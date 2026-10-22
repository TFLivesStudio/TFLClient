// Copyright (C) 2026 Santiagolxx, Notstaff and CubicLauncher contributors
// SPDX-License-Identifier: AGPL-3.0-or-later
//! V2 envelopes use random AES keys held by the OS vault, never derived from UUIDs.
//! The legacy public-key derivation is read-only and exists solely for migration.
#![cfg(feature = "auth")]
use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};
use zeroize::Zeroizing;

static STORAGE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

const SERVICE: &str = "com.tflives.tflclient.sessions.v2";
const MAX_RECORD: u64 = 64 * 1024;

#[derive(Serialize, Deserialize, zeroize::Zeroize, zeroize::ZeroizeOnDrop)]
pub(crate) struct SessionSecrets {
    pub version: u8,
    #[serde(default)]
    pub expires_at: Option<u64>,
    pub access: String,
    pub refresh: Option<String>,
    pub client_token: Option<String>,
    pub issuer: Option<String>,
}

pub(crate) trait Vault {
    fn get(&self, account: &str) -> crate::Result<Option<String>>;
    fn set(&self, account: &str, value: &str) -> crate::Result<()>;
    fn delete(&self, account: &str) -> crate::Result<()>;
}
struct OsVault;
fn failure(code: &str) -> crate::Error {
    crate::Error::AuthError(code.into())
}
impl Vault for OsVault {
    fn get(&self, account: &str) -> crate::Result<Option<String>> {
        let entry = keyring::Entry::new(SERVICE, account)
            .map_err(|_| failure("SECURE_STORE_UNAVAILABLE"))?;
        match entry.get_password() {
            Ok(value) => Ok(Some(value)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(_) => Err(failure("SECURE_STORE_UNAVAILABLE")),
        }
    }
    fn set(&self, account: &str, value: &str) -> crate::Result<()> {
        keyring::Entry::new(SERVICE, account)
            .and_then(|e| e.set_password(value))
            .map_err(|_| failure("SECURE_STORE_UNAVAILABLE"))
    }
    fn delete(&self, account: &str) -> crate::Result<()> {
        let entry = keyring::Entry::new(SERVICE, account)
            .map_err(|_| failure("SECURE_STORE_UNAVAILABLE"))?;
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(_) => Err(failure("SECURE_STORE_UNAVAILABLE")),
        }
    }
}

pub struct SecureStorage;
impl SecureStorage {
    fn root() -> crate::Result<PathBuf> {
        let mut p = dirs::data_dir().ok_or_else(|| failure("SECURE_STORE_UNAVAILABLE"))?;
        p.push("TFLClient");
        p.push("sessions-v2");
        Ok(p)
    }
    pub(crate) fn key(provider: &str, uuid: &str) -> String {
        let mut h = Sha256::new();
        h.update(provider);
        h.update([0]);
        h.update(uuid);
        format!("{:x}", h.finalize())
    }
    pub(crate) fn save(provider: &str, uuid: &str, record: &SessionSecrets) -> crate::Result<()> {
        let _guard = STORAGE_LOCK
            .lock()
            .map_err(|_| failure("SECURE_STORE_UNAVAILABLE"))?;
        Self::write_at(&OsVault, &Self::root()?, &Self::key(provider, uuid), record)
    }
    pub(crate) fn load(provider: &str, uuid: &str) -> crate::Result<Option<SessionSecrets>> {
        let _guard = STORAGE_LOCK
            .lock()
            .map_err(|_| failure("SECURE_STORE_UNAVAILABLE"))?;
        Self::read_at(&OsVault, &Self::root()?, &Self::key(provider, uuid))
    }
    pub(crate) fn migrate(
        provider: &str,
        uuid: &str,
        record: SessionSecrets,
    ) -> crate::Result<SessionSecrets> {
        let _guard = STORAGE_LOCK
            .lock()
            .map_err(|_| failure("SECURE_STORE_UNAVAILABLE"))?;
        let root = Self::root()?;
        let account = Self::key(provider, uuid);
        if let Some(current) = Self::read_at(&OsVault, &root, &account)? {
            return Ok(current);
        }
        Self::write_at(&OsVault, &root, &account, &record)?;
        Ok(record)
    }
    pub(crate) fn delete(provider: &str, uuid: &str) -> crate::Result<()> {
        let _guard = STORAGE_LOCK
            .lock()
            .map_err(|_| failure("SECURE_STORE_UNAVAILABLE"))?;
        let key = Self::key(provider, uuid);
        let path = Self::root()?.join(format!("{key}.enc"));
        // No record means no need to initialize a potentially unavailable vault.
        if path.exists() {
            OsVault.delete(&key)?;
            fs::remove_file(path)?;
        }
        Self::delete_legacy(uuid)
    }
    fn vault_key(
        vault: &impl Vault,
        account: &str,
        create: bool,
    ) -> crate::Result<Zeroizing<Vec<u8>>> {
        let encoded = match vault.get(account)? {
            Some(key) => Zeroizing::new(key),
            None if create => {
                let mut key = Zeroizing::new([0u8; 32]);
                rand::rng().fill_bytes(&mut *key);
                let encoded = Zeroizing::new(STANDARD.encode(*key));
                vault.set(account, &encoded)?;
                let verified = Zeroizing::new(
                    vault
                        .get(account)?
                        .ok_or_else(|| failure("SECURE_STORE_VERIFY_FAILED"))?,
                );
                if *verified != *encoded {
                    return Err(failure("SECURE_STORE_VERIFY_FAILED"));
                }
                encoded
            }
            None => return Err(failure("SESSION_KEY_MISSING")),
        };
        let key = Zeroizing::new(
            STANDARD
                .decode(&*encoded)
                .map_err(|_| failure("SESSION_KEY_INVALID"))?,
        );
        if key.len() != 32 {
            return Err(failure("SESSION_KEY_INVALID"));
        }
        Ok(key)
    }
    fn write_at(
        vault: &impl Vault,
        root: &Path,
        account: &str,
        record: &SessionSecrets,
    ) -> crate::Result<()> {
        if root.exists() && fs::symlink_metadata(root)?.file_type().is_symlink() {
            return Err(failure("SESSION_PATH_INVALID"));
        }
        fs::create_dir_all(root)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(root, fs::Permissions::from_mode(0o700))?;
        }
        let target = root.join(format!("{account}.enc"));
        let key = Self::vault_key(vault, account, true)?;
        let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| failure("SESSION_KEY_INVALID"))?;
        let mut nonce = [0u8; 12];
        rand::rng().fill_bytes(&mut nonce);
        let plain = Zeroizing::new(
            serde_json::to_vec(record).map_err(|_| failure("SESSION_FORMAT_INVALID"))?,
        );
        if plain.len() > MAX_RECORD as usize - 64 {
            return Err(failure("SESSION_TOO_LARGE"));
        }
        let encrypted = cipher
            .encrypt(Nonce::from_slice(&nonce), plain.as_slice())
            .map_err(|_| failure("SESSION_ENCRYPTION_FAILED"))?;
        let temp = root.join(format!(".{}.tmp", uuid::Uuid::new_v4()));
        let result = (|| {
            let mut options = fs::OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            let mut file = options.open(&temp)?;
            file.write_all(&nonce)?;
            file.write_all(&encrypted)?;
            file.sync_all()?;
            fs::rename(&temp, &target)?;
            let checked = Self::read_at(vault, root, account)?
                .ok_or_else(|| failure("SECURE_STORE_VERIFY_FAILED"))?;
            let verify = Zeroizing::new(
                serde_json::to_vec(&checked).map_err(|_| failure("SESSION_FORMAT_INVALID"))?,
            );
            if *verify != *plain {
                return Err(failure("SECURE_STORE_VERIFY_FAILED"));
            }
            Ok(())
        })();
        let _ = fs::remove_file(temp);
        result
    }
    fn read_at(
        vault: &impl Vault,
        root: &Path,
        account: &str,
    ) -> crate::Result<Option<SessionSecrets>> {
        let path = root.join(format!("{account}.enc"));
        if !path.exists() {
            return Ok(None);
        }
        let data = Self::bounded_file(&path)?;
        let key = Self::vault_key(vault, account, false)?;
        let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| failure("SESSION_KEY_INVALID"))?;
        if data.len() < 28 {
            return Err(failure("SESSION_FORMAT_INVALID"));
        }
        let plain = Zeroizing::new(
            cipher
                .decrypt(Nonce::from_slice(&data[..12]), &data[12..])
                .map_err(|_| failure("SESSION_DECRYPTION_FAILED"))?,
        );
        let record: SessionSecrets =
            serde_json::from_slice(&plain).map_err(|_| failure("SESSION_FORMAT_INVALID"))?;
        if record.version != 2 {
            return Err(failure("SESSION_FORMAT_INVALID"));
        }
        Ok(Some(record))
    }
    fn bounded_file(path: &Path) -> crate::Result<Vec<u8>> {
        let m = fs::symlink_metadata(path)?;
        if !m.is_file() || m.len() > MAX_RECORD {
            return Err(failure("SESSION_PATH_INVALID"));
        }
        use std::io::Read;
        let mut bytes = Vec::new();
        fs::File::open(path)?
            .take(MAX_RECORD + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() as u64 > MAX_RECORD {
            return Err(failure("SESSION_TOO_LARGE"));
        }
        Ok(bytes)
    }
    fn legacy_dir() -> crate::Result<PathBuf> {
        Ok(dirs::data_dir()
            .or_else(dirs::home_dir)
            .ok_or_else(|| failure("SESSION_PATH_INVALID"))?
            .join("claunch-rs")
            .join("secrets"))
    }
    fn legacy_path(uuid: &str, kind: &str) -> crate::Result<PathBuf> {
        if uuid.is_empty()
            || !uuid
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            return Err(failure("SESSION_PATH_INVALID"));
        }
        Ok(Self::legacy_dir()?.join(format!("{uuid}_{kind}.enc")))
    }
    pub(crate) fn legacy(uuid: &str, kind: &str) -> crate::Result<Option<String>> {
        let path = Self::legacy_path(uuid, kind)?;
        if !path.exists() {
            return Ok(None);
        }
        Self::decode_legacy(uuid, &Self::bounded_file(&path)?).map(Some)
    }
    fn decode_legacy(uuid: &str, data: &[u8]) -> crate::Result<String> {
        if data.len() < 28 {
            return Err(failure("LEGACY_SESSION_INVALID"));
        }
        let mut h = Sha256::new();
        h.update(uuid.as_bytes());
        h.update(b"claunch-rs-secure-salt-v1");
        let key = Zeroizing::new(h.finalize().to_vec());
        let cipher =
            Aes256Gcm::new_from_slice(&key).map_err(|_| failure("LEGACY_SESSION_INVALID"))?;
        let plain = Zeroizing::new(
            cipher
                .decrypt(Nonce::from_slice(&data[..12]), &data[12..])
                .map_err(|_| failure("LEGACY_SESSION_INVALID"))?,
        );
        String::from_utf8(plain.to_vec()).map_err(|_| failure("LEGACY_SESSION_INVALID"))
    }
    pub(crate) fn delete_legacy(uuid: &str) -> crate::Result<()> {
        for kind in ["access", "refresh", "client"] {
            let path = Self::legacy_path(uuid, kind)?;
            match fs::remove_file(path) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(e.into()),
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    #[derive(Default)]
    struct FakeVault {
        value: RefCell<Option<String>>,
        fail: bool,
    }
    impl Vault for FakeVault {
        fn get(&self, _: &str) -> crate::Result<Option<String>> {
            if self.fail {
                Err(failure("SECURE_STORE_UNAVAILABLE"))
            } else {
                Ok(self.value.borrow().clone())
            }
        }
        fn set(&self, _: &str, v: &str) -> crate::Result<()> {
            *self.value.borrow_mut() = Some(v.into());
            Ok(())
        }
        fn delete(&self, _: &str) -> crate::Result<()> {
            self.value.borrow_mut().take();
            Ok(())
        }
    }
    fn record() -> SessionSecrets {
        SessionSecrets {
            version: 2,
            expires_at: Some(2000000000),
            access: "private-access".into(),
            refresh: Some("private-refresh".into()),
            client_token: None,
            issuer: Some("issuer".into()),
        }
    }
    #[test]
    fn random_os_key_roundtrip_ciphertext_and_delete() {
        let dir = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
        let vault = FakeVault::default();
        let data = record();
        SecureStorage::write_at(&vault, &dir, "test", &data).unwrap();
        assert!(
            !String::from_utf8_lossy(&fs::read(dir.join("test.enc")).unwrap()).contains("private")
        );
        let loaded = SecureStorage::read_at(&vault, &dir, "test")
            .unwrap()
            .unwrap();
        assert_eq!(loaded.access, data.access);
        assert_eq!(loaded.issuer, data.issuer);
        vault.delete("test").unwrap();
        assert!(SecureStorage::read_at(&vault, &dir, "test").is_err());
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn unavailable_vault_does_not_write_or_replace_record() {
        let dir = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("test.enc"), b"preserve").unwrap();
        assert!(
            SecureStorage::write_at(
                &FakeVault {
                    fail: true,
                    ..Default::default()
                },
                &dir,
                "test",
                &record()
            )
            .is_err()
        );
        assert_eq!(fs::read(dir.join("test.enc")).unwrap(), b"preserve");
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn legacy_decrypts_original_format_without_new_legacy_writes() {
        let mut h = Sha256::new();
        h.update(b"legacy-id");
        h.update(b"claunch-rs-secure-salt-v1");
        let key = h.finalize();
        let cipher = Aes256Gcm::new_from_slice(&key).unwrap();
        let nonce = [7; 12];
        let mut bytes = nonce.to_vec();
        bytes.extend(
            cipher
                .encrypt(Nonce::from_slice(&nonce), b"old-refresh".as_slice())
                .unwrap(),
        );
        assert_eq!(
            SecureStorage::decode_legacy("legacy-id", &bytes).unwrap(),
            "old-refresh"
        );
        assert!(SecureStorage::decode_legacy("another-id", &bytes).is_err());
    }
    #[test]
    fn provider_namespaces_and_path_validation() {
        assert_ne!(
            SecureStorage::key("microsoft", "a"),
            SecureStorage::key("yggdrasil:https://example.test", "a")
        );
        assert!(SecureStorage::legacy_path("../outside", "access").is_err());
    }
}
