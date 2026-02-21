<script lang="ts">
  import { decrypt } from "../lib/wasm";
  import { listKeys, getSecretKey, type StoredKey } from "../lib/keystore";

  let ciphertext = $state("");
  let passphrase = $state("");
  let output = $state("");
  let error = $state("");

  async function handleDecrypt() {
    error = "";
    output = "";
    if (!ciphertext.trim()) {
      error = "Paste an encrypted PGP message.";
      return;
    }

    const keys = await listKeys();
    const ownKeys = keys.filter((k) => k.isOwn);

    if (ownKeys.length === 0) {
      error = "No private keys available. Generate or import a key in the Keys tab.";
      return;
    }

    for (const key of ownKeys) {
      const secretKey = await getSecretKey(key.fingerprint);
      if (!secretKey) continue;

      try {
        output = decrypt(ciphertext, secretKey, passphrase || undefined);
        return;
      } catch {
        // Try next key
      }
    }

    error = "Decryption failed. You may not have the correct private key, or the passphrase is wrong.";
  }

  async function copyOutput() {
    await navigator.clipboard.writeText(output);
  }
</script>

<div class="card" style="display: flex; flex-direction: column; gap: 1rem;">
  <h2 style="font-size: 1rem; font-weight: 600;">Decrypt Message</h2>

  <textarea
    class="textarea"
    placeholder="Paste encrypted PGP message here..."
    bind:value={ciphertext}
    rows="8"
  ></textarea>

  <input
    type="password"
    class="input"
    placeholder="Passphrase (if key is protected)"
    bind:value={passphrase}
  />

  <button class="btn btn-primary" onclick={handleDecrypt} disabled={!ciphertext.trim()}>
    Decrypt
  </button>

  {#if error}
    <p class="error">{error}</p>
  {/if}

  {#if output}
    <div style="position: relative;">
      <textarea class="textarea" readonly value={output} rows="6"></textarea>
      <button
        class="btn"
        style="position: absolute; top: 0.5rem; right: 0.5rem; font-size: 0.75rem; padding: 0.25rem 0.5rem;"
        onclick={copyOutput}
      >Copy</button>
    </div>
  {/if}
</div>
