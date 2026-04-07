import { useEffect, useState } from 'react';
import { StatusBar } from 'expo-status-bar';
import {
  StyleSheet,
  View,
  ActivityIndicator,
  TouchableOpacity,
  Text,
  useWindowDimensions,
} from 'react-native';
import {
  SafeAreaProvider,
  useSafeAreaInsets,
} from 'react-native-safe-area-context';
import lkd from './services/lkd';
import AccessTokenDialog from './components/AccessTokenDialog';
import Sidebar from './components/Sidebar';
import InboxView from './views/InboxView';
import TasksView from './views/TasksView';
import SettingsView from './views/SettingsView';
import MarkdownViewer from './views/MarkdownViewer';

function AppContent() {
  const { width } = useWindowDimensions();
  const insets = useSafeAreaInsets();
  const isMobile = width < 600;

  const [loading, setLoading] = useState(true);
  const [showTokenDialog, setShowTokenDialog] = useState(false);
  const [sidebarCollapsed, setSidebarCollapsed] = useState(true);
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
    if (isMobile) setSidebarCollapsed(true);
  };

  const handleFileOpen = (file) => {
    setViewingFile(file);
    if (isMobile) setSidebarCollapsed(true);
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
      return (
        <MarkdownViewer
          uuid={viewingFile.uuid}
          title={viewingFile.label}
          onClose={handleFileClose}
        />
      );
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
          {isMobile ? (
            <>
              <View style={styles.mainContent}>{renderContent()}</View>
              {sidebarCollapsed && (
                <TouchableOpacity
                  style={[
                    styles.hamburger,
                    { top: Math.max(insets.top, 8) + 4 },
                  ]}
                  onPress={() => setSidebarCollapsed(false)}
                >
                  <Text style={styles.hamburgerIcon}>☰</Text>
                </TouchableOpacity>
              )}
              {!sidebarCollapsed && (
                <View style={StyleSheet.absoluteFill}>
                  <Sidebar
                    collapsed={false}
                    onToggleCollapse={() => setSidebarCollapsed(true)}
                    activeView={activeView}
                    onNavigate={handleNavigate}
                    onFileOpen={handleFileOpen}
                    mobile
                  />
                </View>
              )}
            </>
          ) : (
            <>
              <Sidebar
                collapsed={sidebarCollapsed}
                onToggleCollapse={() => setSidebarCollapsed(!sidebarCollapsed)}
                activeView={activeView}
                onNavigate={handleNavigate}
                onFileOpen={handleFileOpen}
              />
              <View style={styles.mainContent}>{renderContent()}</View>
            </>
          )}
        </View>
      )}
      <StatusBar style="auto" />
    </View>
  );
}

export default function App() {
  return (
    <SafeAreaProvider>
      <AppContent />
    </SafeAreaProvider>
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
  hamburger: {
    position: 'absolute',
    left: 12,
    width: 40,
    height: 40,
    borderRadius: 20,
    backgroundColor: '#fff',
    alignItems: 'center',
    justifyContent: 'center',
    elevation: 4,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.2,
    shadowRadius: 4,
    zIndex: 10,
  },
  hamburgerIcon: {
    fontSize: 20,
    color: '#333',
  },
});
