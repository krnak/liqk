# Oxigraph Gate

An HTTP proxy service for [oxigraph-cli](https://github.com/oxigraph/oxigraph) that adds access token authentication.

## Features

- Token-based authentication for all oxigraph endpoints
- Automatic token generation with 128-bit cryptographic entropy
- Browser login page for manual token entry
- Cookie-based session persistence
- Configurable upstream oxigraph URL

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
| `ACCESS_TOKEN` | 32-character hex string | Auto-generated |
| `OXIGRAPH_URL` | Upstream oxigraph URL | `http://localhost:7878` |

If no valid token exists on startup, a new one is generated and saved to `.env`.

### Example `.env`

```
ACCESS_TOKEN=a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6
OXIGRAPH_URL=http://localhost:7878
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

#### Browser Session

Navigate to `http://localhost:8080` in a browser. You will be redirected to the login page where you can enter the token. After successful authentication, a session cookie is set.

### Proxied Endpoints

All oxigraph endpoints are proxied:

| Endpoint | Description |
|----------|-------------|
| `/query` | SPARQL query (GET/POST) |
| `/update` | SPARQL update (POST) |
| `/store` | Graph Store Protocol |
| `/` | YASGUI interface |

## Logging

The service logs to stdout with structured logging. Set `RUST_LOG` environment variable to control log level.

### Startup Output

```
INFO  ========================================
INFO          Oxigraph Gate Starting
INFO  ========================================
INFO  Listen URL:    http://0.0.0.0:8080
INFO  Oxigraph URL:  http://localhost:7878
INFO  Access Token:  a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6
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
      │ Token validated?
      │
      ├─── No ──► /gate/login (HTML form)
      │
      ▼ Yes
┌─────────────┐
│ Oxigraph    │ :7878
│ Server      │
└─────────────┘
```

## Security Notes

- The `.env` file contains the access token and should not be committed to version control
- Tokens are validated using constant-time comparison
- Session cookies are HTTP-only to prevent XSS attacks
- For production use, deploy behind HTTPS reverse proxy

## License

MIT
