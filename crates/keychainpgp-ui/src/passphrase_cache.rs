//! Secure in-memory passphrase cache with automatic expiration.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use zeroize::Zeroize;

/// Entry in the passphrase cache.
struct CacheEntry {
    passphrase: Vec<u8>,
    stored_at: Instant,
}

impl Drop for CacheEntry {
    fn drop(&mut self) {
        self.passphrase.zeroize();
    }
}

/// A cache that stores passphrases by key fingerprint with automatic expiration.
pub struct PassphraseCache {
    entries: HashMap<String, CacheEntry>,
    ttl: Duration,
}

impl PassphraseCache {
    /// Create a new cache with the given time-to-live duration.
    pub fn new(ttl_secs: u64) -> Self {
        Self {
            entries: HashMap::new(),
            ttl: Duration::from_secs(ttl_secs),
        }
    }

    /// Update the TTL duration. Does not affect already-cached entries.
    pub fn set_ttl(&mut self, ttl_secs: u64) {
        self.ttl = Duration::from_secs(ttl_secs);
    }

    /// Store a passphrase for the given fingerprint.
    /// Also purges any expired entries to prevent unbounded memory growth.
    pub fn store(&mut self, fingerprint: &str, passphrase: &[u8]) {
        // Purge expired entries on each store to bound memory usage
        let ttl = self.ttl;
        self.entries
            .retain(|_, entry| entry.stored_at.elapsed() < ttl);

        self.entries.insert(
            fingerprint.to_string(),
            CacheEntry {
                passphrase: passphrase.to_vec(),
                stored_at: Instant::now(),
            },
        );
    }

    /// Get a cached passphrase if it exists and hasn't expired.
    pub fn get(&self, fingerprint: &str) -> Option<&[u8]> {
        self.entries.get(fingerprint).and_then(|entry| {
            if entry.stored_at.elapsed() < self.ttl {
                Some(entry.passphrase.as_slice())
            } else {
                None
            }
        })
    }

    /// Clear all cached passphrases.
    pub fn clear_all(&mut self) {
        self.entries.clear();
    }
}

impl Drop for PassphraseCache {
    fn drop(&mut self) {
        self.clear_all();
    }
}
