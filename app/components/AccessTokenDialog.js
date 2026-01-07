import React, { useState } from 'react';
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

export default function AccessTokenDialog({ visible, onSuccess }) {
  const [token, setToken] = useState('');
  const [baseUrl, setBaseUrl] = useState(lkd.getBaseUrl());
  const [error, setError] = useState('');
  const [validating, setValidating] = useState(false);

  const handleConnect = async () => {
    if (!token.trim()) {
      setError('Please enter an access token');
      return;
    }

    setError('');
    setValidating(true);

    try {
      const isValid = await lkd.validateToken(token.trim(), baseUrl.trim());

      if (isValid) {
        await lkd.setBaseUrl(baseUrl.trim());
        await lkd.setToken(token.trim());
        onSuccess();
      } else {
        setError('Invalid token or unable to connect to server');
      }
    } catch (err) {
      setError('Connection failed: ' + err.message);
    } finally {
      setValidating(false);
    }
  };

  return (
    <Modal visible={visible} animationType="fade" transparent>
      <View style={styles.overlay}>
        <View style={styles.dialog}>
          <Text style={styles.title}>Connect to LKD</Text>
          <Text style={styles.subtitle}>
            Enter your access token to connect to the Liqk Knowledge Database
          </Text>

          <Text style={styles.label}>Server URL</Text>
          <TextInput
            style={styles.input}
            value={baseUrl}
            onChangeText={setBaseUrl}
            placeholder="http://localhost:8080"
            autoCapitalize="none"
            autoCorrect={false}
          />

          <Text style={styles.label}>Access Token</Text>
          <TextInput
            style={styles.input}
            value={token}
            onChangeText={setToken}
            placeholder="Enter 32-character hex token"
            autoCapitalize="none"
            autoCorrect={false}
            secureTextEntry
          />

          {error ? <Text style={styles.error}>{error}</Text> : null}

          <TouchableOpacity
            style={[styles.button, validating && styles.buttonDisabled]}
            onPress={handleConnect}
            disabled={validating}
          >
            {validating ? (
              <ActivityIndicator color="#fff" />
            ) : (
              <Text style={styles.buttonText}>Connect</Text>
            )}
          </TouchableOpacity>

          <Text style={styles.hint}>
            The access token can be found in the gate service logs or .env file
          </Text>
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
    maxWidth: 400,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.25,
    shadowRadius: 8,
    elevation: 5,
  },
  title: {
    fontSize: 22,
    fontWeight: '600',
    marginBottom: 8,
    color: '#333',
  },
  subtitle: {
    fontSize: 14,
    color: '#666',
    marginBottom: 20,
    lineHeight: 20,
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
    marginBottom: 16,
    backgroundColor: '#fafafa',
  },
  error: {
    color: '#dc3545',
    fontSize: 14,
    marginBottom: 16,
  },
  button: {
    backgroundColor: '#007bff',
    borderRadius: 8,
    padding: 14,
    alignItems: 'center',
  },
  buttonDisabled: {
    backgroundColor: '#6c9bd1',
  },
  buttonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
  hint: {
    fontSize: 12,
    color: '#888',
    marginTop: 16,
    textAlign: 'center',
    lineHeight: 18,
  },
});
