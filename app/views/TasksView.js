import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  TouchableOpacity,
  FlatList,
  StyleSheet,
  ActivityIndicator,
} from 'react-native';
import lkd from '../services/lkd';
import AddTaskDialog from '../components/AddTaskDialog';
import ReadmeModal from '../components/ReadmeModal';

const TABS = [
  { key: 'priority', label: 'Priority' },
  { key: 'all', label: 'All' },
  { key: 'completed', label: 'Completed' },
  { key: 'actions', label: 'Actions' },
];

const PRIORITY_COLORS = {
  'priority-highest': '#dc3545',
  'priority-high': '#fd7e14',
  'priority-medium': '#ffc107',
  'priority-pinned': '#6f42c1',
  'priority-none': '#6c757d',
  'priority-low': '#adb5bd',
};

function TaskItem({ task, onMarkDone, onMarkTrashed, onReadmePress }) {
  const priorityColor = PRIORITY_COLORS[task.priority] || '#6c757d';
  const [updating, setUpdating] = useState(false);

  const handleDone = async () => {
    setUpdating(true);
    await onMarkDone(task);
    setUpdating(false);
  };

  const handleTrash = async () => {
    setUpdating(true);
    await onMarkTrashed(task);
    setUpdating(false);
  };

  return (
    <View style={styles.taskItem}>
      <TouchableOpacity
        style={styles.actionBtn}
        onPress={handleDone}
        disabled={updating}
      >
        <Text style={styles.actionIcon}>✅</Text>
      </TouchableOpacity>
      <TouchableOpacity
        style={styles.actionBtn}
        onPress={handleTrash}
        disabled={updating}
      >
        <Text style={styles.actionIcon}>❎</Text>
      </TouchableOpacity>
      <View style={[styles.priorityDot, { backgroundColor: priorityColor }]} />
      <View style={styles.taskContent}>
        <Text style={styles.taskTitle} numberOfLines={2}>
          {task.title}
        </Text>
        {task.project && (
          <Text style={styles.taskProject} numberOfLines={1}>
            {task.project}
          </Text>
        )}
      </View>
      {task.readmeUuid && (
        <TouchableOpacity
          style={styles.actionBtn}
          onPress={() => onReadmePress(task)}
        >
          <Text style={styles.actionIcon}>📝</Text>
        </TouchableOpacity>
      )}
      {task.status && (
        <Text style={styles.taskStatus}>
          {task.status.replace('task-status-', '')}
        </Text>
      )}
    </View>
  );
}

function ActionItem({ action }) {
  const date = new Date(action.created * 1000);
  const timeStr = date.toLocaleString();

  return (
    <View style={styles.actionItem}>
      <View style={styles.actionHeader}>
        <Text style={styles.actionSubject} numberOfLines={1}>
          {action.subjectTitle || 'Unknown'}
        </Text>
        <Text style={styles.actionTime}>{timeStr}</Text>
      </View>
      <Text style={styles.actionDetail}>
        {action.property}: {action.oldValue || '(none)'} → {action.newValue}
      </Text>
    </View>
  );
}

export default function TasksView() {
  const [activeTab, setActiveTab] = useState('priority');
  const [tasks, setTasks] = useState([]);
  const [actions, setActions] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [showAddDialog, setShowAddDialog] = useState(false);
  const [readmeTask, setReadmeTask] = useState(null);

  useEffect(() => {
    loadData();
  }, [activeTab]);

  const loadData = async () => {
    setLoading(true);
    setError(null);
    try {
      if (activeTab === 'actions') {
        const result = await lkd.getActions();
        setActions(result);
      } else {
        let result;
        switch (activeTab) {
          case 'priority':
            result = await lkd.getPriorityTasks();
            break;
          case 'all':
            result = await lkd.getAllTasks();
            break;
          case 'completed':
            result = await lkd.getCompletedTasks();
            break;
        }
        setTasks(result);
      }
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  const handleMarkDone = async (task) => {
    try {
      await lkd.updateTaskStatus(task.uri, 'task-status-done');
      loadData();
    } catch (err) {
      setError(err.message);
    }
  };

  const handleMarkTrashed = async (task) => {
    try {
      await lkd.updateTaskStatus(task.uri, 'task-status-trashed');
      loadData();
    } catch (err) {
      setError(err.message);
    }
  };

  const handleReadmePress = (task) => {
    setReadmeTask(task);
  };

  const renderContent = () => {
    if (loading) {
      return (
        <View style={styles.loading}>
          <ActivityIndicator size="large" color="#007bff" />
        </View>
      );
    }

    if (error) {
      return (
        <View style={styles.error}>
          <Text style={styles.errorText}>{error}</Text>
          <TouchableOpacity onPress={loadData}>
            <Text style={styles.retryBtn}>Retry</Text>
          </TouchableOpacity>
        </View>
      );
    }

    if (activeTab === 'actions') {
      if (actions.length === 0) {
        return (
          <View style={styles.empty}>
            <Text style={styles.emptyText}>No actions</Text>
          </View>
        );
      }
      return (
        <FlatList
          data={actions}
          keyExtractor={(item) => item.uri}
          renderItem={({ item }) => <ActionItem action={item} />}
          style={styles.list}
          contentContainerStyle={styles.listContent}
        />
      );
    }

    if (tasks.length === 0) {
      return (
        <View style={styles.empty}>
          <Text style={styles.emptyText}>No tasks</Text>
        </View>
      );
    }

    return (
      <FlatList
        data={tasks}
        keyExtractor={(item) => item.uri}
        renderItem={({ item }) => (
          <TaskItem
            task={item}
            onMarkDone={handleMarkDone}
            onMarkTrashed={handleMarkTrashed}
            onReadmePress={handleReadmePress}
          />
        )}
        style={styles.list}
        contentContainerStyle={styles.listContent}
      />
    );
  };

  const handleTaskAdded = () => {
    loadData();
  };

  return (
    <View style={styles.container}>
      <Text style={styles.title}>Tasks</Text>

      <View style={styles.tabs}>
        {TABS.map((tab) => (
          <TouchableOpacity
            key={tab.key}
            style={[styles.tab, activeTab === tab.key && styles.tabActive]}
            onPress={() => setActiveTab(tab.key)}
          >
            <Text
              style={[
                styles.tabText,
                activeTab === tab.key && styles.tabTextActive,
              ]}
            >
              {tab.label}
            </Text>
          </TouchableOpacity>
        ))}
      </View>

      {renderContent()}

      <TouchableOpacity
        style={styles.fab}
        onPress={() => setShowAddDialog(true)}
      >
        <Text style={styles.fabIcon}>+</Text>
      </TouchableOpacity>

      <AddTaskDialog
        visible={showAddDialog}
        onClose={() => setShowAddDialog(false)}
        onTaskAdded={handleTaskAdded}
      />

      <ReadmeModal
        visible={!!readmeTask}
        onClose={() => setReadmeTask(null)}
        uuid={readmeTask?.readmeUuid}
        title={readmeTask?.title}
      />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#fff',
    position: 'relative',
  },
  fab: {
    position: 'absolute',
    right: 24,
    bottom: 24,
    width: 56,
    height: 56,
    borderRadius: 12,
    backgroundColor: '#007bff',
    alignItems: 'center',
    justifyContent: 'center',
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.3,
    shadowRadius: 4,
    elevation: 5,
  },
  fabIcon: {
    fontSize: 32,
    color: '#fff',
    lineHeight: 34,
  },
  title: {
    fontSize: 24,
    fontWeight: '600',
    color: '#333',
    padding: 24,
    paddingBottom: 16,
  },
  tabs: {
    flexDirection: 'row',
    borderBottomWidth: 1,
    borderBottomColor: '#e0e0e0',
    paddingHorizontal: 24,
  },
  tab: {
    paddingVertical: 12,
    paddingHorizontal: 16,
    marginRight: 8,
    borderBottomWidth: 2,
    borderBottomColor: 'transparent',
  },
  tabActive: {
    borderBottomColor: '#007bff',
  },
  tabText: {
    fontSize: 14,
    color: '#666',
    fontWeight: '500',
  },
  tabTextActive: {
    color: '#007bff',
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
  empty: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
    padding: 40,
  },
  emptyText: {
    fontSize: 14,
    color: '#888',
  },
  list: {
    flex: 1,
  },
  listContent: {
    padding: 16,
  },
  taskItem: {
    flexDirection: 'row',
    alignItems: 'center',
    padding: 12,
    backgroundColor: '#f8f9fa',
    borderRadius: 8,
    marginBottom: 8,
  },
  actionBtn: {
    width: 24,
    height: 24,
    alignItems: 'center',
    justifyContent: 'center',
  },
  actionIcon: {
    fontSize: 14,
  },
  priorityDot: {
    width: 10,
    height: 10,
    borderRadius: 5,
    marginRight: 12,
  },
  taskContent: {
    flex: 1,
  },
  taskTitle: {
    fontSize: 14,
    color: '#333',
  },
  taskProject: {
    fontSize: 12,
    color: '#888',
    marginTop: 2,
  },
  taskStatus: {
    fontSize: 12,
    color: '#888',
    marginLeft: 8,
    textTransform: 'capitalize',
  },
  actionItem: {
    padding: 12,
    backgroundColor: '#f8f9fa',
    borderRadius: 8,
    marginBottom: 8,
  },
  actionHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 4,
  },
  actionSubject: {
    fontSize: 14,
    fontWeight: '500',
    color: '#333',
    flex: 1,
  },
  actionTime: {
    fontSize: 12,
    color: '#888',
    marginLeft: 8,
  },
  actionDetail: {
    fontSize: 13,
    color: '#666',
  },
});
