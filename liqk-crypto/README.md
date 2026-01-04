# liqk-crypto

A command-line tool for file encryption using ChaCha20Poly1305 and X-Wing KEM (post-quantum hybrid key encapsulation).

## Features

- **Post-quantum secure**: Uses X-Wing KEM (ML-KEM 768 + X25519 hybrid) for key encapsulation
- **Authenticated encryption**: ChaCha20Poly1305 AEAD for symmetric encryption
- **PEM key format**: Human-readable key files
- **Simple CLI**: Three commands for key generation, encryption, and decryption

## Installation

```bash
cargo install --path .
```

Or build from source:

```bash
cargo build --release
```

## Usage

### Generate a key pair

```bash
liqk-crypto keygen --sk secret.pem --pk public.pem
```

### Generate a key pair with manual seed

For deterministic key generation, use the `--seed` flag to enter 32 bytes of randomness as a hexadecimal string (64 characters):

```bash
liqk-crypto keygen --sk secret.pem --pk public.pem --seed
Enter seed (64 hex characters): 0102030405060708091011121314151617181920212223242526272829303132
```

The same seed will always produce the same key pair.

### Encrypt a file

```bash
liqk-crypto encrypt --pk public.pem --input plaintext.txt --output encrypted.bin
```

### Decrypt a file

```bash
liqk-crypto decrypt --sk secret.pem --input encrypted.bin --output decrypted.txt
```

## Cryptographic Details

| Component | Algorithm |
|-----------|-----------|
| KEM | X-Wing KEM Draft 06 (ML-KEM 768 + X25519) |
| AEAD | ChaCha20Poly1305 |
| KDF | HKDF-SHA256 |

### Key Sizes

- Secret key: 2464 bytes (PEM encoded)
- Public key: 1216 bytes (PEM encoded)
- KEM ciphertext: 1120 bytes

### Encrypted File Format

```
┌─────────────┬──────────────────┬─────────────────────┐
│ Nonce (12B) │ KEM CT (1120B)   │ AEAD Ciphertext     │
└─────────────┴──────────────────┴─────────────────────┘
```

- **Nonce**: 12 random bytes for ChaCha20Poly1305
- **KEM Ciphertext**: X-Wing encapsulated key (ML-KEM 768 ciphertext + X25519 public key)
- **AEAD Ciphertext**: ChaCha20Poly1305 encrypted data with 16-byte auth tag

### Key Derivation

The shared secret from X-Wing KEM is processed through HKDF-SHA256:

```
symmetric_key = HKDF-Expand(
    HKDF-Extract(salt=None, ikm=shared_secret),
    info="liqk-crypto-chacha20poly1305",
    length=32
)
```

## Building

```bash
cargo build
```

## Testing

```bash
cargo test
```

Tests include:
- Basic encryption/decryption roundtrip
- Empty file handling
- Large file (1MB) roundtrip
- Wrong key rejection
- Tampered ciphertext detection

## Dependencies

- `libcrux-kem` - X-Wing KEM implementation
- `chacha20poly1305` - AEAD cipher
- `hkdf` / `sha2` - Key derivation
- `clap` - CLI parsing
- `pem` - PEM encoding/decoding
- `rand` - Cryptographic randomness
- `hex` - Hexadecimal encoding/decoding

## License

MIT
