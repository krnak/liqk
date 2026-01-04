use anyhow::{anyhow, Context, Result};
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};
use clap::{Parser, Subcommand};
use hkdf::Hkdf;
use libcrux_kem::{Algorithm, Ct, PrivateKey, PublicKey, Ss};
use pem::Pem;
use rand::RngCore;
use sha2::Sha256;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

const XWING_SK_LABEL: &str = "XWING SECRET KEY";
const XWING_PK_LABEL: &str = "XWING PUBLIC KEY";
const NONCE_SIZE: usize = 12;
const HKDF_INFO: &[u8] = b"liqk-crypto-chacha20poly1305";

// X-Wing KEM ciphertext size: ML-KEM 768 (1088 bytes) + X25519 (32 bytes)
const XWING_CT_SIZE: usize = 1120;
const XWING_SEED_SIZE: usize = 32;

#[derive(Parser)]
#[command(name = "liqk-crypto")]
#[command(about = "File encryption using ChaCha20Poly1305 and X-Wing KEM")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a new key pair
    Keygen {
        /// Path to write the secret key
        #[arg(long)]
        sk: PathBuf,
        /// Path to write the public key
        #[arg(long)]
        pk: PathBuf,
        /// Prompt for manual seed input (64 hex characters = 32 bytes)
        #[arg(long)]
        seed: bool,
    },
    /// Encrypt a file to a public key
    Encrypt {
        /// Path to the public key
        #[arg(long)]
        pk: PathBuf,
        /// Path to the input file
        #[arg(long)]
        input: PathBuf,
        /// Path to the output encrypted file
        #[arg(long)]
        output: PathBuf,
    },
    /// Decrypt a file with a secret key
    Decrypt {
        /// Path to the secret key
        #[arg(long)]
        sk: PathBuf,
        /// Path to the input encrypted file
        #[arg(long)]
        input: PathBuf,
        /// Path to the output decrypted file
        #[arg(long)]
        output: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Keygen { sk, pk, seed } => keygen(&sk, &pk, seed),
        Commands::Encrypt { pk, input, output } => encrypt(&pk, &input, &output),
        Commands::Decrypt { sk, input, output } => decrypt(&sk, &input, &output),
    }
}

fn keygen(sk_path: &PathBuf, pk_path: &PathBuf, manual_seed: bool) -> Result<()> {
    let (secret_key, public_key) = if manual_seed {
        let seed = read_seed_from_terminal()?;
        libcrux_kem::key_gen_derand(Algorithm::XWingKemDraft06, &seed)
            .map_err(|e| anyhow!("Key generation failed: {:?}", e))?
    } else {
        let mut rng = rand::rng();
        libcrux_kem::key_gen(Algorithm::XWingKemDraft06, &mut rng)
            .map_err(|e| anyhow!("Key generation failed: {:?}", e))?
    };

    let sk_pem = Pem::new(XWING_SK_LABEL, secret_key.encode());
    let pk_pem = Pem::new(XWING_PK_LABEL, public_key.encode());

    fs::write(sk_path, pem::encode(&sk_pem)).context("Failed to write secret key")?;
    fs::write(pk_path, pem::encode(&pk_pem)).context("Failed to write public key")?;

    println!("Key pair generated successfully");
    println!("  Secret key: {}", sk_path.display());
    println!("  Public key: {}", pk_path.display());

    Ok(())
}

fn read_seed_from_terminal() -> Result<[u8; XWING_SEED_SIZE]> {
    print!("Enter seed (64 hex characters): ");
    io::stdout().flush().context("Failed to flush stdout")?;

    let mut input = String::new();
    io::stdin()
        .lock()
        .read_line(&mut input)
        .context("Failed to read from stdin")?;

    let input = input.trim();

    if input.len() != XWING_SEED_SIZE * 2 {
        return Err(anyhow!(
            "Invalid seed length: expected {} hex characters, got {}",
            XWING_SEED_SIZE * 2,
            input.len()
        ));
    }

    let bytes = hex::decode(input).context("Invalid hexadecimal string")?;

    bytes
        .try_into()
        .map_err(|_| anyhow!("Failed to convert seed to fixed-size array"))
}

fn encrypt(pk_path: &PathBuf, input_path: &PathBuf, output_path: &PathBuf) -> Result<()> {
    let pk_pem_str = fs::read_to_string(pk_path).context("Failed to read public key")?;
    let pk_pem = pem::parse(&pk_pem_str).context("Failed to parse public key PEM")?;

    if pk_pem.tag() != XWING_PK_LABEL {
        return Err(anyhow!(
            "Invalid public key PEM label: expected '{}', got '{}'",
            XWING_PK_LABEL,
            pk_pem.tag()
        ));
    }

    let public_key = PublicKey::decode(Algorithm::XWingKemDraft06, pk_pem.contents())
        .map_err(|e| anyhow!("Failed to decode public key: {:?}", e))?;

    let plaintext = fs::read(input_path).context("Failed to read input file")?;

    let mut rng = rand::rng();

    let (shared_secret, ciphertext_kem) = public_key
        .encapsulate(&mut rng)
        .map_err(|e| anyhow!("Encapsulation failed: {:?}", e))?;

    let symmetric_key = derive_key(&shared_secret)?;

    let mut nonce_bytes = [0u8; NONCE_SIZE];
    rng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let cipher = ChaCha20Poly1305::new_from_slice(&symmetric_key)
        .map_err(|e| anyhow!("Failed to create cipher: {:?}", e))?;

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| anyhow!("Encryption failed: {:?}", e))?;

    // Output format: nonce || kem_ciphertext || symmetric_ciphertext
    let kem_ct_bytes = ciphertext_kem.encode();
    let mut output = Vec::with_capacity(NONCE_SIZE + kem_ct_bytes.len() + ciphertext.len());
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&kem_ct_bytes);
    output.extend_from_slice(&ciphertext);

    fs::write(output_path, &output).context("Failed to write encrypted file")?;

    println!("File encrypted successfully");
    println!("  Input: {}", input_path.display());
    println!("  Output: {}", output_path.display());

    Ok(())
}

fn decrypt(sk_path: &PathBuf, input_path: &PathBuf, output_path: &PathBuf) -> Result<()> {
    let sk_pem_str = fs::read_to_string(sk_path).context("Failed to read secret key")?;
    let sk_pem = pem::parse(&sk_pem_str).context("Failed to parse secret key PEM")?;

    if sk_pem.tag() != XWING_SK_LABEL {
        return Err(anyhow!(
            "Invalid secret key PEM label: expected '{}', got '{}'",
            XWING_SK_LABEL,
            sk_pem.tag()
        ));
    }

    let secret_key = PrivateKey::decode(Algorithm::XWingKemDraft06, sk_pem.contents())
        .map_err(|e| anyhow!("Failed to decode secret key: {:?}", e))?;

    let encrypted = fs::read(input_path).context("Failed to read encrypted file")?;

    let min_size = NONCE_SIZE + XWING_CT_SIZE;

    if encrypted.len() < min_size {
        return Err(anyhow!(
            "Encrypted file too small: expected at least {} bytes, got {}",
            min_size,
            encrypted.len()
        ));
    }

    let nonce_bytes = &encrypted[..NONCE_SIZE];
    let kem_ct_bytes = &encrypted[NONCE_SIZE..NONCE_SIZE + XWING_CT_SIZE];
    let ciphertext = &encrypted[NONCE_SIZE + XWING_CT_SIZE..];

    let kem_ciphertext = Ct::decode(Algorithm::XWingKemDraft06, kem_ct_bytes)
        .map_err(|e| anyhow!("Failed to decode KEM ciphertext: {:?}", e))?;

    let shared_secret = kem_ciphertext
        .decapsulate(&secret_key)
        .map_err(|e| anyhow!("Decapsulation failed: {:?}", e))?;

    let symmetric_key = derive_key(&shared_secret)?;

    let nonce = Nonce::from_slice(nonce_bytes);
    let cipher = ChaCha20Poly1305::new_from_slice(&symmetric_key)
        .map_err(|e| anyhow!("Failed to create cipher: {:?}", e))?;

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| anyhow!("Decryption failed: {:?}", e))?;

    fs::write(output_path, &plaintext).context("Failed to write decrypted file")?;

    println!("File decrypted successfully");
    println!("  Input: {}", input_path.display());
    println!("  Output: {}", output_path.display());

    Ok(())
}

fn derive_key(shared_secret: &Ss) -> Result<[u8; 32]> {
    let ss_bytes: Vec<u8> = shared_secret.encode();
    let hkdf = Hkdf::<Sha256>::new(None, &ss_bytes);
    let mut key = [0u8; 32];
    hkdf.expand(HKDF_INFO, &mut key)
        .map_err(|e| anyhow!("HKDF expand failed: {:?}", e))?;
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_roundtrip() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let sk_path = temp_dir.path().join("secret.pem");
        let pk_path = temp_dir.path().join("public.pem");
        let input_path = temp_dir.path().join("input.txt");
        let encrypted_path = temp_dir.path().join("encrypted.bin");
        let decrypted_path = temp_dir.path().join("decrypted.txt");

        // Generate keys
        keygen(&sk_path, &pk_path, false)?;

        // Create test input
        let original_content = b"Hello, World! This is a test message for encryption.";
        fs::write(&input_path, original_content)?;

        // Encrypt
        encrypt(&pk_path, &input_path, &encrypted_path)?;

        // Verify encrypted file is different from original
        let encrypted_content = fs::read(&encrypted_path)?;
        assert_ne!(encrypted_content.as_slice(), original_content);

        // Decrypt
        decrypt(&sk_path, &encrypted_path, &decrypted_path)?;

        // Verify roundtrip
        let decrypted_content = fs::read(&decrypted_path)?;
        assert_eq!(decrypted_content, original_content);

        Ok(())
    }

    #[test]
    fn test_roundtrip_empty_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let sk_path = temp_dir.path().join("secret.pem");
        let pk_path = temp_dir.path().join("public.pem");
        let input_path = temp_dir.path().join("empty.txt");
        let encrypted_path = temp_dir.path().join("encrypted.bin");
        let decrypted_path = temp_dir.path().join("decrypted.txt");

        keygen(&sk_path, &pk_path, false)?;

        let original_content = b"";
        fs::write(&input_path, original_content)?;

        encrypt(&pk_path, &input_path, &encrypted_path)?;
        decrypt(&sk_path, &encrypted_path, &decrypted_path)?;

        let decrypted_content = fs::read(&decrypted_path)?;
        assert_eq!(decrypted_content, original_content);

        Ok(())
    }

    #[test]
    fn test_roundtrip_large_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let sk_path = temp_dir.path().join("secret.pem");
        let pk_path = temp_dir.path().join("public.pem");
        let input_path = temp_dir.path().join("large.bin");
        let encrypted_path = temp_dir.path().join("encrypted.bin");
        let decrypted_path = temp_dir.path().join("decrypted.bin");

        keygen(&sk_path, &pk_path, false)?;

        // Create 1MB of random data
        let mut original_content = vec![0u8; 1024 * 1024];
        rand::rng().fill_bytes(&mut original_content);
        fs::write(&input_path, &original_content)?;

        encrypt(&pk_path, &input_path, &encrypted_path)?;
        decrypt(&sk_path, &encrypted_path, &decrypted_path)?;

        let decrypted_content = fs::read(&decrypted_path)?;
        assert_eq!(decrypted_content, original_content);

        Ok(())
    }

    #[test]
    fn test_wrong_key_fails() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let sk1_path = temp_dir.path().join("secret1.pem");
        let pk1_path = temp_dir.path().join("public1.pem");
        let sk2_path = temp_dir.path().join("secret2.pem");
        let pk2_path = temp_dir.path().join("public2.pem");
        let input_path = temp_dir.path().join("input.txt");
        let encrypted_path = temp_dir.path().join("encrypted.bin");
        let decrypted_path = temp_dir.path().join("decrypted.txt");

        // Generate two different key pairs
        keygen(&sk1_path, &pk1_path, false)?;
        keygen(&sk2_path, &pk2_path, false)?;

        let original_content = b"Secret message";
        fs::write(&input_path, original_content)?;

        // Encrypt with key pair 1
        encrypt(&pk1_path, &input_path, &encrypted_path)?;

        // Try to decrypt with key pair 2 - should fail
        let result = decrypt(&sk2_path, &encrypted_path, &decrypted_path);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_tampered_ciphertext_fails() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let sk_path = temp_dir.path().join("secret.pem");
        let pk_path = temp_dir.path().join("public.pem");
        let input_path = temp_dir.path().join("input.txt");
        let encrypted_path = temp_dir.path().join("encrypted.bin");
        let decrypted_path = temp_dir.path().join("decrypted.txt");

        keygen(&sk_path, &pk_path, false)?;

        let original_content = b"Secret message";
        fs::write(&input_path, original_content)?;

        encrypt(&pk_path, &input_path, &encrypted_path)?;

        // Tamper with the encrypted file
        let mut encrypted_content = fs::read(&encrypted_path)?;
        if let Some(last_byte) = encrypted_content.last_mut() {
            *last_byte ^= 0xFF;
        }
        fs::write(&encrypted_path, &encrypted_content)?;

        // Decryption should fail due to authentication
        let result = decrypt(&sk_path, &encrypted_path, &decrypted_path);
        assert!(result.is_err());

        Ok(())
    }
}
