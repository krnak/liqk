# Filesystem

The gate service provides file storage with RDF metadata indexing in Oxigraph.

## Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/file/{path}` | Retrieve a file by its logical path |
| GET | `/upload` | HTML upload form |
| POST | `/upload` | Upload files to the `upload` directory |

All endpoints require authentication via access token (header or cookie).

## Storage

Files are stored in the `files/` directory with UUID-based names:
```
files/
  {uuid}.{extension}
  {uuid}.{extension}
  ...
```

Original filenames are preserved in RDF metadata, not on disk.

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

## Path Resolution

When requesting `/file/upload/document.pdf`:

1. Split path into segments: `["upload", "document.pdf"]`
2. Find root directory: `?root rdfs:label "/"`
3. Traverse directories by label via `posix:includes`
4. Find file with matching `rdfs:label`
5. Read `liqk:storedAs` to get disk filename
6. Serve file from `files/{storedAs}`

Returns 404 if path doesn't exist in the graph.

### Example Query

For path `/file/upload/document.pdf`:

```sparql
PREFIX posix: <http://www.w3.org/ns/posix/stat#>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
PREFIX liqk: <http://liqk.org/schema#>

SELECT ?storedAs FROM <http://liqk.org/graph/filesystem> WHERE {
    ?root a posix:Directory ;
          rdfs:label "/" .
    ?root posix:includes ?dir0 .
    ?dir0 rdfs:label "upload" .
    ?dir0 posix:includes ?file .
    ?file rdfs:label "document.pdf" .
    ?file liqk:storedAs ?storedAs .
}
```
