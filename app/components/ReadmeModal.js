import React, { useState, useEffect } from 'react';
import {
  Modal,
  View,
  Text,
  ScrollView,
  TouchableOpacity,
  StyleSheet,
  ActivityIndicator,
} from 'react-native';
import lkd from '../services/lkd';

export default function ReadmeModal({ visible, onClose, uuid, title }) {
  const [content, setContent] = useState('');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    if (visible && uuid) {
      loadContent();
    }
  }, [visible, uuid]);

  const loadContent = async () => {
    setLoading(true);
    setError(null);
    try {
      const text = await lkd.getFileByUuid(uuid);
      setContent(text);
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  return (
    <Modal visible={visible} animationType="fade" transparent>
      <View style={styles.overlay}>
        <View style={styles.modal}>
          <View style={styles.header}>
            <Text style={styles.title} numberOfLines={1}>
              {title}
            </Text>
            <TouchableOpacity style={styles.closeBtn} onPress={onClose}>
              <Text style={styles.closeIcon}>✕</Text>
            </TouchableOpacity>
          </View>

          {loading ? (
            <View style={styles.loading}>
              <ActivityIndicator size="large" color="#007bff" />
            </View>
          ) : error ? (
            <View style={styles.error}>
              <Text style={styles.errorText}>{error}</Text>
              <TouchableOpacity onPress={loadContent}>
                <Text style={styles.retryBtn}>Retry</Text>
              </TouchableOpacity>
            </View>
          ) : (
            <ScrollView style={styles.content}>
              <Text style={styles.markdown}>{content}</Text>
            </ScrollView>
          )}
        </View>
      </View>
    </Modal>
  );
}

const styles = StyleSheet.create({
  overlay: {
    flex: 1,
    backgroundColor: 'rgba(0, 0, 0, 0.5)',
    justifyContent: 'center',
    alignItems: 'center',
    padding: 20,
  },
  modal: {
    backgroundColor: '#fff',
    borderRadius: 12,
    width: '100%',
    maxWidth: 600,
    maxHeight: '80%',
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.25,
    shadowRadius: 8,
    elevation: 5,
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    padding: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#e0e0e0',
  },
  title: {
    flex: 1,
    fontSize: 18,
    fontWeight: '600',
    color: '#333',
  },
  closeBtn: {
    padding: 4,
    marginLeft: 12,
  },
  closeIcon: {
    fontSize: 18,
    color: '#666',
  },
  loading: {
    padding: 60,
    alignItems: 'center',
  },
  error: {
    padding: 40,
    alignItems: 'center',
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
    padding: 20,
  },
  markdown: {
    fontSize: 15,
    lineHeight: 24,
    color: '#333',
  },
});
