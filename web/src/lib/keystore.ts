/**
 * Browser key storage using IndexedDB.
 *
 * Secret keys are encrypted with AES-256-GCM using a non-extractable wrapping
 * key held in memory. The key is lost on page refresh (secrets become
 * inaccessible — this is by design for a browser-based ephemeral PGP tool).
 */

const DB_NAME = "keychainpgp";
const DB_VERSION = 1;
const STORE_NAME = "keys";

/** In-memory wrapping key — non-extractable, lost on page reload. */
let cachedWrappingKey: CryptoKey | null = null;

export interface StoredKey {
  fingerprint: string;
  name: string | null;
  email: string | null;
  publicKey: string;
  /** Encrypted secret key (base64), or null for contact keys. */
  encryptedSecretKey: string | null;
  /** AES-GCM IV (base64). */
  iv: string | null;
  isOwn: boolean;
  addedAt: number;
}

function openDb(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const req = indexedDB.open(DB_NAME, DB_VERSION);
    req.onupgradeneeded = () => {
      const db = req.result;
      if (!db.objectStoreNames.contains(STORE_NAME)) {
        db.createObjectStore(STORE_NAME, { keyPath: "fingerprint" });
      }
    };
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error);
  });
}

/** Get or generate the AES-256-GCM wrapping key for this session. */
async function getWrappingKey(): Promise<CryptoKey> {
  if (cachedWrappingKey) return cachedWrappingKey;

  cachedWrappingKey = await crypto.subtle.generateKey(
    { name: "AES-GCM", length: 256 },
    false, // non-extractable — cannot be read from JS
    ["encrypt", "decrypt"],
  );
  return cachedWrappingKey;
}

async function encryptSecret(
  plaintext: Uint8Array,
): Promise<{ ciphertext: string; iv: string }> {
  const key = await getWrappingKey();
  const iv = crypto.getRandomValues(new Uint8Array(12));
  const encrypted = await crypto.subtle.encrypt(
    { name: "AES-GCM", iv },
    key,
    plaintext,
  );
  return {
    ciphertext: btoa(String.fromCharCode(...new Uint8Array(encrypted))),
    iv: btoa(String.fromCharCode(...iv)),
  };
}

async function decryptSecret(
  ciphertext: string,
  ivBase64: string,
): Promise<Uint8Array> {
  const key = await getWrappingKey();
  const iv = Uint8Array.from(atob(ivBase64), (c) => c.charCodeAt(0));
  const data = Uint8Array.from(atob(ciphertext), (c) => c.charCodeAt(0));
  const decrypted = await crypto.subtle.decrypt(
    { name: "AES-GCM", iv },
    key,
    data,
  );
  return new Uint8Array(decrypted);
}

export async function listKeys(): Promise<StoredKey[]> {
  const db = await openDb();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE_NAME, "readonly");
    const store = tx.objectStore(STORE_NAME);
    const req = store.getAll();
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error);
  });
}

export async function getKey(fingerprint: string): Promise<StoredKey | null> {
  const db = await openDb();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE_NAME, "readonly");
    const store = tx.objectStore(STORE_NAME);
    const req = store.get(fingerprint);
    req.onsuccess = () => resolve(req.result ?? null);
    req.onerror = () => reject(req.error);
  });
}

export async function storeKey(
  fingerprint: string,
  name: string | null,
  email: string | null,
  publicKey: string,
  secretKey: Uint8Array | null,
): Promise<void> {
  let encryptedSecretKey: string | null = null;
  let iv: string | null = null;

  if (secretKey) {
    const encrypted = await encryptSecret(secretKey);
    encryptedSecretKey = encrypted.ciphertext;
    iv = encrypted.iv;
  }

  const record: StoredKey = {
    fingerprint,
    name,
    email,
    publicKey,
    encryptedSecretKey,
    iv,
    isOwn: secretKey !== null,
    addedAt: Date.now(),
  };

  const db = await openDb();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE_NAME, "readwrite");
    const store = tx.objectStore(STORE_NAME);
    store.put(record);
    tx.oncomplete = () => resolve();
    tx.onerror = () => reject(tx.error);
  });
}

/**
 * Retrieve the decrypted secret key as raw bytes.
 *
 * Callers MUST call `.fill(0)` on the returned `Uint8Array` after use
 * to zeroize the secret key material from memory.
 */
export async function getSecretKey(fingerprint: string): Promise<Uint8Array | null> {
  const record = await getKey(fingerprint);
  if (!record?.encryptedSecretKey || !record.iv) return null;
  try {
    return await decryptSecret(record.encryptedSecretKey, record.iv);
  } catch {
    // Wrapping key lost (new session or page refresh) — secret is inaccessible
    return null;
  }
}

export async function deleteKey(fingerprint: string): Promise<void> {
  const db = await openDb();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE_NAME, "readwrite");
    const store = tx.objectStore(STORE_NAME);
    store.delete(fingerprint);
    tx.oncomplete = () => resolve();
    tx.onerror = () => reject(tx.error);
  });
}
