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

function ProjectItem({ project, onPress }) {
  return (
    <TouchableOpacity style={styles.projectItem} onPress={() => onPress(project)}>
      <View style={styles.projectContent}>
        <Text style={styles.projectTitle} numberOfLines={1}>
          {project.title}
        </Text>
        {project.abbrv && (
          <Text style={styles.projectAbbrv} numberOfLines={1}>
            {project.abbrv}
          </Text>
        )}
      </View>
    </TouchableOpacity>
  );
}

export default function ProjectsView({ onSelectProject }) {
  const [projects, setProjects] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

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

    if (projects.length === 0) {
      return (
        <View style={styles.empty}>
          <Text style={styles.emptyText}>No projects</Text>
        </View>
      );
    }

    return (
      <FlatList
        data={projects}
        keyExtractor={(item) => item.uri}
        renderItem={({ item }) => (
          <ProjectItem project={item} onPress={onSelectProject} />
        )}
        style={styles.list}
        contentContainerStyle={styles.listContent}
      />
    );
  };

  return (
    <View style={styles.container}>
      <Text style={styles.title}>Projects</Text>
      {renderContent()}
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
  projectItem: {
    flexDirection: 'row',
    alignItems: 'center',
    padding: 12,
    backgroundColor: '#f8f9fa',
    borderRadius: 8,
    marginBottom: 8,
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
});
