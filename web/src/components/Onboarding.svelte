<script lang="ts">
  import { completeOnboarding } from "../lib/preferences";
  import { t } from "../lib/i18n.svelte";
  import { generateKeyPair, encrypt, decrypt, inspectKey } from "../lib/wasm";
  import { storeKey, getSecretKey } from "../lib/keystore";

  let { onDismiss }: { onDismiss: () => void } = $props();

  const TOTAL = 4;
  let step = $state(0);

  /* Step 1 — key generation */
  let name = $state("");
  let email = $state("");
  let passphrase = $state("");
  let genError = $state("");
  let generating = $state(false);
  let keyGenerated = $state(false);
  let publicKey = $state("");
  let fingerprint = $state("");

  /* Step 2 — encrypt / decrypt demo */
  let encryptedMsg = $state("");
  let decryptedMsg = $state("");
  let decrypting = $state(false);
  let decryptError = $state("");

  function skip() {
    completeOnboarding();
    onDismiss();
  }

  function finish() {
    completeOnboarding();
    onDismiss();
  }

  function prev() {
    if (step > 0) step--;
  }

  function next() {
    if (step === 1 && !keyGenerated) return;
    if (step < TOTAL - 1) step++;
  }

  async function handleGenerate() {
    genError = "";
    if (!name.trim() || !email.trim()) {
      genError = t("keygen_error_required");
      return;
    }
    generating = true;
    try {
      const kp = generateKeyPair(name, email, passphrase || undefined);
      const info = inspectKey(kp.public_key);
      await storeKey(
        kp.fingerprint,
        info.user_ids[0]?.name ?? null,
        info.user_ids[0]?.email ?? null,
        kp.public_key,
        kp.secret_key,
      );
      publicKey = kp.public_key;
      fingerprint = kp.fingerprint;
      keyGenerated = true;

      /* Pre-encrypt the demo message with the user's new public key */
      const msg = t("onboarding_secret_message");
      encryptedMsg = encrypt(msg, [kp.public_key]);
    } catch (e) {
      genError = String(e);
    } finally {
      generating = false;
    }
  }

  async function handleDecrypt() {
    decryptError = "";
    decrypting = true;
    let sk: Uint8Array | null = null;
    try {
      sk = await getSecretKey(fingerprint);
      if (!sk) {
        decryptError = "Could not retrieve secret key.";
        return;
      }
      const skStr = new TextDecoder().decode(sk);
      decryptedMsg = decrypt(encryptedMsg, skStr, passphrase || undefined);
    } catch (e) {
      decryptError = String(e);
    } finally {
      sk?.fill(0);
      decrypting = false;
    }
  }
</script>

<div class="card wizard">
  <!-- Header: dots + skip -->
  <div class="wizard-header">
    <div class="wizard-dots">
      {#each Array(TOTAL) as _, i}
        <span
          class="wizard-dot"
          class:active={i === step}
          class:completed={i < step}
        ></span>
      {/each}
    </div>
    <button class="wizard-skip" onclick={skip}>{t("onboarding_skip")}</button>
  </div>

  <!-- Step content -->
  <div class="wizard-content">
    {#if step === 0}
      <!-- Welcome -->
      <h2>{t("onboarding_welcome_title")}</h2>
      <p>{t("onboarding_welcome_desc")}</p>

    {:else if step === 1}
      <!-- Generate key -->
      <h2>{t("onboarding_gen_title")}</h2>
      <p>{t("onboarding_gen_desc")}</p>

      {#if keyGenerated}
        <p class="success">{t("onboarding_gen_done")}</p>
      {:else}
        <div class="wizard-form">
          <input class="input" placeholder={t("keygen_name")} bind:value={name} />
          <input class="input" placeholder={t("keygen_email")} type="email" bind:value={email} />
          <input class="input" placeholder={t("keygen_passphrase")} type="password" bind:value={passphrase} />
          {#if genError}
            <p class="error">{genError}</p>
          {/if}
          <button class="btn btn-primary" onclick={handleGenerate} disabled={generating}>
            {generating ? t("onboarding_gen_generating") : t("onboarding_gen_btn")}
          </button>
        </div>
      {/if}

    {:else if step === 2}
      <!-- Decrypt demo -->
      <h2>{t("onboarding_msg_title")}</h2>
      <p>{t("onboarding_msg_desc")}</p>

      <div class="pgp-block">{encryptedMsg}</div>

      {#if decryptedMsg}
        <p class="success" style="margin-top: 0.75rem;">{t("onboarding_msg_decrypted")}</p>
        <div class="decrypted-block">{decryptedMsg}</div>
      {:else}
        {#if decryptError}
          <p class="error" style="margin-top: 0.5rem;">{decryptError}</p>
        {/if}
        <button
          class="btn btn-primary"
          style="margin-top: 0.75rem;"
          onclick={handleDecrypt}
          disabled={decrypting}
        >
          {t("onboarding_msg_decrypt_btn")}
        </button>
      {/if}

      <p class="wizard-tip">{t("onboarding_msg_tip")}</p>

    {:else if step === 3}
      <!-- Done -->
      <h2>{t("onboarding_done_title")}</h2>
      <p>{t("onboarding_done_desc")}</p>
    {/if}
  </div>

  <!-- Footer: navigation -->
  <div class="wizard-footer">
    {#if step > 0}
      <button class="btn" onclick={prev}>{t("onboarding_prev")}</button>
    {:else}
      <div></div>
    {/if}

    {#if step < TOTAL - 1}
      <button
        class="btn btn-primary"
        onclick={next}
        disabled={step === 1 && !keyGenerated}
      >
        {t("onboarding_next")}
      </button>
    {:else}
      <button class="btn btn-primary" onclick={finish}>{t("onboarding_finish")}</button>
    {/if}
  </div>
</div>

<style>
  .wizard {
    margin-bottom: 1.5rem;
    border-color: var(--color-primary);
    animation: fadeIn 0.3s ease-in;
  }

  .wizard-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.25rem;
  }

  .wizard-dots {
    display: flex;
    gap: 0.5rem;
  }

  .wizard-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: var(--color-border);
    transition: background 0.2s;
  }

  .wizard-dot.active {
    background: var(--color-primary);
  }

  .wizard-dot.completed {
    background: var(--color-success);
  }

  .wizard-skip {
    background: none;
    border: none;
    color: var(--color-text-secondary);
    font-size: 0.8125rem;
    cursor: pointer;
    padding: 0.25rem 0.5rem;
    border-radius: 0.25rem;
    transition: color 0.15s;
  }

  .wizard-skip:hover {
    color: var(--color-text);
  }

  .wizard-content {
    min-height: 180px;
  }

  .wizard-content h2 {
    font-size: 1.125rem;
    font-weight: 600;
    margin-bottom: 0.75rem;
  }

  .wizard-content p {
    font-size: 0.875rem;
    color: var(--color-text-secondary);
    line-height: 1.6;
    margin-bottom: 0.75rem;
  }

  .wizard-form {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    margin-top: 0.5rem;
  }

  .pgp-block {
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    border-radius: 0.5rem;
    padding: 0.75rem;
    font-family: var(--color-font-mono);
    font-size: 0.6875rem;
    max-height: 140px;
    overflow-y: auto;
    white-space: pre-wrap;
    word-break: break-all;
    color: var(--color-text-secondary);
    line-height: 1.4;
  }

  .decrypted-block {
    background: var(--color-bg);
    border: 1px solid var(--color-success);
    border-radius: 0.5rem;
    padding: 0.75rem;
    font-size: 0.875rem;
    white-space: pre-wrap;
    color: var(--color-text);
    line-height: 1.6;
    margin-top: 0.5rem;
  }

  .wizard-tip {
    margin-top: 1rem;
    padding: 0.625rem 0.75rem;
    background: var(--color-bg);
    border-radius: 0.5rem;
    border-left: 3px solid var(--color-primary);
    font-size: 0.8125rem !important;
    color: var(--color-text-secondary);
  }

  .wizard-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-top: 1.25rem;
    padding-top: 1rem;
    border-top: 1px solid var(--color-border);
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(-8px); }
    to { opacity: 1; transform: translateY(0); }
  }

  @media (max-width: 480px) {
    .wizard-content {
      min-height: 160px;
    }
    .pgp-block {
      max-height: 100px;
      font-size: 0.625rem;
    }
  }
</style>
