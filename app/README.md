# Liqk App

React Native (Expo) cross-platform application for task management and knowledge database.

## Prerequisites

- Node.js 18+
- npm or yarn
- For Android builds: Expo account and EAS CLI

## Installation

```bash
cd app
npm install
```

## Running in Development

### Web Browser

```bash
npm run web
```

### Android (with Expo Go)

```bash
npm run android
```

Scan the QR code with the Expo Go app on your Android device.

### iOS (with Expo Go)

```bash
npm run ios
```

Scan the QR code with the Camera app on your iOS device.

### General Development Server

```bash
npm start
```

Press `w` for web, `a` for Android, or `i` for iOS.

## Building for Android

### Install EAS CLI

```bash
npm install -g eas-cli
eas login
```

### Build APK (for local installation)

```bash
eas build --platform android --profile preview
```

This creates an APK for internal distribution. Download from the Expo dashboard when complete.

### Build AAB (for Google Play Store)

```bash
eas build --platform android --profile production
```

This creates an Android App Bundle for Play Store submission.

### Local Build (without Expo servers)

Requires Android SDK installed locally:

```bash
eas build --platform android --profile preview --local
```

## Build Profiles

Configured in `eas.json`:

| Profile | Use Case |
|---------|----------|
| `development` | Development client with debugging |
| `preview` | Internal testing (APK) |
| `production` | Play Store release (AAB) |

## Backend Connection

The app connects to the Gate proxy at `http://localhost:8080`. For production, update the endpoint in `services/lkd.js`.

Ensure the full stack is running:

```bash
# Terminal 1: Oxigraph database
oxigraph serve --location ./oxidata

# Terminal 2: Gate proxy (from repo root)
./gate/target/release/oxigraph-gate

# Terminal 3: App
cd app && npm start
```
