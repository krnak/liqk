#!/usr/bin/env python3
"""Generate cryptographically random access tokens."""

import hashlib
import os
import secrets
from datetime import datetime

def main():
    if os.geteuid() != 0:
        print("Error: must run as root (sudo) — tokens.txt is root-only")
        return 1

    label = input("Label: ").strip()
    if not label:
        print("Error: label cannot be empty")
        return 1

    token = secrets.token_hex(16)
    token_hash = hashlib.sha256(token.encode()).hexdigest()
    timestamp = datetime.now().isoformat()

    with open("tokens.txt", "a") as f:
        f.write(f"{timestamp} {label} {token} {token_hash}\n")
    os.chown("tokens.txt", 0, 0)
    os.chmod("tokens.txt", 0o600)

    print(f"Token: {token}")
    print(f"Hash:  {token_hash}")
    return 0

if __name__ == "__main__":
    exit(main())
