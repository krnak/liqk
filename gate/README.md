# Oxigraph Gate

An HTTP proxy service for [oxigraph-cli](https://github.com/oxigraph/oxigraph) with RDF-based access control.

## Features

- RDF-based access control via policies in `http://liqk.org/graph/access`
- Token authentication via SHA-256 hash lookup
- Browser login page for session cookie setup
- Cookie-based session persistence
- Configurable upstream oxigraph URL
- RDF-indexed file storage with upload/download endpoints

## Installation

### From Source

Requires Rust 1.70+.

```bash
cargo build --release
```

The binary will be at `./target/release/oxigraph-gate`.

## Configuration

Configuration is stored in a `.env` file in the working directory.

| Variable | Description | Default |
|----------|-------------|---------|
| `OXIGRAPH_URL` | Upstream oxigraph URL | `http://localhost:7878` |
| `SECURE_COOKIES` | Set cookie Secure flag (requires HTTPS) | `true` |
| `FILES_DIR` | Directory for file storage | `../files` |

If no valid token exists on startup, a new one is generated and saved to `.env`.

### Example `.env`

**Production (behind HTTPS proxy):**
```
OXIGRAPH_URL=http://localhost:7878
FILES_DIR=/var/lib/liqk/files
```

**Development (local HTTP only):**
```
OXIGRAPH_URL=http://localhost:7878
SECURE_COOKIES=false
FILES_DIR=../files
```

## Usage

### Start the Proxy

```bash
# Start oxigraph first
oxigraph serve --location ./data

# Start the gate proxy
./target/release/oxigraph-gate
```

The proxy listens on `0.0.0.0:8080`. The access token is logged at startup.

### Authentication Methods

#### HTTP Header

Use `X-Access-Token` header:

```bash
curl -H "X-Access-Token: YOUR_TOKEN" http://localhost:8080/query?query=SELECT%20*%20WHERE%20{?s%20?p%20?o}
```

Or `Authorization: Bearer` header:

```bash
curl -H "Authorization: Bearer YOUR_TOKEN" http://localhost:8080/query?query=SELECT%20*%20WHERE%20{?s%20?p%20?o}
```

#### Browser Session (Cookie Authentication)

Navigate to `http://localhost:8080/gate/login` in a browser to enter your token. After successful authentication, a session cookie is set and all subsequent requests from that browser session are automatically authenticated.

The session cookie:
- Stores the token for RDF-based access control lookup
- Expires after 3 months
- Is not sent with cross-origin requests (for security)

### Proxied Endpoints

All oxigraph endpoints are proxied:

| Endpoint | Description |
|----------|-------------|
| `/query` | SPARQL query (GET/POST) |
| `/update` | SPARQL update (POST) |
| `/store` | Graph Store Protocol |
| `/` | YASGUI interface |

### File Storage

The gate includes an RDF-indexed file storage system. Files are stored on disk and indexed in Oxigraph using the `http://liqk.org/graph/filesystem` graph.

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/res` | POST | Upload new files (multipart/form-data) |
| `/res/{uuid}` | GET | Download file by UUID |
| `/res/{uuid}` | PUT | Replace file content (keeping same UUID) |

#### Upload Files

```bash
# Upload via curl
curl -X POST -H "X-Access-Token: YOUR_TOKEN" \
  -F "files=@document.pdf" \
  http://localhost:8080/res

# Response (JSON)
{"success":true,"files":[{"filename":"document.pdf","uuid":"550e8400-e29b-41d4-a716-446655440000"}]}
```

#### Download by UUID

```bash
curl -H "X-Access-Token: YOUR_TOKEN" \
  http://localhost:8080/res/550e8400-e29b-41d4-a716-446655440000 \
  -o document.pdf
```

#### Replace File Content

```bash
curl -X PUT -H "X-Access-Token: YOUR_TOKEN" \
  --data-binary @updated_document.pdf \
  http://localhost:8080/res/550e8400-e29b-41d4-a716-446655440000
```

The file content is replaced while keeping the same UUID. The file size is updated in the RDF metadata.

#### Storage Details

- Files are stored in the directory specified by `FILES_DIR` (default: `../files/`)
- Each file is renamed to `{uuid}.{extension}` on disk
- Metadata (original name, size, MIME type, timestamp) is stored in Oxigraph
- Maximum upload size: 4 GB

### Access Control

Access is controlled via RDF-based policies stored in the `http://liqk.org/graph/access` graph. See [liqk-schema.md](../liqk-schema.md) for the full access vocabulary.

| Endpoint | Resource | Required Rank |
|----------|----------|---------------|
| `/`, `/query` | `<http://liqk.org/graph>` | 1 (view) |
| `/update` | `<http://liqk.org/graph>` | 3 (edit) |
| `POST /res` | `<http://liqk.org/schema#action-upload-file>` | 3 (edit) |
| `GET /res/{uuid}` | `<urn:uuid:{uuid}>` | 1 (view) |
| `PUT /res/{uuid}` | `<urn:uuid:{uuid}>` | 3 (edit) |

Access ranks:
- **4** (admin): Full administrative access
- **3** (edit): Can modify data
- **2** (comment): Can add comments
- **1** (view): Read-only access
- **0** (none): No access

Tokens are authenticated via SHA-256 hash comparison against stored `liqk:AccessToken` resources.

## Logging

The service logs to stdout with structured logging. Set `RUST_LOG` environment variable to control log level.

### Startup Output

```
INFO  ========================================
INFO          Oxigraph Gate Starting
INFO  ========================================
INFO  Listen URL:    http://0.0.0.0:8080
INFO  Oxigraph URL:  http://localhost:7878
INFO  ========================================
```

### Logged Events

| Event | Level | Fields |
|-------|-------|--------|
| Successful login | INFO | `client` |
| Failed login | WARN | `client` |
| Proxied request | INFO | `client`, `method`, `path`, `status`, `bytes` |
| Unauthorized request | WARN | `client`, `method`, `path` |
| Proxy error | WARN | `client`, `method`, `path`, `error` |

### Example Log Output

```
INFO  client=127.0.0.1:52431 Login successful
INFO  client=127.0.0.1:52431 method=GET path=/query?query=SELECT... status=200 OK bytes=1234 Request proxied
WARN  client=192.168.1.5:41022 Login failed - invalid token
```

### Log Level Control

```bash
# Default (info level)
./oxigraph-gate

# Debug level
RUST_LOG=debug ./oxigraph-gate

# Only warnings and errors
RUST_LOG=warn ./oxigraph-gate
```

## Architecture

```
Client Request
      │
      ▼
┌─────────────┐
│ Oxigraph    │ :8080
│ Gate        │
└─────┬───────┘
      │ Access rank sufficient?
      │
      ├─── No ──► 403 Forbidden
      │
      ▼ Yes
┌─────────────┐
│ Oxigraph    │ :7878
│ Server      │
└─────────────┘

/gate/login ──► Browser session setup (manual navigation)
```

## Security Notes

### Cookie Security
Session cookies have the following security attributes:
- **HttpOnly**: Prevents JavaScript access (XSS protection)
- **SameSite=Strict**: Prevents cross-site request forgery (CSRF)
- **Secure**: Only sent over HTTPS (when `SECURE_COOKIES=true`)
- **Max-Age**: Sessions expire after 3 months

### CORS Policy
- Cross-origin requests are allowed for SPARQL client compatibility
- Credentials (cookies) are NOT sent with cross-origin requests
- Cross-origin clients must authenticate via `X-Access-Token` or `Authorization: Bearer` headers
- Same-origin browser requests use cookie authentication normally

### Production Deployment
- Always deploy behind an HTTPS reverse proxy (nginx, Caddy, etc.)
- Keep `SECURE_COOKIES=true` (default) in production
- Only set `SECURE_COOKIES=false` for local development without HTTPS

## License

MIT
