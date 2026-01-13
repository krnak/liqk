# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Liqk is a multi-platform task management and knowledge database application with:
- **React Native frontend** (Expo) - cross-platform web/iOS/Android app
- **Rust HTTP proxy gateway** (gate) - authentication layer for Oxigraph
- **Post-quantum crypto tool** (liqk-crypto) - file encryption CLI
- **RDF triple-store** (Oxigraph) - semantic data storage

## Common Commands

### Frontend (app/)
```bash
cd app
npm install
npm start          # Start Expo dev server
npm run web        # Run in browser
npm run android    # Run on Android
npm run ios        # Run on iOS
```

### Gateway (gate/)
```bash
cd gate
cargo build --release
RUST_LOG=debug ./target/release/oxigraph-gate    # Run with debug logging
cargo test
```

### Crypto Tool (liqk-crypto/)
```bash
cd liqk-crypto
cargo build --release
cargo test                                        # Run all tests
./target/release/liqk-crypto keygen --sk sk.pem --pk pk.pem
./target/release/liqk-crypto encrypt --pk pk.pem --input file.txt --output file.bin
./target/release/liqk-crypto decrypt --sk sk.pem --input file.bin --output file.txt
```

### Full Stack Development
```bash
# Terminal 1: Start Oxigraph database
oxigraph serve --location ./oxidata

# Terminal 2: Start the gateway (from repo root)
./gate/target/release/oxigraph-gate

# Terminal 3: Start the app
cd app && npm start
```

## Architecture

### Data Flow
```
React Native App ──► Gate Proxy (8080) ──► Oxigraph (7878)
                         │
                         ▼
                    files/ directory
```

### RDF Graphs
- `http://liqk.org/graph/filesystem` - File metadata (POSIX ontology)
- `http://liqk.org/graph/kairos` - Task/project data

### Authentication
- Token-based: 32-char hex string in `gate/.env`
- Methods: `X-Access-Token` header, `Authorization: Bearer`, or browser cookies
- Token validation uses constant-time comparison

### LKD Service (app/services/lkd.js)
Client-side API wrapper handling:
- Token storage via AsyncStorage
- SPARQL query execution
- File upload/download
- Directory browsing at `/file/{path}`

## Schema (liqk-schema.md)

Namespace: `http://liqk.org/schema#` (prefix: `liqk:`)

**Classes**: Project, Task, ModifyAction (audit trail)

**Key predicates**: priority, project, readme, status, title, task-status, abbrv

**Priority values** (ranked 6→1): priority-highest, priority-high, priority-medium, priority-pinned, priority-none, priority-low

**Project statuses**: project-status-completed, project-status-focus, project-status-inactive, project-status-life-long, project-status-peripheral

**Task statuses**: task-status-done, task-status-hall-of-fame, task-status-not-started, task-status-trashed

## File Storage

Files stored in `files/` as `{uuid}.{extension}`. Metadata (original name, size, MIME type) stored in RDF, not on disk.

**Endpoints**:
- `GET /file/{path}` - Browse/download by path
- `GET /res/{uuid}` - Download by UUID
- `PUT /res/{uuid}` - Replace file content
- `POST /upload` - Upload files

## Configuration

### gate/.env
```
ACCESS_TOKEN=<32-char-hex>
OXIGRAPH_URL=http://localhost:7878
SECURE_COOKIES=true      # false for local dev without HTTPS
FILES_DIR=../files       # directory for file storage
```

### Root .env (Backblaze backup)
```
BACKBLAZE_KEY_ID=...
BACKBLAZE_APPLICATION_KEY=...
BACKBLAZE_BUCKET=liqk00
BACKBLAZE_ENDPOINT=https://s3.eu-central-003.backblazeb2.com
```

## Security Notes

- SPARQL injection prevention implemented in gate
- Cookie security: HttpOnly, SameSite=Strict, Secure flag
- liqk-crypto uses X-Wing KEM (post-quantum hybrid: ML-KEM 768 + X25519) + ChaCha20Poly1305
