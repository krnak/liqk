import React, { useState, useEffect, useRef, useMemo } from 'react';
import {
  View,
  Text,
  TouchableOpacity,
  ScrollView,
  StyleSheet,
  ActivityIndicator,
  Animated,
  PanResponder,
  Platform,
} from 'react-native';
import lkd from '../services/lkd';

const STATUS_CATEGORIES = [
  { key: 'focus', label: 'Focus', color: '#28a745', statusValue: 'project-status-focus' },
  { key: 'peripheral', label: 'Peripheral', color: '#ffc107', statusValue: 'project-status-peripheral' },
  { key: 'life-long', label: 'Life-long', color: '#fd7e14', statusValue: 'project-status-life-long' },
  { key: 'inactive', label: 'Inactive', color: '#6c757d', statusValue: 'project-status-inactive' },
  { key: 'completed', label: 'Completed', color: '#8B4513', statusValue: 'project-status-completed' },
];

const STATUS_KEY_MAP = {};
for (const cat of STATUS_CATEGORIES) {
  STATUS_KEY_MAP[cat.statusValue] = cat.key;
}

const AUTO_SCROLL_EDGE = 50;
const AUTO_SCROLL_MAX_SPEED = 8;

function DraggableProjectItem({ project, onPress, onDragStart, onDragMove, onDragEnd, isDraggingThis }) {
  const activeRef = useRef(false);
  const cbRef = useRef({ onDragStart, onDragMove, onDragEnd, onPress, project });
  cbRef.current = { onDragStart, onDragMove, onDragEnd, onPress, project };

  const panResponder = useRef(
    PanResponder.create({
      onStartShouldSetPanResponder: () => true,
      onPanResponderMove: (_, gs) => {
        if (!activeRef.current && (Math.abs(gs.dx) > 3 || Math.abs(gs.dy) > 3)) {
          activeRef.current = true;
          cbRef.current.onDragStart(cbRef.current.project, gs.moveX, gs.moveY);
        }
        if (activeRef.current) {
          cbRef.current.onDragMove(gs.moveX, gs.moveY);
        }
      },
      onPanResponderRelease: () => {
        if (activeRef.current) {
          cbRef.current.onDragEnd(false);
        }
        activeRef.current = false;
      },
      onPanResponderTerminate: () => {
        // Don't end drag — container PanResponder takes over
        activeRef.current = false;
      },
    })
  ).current;

  return (
    <View style={[styles.projectItem, isDraggingThis && styles.projectItemDragging]}>
      <View {...panResponder.panHandlers} style={styles.dragHandle}>
        <Text style={styles.dragHandleIcon}>⠿</Text>
      </View>
      <TouchableOpacity
        style={styles.projectContent}
        onPress={() => cbRef.current.onPress(cbRef.current.project)}
        activeOpacity={0.7}
      >
        <Text style={styles.projectTitle} numberOfLines={1}>
          {project.title}
        </Text>
        {project.abbrv && (
          <Text style={styles.projectAbbrv} numberOfLines={1}>
            {project.abbrv}
          </Text>
        )}
      </TouchableOpacity>
    </View>
  );
}

function CategorySection({ category, projects, onSelectProject, onDragStart, onDragMove, onDragEnd, draggingUri, isHovered, sectionRef }) {
  return (
    <View
      ref={sectionRef}
      style={[
        styles.categorySection,
        isHovered && { backgroundColor: category.color + '30', borderColor: category.color },
      ]}
    >
      <View style={[styles.categoryHeader, { borderLeftColor: category.color }]}>
        <View style={[styles.categoryDot, { backgroundColor: category.color }]} />
        <Text style={styles.categoryLabel}>{category.label}</Text>
        <Text style={styles.categoryCount}>{projects.length}</Text>
      </View>
      {projects.length === 0 ? (
        <View style={styles.emptyCategory}>
          <Text style={styles.emptyCategoryText}>
            {isHovered ? 'Drop here' : 'No projects'}
          </Text>
        </View>
      ) : (
        projects.map((project) => (
          <DraggableProjectItem
            key={project.uri}
            project={project}
            onPress={onSelectProject}
            onDragStart={onDragStart}
            onDragMove={onDragMove}
            onDragEnd={onDragEnd}
            isDraggingThis={draggingUri === project.uri}
          />
        ))
      )}
    </View>
  );
}

export default function ProjectsView({ onSelectProject }) {
  const [projects, setProjects] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [drag, setDrag] = useState(null);
  const [hoverKey, setHoverKey] = useState(null);

  const dragPos = useRef(new Animated.ValueXY()).current;
  const categoryRefs = useRef({});
  const categoryLayouts = useRef({});
  const dragRef = useRef(null);
  const hoverKeyRef = useRef(null);

  // Scroll tracking
  const scrollRef = useRef(null);
  const scrollAreaRef = useRef(null);
  const scrollOffset = useRef(0);
  const scrollOffsetAtDragStart = useRef(0);
  const scrollBounds = useRef({ y: 0, height: 0 });
  const contentHeight = useRef(0);

  // Auto-scroll
  const autoScrollSpeed = useRef(0);
  const autoScrollRaf = useRef(null);
  const lastMoveY = useRef(0);

  // Container PanResponder — takes over from drag handle once drag starts.
  // Lives outside the ScrollView so scrollTo can't disrupt it.
  const handleDragMoveRef = useRef(null);
  const handleDragEndRef = useRef(null);
  const containerPanResponder = useRef(
    PanResponder.create({
      onStartShouldSetPanResponder: () => false,
      onMoveShouldSetPanResponder: () => !!dragRef.current,
      onPanResponderMove: (_, gs) => {
        handleDragMoveRef.current?.(gs.moveX, gs.moveY);
      },
      onPanResponderRelease: () => {
        handleDragEndRef.current?.();
      },
      onPanResponderTerminate: () => {
        handleDragEndRef.current?.();
      },
    })
  ).current;

  useEffect(() => {
    return () => {
      if (autoScrollRaf.current) {
        cancelAnimationFrame(autoScrollRaf.current);
      }
    };
  }, []);

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await lkd.getProjects();
      setProjects(result);
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  const grouped = useMemo(() => {
    const groups = {};
    for (const cat of STATUS_CATEGORIES) {
      groups[cat.key] = [];
    }
    groups['other'] = [];

    for (const project of projects) {
      const key = STATUS_KEY_MAP[project.status];
      if (key) {
        groups[key].push(project);
      } else {
        groups['other'].push(project);
      }
    }

    for (const key of Object.keys(groups)) {
      groups[key].sort((a, b) => (a.rank ?? Infinity) - (b.rank ?? Infinity));
    }

    return groups;
  }, [projects]);

  const measureCategories = () => {
    return Promise.all(
      Object.entries(categoryRefs.current).map(
        ([key, ref]) =>
          new Promise((resolve) => {
            if (ref?.measureInWindow) {
              ref.measureInWindow((x, y, w, h) => {
                categoryLayouts.current[key] = { y, height: h };
                resolve();
              });
            } else {
              resolve();
            }
          })
      )
    );
  };

  const findHoveredCategory = (screenY) => {
    const scrollDelta = scrollOffset.current - scrollOffsetAtDragStart.current;
    for (const [key, layout] of Object.entries(categoryLayouts.current)) {
      const adjustedY = layout.y - scrollDelta;
      if (screenY >= adjustedY && screenY <= adjustedY + layout.height) {
        return key;
      }
    }
    return null;
  };

  const updateHover = (screenY) => {
    const key = findHoveredCategory(screenY);
    if (key !== hoverKeyRef.current) {
      hoverKeyRef.current = key;
      setHoverKey(key);
    }
  };

  const startAutoScroll = () => {
    if (autoScrollRaf.current) return;
    const step = () => {
      const speed = autoScrollSpeed.current;
      if (Math.abs(speed) > 0.1 && scrollRef.current) {
        const maxOff = Math.max(0, contentHeight.current - scrollBounds.current.height);
        const newOff = Math.max(0, Math.min(maxOff, scrollOffset.current + speed));
        if (newOff !== scrollOffset.current) {
          scrollOffset.current = newOff;
          scrollRef.current.scrollTo({ y: newOff, animated: false });
          updateHover(lastMoveY.current);
        }
      }
      autoScrollRaf.current = requestAnimationFrame(step);
    };
    autoScrollRaf.current = requestAnimationFrame(step);
  };

  const stopAutoScroll = () => {
    autoScrollSpeed.current = 0;
    if (autoScrollRaf.current) {
      cancelAnimationFrame(autoScrollRaf.current);
      autoScrollRaf.current = null;
    }
  };

  const checkAutoScroll = (screenY) => {
    const { y: top, height } = scrollBounds.current;
    const bottom = top + height;

    if (screenY < top + AUTO_SCROLL_EDGE) {
      const ratio = Math.min(1, (top + AUTO_SCROLL_EDGE - screenY) / AUTO_SCROLL_EDGE);
      autoScrollSpeed.current = -AUTO_SCROLL_MAX_SPEED * ratio;
      startAutoScroll();
    } else if (screenY > bottom - AUTO_SCROLL_EDGE) {
      const ratio = Math.min(1, (screenY - (bottom - AUTO_SCROLL_EDGE)) / AUTO_SCROLL_EDGE);
      autoScrollSpeed.current = AUTO_SCROLL_MAX_SPEED * ratio;
      startAutoScroll();
    } else {
      autoScrollSpeed.current = 0;
    }
  };

  const handleDragStart = async (project, x, y) => {
    const sourceKey = STATUS_KEY_MAP[project.status] || 'other';

    // Measure scroll area bounds
    await new Promise((resolve) => {
      if (scrollAreaRef.current?.measureInWindow) {
        scrollAreaRef.current.measureInWindow((sx, sy, sw, sh) => {
          scrollBounds.current = { y: sy, height: sh };
          resolve();
        });
      } else {
        resolve();
      }
    });

    await measureCategories();
    scrollOffsetAtDragStart.current = scrollOffset.current;
    lastMoveY.current = y;

    dragPos.setValue({ x: x - 90, y: y - 30 });
    const data = { project, sourceKey };
    dragRef.current = data;
    setDrag(data);
  };

  const handleDragMove = (x, y) => {
    dragPos.setValue({ x: x - 90, y: y - 30 });
    lastMoveY.current = y;
    updateHover(y);
    checkAutoScroll(y);
  };

  const handleDragEnd = async () => {
    stopAutoScroll();

    const currentDrag = dragRef.current;
    const currentHover = hoverKeyRef.current;

    dragRef.current = null;
    hoverKeyRef.current = null;
    setDrag(null);
    setHoverKey(null);

    if (!currentDrag || !currentHover || currentDrag.sourceKey === currentHover) {
      return;
    }

    const targetCat = STATUS_CATEGORIES.find((c) => c.key === currentHover);
    if (!targetCat) return;

    const projectUri = currentDrag.project.uri;

    setProjects((prev) =>
      prev.map((p) => (p.uri === projectUri ? { ...p, status: targetCat.statusValue } : p))
    );

    try {
      await lkd.updateProjectStatus(projectUri, targetCat.statusValue);
      const result = await lkd.getProjects();
      setProjects(result);
    } catch (err) {
      setError(err.message);
      const result = await lkd.getProjects();
      setProjects(result);
    }
  };

  // Keep container PanResponder callbacks current
  handleDragMoveRef.current = handleDragMove;
  handleDragEndRef.current = handleDragEnd;

  const setCategoryRef = (key) => (ref) => {
    categoryRefs.current[key] = ref;
  };

  if (loading) {
    return (
      <View style={styles.container} {...containerPanResponder.panHandlers}>
        <Text style={styles.title}>Projects</Text>
        <View style={styles.loading}>
          <ActivityIndicator size="large" color="#007bff" />
        </View>
      </View>
    );
  }

  if (error) {
    return (
      <View style={styles.container} {...containerPanResponder.panHandlers}>
        <Text style={styles.title}>Projects</Text>
        <View style={styles.error}>
          <Text style={styles.errorText}>{error}</Text>
          <TouchableOpacity onPress={loadData}>
            <Text style={styles.retryBtn}>Retry</Text>
          </TouchableOpacity>
        </View>
      </View>
    );
  }

  return (
    <View style={styles.container} {...containerPanResponder.panHandlers}>
      <Text style={styles.title}>Projects</Text>
      <View style={styles.scrollArea} ref={scrollAreaRef}>
        <ScrollView
          ref={scrollRef}
          contentContainerStyle={styles.listContent}
          scrollEnabled={!drag}
          onScroll={(e) => { scrollOffset.current = e.nativeEvent.contentOffset.y; }}
          scrollEventThrottle={16}
          onContentSizeChange={(w, h) => { contentHeight.current = h; }}
        >
          {STATUS_CATEGORIES.map((cat) => (
            <CategorySection
              key={cat.key}
              category={cat}
              projects={grouped[cat.key]}
              onSelectProject={onSelectProject}
              onDragStart={handleDragStart}
              onDragMove={handleDragMove}
              onDragEnd={handleDragEnd}
              draggingUri={drag?.project.uri}
              isHovered={hoverKey === cat.key && drag?.sourceKey !== cat.key}
              sectionRef={setCategoryRef(cat.key)}
            />
          ))}
          {grouped['other'].length > 0 && (
            <CategorySection
              category={{ key: 'other', label: 'Other', color: '#adb5bd', statusValue: null }}
              projects={grouped['other']}
              onSelectProject={onSelectProject}
              onDragStart={handleDragStart}
              onDragMove={handleDragMove}
              onDragEnd={handleDragEnd}
              draggingUri={drag?.project.uri}
              isHovered={false}
              sectionRef={setCategoryRef('other')}
            />
          )}
        </ScrollView>
      </View>

      {drag && (
        <Animated.View
          pointerEvents="none"
          style={[
            styles.dragGhost,
            { transform: dragPos.getTranslateTransform() },
          ]}
        >
          <Text style={styles.dragGhostTitle} numberOfLines={1}>
            {drag.project.title}
          </Text>
        </Animated.View>
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#fff',
  },
  title: {
    fontSize: 24,
    fontWeight: '600',
    color: '#333',
    padding: 24,
    paddingBottom: 16,
  },
  loading: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
  },
  error: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
    padding: 40,
  },
  errorText: {
    fontSize: 14,
    color: '#dc3545',
    textAlign: 'center',
    marginBottom: 16,
  },
  retryBtn: {
    fontSize: 16,
    color: '#007bff',
  },
  scrollArea: {
    flex: 1,
  },
  listContent: {
    padding: 16,
    paddingBottom: 32,
  },
  categorySection: {
    marginBottom: 16,
    borderRadius: 8,
    overflow: 'hidden',
    borderWidth: 2,
    borderColor: 'transparent',
  },
  categoryHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingVertical: 10,
    paddingHorizontal: 12,
    borderLeftWidth: 4,
    backgroundColor: '#f8f9fa',
  },
  categoryDot: {
    width: 8,
    height: 8,
    borderRadius: 4,
    marginRight: 8,
  },
  categoryLabel: {
    fontSize: 14,
    fontWeight: '600',
    color: '#333',
    flex: 1,
  },
  categoryCount: {
    fontSize: 13,
    color: '#888',
  },
  emptyCategory: {
    paddingVertical: 16,
    paddingHorizontal: 12,
    minHeight: 48,
    alignItems: 'center',
    justifyContent: 'center',
  },
  emptyCategoryText: {
    fontSize: 13,
    color: '#bbb',
    fontStyle: 'italic',
  },
  projectItem: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingVertical: 10,
    paddingRight: 12,
    backgroundColor: '#fff',
    borderBottomWidth: StyleSheet.hairlineWidth,
    borderBottomColor: '#eee',
  },
  projectItemDragging: {
    opacity: 0.3,
  },
  dragHandle: {
    width: 36,
    height: 36,
    alignItems: 'center',
    justifyContent: 'center',
    ...Platform.select({ web: { cursor: 'grab' } }),
  },
  dragHandleIcon: {
    fontSize: 18,
    color: '#ccc',
    ...Platform.select({ web: { userSelect: 'none' } }),
  },
  projectContent: {
    flex: 1,
  },
  projectTitle: {
    fontSize: 14,
    color: '#333',
  },
  projectAbbrv: {
    fontSize: 12,
    color: '#888',
    marginTop: 2,
  },
  dragGhost: {
    position: 'absolute',
    width: 180,
    padding: 12,
    backgroundColor: '#fff',
    borderRadius: 8,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 4 },
    shadowOpacity: 0.2,
    shadowRadius: 8,
    elevation: 8,
  },
  dragGhostTitle: {
    fontSize: 14,
    fontWeight: '500',
    color: '#333',
  },
});
