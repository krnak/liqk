import React, { useState, useEffect, useMemo } from 'react';
import {
  Modal,
  View,
  Text,
  TextInput,
  TouchableOpacity,
  StyleSheet,
  ActivityIndicator,
} from 'react-native';
import lkd from '../services/lkd';

export default function AddTaskDialog({ visible, onClose, onTaskAdded }) {
  const [text, setText] = useState('');
  const [abbreviations, setAbbreviations] = useState({ projects: [], priorities: [] });
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState(null);

  useEffect(() => {
    if (visible) {
      loadAbbreviations();
      setText('');
    }
  }, [visible]);

  const loadAbbreviations = async () => {
    setLoading(true);
    setError(null);
    try {
      const abbrvs = await lkd.getAbbreviations();
      setAbbreviations(abbrvs);
    } catch (err) {
      setError('Failed to load abbreviations: ' + err.message);
    } finally {
      setLoading(false);
    }
  };

  const parsed = useMemo(() => {
    const lines = text.split('\n');
    const firstLine = lines[0] || '';
    const restLines = lines.slice(1).join('\n').trim();
    const words = firstLine.split(/\s+/).filter(Boolean);

    let priority = null;
    const projects = [];
    const titleWords = [];

    for (const word of words) {
      const lowerWord = word.toLowerCase();

      // Check priority (first match wins)
      if (!priority) {
        const matchedPriority = abbreviations.priorities.find(
          (p) => p.abbrv.toLowerCase() === lowerWord
        );
        if (matchedPriority) {
          priority = matchedPriority;
          continue;
        }
      }

      // Check projects (multiple allowed)
      const matchedProject = abbreviations.projects.find(
        (p) => p.abbrv.toLowerCase() === lowerWord
      );
      if (matchedProject) {
        if (!projects.find((p) => p.uri === matchedProject.uri)) {
          projects.push(matchedProject);
        }
        continue;
      }

      // Not a tag, keep as title word
      titleWords.push(word);
    }

    const title = titleWords.join(' ');
    const hasReadme = restLines.length > 0;
    const readmeContent = hasReadme ? `# ${title}\n\n${restLines}` : null;

    return {
      title,
      priority,
      projects,
      hasReadme,
      readmeContent,
    };
  }, [text, abbreviations]);

  const handleAdd = async () => {
    if (!parsed.title.trim()) {
      setError('Title cannot be empty');
      return;
    }

    setSaving(true);
    setError(null);

    try {
      await lkd.createTask(
        parsed.title.trim(),
        parsed.priority?.uri || null,
        parsed.projects.map((p) => p.uri),
        parsed.readmeContent
      );
      onTaskAdded?.();
      onClose();
    } catch (err) {
      setError('Failed to create task: ' + err.message);
    } finally {
      setSaving(false);
    }
  };

  return (
    <Modal visible={visible} animationType="fade" transparent>
      <View style={styles.overlay}>
        <View style={styles.dialog}>
          <Text style={styles.title}>Add Task</Text>

          {loading ? (
            <View style={styles.loading}>
              <ActivityIndicator size="small" color="#007bff" />
              <Text style={styles.loadingText}>Loading abbreviations...</Text>
            </View>
          ) : (
            <>
              <TextInput
                style={styles.input}
                value={text}
                onChangeText={setText}
                placeholder="Enter task... (use abbrvs for priority/projects)"
                multiline
                numberOfLines={3}
                autoFocus
              />

              <View style={styles.output}>
                <View style={styles.outputRow}>
                  <Text style={styles.outputLabel}>Title:</Text>
                  <Text style={styles.outputValue} numberOfLines={2}>
                    {parsed.title || '(empty)'}
                  </Text>
                </View>
                <View style={styles.outputRow}>
                  <Text style={styles.outputLabel}>Priority:</Text>
                  <Text style={styles.outputValue}>
                    {parsed.priority?.abbrv || '(none)'}
                  </Text>
                </View>
                <View style={styles.outputRow}>
                  <Text style={styles.outputLabel}>Projects:</Text>
                  <Text style={styles.outputValue}>
                    {parsed.projects.length > 0
                      ? parsed.projects.map((p) => p.title || p.abbrv).join(', ')
                      : '(none)'}
                  </Text>
                </View>
                <View style={styles.outputRow}>
                  <Text style={styles.outputLabel}>Readme:</Text>
                  <Text style={styles.outputValue}>
                    {parsed.hasReadme ? '📝 will be created' : '(none)'}
                  </Text>
                </View>
              </View>

              {error && <Text style={styles.error}>{error}</Text>}

              <View style={styles.buttons}>
                <TouchableOpacity
                  style={styles.closeBtn}
                  onPress={onClose}
                  disabled={saving}
                >
                  <Text style={styles.closeBtnText}>Close</Text>
                </TouchableOpacity>
                <TouchableOpacity
                  style={[styles.addBtn, saving && styles.addBtnDisabled]}
                  onPress={handleAdd}
                  disabled={saving || !parsed.title.trim()}
                >
                  {saving ? (
                    <ActivityIndicator size="small" color="#fff" />
                  ) : (
                    <Text style={styles.addBtnText}>Add</Text>
                  )}
                </TouchableOpacity>
              </View>
            </>
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
  dialog: {
    backgroundColor: '#fff',
    borderRadius: 12,
    padding: 24,
    width: '100%',
    maxWidth: 500,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.25,
    shadowRadius: 8,
    elevation: 5,
  },
  title: {
    fontSize: 20,
    fontWeight: '600',
    color: '#333',
    marginBottom: 16,
  },
  loading: {
    flexDirection: 'row',
    alignItems: 'center',
    padding: 20,
  },
  loadingText: {
    marginLeft: 10,
    color: '#666',
  },
  input: {
    borderWidth: 1,
    borderColor: '#ddd',
    borderRadius: 8,
    padding: 12,
    fontSize: 16,
    backgroundColor: '#fafafa',
    minHeight: 80,
    textAlignVertical: 'top',
  },
  output: {
    marginTop: 16,
    padding: 12,
    backgroundColor: '#f0f4f8',
    borderRadius: 8,
  },
  outputRow: {
    flexDirection: 'row',
    marginBottom: 6,
  },
  outputLabel: {
    width: 70,
    fontSize: 13,
    fontWeight: '600',
    color: '#555',
  },
  outputValue: {
    flex: 1,
    fontSize: 13,
    color: '#333',
  },
  error: {
    color: '#dc3545',
    fontSize: 14,
    marginTop: 12,
  },
  buttons: {
    flexDirection: 'row',
    justifyContent: 'flex-end',
    marginTop: 20,
    gap: 12,
  },
  closeBtn: {
    paddingVertical: 10,
    paddingHorizontal: 20,
    borderRadius: 8,
    borderWidth: 1,
    borderColor: '#ccc',
  },
  closeBtnText: {
    fontSize: 16,
    color: '#666',
  },
  addBtn: {
    paddingVertical: 10,
    paddingHorizontal: 24,
    borderRadius: 8,
    backgroundColor: '#007bff',
    minWidth: 80,
    alignItems: 'center',
  },
  addBtnDisabled: {
    backgroundColor: '#6c9bd1',
  },
  addBtnText: {
    fontSize: 16,
    color: '#fff',
    fontWeight: '600',
  },
});
