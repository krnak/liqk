import { useEffect, useState } from 'react';
import { StatusBar } from 'expo-status-bar';
import { StyleSheet, View, ActivityIndicator } from 'react-native';
import lkd from './services/lkd';
import AccessTokenDialog from './components/AccessTokenDialog';
import Sidebar from './components/Sidebar';
import InboxView from './views/InboxView';
import TasksView from './views/TasksView';
import SettingsView from './views/SettingsView';
import MarkdownViewer from './views/MarkdownViewer';

export default function App() {
  const [loading, setLoading] = useState(true);
  const [showTokenDialog, setShowTokenDialog] = useState(false);
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);
  const [activeView, setActiveView] = useState('tasks');
  const [viewingFile, setViewingFile] = useState(null);

  useEffect(() => {
    async function initLkd() {
      const hasToken = await lkd.init();
      setShowTokenDialog(!hasToken);
      setLoading(false);
    }
    initLkd();
  }, []);

  const handleTokenSuccess = () => {
    setShowTokenDialog(false);
  };

  const handleTokenCleared = () => {
    setShowTokenDialog(true);
  };

  const handleNavigate = (view) => {
    setViewingFile(null);
    setActiveView(view);
  };

  const handleFileOpen = (filePath) => {
    setViewingFile(filePath);
  };

  const handleFileClose = () => {
    setViewingFile(null);
  };

  if (loading) {
    return (
      <View style={styles.loadingContainer}>
        <ActivityIndicator size="large" color="#007bff" />
        <StatusBar style="auto" />
      </View>
    );
  }

  const renderContent = () => {
    if (viewingFile) {
      return <MarkdownViewer filePath={viewingFile} onClose={handleFileClose} />;
    }

    switch (activeView) {
      case 'inbox':
        return <InboxView />;
      case 'settings':
        return <SettingsView onTokenCleared={handleTokenCleared} />;
      case 'tasks':
      default:
        return <TasksView />;
    }
  };

  return (
    <View style={styles.container}>
      <AccessTokenDialog
        visible={showTokenDialog}
        onSuccess={handleTokenSuccess}
      />
      {!showTokenDialog && (
        <View style={styles.appLayout}>
          <Sidebar
            collapsed={sidebarCollapsed}
            onToggleCollapse={() => setSidebarCollapsed(!sidebarCollapsed)}
            activeView={activeView}
            onNavigate={handleNavigate}
            onFileOpen={handleFileOpen}
          />
          <View style={styles.mainContent}>{renderContent()}</View>
        </View>
      )}
      <StatusBar style="auto" />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#fff',
  },
  loadingContainer: {
    flex: 1,
    backgroundColor: '#fff',
    alignItems: 'center',
    justifyContent: 'center',
  },
  appLayout: {
    flex: 1,
    flexDirection: 'row',
  },
  mainContent: {
    flex: 1,
  },
});
