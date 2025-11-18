use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;

const PBKDF2_ITERATIONS: u32 = 100_000;
const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 32;

pub fn derive_key(password: &str, salt: &[u8; SALT_SIZE]) -> Key<Aes256Gcm> {
    let mut key_bytes = [0u8; 32];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, PBKDF2_ITERATIONS, &mut key_bytes);
    *Key::<Aes256Gcm>::from_slice(&key_bytes)
}

pub fn encrypt_data(data: &[u8], password: &str, salt: &[u8; SALT_SIZE]) -> Result<(Vec<u8>, [u8; NONCE_SIZE]), aes_gcm::Error> {
    let key = derive_key(password, salt);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    
    let nonce_array: [u8; NONCE_SIZE] = nonce.as_slice().try_into()
        .map_err(|_| aes_gcm::Error)?;
    
    let ciphertext = cipher.encrypt(&nonce, data)?;
    
    Ok((ciphertext, nonce_array))
}

pub fn decrypt_data(
    ciphertext: &[u8],
    password: &str,
    salt: &[u8; SALT_SIZE],
    nonce: &[u8; NONCE_SIZE],
) -> Result<Vec<u8>, aes_gcm::Error> {
    let key = derive_key(password, salt);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(nonce);
    
    cipher.decrypt(nonce, ciphertext)
}

pub fn generate_salt() -> [u8; SALT_SIZE] {
    let mut salt = [0u8; SALT_SIZE];
    use rand::RngCore;
    rand::thread_rng().fill_bytes(&mut salt);
    salt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_round_trip() {
        let data = b"Hello, World!";
        let password = "test_password_123";
        let salt = generate_salt();

        let (ciphertext, nonce) = encrypt_data(data, password, &salt).unwrap();
        let decrypted = decrypt_data(&ciphertext, password, &salt, &nonce).unwrap();

        assert_eq!(data, decrypted.as_slice());
    }

    #[test]
    fn test_wrong_password_fails() {
        let data = b"Hello, World!";
        let password = "test_password_123";
        let wrong_password = "wrong_password";
        let salt = generate_salt();

        let (ciphertext, nonce) = encrypt_data(data, password, &salt).unwrap();
        let result = decrypt_data(&ciphertext, wrong_password, &salt, &nonce);

        assert!(result.is_err());
    }
}

