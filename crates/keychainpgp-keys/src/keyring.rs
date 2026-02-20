//! High-level keyring interface combining SQLite storage and OS credentials.

use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use keychainpgp_core::types::TrustLevel;
use secrecy::SecretBox;

use crate::credential::CredentialStore;
use crate::error::{Error, Result};
use crate::storage::{KeyRecord, KeyStorage};

/// The main keyring interface. Manages both public keys (SQLite) and
/// private keys (OS credential store with file-based fallback).
pub struct Keyring {
    storage: KeyStorage,
    credentials: CredentialStore,
    data_dir: PathBuf,
}

impl Keyring {
    /// Open the keyring using the default platform data directory.
    pub fn open_default() -> Result<Self> {
        let dirs = ProjectDirs::from("com", "keychainpgp", "KeychainPGP")
            .ok_or_else(|| Error::CredentialStore {
                reason: "could not determine application data directory".into(),
            })?;

        let data_dir = dirs.data_dir().to_path_buf();
        std::fs::create_dir_all(&data_dir)?;

        let db_path = data_dir.join("keyring.db");
        let storage = KeyStorage::open(&db_path)?;
        let credentials = CredentialStore::new(&data_dir)?;

        Ok(Self { storage, credentials, data_dir })
    }

    /// Open the keyring at a specific directory (for testing).
    pub fn open_at(data_dir: &Path) -> Result<Self> {
        std::fs::create_dir_all(data_dir)?;
        let db_path = data_dir.join("keyring.db");
        let storage = KeyStorage::open(&db_path)?;
        let credentials = CredentialStore::new(data_dir)?;
        Ok(Self {
            storage,
            credentials,
            data_dir: data_dir.to_path_buf(),
        })
    }

    /// Get the data directory path.
    #[must_use]
    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    /// Store a generated key pair (public key in DB, private key in credential store).
    pub fn store_generated_key(
        &self,
        record: KeyRecord,
        secret_key: &[u8],
    ) -> Result<()> {
        // Store private key
        self.credentials.store_secret_key(&record.fingerprint, secret_key)?;

        // Store public key in SQLite
        self.storage.insert(&record)?;

        Ok(())
    }

    /// Import a public key into the keyring.
    pub fn import_public_key(&self, record: KeyRecord) -> Result<()> {
        self.storage.insert(&record)
    }

    /// Get a key record by fingerprint.
    pub fn get_key(&self, fingerprint: &str) -> Result<Option<KeyRecord>> {
        self.storage.get(fingerprint)
    }

    /// List all keys in the keyring.
    pub fn list_keys(&self) -> Result<Vec<KeyRecord>> {
        self.storage.list_all()
    }

    /// Search keys by name, email, or fingerprint fragment.
    pub fn search_keys(&self, query: &str) -> Result<Vec<KeyRecord>> {
        self.storage.search(query)
    }

    /// Delete a key from the keyring (both public and private if present).
    pub fn delete_key(&self, fingerprint: &str) -> Result<bool> {
        // Try to delete private key (ignore errors if not present)
        let _ = self.credentials.delete_secret_key(fingerprint);

        self.storage.delete(fingerprint)
    }

    /// Retrieve the secret key for the given fingerprint.
    pub fn get_secret_key(&self, fingerprint: &str) -> Result<SecretBox<Vec<u8>>> {
        self.credentials.get_secret_key(fingerprint)
    }

    /// Check if a secret key exists for the given fingerprint.
    pub fn has_secret_key(&self, fingerprint: &str) -> bool {
        self.credentials.has_secret_key(fingerprint)
    }

    /// Update the trust level for a key.
    pub fn set_trust(&self, fingerprint: &str, trust: TrustLevel) -> Result<bool> {
        self.storage.set_trust(fingerprint, trust)
    }
}
