<script lang="ts">
  import { verify } from "../lib/wasm";
  import { listKeys } from "../lib/keystore";
  import { t } from "../lib/i18n.svelte";

  let signedMessage = $state("");
  let result = $state<{ valid: boolean; fingerprint: string | null } | null>(null);
  let signerName = $state("");
  let error = $state("");

  async function handleVerify() {
    error = "";
    result = null;
    signerName = "";

    if (!signedMessage.trim()) {
      error = t("verify_error_empty");
      return;
    }

    const keys = await listKeys();
    if (keys.length === 0) {
      error = t("verify_error_no_keys");
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
  <h2 style="font-size: 1rem; font-weight: 600;">{t("verify_title")}</h2>

  <textarea
    class="textarea"
    placeholder={t("verify_placeholder")}
    bind:value={signedMessage}
    rows="8"
  ></textarea>

  <button class="btn btn-primary" onclick={handleVerify} disabled={!signedMessage.trim()}>
    {t("verify_btn")}
  </button>

  {#if error}
    <p class="error">{error}</p>
  {/if}

  {#if result}
    {#if result.valid}
      <div style="padding: 1rem; border-radius: 0.5rem; border: 1px solid var(--color-success); background: rgba(34, 197, 94, 0.1);">
        <p class="success" style="font-weight: 600;">{t("verify_valid")}</p>
        <p style="font-size: 0.875rem; color: var(--color-text-secondary); margin-top: 0.25rem;">
          {t("verify_signed_by", { name: signerName })}
        </p>
        {#if result.fingerprint}
          <p style="font-size: 0.75rem; color: var(--color-text-secondary); font-family: var(--color-font-mono);">
            {result.fingerprint}
          </p>
        {/if}
      </div>
    {:else}
      <div style="padding: 1rem; border-radius: 0.5rem; border: 1px solid var(--color-danger); background: rgba(239, 68, 68, 0.1);">
        <p class="error" style="font-weight: 600;">{t("verify_failed")}</p>
        <p style="font-size: 0.875rem; color: var(--color-text-secondary); margin-top: 0.25rem;">
          {t("verify_signer_not_found")}
        </p>
      </div>
    {/if}
  {/if}
</div>
