#!/usr/bin/env python3
"""Generate cryptographically random access tokens."""

import hashlib
import secrets
from datetime import datetime

def main():
    label = input("Label: ").strip()
    if not label:
        print("Error: label cannot be empty")
        return 1

    token = secrets.token_hex(16)
    token_hash = hashlib.sha256(token.encode()).hexdigest()
    timestamp = datetime.now().isoformat()

    with open("tokens.txt", "a") as f:
        f.write(f"{timestamp} {label} {token} {token_hash}\n")

    print(f"Token: {token}")
    print(f"Hash:  {token_hash}")
    return 0

if __name__ == "__main__":
    exit(main())
