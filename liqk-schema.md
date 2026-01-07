# liqk Schema Identifiers

> **Namespace URI:** `http://liqk.org/schema#`
>
> **Prefix:** `liqk:`

## Classes

- `liqk:Project`
- `liqk:Task`
- `liqk:ModifyAction`

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

## Priority values

- `liqk:priority-highest`
- `liqk:priority-high`
- `liqk:priority-medium`
- `liqk:priority-pinned`
- `liqk:priority-none`
- `liqk:priority-low`

## Priority ranks (`liqk:priority-rank`)

- `liqk:priority-highest` → 6
- `liqk:priority-high` → 5
- `liqk:priority-medium` → 4
- `liqk:priority-pinned` → 3
- `liqk:priority-none` → 2
- `liqk:priority-low` → 1

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
