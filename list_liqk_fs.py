#!/usr/bin/env python3
import requests
import json

SPARQL_ENDPOINT = "http://localhost:7878"
GRAPH = "<http://liqk.org/graph/filesystem>"

def sparql_query(query):
    resp = requests.post(
        f"{SPARQL_ENDPOINT}/query",
        headers={
            "Content-Type": "application/sparql-query",
            "Accept": "application/json"
        },
        data=query
    )
    return resp.json() if resp.status_code == 200 else None

# Get all directories and files with their parent relationships
query = f"""
PREFIX posix: <http://www.w3.org/ns/posix/stat#>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

SELECT ?item ?label ?type ?parent ?parentLabel
FROM {GRAPH}
WHERE {{
    ?item rdfs:label ?label .
    {{
        ?item a posix:Directory .
        BIND("dir" AS ?type)
    }} UNION {{
        ?item a posix:File .
        BIND("file" AS ?type)
    }}
    OPTIONAL {{
        ?parent posix:includes ?item .
        ?parent rdfs:label ?parentLabel .
    }}
}}
ORDER BY ?parentLabel ?type ?label
"""

result = sparql_query(query)

if not result:
    print("Failed to query database")
    exit(1)

# Build tree structure
nodes = {}  # uri -> {label, type, children}
root_nodes = []

for binding in result['results']['bindings']:
    uri = binding['item']['value']
    label = binding['label']['value']
    node_type = binding['type']['value']
    parent_uri = binding.get('parent', {}).get('value')
    
    if uri not in nodes:
        nodes[uri] = {'label': label, 'type': node_type, 'children': [], 'uri': uri}
    
    if parent_uri:
        if parent_uri not in nodes:
            parent_label = binding.get('parentLabel', {}).get('value', '?')
            nodes[parent_uri] = {'label': parent_label, 'type': 'dir', 'children': [], 'uri': parent_uri}
        if uri not in [c['uri'] for c in nodes[parent_uri]['children']]:
            nodes[parent_uri]['children'].append(nodes[uri])
    else:
        if nodes[uri] not in root_nodes:
            root_nodes.append(nodes[uri])

def get_uuid(uri):
    return uri.replace('urn:uuid:', '')

def print_tree(node, prefix="", is_last=True):
    connector = "└── " if is_last else "├── "
    uuid_short = get_uuid(node['uri'])[:8]
    
    if node['type'] == 'dir':
        print(f"{prefix}{connector}📁 {node['label']} ({uuid_short}...)")
    else:
        print(f"{prefix}    {connector}📄 {node['label'][:50]} ({uuid_short}...)")
    
    # Sort children: directories first, then files
    children = sorted(node['children'], key=lambda x: (0 if x['type'] == 'dir' else 1, x['label']))
    
    for i, child in enumerate(children):
        is_last_child = (i == len(children) - 1)
        new_prefix = prefix + ("    " if is_last else "│   ")
        print_tree(child, new_prefix, is_last_child)

print("=== LIQK Filesystem ===\n")

# Find and print from root
for node in root_nodes:
    if node['label'] == '/':
        # Sort root's children
        node['children'] = sorted(node['children'], key=lambda x: (0 if x['type'] == 'dir' else 1, x['label']))
        print(f"📁 / (root: {get_uuid(node['uri'])[:8]}...)")
        for i, child in enumerate(node['children']):
            print_tree(child, "", i == len(node['children']) - 1)
        break

# Count totals
dirs = sum(1 for n in nodes.values() if n['type'] == 'dir')
files = sum(1 for n in nodes.values() if n['type'] == 'file')
print(f"\n=== Total: {dirs} directories, {files} files ===")
