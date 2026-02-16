# Filesystem

The gate service provides file storage with RDF metadata indexing in Oxigraph.

## Endpoints

| Method | Path | Description |
|--------|------|-------------|
| POST | `/res` | Upload new files (multipart/form-data) |
| GET | `/res/{uuid}` | Download file by UUID |
| PUT | `/res/{uuid}` | Replace file content (keeping same UUID) |

All endpoints require authentication via access token (header or cookie).

| Endpoint | Resource | Required Rank |
|----------|----------|---------------|
| `POST /res` | `<http://liqk.org/schema#action-upload-file>` | 3 (edit) |
| `GET /res/{uuid}` | `<urn:uuid:{uuid}>` | 1 (view) |
| `PUT /res/{uuid}` | `<urn:uuid:{uuid}>` | 3 (edit) |

## Storage

Files are stored in the directory specified by `FILES_DIR` (default: `../files/`) with UUID-based names:
```
files/
  {uuid}.{extension}
  {uuid}.{extension}
  ...
```

Original filenames are preserved in RDF metadata, not on disk. Maximum upload size: 4 GB.

## RDF Graph

All metadata is stored in the named graph `<http://liqk.org/graph/filesystem>`.

### Ontologies

| Prefix | URI |
|--------|-----|
| posix | `http://www.w3.org/ns/posix/stat#` |
| rdfs | `http://www.w3.org/2000/01/rdf-schema#` |
| dc | `http://purl.org/dc/terms/` |
| liqk | `http://liqk.org/schema#` |

### Directory Structure

Directories use UUID URNs as identifiers with labels for names:

```turtle
<urn:uuid:...> a posix:Directory ;
    rdfs:label "/" .                    # root directory

<urn:uuid:...> a posix:Directory ;
    rdfs:label "upload" .               # upload directory

<urn:uuid:root> posix:includes <urn:uuid:upload> .
```

The root directory has `rdfs:label "/"`. Child directories are linked via `posix:includes`.

### File Records

When a file is uploaded, the following triples are created:

```turtle
<urn:uuid:{file-uuid}> a posix:File ;
    rdfs:label "original-filename.ext" ;
    posix:size 12345 ;
    dc:format "application/pdf" ;
    dc:created "2024-01-15T10:30:00Z"^^xsd:dateTime ;
    liqk:storedAs "{file-uuid}.ext" .

<urn:uuid:upload-dir> posix:includes <urn:uuid:{file-uuid}> .
```

| Predicate | Description |
|-----------|-------------|
| `rdfs:label` | Original filename |
| `posix:size` | File size in bytes |
| `dc:format` | MIME type |
| `dc:created` | Upload timestamp (ISO 8601) |
| `liqk:storedAs` | Actual filename on disk |

## UUID Resolution

When requesting `/res/{uuid}`:

1. Look up `liqk:storedAs` for `<urn:uuid:{uuid}>`
2. Serve file from `files/{storedAs}`

Returns 404 if UUID doesn't exist in the graph.

### Example Query

```sparql
PREFIX liqk: <http://liqk.org/schema#>

SELECT ?storedAs FROM <http://liqk.org/graph/filesystem> WHERE {
    <urn:uuid:550e8400-e29b-41d4-a716-446655440000> liqk:storedAs ?storedAs .
}
```
