#!/usr/bin/env node

const { S3Client, HeadObjectCommand } = require("@aws-sdk/client-s3");
require("dotenv").config({ path: require("path").resolve(__dirname, "../.env") });

const key = process.argv[2];
if (!key) {
  console.error("Usage: node b2-check.js <object-key>");
  process.exit(2);
}

async function main() {
  const keyId = process.env.BACKBLAZE_KEY_ID;
  const applicationKey = process.env.BACKBLAZE_APPLICATION_KEY;
  const bucket = process.env.BACKBLAZE_BUCKET;
  const endpoint = process.env.BACKBLAZE_ENDPOINT;

  if (!keyId || !applicationKey || !bucket || !endpoint) {
    console.error("Missing BACKBLAZE_* environment variables");
    process.exit(2);
  }

  const client = new S3Client({
    endpoint: endpoint,
    region: new URL(endpoint).hostname.split(".")[1],
    credentials: {
      accessKeyId: keyId,
      secretAccessKey: applicationKey,
    },
  });

  try {
    await client.send(new HeadObjectCommand({ Bucket: bucket, Key: key }));
    process.exit(0);
  } catch (err) {
    if (err.name === "NotFound" || err.$metadata?.httpStatusCode === 404) {
      process.exit(1);
    }
    console.error(`B2 check failed: ${err.message}`);
    process.exit(2);
  }
}

main();
