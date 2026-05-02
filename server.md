scp oxigraph-gate liqk:/root/gate
scp gate/.env liqk/root/gate
scp /home/agi/.cargo/bin/oxigraph liqk:/root
scp nginx.conf liqk:/etc/nginx/sites-enabled/liqk.kairos.to.conf

# MCP sidecar
cd mcp && npm install && npm run build && cd ..
ssh liqk 'mkdir -p /root/mcp'
scp -r mcp/dist mcp/package.json mcp/package-lock.json liqk:/root/mcp/
ssh liqk 'cd /root/mcp && npm install --omit=dev'
# run: cd /root/mcp && GATE_URL=http://127.0.0.1:8080 BIND_ADDR=127.0.0.1:8090 node dist/server.js
# (wire as a systemd unit for production)
#
# Auth: the sidecar accepts the gate token in either
#   Authorization: Bearer <token>           (preferred)
#   ?token=<token>                          (used by claude.ai connectors — UI is OAuth-only)
# Logs to /var/log/liqk-mcp.log. Token-in-URL gets written to nginx access logs;
# rotate via gen-token.py if it leaks.
