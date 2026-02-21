<script lang="ts">
  import { verify } from "../lib/wasm";
  import { listKeys } from "../lib/keystore";

  let signedMessage = $state("");
  let result = $state<{ valid: boolean; fingerprint: string | null } | null>(null);
  let signerName = $state("");
  let error = $state("");

  async function handleVerify() {
    error = "";
    result = null;
    signerName = "";

    if (!signedMessage.trim()) {
      error = "Paste a signed PGP message.";
      return;
    }

    const keys = await listKeys();
    if (keys.length === 0) {
      error = "No keys in keyring. Import the signer's public key first.";
      return;
    }

    for (const key of keys) {
      try {
        const r = verify(signedMessage, key.publicKey);
        if (r.valid) {
          result = { valid: true, fingerprint: r.signer_fingerprint };
          signerName = key.name ?? key.email ?? key.fingerprint.slice(-16);
          return;
        }
      } catch {
        // Try next key
      }
    }

    result = { valid: false, fingerprint: null };
  }
</script>

<div class="card" style="display: flex; flex-direction: column; gap: 1rem;">
  <h2 style="font-size: 1rem; font-weight: 600;">Verify Signature</h2>

  <textarea
    class="textarea"
    placeholder="Paste signed PGP message here..."
    bind:value={signedMessage}
    rows="8"
  ></textarea>

  <button class="btn btn-primary" onclick={handleVerify} disabled={!signedMessage.trim()}>
    Verify
  </button>

  {#if error}
    <p class="error">{error}</p>
  {/if}

  {#if result}
    {#if result.valid}
      <div style="padding: 1rem; border-radius: 0.5rem; border: 1px solid var(--success); background: rgba(34, 197, 94, 0.1);">
        <p class="success" style="font-weight: 600;">Valid Signature</p>
        <p style="font-size: 0.875rem; color: var(--text-secondary); margin-top: 0.25rem;">
          Signed by: {signerName}
        </p>
        {#if result.fingerprint}
          <p style="font-size: 0.75rem; color: var(--text-secondary); font-family: var(--font-mono);">
            {result.fingerprint}
          </p>
        {/if}
      </div>
    {:else}
      <div style="padding: 1rem; border-radius: 0.5rem; border: 1px solid var(--danger); background: rgba(239, 68, 68, 0.1);">
        <p class="error" style="font-weight: 600;">Verification Failed</p>
        <p style="font-size: 0.875rem; color: var(--text-secondary); margin-top: 0.25rem;">
          The signer's key may not be in your keyring.
        </p>
      </div>
    {/if}
  {/if}
</div>
