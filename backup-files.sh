#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
FILES_DIR="${SCRIPT_DIR}/files"
OXIGRAPH_URL="http://localhost:7878"
TMP_DIR=$(mktemp -d "${SCRIPT_DIR}/tmp.backup-files.XXXXXX")

trap 'rm -rf "$TMP_DIR"' EXIT

UPLOADED=0
SKIPPED=0
TOTAL=0

for FILE in "${FILES_DIR}"/*; do
    [ -f "$FILE" ] || continue
    TOTAL=$((TOTAL + 1))

    BASENAME=$(basename "$FILE")
    UUID="${BASENAME%.*}"

    echo "Processing ${BASENAME}..."

    # 1. Compute IPFS CID
    CID=$(node scripts/ipfs-cid.js "$FILE")
    OBJ_KEY="${CID}.zip.enc"

    # 2. Check if already in B2
    if node scripts/b2-check.js "$OBJ_KEY"; then
        echo "  Already in cloud: ${OBJ_KEY}"
        SKIPPED=$((SKIPPED + 1))
    else
        # 3. Compress, encrypt, upload
        echo "  Compressing..."
        zip -j "${TMP_DIR}/${CID}.zip" "$FILE"

        echo "  Encrypting..."
        liqk-crypto encrypt \
            --pk pk.pem \
            --input "${TMP_DIR}/${CID}.zip" \
            --output "${TMP_DIR}/${OBJ_KEY}"

        echo "  Uploading ${OBJ_KEY}..."
        node scripts/b2-upload.js "${TMP_DIR}/${OBJ_KEY}"

        rm -f "${TMP_DIR}/${CID}.zip" "${TMP_DIR}/${OBJ_KEY}"
        UPLOADED=$((UPLOADED + 1))
    fi

    # 4. Record CID in RDF
    curl -s -f -X POST "${OXIGRAPH_URL}/update" \
        -H "Content-Type: application/sparql-update" \
        --data-raw "PREFIX dc: <http://purl.org/dc/terms/>
INSERT DATA {
    GRAPH <http://liqk.org/graph/filesystem> {
        <urn:uuid:${UUID}> dc:identifier \"ipfs://${CID}\" .
    }
}"

    echo "  CID recorded: ipfs://${CID}"
done

echo ""
echo "Done. Total: ${TOTAL}, Uploaded: ${UPLOADED}, Skipped: ${SKIPPED}"
