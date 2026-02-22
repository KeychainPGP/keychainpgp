<script lang="ts">
  import { generateKeyPair, inspectKey } from "../lib/wasm";
  import { listKeys, storeKey, deleteKey, type StoredKey } from "../lib/keystore";
  import { t } from "../lib/i18n.svelte";

  let keys: StoredKey[] = $state([]);
  let showGenerate = $state(false);
  let showImport = $state(false);

  // Generate form
  let genName = $state("");
  let genEmail = $state("");
  let genPassphrase = $state("");
  let genError = $state("");
  let generating = $state(false);

  // Import form
  let importData = $state("");
  let importError = $state("");
  let importing = $state(false);

  let statusMsg = $state("");

  async function refresh() {
    keys = await listKeys();
  }

  refresh();

  async function handleGenerate() {
    genError = "";
    if (!genName.trim() || !genEmail.trim()) {
      genError = t("keygen_error_required");
      return;
    }
    generating = true;
    try {
      const kp = generateKeyPair(genName, genEmail, genPassphrase || undefined);
      const info = inspectKey(kp.public_key);
      await storeKey(
        kp.fingerprint,
        info.user_ids[0]?.name ?? null,
        info.user_ids[0]?.email ?? null,
        kp.public_key,
        kp.secret_key,
      );
      await refresh();
      showGenerate = false;
      genName = "";
      genEmail = "";
      genPassphrase = "";
      statusMsg = t("keygen_success");
      setTimeout(() => (statusMsg = ""), 3000);
    } catch (e) {
      genError = String(e);
    } finally {
      generating = false;
    }
  }

  async function handleImport() {
    importError = "";
    if (!importData.trim()) {
      importError = t("import_error_empty");
      return;
    }
    importing = true;
    try {
      const info = inspectKey(importData);
      const name = info.user_ids[0]?.name ?? null;
      const email = info.user_ids[0]?.email ?? null;
      const secretKey = info.has_secret_key ? importData : null;
      await storeKey(info.fingerprint, name, email, importData, secretKey);
      await refresh();
      showImport = false;
      importData = "";
      statusMsg = t("import_success", { name: name ?? email ?? info.fingerprint.slice(-16) });
      setTimeout(() => (statusMsg = ""), 3000);
    } catch (e) {
      importError = String(e);
    } finally {
      importing = false;
    }
  }

  async function handleDelete(fp: string) {
    await deleteKey(fp);
    await refresh();
    statusMsg = t("key_deleted");
    setTimeout(() => (statusMsg = ""), 3000);
  }

  async function handleExport(key: StoredKey) {
    await navigator.clipboard.writeText(key.publicKey);
    statusMsg = t("key_exported");
    setTimeout(() => (statusMsg = ""), 3000);
  }
</script>

<div class="card" style="display: flex; flex-direction: column; gap: 1rem;">
  <div style="display: flex; align-items: center; justify-content: space-between;">
    <h2 style="font-size: 1rem; font-weight: 600;">{t("keys_title")}</h2>
    <div style="display: flex; gap: 0.5rem;">
      <button class="btn btn-primary" onclick={() => { showGenerate = !showGenerate; showImport = false; }}>
        {t("keys_generate_btn")}
      </button>
      <button class="btn" onclick={() => { showImport = !showImport; showGenerate = false; }}>
        {t("keys_import_btn")}
      </button>
    </div>
  </div>

  {#if statusMsg}
    <p class="success">{statusMsg}</p>
  {/if}

  {#if showGenerate}
    <div style="display: flex; flex-direction: column; gap: 0.75rem; padding: 1rem; border: 1px solid var(--color-border); border-radius: 0.5rem;">
      <p style="font-size: 0.875rem; font-weight: 500;">{t("keygen_title")}</p>
      <input class="input" placeholder={t("keygen_name")} bind:value={genName} />
      <input class="input" placeholder={t("keygen_email")} bind:value={genEmail} />
      <input class="input" type="password" placeholder={t("keygen_passphrase")} bind:value={genPassphrase} />
      {#if genError}
        <p class="error">{genError}</p>
      {/if}
      <div style="display: flex; gap: 0.5rem; justify-content: flex-end;">
        <button class="btn" onclick={() => showGenerate = false}>{t("keygen_cancel")}</button>
        <button class="btn btn-primary" onclick={handleGenerate} disabled={generating}>
          {generating ? t("keygen_loading") : t("keygen_submit")}
        </button>
      </div>
    </div>
  {/if}

  {#if showImport}
    <div style="display: flex; flex-direction: column; gap: 0.75rem; padding: 1rem; border: 1px solid var(--color-border); border-radius: 0.5rem;">
      <p style="font-size: 0.875rem; font-weight: 500;">{t("import_title")}</p>
      <textarea class="textarea" placeholder={t("import_placeholder")} bind:value={importData} rows="6"></textarea>
      {#if importError}
        <p class="error">{importError}</p>
      {/if}
      <div style="display: flex; gap: 0.5rem; justify-content: flex-end;">
        <button class="btn" onclick={() => showImport = false}>{t("import_cancel")}</button>
        <button class="btn btn-primary" onclick={handleImport} disabled={importing}>
          {importing ? t("import_loading") : t("import_submit")}
        </button>
      </div>
    </div>
  {/if}

  {#if keys.length === 0}
    <p style="font-size: 0.875rem; color: var(--color-text-secondary); text-align: center; padding: 2rem 0;">
      {t("keys_empty")}
    </p>
  {:else}
    <div style="display: flex; flex-direction: column; gap: 0.5rem;">
      {#each keys as key (key.fingerprint)}
        <div class="key-row" style="display: flex; align-items: center; justify-content: space-between; padding: 0.75rem; border: 1px solid var(--color-border); border-radius: 0.5rem;">
          <div style="min-width: 0; flex: 1;">
            <p style="font-size: 0.875rem; font-weight: 500;">
              {key.name ?? t("keys_unnamed")}
              {#if key.isOwn}
                <span style="font-size: 0.75rem; color: var(--color-primary); font-weight: normal;">{t("keys_own_label")}</span>
              {/if}
            </p>
            <p style="font-size: 0.75rem; color: var(--color-text-secondary);">
              {key.email ?? ""}
            </p>
            <p style="font-size: 0.625rem; color: var(--color-text-secondary); font-family: var(--color-font-mono);">
              {key.fingerprint.slice(-16)}
            </p>
          </div>
          <div class="key-actions" style="display: flex; gap: 0.375rem; flex-shrink: 0;">
            <button class="btn" style="font-size: 0.75rem; padding: 0.25rem 0.5rem;" onclick={() => handleExport(key)}>
              {t("key_export_btn")}
            </button>
            <button class="btn btn-danger" style="font-size: 0.75rem; padding: 0.25rem 0.5rem;" onclick={() => handleDelete(key.fingerprint)}>
              {t("key_delete_btn")}
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  @media (max-width: 480px) {
    .key-row {
      flex-direction: column !important;
      align-items: flex-start !important;
      gap: 0.5rem;
    }
    .key-actions {
      align-self: flex-end;
    }
  }
</style>
