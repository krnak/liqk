import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  ScrollView,
  TouchableOpacity,
  StyleSheet,
  ActivityIndicator,
} from 'react-native';
import lkd from '../services/lkd';

export default function MarkdownViewer({ filePath, onClose }) {
  const [content, setContent] = useState('');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    loadFile();
  }, [filePath]);

  const loadFile = async () => {
    setLoading(true);
    setError(null);
    try {
      const text = await lkd.getFileContent(filePath);
      setContent(text);
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  const fileName = filePath.split('/').pop();

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <TouchableOpacity style={styles.backBtn} onPress={onClose}>
          <Text style={styles.backIcon}>←</Text>
        </TouchableOpacity>
        <Text style={styles.fileName} numberOfLines={1}>
          {fileName}
        </Text>
        <View style={styles.headerSpacer} />
      </View>

      {loading ? (
        <View style={styles.loading}>
          <ActivityIndicator size="large" color="#007bff" />
        </View>
      ) : error ? (
        <View style={styles.error}>
          <Text style={styles.errorText}>{error}</Text>
          <TouchableOpacity onPress={loadFile}>
            <Text style={styles.retryBtn}>Retry</Text>
          </TouchableOpacity>
        </View>
      ) : (
        <ScrollView style={styles.content}>
          <Text style={styles.markdown}>{content}</Text>
        </ScrollView>
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#fff',
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: 16,
    paddingVertical: 12,
    borderBottomWidth: 1,
    borderBottomColor: '#e0e0e0',
    backgroundColor: '#f8f8f8',
  },
  backBtn: {
    padding: 8,
    marginRight: 8,
  },
  backIcon: {
    fontSize: 20,
    color: '#007bff',
  },
  fileName: {
    flex: 1,
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
  },
  headerSpacer: {
    width: 40,
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
  content: {
    flex: 1,
    padding: 20,
  },
  markdown: {
    fontSize: 15,
    lineHeight: 24,
    color: '#333',
    fontFamily: 'monospace',
  },
});
