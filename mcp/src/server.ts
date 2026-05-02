import express, { type Request as ExpressRequest, type Response as ExpressResponse } from "express";
import { randomUUID } from "node:crypto";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StreamableHTTPServerTransport } from "@modelcontextprotocol/sdk/server/streamableHttp.js";
import { z } from "zod";

const GATE_URL = (process.env.GATE_URL ?? "http://127.0.0.1:8080").replace(/\/$/, "");
const BIND_ADDR = process.env.BIND_ADDR ?? "127.0.0.1:8090";

const __dirname = dirname(fileURLToPath(import.meta.url));
const DOCS_DIR = process.env.DOCS_DIR ?? resolve(__dirname, "..", "docs");

function loadDoc(name: string): string {
  try {
    return readFileSync(resolve(DOCS_DIR, name), "utf8");
  } catch (e) {
    console.warn(`could not load ${name} from ${DOCS_DIR}: ${(e as Error).message}`);
    return `(${name} unavailable on this server)`;
  }
}

const SCHEMA_DOC = loadDoc("liqk-schema.md");
const FILESYSTEM_DOC = loadDoc("filesystem.md");

const UUID_RE = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

const WRITE_KEYWORDS = /\b(?:INSERT|DELETE|LOAD|CLEAR|DROP|CREATE|ADD|MOVE|COPY)\b/i;

function stripCommentsAndStrings(query: string): string {
  let out = "";
  let i = 0;
  const n = query.length;
  while (i < n) {
    const c = query[i];
    if (c === "#") {
      while (i < n && query[i] !== "\n") i++;
      continue;
    }
    if (c === '"' || c === "'") {
      const triple = query.slice(i, i + 3) === c.repeat(3);
      const quote = triple ? c.repeat(3) : c;
      i += quote.length;
      while (i < n) {
        if (query[i] === "\\" && i + 1 < n) {
          i += 2;
          continue;
        }
        if (query.slice(i, i + quote.length) === quote) {
          i += quote.length;
          break;
        }
        i++;
      }
      continue;
    }
    out += c;
    i++;
  }
  return out;
}

function isTextualContentType(ct: string): boolean {
  const lower = ct.toLowerCase();
  return (
    lower.startsWith("text/") ||
    lower.includes("application/json") ||
    lower.includes("application/xml") ||
    lower.includes("application/sparql-results+json") ||
    lower.includes("application/n-triples") ||
    lower.includes("application/n-quads") ||
    lower.includes("application/trig") ||
    lower.includes("application/ld+json") ||
    lower.includes("+json") ||
    lower.includes("+xml")
  );
}

type ToolResult = {
  content: Array<
    | { type: "text"; text: string }
    | { type: "resource"; resource: { uri: string; mimeType: string; blob: string } }
  >;
  isError?: boolean;
};

function errorResult(message: string): ToolResult {
  return { isError: true, content: [{ type: "text", text: message }] };
}

function buildServer(bearer: string): McpServer {
  const server = new McpServer({ name: "liqk-mcp", version: "0.1.0" });

  server.tool(
    "read_file",
    "Fetch a stored file by UUID. See resource `liqk://filesystem` for how to locate UUIDs.",
    {
      uuid: z.string().regex(UUID_RE, "must be a valid UUID").describe("Bare file UUID, no extension."),
    },
    async ({ uuid }): Promise<ToolResult> => {
      const url = `${GATE_URL}/res/${encodeURIComponent(uuid)}`;
      let resp: Response;
      try {
        resp = await fetch(url, { headers: { Authorization: bearer } });
      } catch (e) {
        return errorResult(`gate request failed: ${(e as Error).message}`);
      }
      const ct = resp.headers.get("content-type") ?? "application/octet-stream";
      if (!resp.ok) {
        const body = await resp.text().catch(() => "");
        return errorResult(`gate ${resp.status}: ${body.slice(0, 500)}`);
      }
      if (isTextualContentType(ct)) {
        const text = await resp.text();
        return { content: [{ type: "text", text }] };
      }
      const buf = Buffer.from(await resp.arrayBuffer());
      return {
        content: [
          {
            type: "resource",
            resource: {
              uri: `liqk://res/${uuid}`,
              mimeType: ct.split(";")[0].trim(),
              blob: buf.toString("base64"),
            },
          },
        ],
      };
    },
  );

  server.tool(
    "sparql_query",
    "Read-only SPARQL (SELECT/ASK/CONSTRUCT/DESCRIBE) against liqk's Oxigraph. Read resources `liqk://schema` and `liqk://filesystem` for vocab and graph URIs before writing non-trivial queries.",
    {
      query: z.string().min(1).max(20_000).describe("SPARQL 1.1 query. Default graph is empty — use FROM or GRAPH."),
    },
    async ({ query }): Promise<ToolResult> => {
      const stripped = stripCommentsAndStrings(query);
      if (WRITE_KEYWORDS.test(stripped)) {
        return errorResult(
          "sparql_query is read-only — rejected because the query contains an UPDATE keyword. Use SELECT/ASK/CONSTRUCT/DESCRIBE only.",
        );
      }
      let resp: Response;
      try {
        resp = await fetch(`${GATE_URL}/query`, {
          method: "POST",
          headers: {
            "Content-Type": "application/sparql-query",
            Accept: "application/sparql-results+json, application/json",
            Authorization: bearer,
          },
          body: query,
        });
      } catch (e) {
        return errorResult(`gate request failed: ${(e as Error).message}`);
      }
      const text = await resp.text();
      if (!resp.ok) {
        return errorResult(`gate ${resp.status}: ${text.slice(0, 1000)}`);
      }
      return { content: [{ type: "text", text }] };
    },
  );

  server.tool(
    "sparql_update",
    "Execute a SPARQL 1.1 UPDATE (INSERT/DELETE/LOAD/CLEAR/etc.) against liqk's Oxigraph. Requires edit access (rank >= 3) on `<http://liqk.org/graph>` for the bearer token. Read `liqk://schema` and `liqk://filesystem` before writing non-trivial updates.",
    {
      update: z.string().min(1).max(50_000).describe("SPARQL 1.1 UPDATE. Use GRAPH <...> { ... } to scope writes to the right named graph."),
    },
    async ({ update }): Promise<ToolResult> => {
      let resp: Response;
      try {
        resp = await fetch(`${GATE_URL}/update`, {
          method: "POST",
          headers: {
            "Content-Type": "application/sparql-update",
            Authorization: bearer,
          },
          body: update,
        });
      } catch (e) {
        return errorResult(`gate request failed: ${(e as Error).message}`);
      }
      const text = await resp.text();
      if (!resp.ok) {
        return errorResult(`gate ${resp.status}: ${text.slice(0, 1000)}`);
      }
      return { content: [{ type: "text", text: text || "ok" }] };
    },
  );

  server.tool(
    "uuid_v4",
    "Generate a fresh random UUID (v4). Useful when minting subject URIs for new resources before writing SPARQL.",
    {},
    async (): Promise<ToolResult> => ({
      content: [{ type: "text", text: randomUUID() }],
    }),
  );

  server.resource(
    "liqk-schema",
    "liqk://schema",
    {
      title: "liqk ontology",
      description:
        "Full reference for the liqk: vocabulary — classes, predicates, status enums, priority ranks, ModifyAction shape, and Turtle examples. Read this before writing non-trivial SPARQL against the kairos graph.",
      mimeType: "text/markdown",
    },
    async () => ({
      contents: [{ uri: "liqk://schema", mimeType: "text/markdown", text: SCHEMA_DOC }],
    }),
  );

  server.resource(
    "liqk-filesystem",
    "liqk://filesystem",
    {
      title: "liqk filesystem graph",
      description:
        "Reference for the filesystem named graph: posix:File / posix:Directory layout, predicates (rdfs:label, posix:size, dc:format, dc:created, liqk:storedAs), and how `/res/{uuid}` resolves files. Read this before navigating files or directories.",
      mimeType: "text/markdown",
    },
    async () => ({
      contents: [{ uri: "liqk://filesystem", mimeType: "text/markdown", text: FILESYSTEM_DOC }],
    }),
  );

  return server;
}

const app = express();
app.use(express.json({ limit: "1mb" }));

app.post("/", async (req: ExpressRequest, res: ExpressResponse) => {
  const header = req.headers.authorization;
  const queryToken = typeof req.query.token === "string" ? req.query.token : undefined;
  const auth =
    header && /^Bearer\s+\S+/i.test(header)
      ? header
      : queryToken
        ? `Bearer ${queryToken}`
        : undefined;
  if (!auth) {
    res.status(401).json({
      jsonrpc: "2.0",
      error: {
        code: -32001,
        message: "missing token: pass Authorization: Bearer <token> or ?token=<token>",
      },
      id: null,
    });
    return;
  }
  const transport = new StreamableHTTPServerTransport({ sessionIdGenerator: undefined });
  res.on("close", () => {
    transport.close();
  });
  const server = buildServer(auth);
  try {
    await server.connect(transport);
    await transport.handleRequest(req, res, req.body);
  } catch (e) {
    if (!res.headersSent) {
      res.status(500).json({
        jsonrpc: "2.0",
        error: { code: -32603, message: `internal: ${(e as Error).message}` },
        id: null,
      });
    }
  }
});

app.get("/", (_req: ExpressRequest, res: ExpressResponse) => {
  res.status(405).json({
    jsonrpc: "2.0",
    error: { code: -32000, message: "GET not supported; use POST for stateless MCP" },
    id: null,
  });
});

app.delete("/", (_req: ExpressRequest, res: ExpressResponse) => {
  res.status(405).json({
    jsonrpc: "2.0",
    error: { code: -32000, message: "session termination not supported in stateless mode" },
    id: null,
  });
});

const [host, portStr] = BIND_ADDR.split(":");
const port = Number(portStr ?? "8090");
app.listen(port, host, () => {
  console.log(`liqk-mcp listening on ${host}:${port}, gate=${GATE_URL}`);
});
