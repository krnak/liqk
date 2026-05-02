# liqk-mcp

Remote MCP server that exposes read-only liqk tools to the Claude mobile app.

It sits behind nginx at `https://liqk.kairos.to/mcp` and forwards the caller's
token to the gate verbatim. The sidecar holds no credentials of its own — gate
access ranks are the only gate.

Two token sources are accepted (header is preferred; query string exists because
the claude.ai connector UI offers no header field):

- `Authorization: Bearer <token>` header
- `?token=<token>` query string

## Tools

- `read_file(uuid)` → `GET /res/{uuid}` on the gate.
- `sparql_query(query)` → `POST /query` with `Content-Type: application/sparql-query`.
  Rejects writes (`INSERT|DELETE|LOAD|CLEAR|DROP|CREATE|ADD|MOVE|COPY`) client-side.

## Develop

```bash
npm install
cp .env.example .env       # edit GATE_URL / BIND_ADDR if needed
npm run dev
```

Smoke-test locally against the production gate:

```bash
GATE_URL=https://liqk.kairos.to npm run dev
# then in another shell:
TOKEN=...   # a rank-≥1 token
curl -s -X POST http://127.0.0.1:8090/ \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -H "Accept: application/json, text/event-stream" \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"smoke","version":"0"}}}'
```

## Build & run

```bash
npm run build
npm start
```

## Connector setup

In claude.ai → Customize → Add custom connector:

- Name: anything (e.g. `liqk`)
- Remote MCP server URL: `https://liqk.kairos.to/mcp?token=<your-rank-≥1-token>`
- Leave OAuth Client ID / Secret empty

The connector syncs to the Claude Android/iOS app automatically.

Note: putting the token in the URL writes it to nginx access logs. The token is
treated as a personal credential — rotate via `python gen-token.py` if it leaks.
