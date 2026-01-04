#!/bin/bash

set -e

DATE=$(date +%Y-%m-%d)
BACKUP_DIR="backups"
BACKUP_FILE="${BACKUP_DIR}/${DATE}-backup.nq"

mkdir -p "${BACKUP_DIR}"

echo "Dumping oxigraph database to ${BACKUP_FILE}..."
oxigraph dump --location oxidata --file "${BACKUP_FILE}"

echo "Canonizing ${BACKUP_FILE}..."
node scripts/rdf-canonize-cli.js "${BACKUP_FILE}" -o "${BACKUP_FILE}"

echo "Compressing ${BACKUP_FILE}..."
zip -j "${BACKUP_FILE}.zip" "${BACKUP_FILE}"
rm "${BACKUP_FILE}"

echo "Encrypting ${BACKUP_FILE}.zip with liqk-crypto..."
./liqk-crypto/target/release/liqk-crypto encrypt --pk pk.pem --input "${BACKUP_FILE}.zip" --output "${BACKUP_FILE}.zip.enc"

echo "Uploading encrypted backup to cloud..."
node scripts/b2-upload.js "${BACKUP_FILE}.zip.enc"

echo "Done. Backup encrypted and uploaded to cloud."
