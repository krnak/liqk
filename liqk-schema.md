# liqk Schema Identifiers

> **Namespace URI:** `http://liqk.org/schema#`
>
> **Prefix:** `liqk:`

## Classes

- `liqk:Project`
- `liqk:Task`
- `liqk:ModifyAction`
- `liqk:AccessPolicy`
- `liqk:AccessToken`

## Actions

- `liqk:action-upload-file`

## Predicates

- `liqk:priority`
- `liqk:project`
- `liqk:readme`
- `liqk:status`
- `liqk:task-status`
- `liqk:title`
- `liqk:modified-property`
- `liqk:old-value`
- `liqk:new-value`
- `liqk:abbrv`
- `liqk:rank`

## Priority values

- `liqk:priority-highest`
- `liqk:priority-high`
- `liqk:priority-medium`
- `liqk:priority-pinned`
- `liqk:priority-none`
- `liqk:priority-low`

## Priority ranks (`liqk:rank`)

| Value | Rank |
|-------|------|
| `liqk:priority-highest` | 6 |
| `liqk:priority-high` | 5 |
| `liqk:priority-medium` | 4 |
| `liqk:priority-pinned` | 3 |
| `liqk:priority-none` | 2 |
| `liqk:priority-low` | 1 |

## Project status values

- `liqk:project-status-completed`
- `liqk:project-status-focus`
- `liqk:project-status-inactive`
- `liqk:project-status-life-long`
- `liqk:project-status-peripheral`

## Task status values

- `liqk:task-status-done`
- `liqk:task-status-hall-of-fame`
- `liqk:task-status-not-started`
- `liqk:task-status-trashed`

---

## ModifyAction

Tracks modifications to Task properties, providing an audit trail of changes.

### External namespace

| Prefix | URI |
|--------|-----|
| `dcterms:` | `http://purl.org/dc/terms/` |

### Properties

| Property | Type | Description |
|----------|------|-------------|
| Subject URI | UUID | Each ModifyAction is identified by a UUID (e.g., `urn:uuid:550e8400-e29b-41d4-a716-446655440000`) |
| `rdf:type` | IRI | Always `liqk:ModifyAction` |
| `liqk:modified-property` | IRI | The predicate that was changed (e.g., `liqk:priority`) |
| `liqk:old-value` | any | The previous value before modification |
| `liqk:new-value` | any | The new value after modification |
| `dcterms:created` | `xsd:integer` | Unix timestamp (seconds since 1970-01-01 00:00:00 UTC) |

### Example (Turtle)

```turtle
@prefix liqk: <http://liqk.org/schema#> .
@prefix dcterms: <http://purl.org/dc/terms/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

<urn:uuid:550e8400-e29b-41d4-a716-446655440000>
    a liqk:ModifyAction ;
    liqk:modified-property liqk:priority ;
    liqk:old-value liqk:priority-low ;
    liqk:new-value liqk:priority-high ;
    dcterms:created "1735689600"^^xsd:integer .
```

---

## abbrv

Provides one or more abbreviation labels for any resource. Useful for short identifiers, aliases, or quick-reference names.

### Properties

| Property | Type | Cardinality | Description |
|----------|------|-------------|-------------|
| `liqk:abbrv` | `xsd:string` | 0..* | Short abbreviation string for the resource |

### Example (Turtle)

```turtle
@prefix liqk: <http://liqk.org/schema#> .

<urn:uuid:a1b2c3d4-e5f6-7890-abcd-ef1234567890>
    a liqk:Project ;
    liqk:title "Artificial General Intelligence Research" ;
    liqk:abbrv "AGI" ;
    liqk:abbrv "agi-research" ;
    liqk:abbrv "ar" .
```

---

## Access Vocabulary

**Graph URI:** `http://liqk.org/graph/access`

### Classes

- `liqk:AccessPolicy`
- `liqk:AccessToken`

### Predicates

| Predicate | Domain | Range | Description |
|-----------|--------|-------|-------------|
| `liqk:access-level` | `AccessPolicy` | IRI | Permission level granted |
| `liqk:policy-type` | `AccessPolicy` | IRI | How access is granted |
| `liqk:policy-target` | `AccessPolicy` | IRI | Resource this policy protects |
| `liqk:policy-grantee` | `AccessPolicy` | IRI | Entity granted access (e.g., AccessToken) |
| `liqk:token-hash` | `AccessToken` | `xsd:string` | SHA-256 hash of the token |

### Access level values (`liqk:rank`)

| Value | Rank |
|-------|------|
| `liqk:access-level-admin` | 4 |
| `liqk:access-level-edit` | 3 |
| `liqk:access-level-comment` | 2 |
| `liqk:access-level-view` | 1 |
| `liqk:access-level-none` | 0 |

### Policy type values

- `liqk:policy-type-public`
- `liqk:policy-type-token`

### Action resources

These IRIs represent actions that can be protected by access policies:

| Resource | Description |
|----------|-------------|
| `liqk:action-upload-file` | Permission to upload new files via `POST /res` |

---

## Gate Access Control

The Oxigraph Gate uses the following resources as policy targets:

| Endpoint | Resource | Required Rank |
|----------|----------|---------------|
| `/`, `/query` | `<http://liqk.org/graph>` | 1 (view) |
| `/update` | `<http://liqk.org/graph>` | 3 (edit) |
| `POST /res` | `liqk:action-upload-file` | 3 (edit) |
| `GET /res/{uuid}` | `<urn:uuid:{uuid}>` | 1 (view) |
| `PUT /res/{uuid}` | `<urn:uuid:{uuid}>` | 3 (edit) |

### Example: Grant edit access to the graph

```turtle
@prefix liqk: <http://liqk.org/schema#> .
@prefix dcterms: <http://purl.org/dc/terms/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

# Token with edit access to the graph
<urn:uuid:token-admin>
    a liqk:AccessToken ;
    liqk:token-hash "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08" ;
    dcterms:created "1736784000"^^xsd:integer .

<urn:uuid:policy-graph-edit>
    a liqk:AccessPolicy ;
    liqk:policy-target <http://liqk.org/graph> ;
    liqk:policy-type liqk:policy-type-token ;
    liqk:access-level liqk:access-level-edit ;
    liqk:policy-grantee <urn:uuid:token-admin> ;
    dcterms:created "1736784000"^^xsd:integer .
```

### Example: Grant upload permission

```turtle
@prefix liqk: <http://liqk.org/schema#> .
@prefix dcterms: <http://purl.org/dc/terms/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

<urn:uuid:policy-upload>
    a liqk:AccessPolicy ;
    liqk:policy-target liqk:action-upload-file ;
    liqk:policy-type liqk:policy-type-token ;
    liqk:access-level liqk:access-level-edit ;
    liqk:policy-grantee <urn:uuid:token-admin> ;
    dcterms:created "1736784000"^^xsd:integer .
```

---

### AccessPolicy

| Property | Type | Cardinality | Description |
|----------|------|-------------|-------------|
| Subject URI | UUID | 1 | `urn:uuid:...` |
| `rdf:type` | IRI | 1 | `liqk:AccessPolicy` |
| `liqk:policy-target` | IRI | 1 | Resource being protected |
| `liqk:policy-type` | IRI | 1 | `liqk:policy-type-public` or `liqk:policy-type-token` |
| `liqk:access-level` | IRI | 1 | Level of access granted |
| `liqk:policy-grantee` | IRI | 0..1 | Required for `liqk:policy-type-token`, links to `AccessToken` |
| `dcterms:created` | `xsd:integer` | 1 | Unix timestamp |

### AccessToken

| Property | Type | Cardinality | Description |
|----------|------|-------------|-------------|
| Subject URI | UUID | 1 | `urn:uuid:...` |
| `rdf:type` | IRI | 1 | `liqk:AccessToken` |
| `liqk:token-hash` | `xsd:string` | 1 | SHA-256 hash of the plaintext token |
| `dcterms:created` | `xsd:integer` | 1 | Unix timestamp |

### Example (Turtle)

```turtle
@prefix liqk: <http://liqk.org/schema#> .
@prefix dcterms: <http://purl.org/dc/terms/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

# Public view access to a project
<urn:uuid:policy-001>
    a liqk:AccessPolicy ;
    liqk:policy-target <urn:uuid:project-123> ;
    liqk:policy-type liqk:policy-type-public ;
    liqk:access-level liqk:access-level-view ;
    dcterms:created "1736784000"^^xsd:integer .

# Token for edit access
<urn:uuid:token-001>
    a liqk:AccessToken ;
    liqk:token-hash "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08" ;
    dcterms:created "1736784000"^^xsd:integer .

# Token-based edit access policy
<urn:uuid:policy-002>
    a liqk:AccessPolicy ;
    liqk:policy-target <urn:uuid:project-123> ;
    liqk:policy-type liqk:policy-type-token ;
    liqk:access-level liqk:access-level-edit ;
    liqk:policy-grantee <urn:uuid:token-001> ;
    dcterms:created "1736784000"^^xsd:integer .
```

### Access Queries

#### Public Access Query (IRI-based)

Returns the maximum access rank for a resource via public policies. Returns 0 if no policy matches.

**Arguments:** `$resource` (IRI)

```sparql
PREFIX liqk: <http://liqk.org/schema#>

SELECT (COALESCE(MAX(?rank), 0) AS ?accessRank)
FROM <http://liqk.org/graph/access>
WHERE {
  ?policy a liqk:AccessPolicy ;
          liqk:policy-type liqk:policy-type-public ;
          liqk:policy-target $resource ;
          liqk:access-level ?level .

  ?level liqk:rank ?rank .
}
```

#### Token Access Query (IRI-based)

Returns the maximum access rank for a resource via token-based policies. Returns 0 if no policy matches.

**Arguments:** `$resource` (IRI), `$tokenHash` (string)

```sparql
PREFIX liqk: <http://liqk.org/schema#>

SELECT (COALESCE(MAX(?rank), 0) AS ?accessRank)
FROM <http://liqk.org/graph/access>
WHERE {
  ?policy a liqk:AccessPolicy ;
          liqk:policy-type liqk:policy-type-token ;
          liqk:policy-target $resource ;
          liqk:access-level ?level ;
          liqk:policy-grantee ?token .

  ?token a liqk:AccessToken ;
         liqk:token-hash $tokenHash .

  ?level liqk:rank ?rank .
}
```

#### Public Access Query (UUID with inheritance)

Returns the maximum access rank for a UUID resource via public policies, supporting inheritance via `posix:includes*`. Returns 0 if no policy matches.

**Arguments:** `$resource` (UUID as `urn:uuid:...`)

```sparql
PREFIX liqk: <http://liqk.org/schema#>
PREFIX posix: <http://www.w3.org/ns/posix/stat#>

SELECT (COALESCE(MAX(?rank), 0) AS ?accessRank)
FROM <http://liqk.org/graph/access>
FROM <http://liqk.org/graph/filesystem>
WHERE {
  ?policy a liqk:AccessPolicy ;
          liqk:policy-type liqk:policy-type-public ;
          liqk:policy-target ?target ;
          liqk:access-level ?level .

  ?level liqk:rank ?rank .
  ?target posix:includes* $resource .
}
```

#### Token Access Query (UUID with inheritance)

Returns the maximum access rank for a UUID resource via token-based policies, supporting inheritance via `posix:includes*`. Returns 0 if no policy matches.

**Arguments:** `$resource` (UUID as `urn:uuid:...`), `$tokenHash` (string)

```sparql
PREFIX liqk: <http://liqk.org/schema#>
PREFIX posix: <http://www.w3.org/ns/posix/stat#>

SELECT (COALESCE(MAX(?rank), 0) AS ?accessRank)
FROM <http://liqk.org/graph/access>
FROM <http://liqk.org/graph/filesystem>
WHERE {
  ?policy a liqk:AccessPolicy ;
          liqk:policy-type liqk:policy-type-token ;
          liqk:policy-target ?target ;
          liqk:access-level ?level ;
          liqk:policy-grantee ?token .

  ?token a liqk:AccessToken ;
         liqk:token-hash $tokenHash .

  ?level liqk:rank ?rank .
  ?target posix:includes* $resource .
}
```
