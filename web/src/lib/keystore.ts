/**
 * Browser key storage using IndexedDB.
 *
 * Secret keys are encrypted with AES-256-GCM using a wrapping key
 * stored in sessionStorage (lost when the tab closes).
 */

const DB_NAME = "keychainpgp";
const DB_VERSION = 1;
const STORE_NAME = "keys";
const WRAPPING_KEY_ID = "keychainpgp-wrapping-key";

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
  const stored = sessionStorage.getItem(WRAPPING_KEY_ID);
  if (stored) {
    const raw = Uint8Array.from(atob(stored), (c) => c.charCodeAt(0));
    return crypto.subtle.importKey("raw", raw, "AES-GCM", true, [
      "encrypt",
      "decrypt",
    ]);
  }

  const key = await crypto.subtle.generateKey(
    { name: "AES-GCM", length: 256 },
    true,
    ["encrypt", "decrypt"],
  );
  const exported = await crypto.subtle.exportKey("raw", key);
  sessionStorage.setItem(
    WRAPPING_KEY_ID,
    btoa(String.fromCharCode(...new Uint8Array(exported))),
  );
  return key;
}

async function encryptSecret(
  plaintext: string,
): Promise<{ ciphertext: string; iv: string }> {
  const key = await getWrappingKey();
  const iv = crypto.getRandomValues(new Uint8Array(12));
  const encoded = new TextEncoder().encode(plaintext);
  const encrypted = await crypto.subtle.encrypt(
    { name: "AES-GCM", iv },
    key,
    encoded,
  );
  return {
    ciphertext: btoa(String.fromCharCode(...new Uint8Array(encrypted))),
    iv: btoa(String.fromCharCode(...iv)),
  };
}

async function decryptSecret(
  ciphertext: string,
  ivBase64: string,
): Promise<string> {
  const key = await getWrappingKey();
  const iv = Uint8Array.from(atob(ivBase64), (c) => c.charCodeAt(0));
  const data = Uint8Array.from(atob(ciphertext), (c) => c.charCodeAt(0));
  const decrypted = await crypto.subtle.decrypt(
    { name: "AES-GCM", iv },
    key,
    data,
  );
  return new TextDecoder().decode(decrypted);
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
  secretKey: string | null,
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

export async function getSecretKey(fingerprint: string): Promise<string | null> {
  const record = await getKey(fingerprint);
  if (!record?.encryptedSecretKey || !record.iv) return null;
  try {
    return await decryptSecret(record.encryptedSecretKey, record.iv);
  } catch {
    // Wrapping key lost (new session) — secret is inaccessible
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
