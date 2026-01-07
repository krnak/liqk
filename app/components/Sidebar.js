import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  TouchableOpacity,
  ScrollView,
  StyleSheet,
  ActivityIndicator,
} from 'react-native';
import lkd from '../services/lkd';

function SectionHeader({ title, collapsed, onToggle }) {
  return (
    <TouchableOpacity style={styles.sectionHeader} onPress={onToggle}>
      <Text style={styles.sectionTitle}>{title}</Text>
      <Text style={styles.sectionChevron}>{collapsed ? '▸' : '▾'}</Text>
    </TouchableOpacity>
  );
}

function MenuItem({ icon, label, onPress, disabled, active }) {
  return (
    <TouchableOpacity
      style={[
        styles.menuItem,
        disabled && styles.menuItemDisabled,
        active && styles.menuItemActive,
      ]}
      onPress={onPress}
      disabled={disabled}
    >
      <Text style={[styles.menuIcon, disabled && styles.menuIconDisabled]}>
        {icon}
      </Text>
      <Text
        style={[styles.menuLabel, disabled && styles.menuLabelDisabled]}
        numberOfLines={1}
      >
        {label}
      </Text>
    </TouchableOpacity>
  );
}

function FilesystemBrowser({ currentPath, onNavigate, onFileOpen }) {
  const [items, setItems] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    loadDirectory();
  }, [currentPath]);

  const loadDirectory = async () => {
    setLoading(true);
    setError(null);
    try {
      const contents = await lkd.listDirectory(currentPath);
      setItems(contents);
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  const isMarkdown = (item) => {
    return (
      item.type === 'file' &&
      (item.label.endsWith('.md') || item.label.endsWith('.markdown'))
    );
  };

  const handleItemPress = (item) => {
    if (item.type === 'directory') {
      const newPath =
        currentPath === '/' ? `/${item.label}` : `${currentPath}/${item.label}`;
      onNavigate(newPath);
    } else if (isMarkdown(item)) {
      const filePath =
        currentPath === '/' ? `/${item.label}` : `${currentPath}/${item.label}`;
      onFileOpen(filePath);
    }
  };

  const handleGoUp = () => {
    if (currentPath === '/') return;
    const segments = currentPath.split('/').filter(Boolean);
    segments.pop();
    onNavigate(segments.length === 0 ? '/' : '/' + segments.join('/'));
  };

  if (loading) {
    return (
      <View style={styles.fsLoading}>
        <ActivityIndicator size="small" color="#666" />
      </View>
    );
  }

  if (error) {
    return (
      <View style={styles.fsError}>
        <Text style={styles.fsErrorText}>{error}</Text>
        <TouchableOpacity onPress={loadDirectory}>
          <Text style={styles.fsRetry}>Retry</Text>
        </TouchableOpacity>
      </View>
    );
  }

  return (
    <View style={styles.fsBrowser}>
      {currentPath !== '/' && (
        <MenuItem icon="↩" label=".." onPress={handleGoUp} />
      )}
      {items.map((item) => (
        <MenuItem
          key={item.uri}
          icon={item.type === 'directory' ? '📁' : '📄'}
          label={item.label}
          onPress={() => handleItemPress(item)}
          disabled={item.type === 'file' && !isMarkdown(item)}
        />
      ))}
      {items.length === 0 && (
        <Text style={styles.fsEmpty}>Empty directory</Text>
      )}
    </View>
  );
}

export default function Sidebar({
  collapsed,
  onToggleCollapse,
  activeView,
  onNavigate,
  onFileOpen,
}) {
  const [fsPath, setFsPath] = useState('/');
  const [sectionsCollapsed, setSectionsCollapsed] = useState({
    favorites: true,
    filesystem: false,
  });

  const toggleSection = (section) => {
    setSectionsCollapsed((prev) => ({
      ...prev,
      [section]: !prev[section],
    }));
  };

  if (collapsed) {
    return (
      <View style={styles.collapsedSidebar}>
        <TouchableOpacity style={styles.expandBtn} onPress={onToggleCollapse}>
          <Text style={styles.expandIcon}>☰</Text>
        </TouchableOpacity>
      </View>
    );
  }

  return (
    <View style={styles.sidebar}>
      <View style={styles.sidebarHeader}>
        <Text style={styles.sidebarTitle}>Liqk</Text>
        <TouchableOpacity style={styles.collapseBtn} onPress={onToggleCollapse}>
          <Text style={styles.collapseIcon}>✕</Text>
        </TouchableOpacity>
      </View>

      <ScrollView style={styles.sidebarContent}>
        {/* Fixed head items */}
        <View style={styles.section}>
          <MenuItem
            icon="✅"
            label="Tasks"
            onPress={() => onNavigate('tasks')}
            active={activeView === 'tasks'}
          />
          <MenuItem
            icon="📥"
            label="Inbox"
            onPress={() => onNavigate('inbox')}
            active={activeView === 'inbox'}
          />
        </View>

        {/* Favorites */}
        <View style={styles.section}>
          <SectionHeader
            title="Favorites"
            collapsed={sectionsCollapsed.favorites}
            onToggle={() => toggleSection('favorites')}
          />
          {!sectionsCollapsed.favorites && (
            <Text style={styles.emptySection}>No favorites yet</Text>
          )}
        </View>

        {/* Filesystem */}
        <View style={styles.section}>
          <SectionHeader
            title="Files"
            collapsed={sectionsCollapsed.filesystem}
            onToggle={() => toggleSection('filesystem')}
          />
          {!sectionsCollapsed.filesystem && (
            <FilesystemBrowser
              currentPath={fsPath}
              onNavigate={setFsPath}
              onFileOpen={onFileOpen}
            />
          )}
        </View>
      </ScrollView>

      {/* Fixed tail items */}
      <View style={styles.sidebarFooter}>
        <MenuItem
          icon="⚙"
          label="Settings"
          onPress={() => onNavigate('settings')}
          active={activeView === 'settings'}
        />
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  sidebar: {
    width: 260,
    backgroundColor: '#f5f5f5',
    borderRightWidth: 1,
    borderRightColor: '#e0e0e0',
    flexDirection: 'column',
  },
  collapsedSidebar: {
    width: 48,
    backgroundColor: '#f5f5f5',
    borderRightWidth: 1,
    borderRightColor: '#e0e0e0',
    alignItems: 'center',
    paddingTop: 12,
  },
  sidebarHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingHorizontal: 16,
    paddingVertical: 12,
    borderBottomWidth: 1,
    borderBottomColor: '#e0e0e0',
  },
  sidebarTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333',
  },
  collapseBtn: {
    padding: 4,
  },
  collapseIcon: {
    fontSize: 16,
    color: '#666',
  },
  expandBtn: {
    padding: 8,
  },
  expandIcon: {
    fontSize: 20,
    color: '#333',
  },
  sidebarContent: {
    flex: 1,
  },
  sidebarFooter: {
    borderTopWidth: 1,
    borderTopColor: '#e0e0e0',
    paddingVertical: 8,
  },
  section: {
    paddingVertical: 4,
  },
  sectionHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingHorizontal: 16,
    paddingVertical: 8,
  },
  sectionTitle: {
    fontSize: 12,
    fontWeight: '600',
    color: '#888',
    textTransform: 'uppercase',
    letterSpacing: 0.5,
  },
  sectionChevron: {
    fontSize: 12,
    color: '#888',
  },
  emptySection: {
    paddingHorizontal: 16,
    paddingVertical: 8,
    fontSize: 13,
    color: '#999',
    fontStyle: 'italic',
  },
  menuItem: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: 16,
    paddingVertical: 10,
  },
  menuItemDisabled: {
    opacity: 0.5,
  },
  menuItemActive: {
    backgroundColor: '#e3e3e3',
  },
  menuIcon: {
    fontSize: 16,
    marginRight: 10,
    width: 20,
    textAlign: 'center',
  },
  menuIconDisabled: {
    opacity: 0.6,
  },
  menuLabel: {
    fontSize: 14,
    color: '#333',
    flex: 1,
  },
  menuLabelDisabled: {
    color: '#999',
  },
  fsBrowser: {
    paddingLeft: 8,
  },
  fsLoading: {
    padding: 16,
    alignItems: 'center',
  },
  fsError: {
    padding: 16,
  },
  fsErrorText: {
    fontSize: 12,
    color: '#c00',
  },
  fsRetry: {
    fontSize: 12,
    color: '#007bff',
    marginTop: 4,
  },
  fsEmpty: {
    paddingHorizontal: 16,
    paddingVertical: 8,
    fontSize: 13,
    color: '#999',
    fontStyle: 'italic',
  },
});
