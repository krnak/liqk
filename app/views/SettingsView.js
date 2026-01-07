import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  TextInput,
  TouchableOpacity,
  StyleSheet,
  Alert,
  ActivityIndicator,
} from 'react-native';
import lkd from '../services/lkd';

export default function SettingsView({ onTokenCleared }) {
  const [baseUrl, setBaseUrl] = useState('');
  const [token, setToken] = useState('');
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    setBaseUrl(lkd.getBaseUrl());
    setToken(lkd.getToken() || '');
  }, []);

  const handleSave = async () => {
    if (!token.trim()) {
      Alert.alert('Error', 'Access token is required');
      return;
    }

    setSaving(true);
    setSaved(false);

    try {
      const isValid = await lkd.validateToken(token.trim(), baseUrl.trim());
      if (isValid) {
        await lkd.setBaseUrl(baseUrl.trim());
        await lkd.setToken(token.trim());
        setSaved(true);
        setTimeout(() => setSaved(false), 2000);
      } else {
        Alert.alert('Error', 'Invalid token or unable to connect to server');
      }
    } catch (err) {
      Alert.alert('Error', 'Connection failed: ' + err.message);
    } finally {
      setSaving(false);
    }
  };

  const handleClearToken = async () => {
    Alert.alert(
      'Clear Token',
      'Are you sure you want to clear the access token? You will need to re-enter it.',
      [
        { text: 'Cancel', style: 'cancel' },
        {
          text: 'Clear',
          style: 'destructive',
          onPress: async () => {
            await lkd.clearToken();
            setToken('');
            onTokenCleared?.();
          },
        },
      ]
    );
  };

  return (
    <View style={styles.container}>
      <Text style={styles.title}>Settings</Text>

      <View style={styles.section}>
        <Text style={styles.sectionTitle}>LKD Connection</Text>

        <View style={styles.field}>
          <Text style={styles.label}>Server URL</Text>
          <TextInput
            style={styles.input}
            value={baseUrl}
            onChangeText={setBaseUrl}
            placeholder="http://localhost:8080"
            autoCapitalize="none"
            autoCorrect={false}
          />
        </View>

        <View style={styles.field}>
          <Text style={styles.label}>Access Token</Text>
          <TextInput
            style={styles.input}
            value={token}
            onChangeText={setToken}
            placeholder="32-character hex token"
            autoCapitalize="none"
            autoCorrect={false}
            secureTextEntry
          />
        </View>

        <View style={styles.buttons}>
          <TouchableOpacity
            style={[styles.saveBtn, saving && styles.saveBtnDisabled]}
            onPress={handleSave}
            disabled={saving}
          >
            {saving ? (
              <ActivityIndicator size="small" color="#fff" />
            ) : (
              <Text style={styles.saveBtnText}>
                {saved ? 'Saved!' : 'Save'}
              </Text>
            )}
          </TouchableOpacity>

          <TouchableOpacity style={styles.clearBtn} onPress={handleClearToken}>
            <Text style={styles.clearBtnText}>Clear Token</Text>
          </TouchableOpacity>
        </View>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#fff',
    padding: 24,
  },
  title: {
    fontSize: 24,
    fontWeight: '600',
    color: '#333',
    marginBottom: 24,
  },
  section: {
    marginBottom: 32,
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#555',
    marginBottom: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#eee',
    paddingBottom: 8,
  },
  field: {
    marginBottom: 16,
  },
  label: {
    fontSize: 14,
    fontWeight: '500',
    color: '#333',
    marginBottom: 6,
  },
  input: {
    borderWidth: 1,
    borderColor: '#ddd',
    borderRadius: 8,
    padding: 12,
    fontSize: 16,
    backgroundColor: '#fafafa',
  },
  buttons: {
    flexDirection: 'row',
    gap: 12,
    marginTop: 8,
  },
  saveBtn: {
    backgroundColor: '#007bff',
    borderRadius: 8,
    paddingVertical: 12,
    paddingHorizontal: 24,
    minWidth: 100,
    alignItems: 'center',
  },
  saveBtnDisabled: {
    backgroundColor: '#6c9bd1',
  },
  saveBtnText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
  clearBtn: {
    borderWidth: 1,
    borderColor: '#dc3545',
    borderRadius: 8,
    paddingVertical: 12,
    paddingHorizontal: 24,
  },
  clearBtnText: {
    color: '#dc3545',
    fontSize: 16,
    fontWeight: '500',
  },
});
