<script lang="ts">
  import { sign } from "../lib/wasm";
  import { listKeys, getSecretKey } from "../lib/keystore";

  let message = $state("");
  let passphrase = $state("");
  let output = $state("");
  let error = $state("");

  async function handleSign() {
    error = "";
    output = "";
    if (!message.trim()) {
      error = "Enter a message to sign.";
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
        output = sign(message, secretKey, passphrase || undefined);
        return;
      } catch {
        // Try next key
      }
    }

    error = "Signing failed. Your key may require a passphrase.";
  }

  async function copyOutput() {
    await navigator.clipboard.writeText(output);
  }
</script>

<div class="card" style="display: flex; flex-direction: column; gap: 1rem;">
  <h2 style="font-size: 1rem; font-weight: 600;">Sign Message</h2>

  <textarea
    class="textarea"
    placeholder="Type your message here..."
    bind:value={message}
    rows="5"
  ></textarea>

  <input
    type="password"
    class="input"
    placeholder="Passphrase (if key is protected)"
    bind:value={passphrase}
  />

  <button class="btn btn-primary" onclick={handleSign} disabled={!message.trim()}>
    Sign
  </button>

  {#if error}
    <p class="error">{error}</p>
  {/if}

  {#if output}
    <div style="position: relative;">
      <textarea class="textarea" readonly value={output} rows="8"></textarea>
      <button
        class="btn"
        style="position: absolute; top: 0.5rem; right: 0.5rem; font-size: 0.75rem; padding: 0.25rem 0.5rem;"
        onclick={copyOutput}
      >Copy</button>
    </div>
  {/if}
</div>
