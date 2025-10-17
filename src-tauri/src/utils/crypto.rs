use std::path::Path;

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::{engine::general_purpose::STANDARD as Base64, Engine as _};
use keyring::Entry;
use pbkdf2::pbkdf2_hmac;
use rand::rngs::OsRng;
use rand::RngCore;
use sha2::{Digest, Sha256};

use crate::error::{AppError, AppResult};

const KEYRING_SERVICE: &str = "cognical.ai.vault";
const VERSION_PREFIX: &str = "v1:";
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;
const PBKDF2_ITERATIONS: u32 = 120_000;

#[derive(Clone)]
pub struct CryptoVault {
    account: String,
}

impl CryptoVault {
    pub fn from_database_path(path: &Path) -> AppResult<Self> {
        let account = account_from_path(path);
        Self::new(&account)
    }

    pub fn new(account_id: &str) -> AppResult<Self> {
        Entry::new(KEYRING_SERVICE, account_id)
            .map_err(|err| AppError::other(format!("无法初始化系统密钥存储: {err}")))?;
        Ok(Self {
            account: account_id.to_string(),
        })
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> AppResult<String> {
        let master = self.load_or_create_master_secret()?;
        encrypt_with_master(&master, plaintext)
    }

    pub fn decrypt(&self, ciphertext: &str) -> AppResult<Vec<u8>> {
        let master = self.load_or_create_master_secret()?;
        decrypt_with_master(&master, ciphertext)
    }

    pub fn clear_master_secret(&self) -> AppResult<()> {
        let entry = self.entry()?;
        match entry.delete_password() {
            Ok(_) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(err) => Err(AppError::other(format!(
                "无法删除系统密钥存储中的凭据: {err}"
            ))),
        }
    }

    fn load_or_create_master_secret(&self) -> AppResult<Vec<u8>> {
        let entry = self.entry()?;
        match entry.get_password() {
            Ok(secret) => decode_master_secret(&secret),
            Err(keyring::Error::NoEntry) => self.create_master_secret(entry),
            Err(err) => Err(AppError::other(format!("无法访问系统密钥存储: {err}"))),
        }
    }

    fn create_master_secret(&self, entry: Entry) -> AppResult<Vec<u8>> {
        let mut secret = vec![0u8; KEY_LEN];
        OsRng.fill_bytes(&mut secret);
        let encoded = Base64.encode(&secret);
        entry
            .set_password(&encoded)
            .map_err(|err| AppError::other(format!("无法写入系统密钥存储: {err}")))?;
        Ok(secret)
    }

    fn entry(&self) -> AppResult<Entry> {
        Entry::new(KEYRING_SERVICE, &self.account)
            .map_err(|err| AppError::other(format!("无法初始化系统密钥存储: {err}")))
    }
}

pub(crate) fn encrypt_with_master(master_secret: &[u8], plaintext: &[u8]) -> AppResult<String> {
    if master_secret.len() != KEY_LEN {
        return Err(AppError::other("主密钥长度无效"));
    }

    let mut salt = [0u8; SALT_LEN];
    OsRng.fill_bytes(&mut salt);
    let key = derive_key(master_secret, &salt);
    let cipher =
        Aes256Gcm::new_from_slice(&key).map_err(|_| AppError::other("无法初始化加密器"))?;

    let mut nonce = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce);

    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), plaintext)
        .map_err(|_| AppError::other("加密失败"))?;

    let mut payload = Vec::with_capacity(SALT_LEN + NONCE_LEN + ciphertext.len());
    payload.extend_from_slice(&salt);
    payload.extend_from_slice(&nonce);
    payload.extend_from_slice(&ciphertext);

    Ok(format!("{VERSION_PREFIX}{}", Base64.encode(payload)))
}

pub(crate) fn decrypt_with_master(master_secret: &[u8], ciphertext: &str) -> AppResult<Vec<u8>> {
    if master_secret.len() != KEY_LEN {
        return Err(AppError::other("主密钥长度无效"));
    }

    let encoded = ciphertext
        .strip_prefix(VERSION_PREFIX)
        .ok_or_else(|| AppError::other("密文格式不受支持"))?;

    let decoded = Base64
        .decode(encoded.as_bytes())
        .map_err(|_| AppError::other("密文损坏，无法解码"))?;

    if decoded.len() <= SALT_LEN + NONCE_LEN {
        return Err(AppError::other("密文数据长度无效"));
    }

    let (salt, rest) = decoded.split_at(SALT_LEN);
    let (nonce_bytes, ciphertext_bytes) = rest.split_at(NONCE_LEN);

    let key = derive_key(master_secret, salt);
    let cipher =
        Aes256Gcm::new_from_slice(&key).map_err(|_| AppError::other("无法初始化解密器"))?;

    cipher
        .decrypt(Nonce::from_slice(nonce_bytes), ciphertext_bytes)
        .map_err(|_| AppError::other("解密失败"))
}

fn derive_key(master: &[u8], salt: &[u8]) -> [u8; KEY_LEN] {
    let mut key = [0u8; KEY_LEN];
    pbkdf2_hmac::<Sha256>(master, salt, PBKDF2_ITERATIONS, &mut key);
    key
}

fn decode_master_secret(encoded: &str) -> AppResult<Vec<u8>> {
    let secret = Base64
        .decode(encoded.as_bytes())
        .map_err(|_| AppError::other("系统密钥存储中的凭据损坏"))?;
    if secret.len() != KEY_LEN {
        return Err(AppError::other("系统密钥存储中的凭据长度无效"));
    }
    Ok(secret)
}

fn account_from_path(path: &Path) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"cognical.ai.settings.v1");
    hasher.update(path.to_string_lossy().as_bytes());
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(32);
    for byte in digest[..16].iter() {
        hex.push_str(&format!("{:02x}", byte));
    }
    format!("deepseek-{}", hex)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_and_decrypt_roundtrip() {
        let master = [42u8; KEY_LEN];
        let ciphertext = encrypt_with_master(&master, b"test-secret").unwrap();
        let decrypted = decrypt_with_master(&master, &ciphertext).unwrap();
        assert_eq!(decrypted, b"test-secret");
    }

    #[test]
    fn encrypt_produces_unique_ciphertext() {
        let master = [7u8; KEY_LEN];
        let first = encrypt_with_master(&master, b"repeatable").unwrap();
        let second = encrypt_with_master(&master, b"repeatable").unwrap();
        assert_ne!(first, second);
    }

    #[test]
    fn decrypt_with_wrong_master_fails() {
        let master = [1u8; KEY_LEN];
        let other = [2u8; KEY_LEN];
        let ciphertext = encrypt_with_master(&master, b"secret").unwrap();
        let result = decrypt_with_master(&other, &ciphertext);
        assert!(result.is_err());
    }
}
