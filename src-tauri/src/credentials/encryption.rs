use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use hkdf::Hkdf;
use rand::RngCore;
use sha2::Sha256;

use super::error::{CredResult, CredentialError};

const HKDF_SALT: &[u8] = b"contentjuiceos-credential-salt-v1";
const HKDF_INFO: &[u8] = b"contentjuiceos-credential-encryption";
const NONCE_SIZE: usize = 12;

/// Derives a 256-bit encryption key from the machine's unique identifier using HKDF-SHA256.
pub fn derive_key() -> CredResult<[u8; 32]> {
    let machine_id = get_machine_id()?;
    let hkdf = Hkdf::<Sha256>::new(Some(HKDF_SALT), machine_id.as_bytes());
    let mut key = [0u8; 32];
    hkdf.expand(HKDF_INFO, &mut key)
        .map_err(|e| CredentialError::Encryption(format!("HKDF expand failed: {e}")))?;
    Ok(key)
}

/// Encrypts plaintext using AES-256-GCM.
///
/// Returns base64(nonce ∥ ciphertext_with_tag).
pub fn encrypt(key: &[u8; 32], plaintext: &str) -> CredResult<String> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| CredentialError::Encryption(format!("Cipher init failed: {e}")))?;

    let mut nonce_bytes = [0u8; NONCE_SIZE];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| CredentialError::Encryption(format!("Encryption failed: {e}")))?;

    // Concatenate nonce + ciphertext (which includes the GCM tag)
    let mut combined = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);

    Ok(BASE64.encode(combined))
}

/// Decrypts a base64-encoded blob produced by [`encrypt`].
pub fn decrypt(key: &[u8; 32], encoded: &str) -> CredResult<String> {
    let combined = BASE64
        .decode(encoded)
        .map_err(|e| CredentialError::Encryption(format!("Base64 decode failed: {e}")))?;

    if combined.len() < NONCE_SIZE + 1 {
        return Err(CredentialError::Encryption(
            "Ciphertext too short".to_string(),
        ));
    }

    let (nonce_bytes, ciphertext) = combined.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| CredentialError::Encryption(format!("Cipher init failed: {e}")))?;

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| CredentialError::Encryption(format!("Decryption failed: {e}")))?;

    String::from_utf8(plaintext)
        .map_err(|e| CredentialError::Encryption(format!("UTF-8 decode failed: {e}")))
}

/// Reads a platform-specific machine identifier.
///
/// - **Windows**: `HKLM\SOFTWARE\Microsoft\Cryptography\MachineGuid`
/// - **macOS**: `IOPlatformUUID` from `ioreg`
/// - **Linux**: `/etc/machine-id`
/// - **Fallback**: hostname + app identifier hash
fn get_machine_id() -> CredResult<String> {
    get_machine_id_platform()
}

#[cfg(target_os = "windows")]
fn get_machine_id_platform() -> CredResult<String> {
    use winreg::enums::HKEY_LOCAL_MACHINE;
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let crypto_key = hklm
        .open_subkey("SOFTWARE\\Microsoft\\Cryptography")
        .map_err(|e| CredentialError::Encryption(format!("Registry open failed: {e}")))?;
    let guid: String = crypto_key
        .get_value("MachineGuid")
        .map_err(|e| CredentialError::Encryption(format!("Registry read failed: {e}")))?;
    Ok(guid)
}

#[cfg(target_os = "macos")]
fn get_machine_id_platform() -> CredResult<String> {
    let output = std::process::Command::new("ioreg")
        .args(["-rd1", "-c", "IOPlatformExpertDevice"])
        .output()
        .map_err(|e| CredentialError::Encryption(format!("ioreg command failed: {e}")))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if line.contains("IOPlatformUUID") {
            if let Some(uuid) = line.split('"').nth(3) {
                return Ok(uuid.to_string());
            }
        }
    }

    get_machine_id_fallback()
}

#[cfg(target_os = "linux")]
fn get_machine_id_platform() -> CredResult<String> {
    match std::fs::read_to_string("/etc/machine-id") {
        Ok(id) => Ok(id.trim().to_string()),
        Err(_) => get_machine_id_fallback(),
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
fn get_machine_id_platform() -> CredResult<String> {
    get_machine_id_fallback()
}

/// Fallback: uses hostname combined with a fixed app identifier.
#[allow(dead_code)]
fn get_machine_id_fallback() -> CredResult<String> {
    use sha2::Digest;
    use std::fmt::Write;

    let hostname = std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "unknown-host".to_string());

    let mut hasher = Sha256::new();
    hasher.update(hostname.as_bytes());
    hasher.update(b"contentjuiceos-machine-fallback");
    let hash = hasher.finalize();

    let mut hex_str = String::with_capacity(64);
    for byte in hash {
        write!(hex_str, "{byte:02x}").unwrap();
    }
    Ok(hex_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> [u8; 32] {
        // Use a fixed key for deterministic tests
        let mut key = [0u8; 32];
        key[0] = 0x42;
        key[31] = 0xFF;
        key
    }

    #[test]
    fn encrypt_decrypt_round_trip() {
        let key = test_key();
        let plaintext = "my-secret-token-12345";
        let encrypted = encrypt(&key, plaintext).unwrap();
        let decrypted = decrypt(&key, &encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn encrypt_produces_different_output_each_time() {
        let key = test_key();
        let plaintext = "same-input";
        let a = encrypt(&key, plaintext).unwrap();
        let b = encrypt(&key, plaintext).unwrap();
        // Random nonces mean different ciphertext each time
        assert_ne!(a, b);
    }

    #[test]
    fn wrong_key_fails_to_decrypt() {
        let key1 = test_key();
        let mut key2 = test_key();
        key2[0] = 0x99;

        let encrypted = encrypt(&key1, "secret").unwrap();
        let result = decrypt(&key2, &encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn tampered_ciphertext_fails() {
        let key = test_key();
        let encrypted = encrypt(&key, "secret").unwrap();

        // Tamper with the base64-encoded data
        let mut bytes = BASE64.decode(&encrypted).unwrap();
        if let Some(last) = bytes.last_mut() {
            *last ^= 0xFF;
        }
        let tampered = BASE64.encode(&bytes);

        let result = decrypt(&key, &tampered);
        assert!(result.is_err());
    }

    #[test]
    fn too_short_ciphertext_fails() {
        let key = test_key();
        let short = BASE64.encode([0u8; 5]);
        let result = decrypt(&key, &short);
        assert!(result.is_err());
    }

    #[test]
    fn derive_key_succeeds() {
        let result = derive_key();
        assert!(result.is_ok());
        let key = result.unwrap();
        assert_eq!(key.len(), 32);
        // Key should not be all zeros
        assert!(key.iter().any(|&b| b != 0));
    }

    #[test]
    fn derive_key_is_deterministic() {
        let k1 = derive_key().unwrap();
        let k2 = derive_key().unwrap();
        assert_eq!(k1, k2);
    }
}
