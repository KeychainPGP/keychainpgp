<script lang="ts">
  import { sign } from "../lib/wasm";
  import { listKeys, getSecretKey } from "../lib/keystore";
  import { t } from "../lib/i18n.svelte";

  let message = $state("");
  let passphrase = $state("");
  let output = $state("");
  let error = $state("");

  async function handleSign() {
    error = "";
    output = "";
    if (!message.trim()) {
      error = t("sign_error_empty");
      return;
    }

    const keys = await listKeys();
    const ownKeys = keys.filter((k) => k.isOwn);

    if (ownKeys.length === 0) {
      error = t("sign_error_no_key");
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

    error = t("sign_error_failed");
  }

  async function copyOutput() {
    await navigator.clipboard.writeText(output);
  }
</script>

<div class="card" style="display: flex; flex-direction: column; gap: 1rem;">
  <h2 style="font-size: 1rem; font-weight: 600;">{t("sign_title")}</h2>

  <textarea
    class="textarea"
    placeholder={t("sign_placeholder")}
    bind:value={message}
    rows="5"
  ></textarea>

  <input
    type="password"
    class="input"
    placeholder={t("sign_passphrase")}
    bind:value={passphrase}
  />

  <button class="btn btn-primary" onclick={handleSign} disabled={!message.trim()}>
    {t("sign_btn")}
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
      >{t("copy_btn")}</button>
    </div>
  {/if}
</div>
