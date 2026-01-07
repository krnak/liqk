import React from 'react';
import { View, Text, StyleSheet } from 'react-native';

export default function InboxView() {
  return (
    <View style={styles.container}>
      <View style={styles.emptyState}>
        <Text style={styles.emptyIcon}>📥</Text>
        <Text style={styles.emptyTitle}>Inbox</Text>
        <Text style={styles.emptyText}>No items in your inbox</Text>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#fff',
  },
  emptyState: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
    padding: 40,
  },
  emptyIcon: {
    fontSize: 48,
    marginBottom: 16,
  },
  emptyTitle: {
    fontSize: 20,
    fontWeight: '600',
    color: '#333',
    marginBottom: 8,
  },
  emptyText: {
    fontSize: 14,
    color: '#888',
  },
});
