# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Liqk is a personal knowledge management system built on RDF/SPARQL. It consists of three main components: a Rust HTTP gateway proxy, an Oxigraph triplestore database, and a React Native (Expo) mobile/web app.

## Architecture

```
app/ (React Native/Expo)  ──►  gate/ (Rust/Axum, :8080)  ──►  Oxigraph (:7878)
                                     ▲             │
                                     │             ▼
mcp/ (TS, :8090) ────────────────────┘         files/ (UUID-named files on disk)
   ▲
   └── Claude mobile (remote MCP connector)
```

- **Oxigraph** — RDF triplestore storing all data (tasks, projects, filesystem metadata, access policies)
- **gate/** (`oxigraph-gate`) — Rust/Axum HTTP proxy on port 8080 that adds RDF-based access control, token authentication, cookie sessions, and file storage endpoints on top of Oxigraph
- **app/** — React Native (Expo) cross-platform client for task management; communicates with gate via SPARQL queries/updates and file REST endpoints
- **mcp/** — TypeScript MCP sidecar on port 8090 (nginx route `/mcp`). Exposes read-only `read_file` and `sparql_query` tools to the Claude mobile app. Accepts the gate token via `Authorization: Bearer …` header or `?token=…` query string (the latter is needed because the claude.ai custom-connector UI is OAuth-only — no bearer field). Forwards the token to the gate so existing access ranks apply.
- **oxidata/** — Oxigraph's on-disk database directory (RocksDB SST files)
- **files/** — UUID-named file storage managed by the gate's filesystem module
- **liqk-crypto/** — Standalone Rust CLI for file encryption (ChaCha20Poly1305 + X-Wing KEM), used in backup pipeline

## Key RDF Graphs

| Graph URI | Purpose |
|-----------|---------|
| `http://liqk.org/graph/kairos` | Tasks, projects, priorities, ModifyActions |
| `http://liqk.org/graph/filesystem` | File/directory metadata (POSIX-style tree) |
| `http://liqk.org/graph/access` | Access policies and token hashes |

The custom ontology namespace is `http://liqk.org/schema#` (prefix `liqk:`). See `liqk-schema.md` for the full vocabulary and `filesystem.md` for the filesystem graph structure.

## Running the Stack

Prerequisites: `pacman -S zip`, `cargo install oxigraph-cli`

```bash
# Terminal 1: Start Oxigraph database
oxigraph serve --location ./oxidata

# Terminal 2: Start the gate proxy (from repo root)
cd gate && cargo run          # dev mode
# or: ./gate/target/release/oxigraph-gate   # release binary

# Terminal 3: Start the app
cd app && npm start           # then press w/a/i for web/android/ios
```

Gate configuration is in `gate/.env` (`OXIGRAPH_URL`, `SECURE_COOKIES`, `FILES_DIR`). Set `SECURE_COOKIES=false` for local dev without HTTPS.

## Build Commands

```bash
# Gate (Rust)
cd gate && cargo build --release
cd gate && cargo build            # debug

# Crypto tool (Rust)
cd liqk-crypto && cargo build --release

# App (Expo/React Native)
cd app && npm install
cd app && npm run web             # web dev server
cd app && npm run android         # Android via Expo Go
cd app && npm run ios             # iOS via Expo Go
```

## Useful Scripts

| Script | Purpose |
|--------|---------|
| `python gen-token.py` | Generate access token, append to `tokens.txt` |
| `./backup.sh` | Dump Oxigraph → canonize → zip → encrypt → upload to B2 |
| `python list_liqk_fs.py` | Print filesystem tree from Oxigraph (queries :7878 directly) |
| `node scripts/rdf-canonize-cli.js` | Canonize N-Quads/N-Triples (URDNA2015) |
| `node scripts/b2-upload.js` | Upload file to Backblaze B2 |

## Gate Source Layout (gate/src/)

| File | Responsibility |
|------|---------------|
| `main.rs` | Axum router setup, CORS config, app state |
| `auth.rs` | Token extraction (header/cookie), login page, session cookies |
| `proxy.rs` | Reverse proxy to Oxigraph with access rank checking |
| `files.rs` | File upload/download/replace, RDF metadata indexing, SHA-256 token hashing, SPARQL access queries |
| `templates.rs` | HTML templates for login page |

## App Source Layout (app/)

| File | Purpose |
|------|---------|
| `services/lkd.js` | LKD service — SPARQL query/update helpers, task CRUD, file operations. Singleton `lkd` instance |
| `App.js` | Root component, routing between views, token dialog |
| `views/TasksView.js` | Task list with priority filtering |
| `views/InboxView.js` | Inbox/activity feed (ModifyActions) |
| `views/SettingsView.js` | Token and base URL configuration |
| `views/MarkdownViewer.js` | Markdown file viewer (fetches via `/res/{uuid}`) |
| `components/Sidebar.js` | Navigation sidebar with filesystem browser |

## Authentication Flow

Tokens are 16-byte hex strings. The gate hashes them with SHA-256 and looks up `liqk:AccessToken` resources in the access graph. Authentication methods: `X-Access-Token` header, `Authorization: Bearer` header, or `oxigraph_gate_token` session cookie. Access policies map tokens to resources with ranked permission levels (0=none through 4=admin).

## Deployment

The production server is `liqk.kairos.to`. Nginx reverse-proxies port 80 to gate on :8080. Deploy commands are in `server.md` (scp binaries, .env, and nginx config to the server).
