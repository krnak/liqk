#!/usr/bin/env node

const { S3Client, PutObjectCommand } = require("@aws-sdk/client-s3");
const { readFileSync, statSync } = require("fs");
const { basename } = require("path");
require("dotenv").config({ path: require("path").resolve(__dirname, "../.env") });

async function uploadFile(filePath) {
  const keyId = process.env.BACKBLAZE_KEY_ID;
  const applicationKey = process.env.BACKBLAZE_APPLICATION_KEY;
  const bucket = process.env.BACKBLAZE_BUCKET;
  const endpoint = process.env.BACKBLAZE_ENDPOINT;

  if (!keyId || !applicationKey || !bucket || !endpoint) {
    console.error("Missing required environment variables:");
    if (!keyId) console.error("  - BACKBLAZE_KEY_ID");
    if (!applicationKey) console.error("  - BACKBLAZE_APPLICATION_KEY");
    if (!bucket) console.error("  - BACKBLAZE_BUCKET");
    if (!endpoint) console.error("  - BACKBLAZE_ENDPOINT");
    process.exit(1);
  }

  // Verify file exists
  try {
    statSync(filePath);
  } catch (err) {
    console.error(`File not found: ${filePath}`);
    process.exit(1);
  }

  const client = new S3Client({
    endpoint: endpoint,
    region: new URL(endpoint).hostname.split(".")[1],
    credentials: {
      accessKeyId: keyId,
      secretAccessKey: applicationKey,
    },
  });

  const fileName = basename(filePath);
  const fileContent = readFileSync(filePath);

  const command = new PutObjectCommand({
    Bucket: bucket,
    Key: fileName,
    Body: fileContent,
  });

  try {
    const response = await client.send(command);
    console.log(`Uploaded: ${fileName} -> s3://${bucket}/${fileName}`);
    console.log(`ETag: ${response.ETag}`);
  } catch (err) {
    console.error(`Upload failed: ${err.message}`);
    process.exit(1);
  }
}

const filePath = process.argv[2];

if (!filePath) {
  console.error("Usage: node b2-upload.js <file-path>");
  process.exit(1);
}

uploadFile(filePath);
