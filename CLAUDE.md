# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Liqk is a task management and knowledge database application with:
- **React Native (Expo) frontend** - cross-platform app (`app/`)
- **Rust HTTP proxy gateway** - authentication and file storage (`gate/`)
- **Post-quantum crypto utility** - file encryption tool (`liqk-crypto/`)
- **Oxigraph** - RDF triple store database (external dependency)

All data is modeled as RDF triples using SPARQL queries. Access control is enforced via RDF-based policies.

## Build & Run Commands

### Prerequisites
```bash
pacman -S zip
cargo install oxigraph-cli
```

### Full Stack Development
```bash
# Terminal 1: Oxigraph database
oxigraph serve --location ./oxidata

# Terminal 2: Gate proxy
cd gate && cargo build --release
./target/release/oxigraph-gate

# Terminal 3: App
cd app && npm install && npm run web
```

### Frontend (app/)
```bash
npm install              # Install dependencies
npm start                # Expo dev server (press w/a/i for platform)
npm run web              # Web browser
npm run android          # Android via Expo Go
npm run ios              # iOS via Expo Go

# Production builds
eas build --platform android --profile preview     # APK
eas build --platform android --profile production  # AAB for Play Store
```

### Gateway (gate/)
```bash
cargo build --release    # Compile
cargo test               # Run tests
RUST_LOG=debug ./target/release/oxigraph-gate  # Run with debug logging
```

### Crypto Tool (liqk-crypto/)
```bash
cargo build --release
cargo test

# CLI usage
./target/release/liqk-crypto keygen --sk sk.pem --pk pk.pem
./target/release/liqk-crypto encrypt --pk pk.pem --input file.txt --output file.bin
./target/release/liqk-crypto decrypt --sk sk.pem --input file.bin --output file.txt
```

## Architecture

```
React Native App (:web)
       │
       │ X-Access-Token header
       ▼
Oxigraph Gate (:8080)     ──► File Storage (files/)
       │
       │ SPARQL proxy
       ▼
Oxigraph Server (:7878)
       │
       ▼
RDF Database (oxidata/)
```

### RDF Graphs
- `http://liqk.org/graph/kairos` - Tasks and projects
- `http://liqk.org/graph/filesystem` - File metadata (POSIX ontology)
- `http://liqk.org/graph/access` - Access control policies

### Namespace
- **Prefix:** `liqk:`
- **URI:** `http://liqk.org/schema#`

### Key Classes
- `liqk:Task` - Work items with priority, status, project links
- `liqk:Project` - Task groupings with status and readme
- `liqk:AccessPolicy` - Permission grants (public or token-based)
- `liqk:AccessToken` - Auth tokens (SHA-256 hashed)
- `liqk:ModifyAction` - Audit trail for property changes

### Access Control
Tokens are validated via SHA-256 hash lookup against `liqk:AccessToken` resources. Access ranks:
- 4 (admin), 3 (edit), 2 (comment), 1 (view), 0 (none)

## Key Files

### Frontend
- `app/App.js` - Root component with routing
- `app/services/lkd.js` - Backend API wrapper (SPARQL queries, file ops)
- `app/views/TasksView.js` - Main task/project UI
- `app/components/Sidebar.js` - Navigation and file browser

### Gateway
- `gate/src/main.rs` - Server setup and Axum router
- `gate/src/files.rs` - File upload/download with RDF indexing
- `gate/src/auth.rs` - Token validation and cookie handling
- `gate/src/proxy.rs` - Oxigraph request forwarding

### Configuration
- `gate/.env` - ACCESS_TOKEN, OXIGRAPH_URL, SECURE_COOKIES, FILES_DIR
- `.env` (root) - Shared config

## Documentation

- `liqk-schema.md` - RDF ontology and access control vocabulary
- `filesystem.md` - File storage architecture and POSIX ontology
- `gate/README.md` - Gateway API, authentication, and security details
- `app/README.md` - Expo development and build profiles
