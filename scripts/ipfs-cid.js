#!/usr/bin/env node

const { createHash } = require("crypto");
const { createReadStream } = require("fs");

const filePath = process.argv[2];
if (!filePath) {
  console.error("Usage: node ipfs-cid.js <file-path>");
  process.exit(1);
}

async function main() {
  const { CID } = await import("multiformats/cid");
  const { create: createDigest } = await import("multiformats/hashes/digest");
  const { base32 } = await import("multiformats/bases/base32");

  const hash = await new Promise((resolve, reject) => {
    const hasher = createHash("sha256");
    const stream = createReadStream(filePath);
    stream.on("data", (chunk) => hasher.update(chunk));
    stream.on("end", () => resolve(hasher.digest()));
    stream.on("error", reject);
  });

  // sha2-256 multihash code = 0x12, raw codec = 0x55
  const digest = createDigest(0x12, hash);
  const cid = CID.createV1(0x55, digest);
  process.stdout.write(cid.toString(base32));
}

main().catch((err) => {
  console.error(err.message);
  process.exit(1);
});
