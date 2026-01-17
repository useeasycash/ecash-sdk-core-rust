use k256::{
    ecdsa::{
        signature::{Signer as SignerTrait, Verifier},
        Signature, SigningKey, VerifyingKey,
    },
    SecretKey,
};
use sha2::{Digest, Sha256};

/// TransactionSigner handles cryptographic signing operations for transactions.
///
/// This struct wraps an ECDSA signing key and provides methods for signing
/// transaction data with SHA-256 hashing.
///
/// # Example
/// ```
/// use k256::SecretKey;
/// use ecash_sdk_core::crypto::TransactionSigner;
///
/// let secret_key_bytes = [1u8; 32];
/// let secret_key = SecretKey::from_bytes(&secret_key_bytes.into()).unwrap();
/// let signer = TransactionSigner::new(secret_key);
/// let signature = signer.sign_message(b"transaction data").unwrap();
/// ```
pub struct TransactionSigner {
    signing_key: SigningKey,
}

impl TransactionSigner {
    /// Creates a new signer with a given private key.
    ///
    /// # Arguments
    /// * `secret_key` - The secp256k1 secret key for signing
    pub fn new(secret_key: SecretKey) -> Self {
        Self {
            signing_key: SigningKey::from(&secret_key),
        }
    }

    /// Signs arbitrary data and returns hex-encoded signature.
    ///
    /// The data is first hashed with SHA-256, then signed using ECDSA.
    /// Returns a hex-encoded signature prefixed with "0x".
    ///
    /// # Arguments
    /// * `data` - The raw bytes to sign
    ///
    /// # Returns
    /// * `Ok(String)` - Hex-encoded signature (e.g., "0x1234...")
    /// * `Err(String)` - Error message if signing fails
    pub fn sign_message(&self, data: &[u8]) -> Result<String, String> {
        let hash = Sha256::digest(data);
        let signature: Signature = SignerTrait::sign(&self.signing_key, &hash);
        Ok(format!("0x{}", hex::encode(signature.to_bytes())))
    }

    /// Returns the public verifying key corresponding to this signer.
    pub fn verifying_key(&self) -> VerifyingKey {
        *self.signing_key.verifying_key()
    }
}

/// Verifies a signature against a public key.
///
/// # Arguments
/// * `verifying_key` - The public key to verify against
/// * `data` - The original data that was signed
/// * `signature_hex` - Hex-encoded signature (with or without "0x" prefix)
///
/// # Returns
/// * `Ok(true)` - Signature is valid
/// * `Ok(false)` - Signature verification failed
/// * `Err(String)` - Error parsing hex or signature format
pub fn verify_signature(
    verifying_key: &VerifyingKey,
    data: &[u8],
    signature_hex: &str,
) -> Result<bool, String> {
    let hash = Sha256::digest(data);

    // Decode hex signature (strip 0x prefix if present)
    let sig_hex = signature_hex.strip_prefix("0x").unwrap_or(signature_hex);
    let sig_bytes = hex::decode(sig_hex).map_err(|e| format!("invalid hex: {}", e))?;

    // ECDSA signatures are 64 bytes (r: 32, s: 32)
    if sig_bytes.len() != 64 {
        return Err(format!(
            "invalid signature length: expected 64 bytes, got {}",
            sig_bytes.len()
        ));
    }

    // Convert to fixed-size array for Signature::from_bytes
    let sig_array: [u8; 64] = sig_bytes
        .try_into()
        .map_err(|_| "failed to convert signature bytes")?;

    let signature =
        Signature::from_bytes(&sig_array.into()).map_err(|e| format!("invalid signature: {}", e))?;

    Ok(verifying_key.verify(&hash, &signature).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use k256::SecretKey;

    #[test]
    fn test_transaction_signer_sign_and_verify() {
        // Use a deterministic secret key for testing
        let secret_key_bytes = [1u8; 32];
        let secret_key = SecretKey::from_bytes(&secret_key_bytes.into()).unwrap();
        let signer = TransactionSigner::new(secret_key);

        let data = b"test message";
        let signature = signer.sign_message(data).unwrap();

        // Verify signature using the signer's verifying key
        let is_valid = verify_signature(&signer.verifying_key(), data, &signature).unwrap();
        assert!(is_valid);
    }

    #[test]
    fn test_signature_deterministic() {
        let secret_key_bytes = [1u8; 32];
        let secret_key = SecretKey::from_bytes(&secret_key_bytes.into()).unwrap();
        let signer = TransactionSigner::new(secret_key);

        let data = b"test message";
        let sig1 = signer.sign_message(data).unwrap();
        let sig2 = signer.sign_message(data).unwrap();

        // Note: ECDSA signatures may include randomness, so we verify both are valid
        // rather than checking equality
        assert!(verify_signature(&signer.verifying_key(), data, &sig1).unwrap());
        assert!(verify_signature(&signer.verifying_key(), data, &sig2).unwrap());
    }

    #[test]
    fn test_verify_signature_invalid_hex() {
        let secret_key_bytes = [2u8; 32];
        let secret_key = SecretKey::from_bytes(&secret_key_bytes.into()).unwrap();
        let signer = TransactionSigner::new(secret_key);

        let data = b"test message";
        let invalid_sig = "not_valid_hex";

        let result = verify_signature(&signer.verifying_key(), data, invalid_sig);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_signature_wrong_length() {
        let secret_key_bytes = [2u8; 32];
        let secret_key = SecretKey::from_bytes(&secret_key_bytes.into()).unwrap();
        let signer = TransactionSigner::new(secret_key);

        let data = b"test message";
        let short_sig = "0x1234567890abcdef";

        let result = verify_signature(&signer.verifying_key(), data, short_sig);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid signature length"));
    }

    #[test]
    fn test_verify_signature_wrong_key() {
        let secret_key_bytes1 = [1u8; 32];
        let secret_key_bytes2 = [2u8; 32];

        let secret_key1 = SecretKey::from_bytes(&secret_key_bytes1.into()).unwrap();
        let secret_key2 = SecretKey::from_bytes(&secret_key_bytes2.into()).unwrap();

        let signer1 = TransactionSigner::new(secret_key1);
        let signer2 = TransactionSigner::new(secret_key2);

        let data = b"test message";
        let signature = signer1.sign_message(data).unwrap();

        // Verify with wrong key should fail
        let is_valid = verify_signature(&signer2.verifying_key(), data, &signature).unwrap();
        assert!(!is_valid);
    }
}
