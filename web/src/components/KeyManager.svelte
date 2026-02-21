<script lang="ts">
  import { generateKeyPair, inspectKey } from "../lib/wasm";
  import { listKeys, storeKey, deleteKey, type StoredKey } from "../lib/keystore";

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
      genError = "Name and email are required.";
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
      statusMsg = "Key pair generated!";
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
      importError = "Paste a PGP public or private key.";
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
      statusMsg = `Imported key for ${name ?? email ?? info.fingerprint.slice(-16)}`;
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
    statusMsg = "Key deleted.";
    setTimeout(() => (statusMsg = ""), 3000);
  }

  async function handleExport(key: StoredKey) {
    await navigator.clipboard.writeText(key.publicKey);
    statusMsg = "Public key copied to clipboard.";
    setTimeout(() => (statusMsg = ""), 3000);
  }
</script>

<div class="card" style="display: flex; flex-direction: column; gap: 1rem;">
  <div style="display: flex; align-items: center; justify-content: space-between;">
    <h2 style="font-size: 1rem; font-weight: 600;">Key Manager</h2>
    <div style="display: flex; gap: 0.5rem;">
      <button class="btn btn-primary" onclick={() => { showGenerate = !showGenerate; showImport = false; }}>
        Generate
      </button>
      <button class="btn" onclick={() => { showImport = !showImport; showGenerate = false; }}>
        Import
      </button>
    </div>
  </div>

  {#if statusMsg}
    <p class="success">{statusMsg}</p>
  {/if}

  {#if showGenerate}
    <div style="display: flex; flex-direction: column; gap: 0.75rem; padding: 1rem; border: 1px solid var(--border); border-radius: 0.5rem;">
      <p style="font-size: 0.875rem; font-weight: 500;">Generate New Key Pair</p>
      <input class="input" placeholder="Name" bind:value={genName} />
      <input class="input" placeholder="Email" bind:value={genEmail} />
      <input class="input" type="password" placeholder="Passphrase (optional)" bind:value={genPassphrase} />
      {#if genError}
        <p class="error">{genError}</p>
      {/if}
      <div style="display: flex; gap: 0.5rem; justify-content: flex-end;">
        <button class="btn" onclick={() => showGenerate = false}>Cancel</button>
        <button class="btn btn-primary" onclick={handleGenerate} disabled={generating}>
          {generating ? "Generating..." : "Generate"}
        </button>
      </div>
    </div>
  {/if}

  {#if showImport}
    <div style="display: flex; flex-direction: column; gap: 0.75rem; padding: 1rem; border: 1px solid var(--border); border-radius: 0.5rem;">
      <p style="font-size: 0.875rem; font-weight: 500;">Import Key</p>
      <textarea class="textarea" placeholder="Paste ASCII-armored PGP key..." bind:value={importData} rows="6"></textarea>
      {#if importError}
        <p class="error">{importError}</p>
      {/if}
      <div style="display: flex; gap: 0.5rem; justify-content: flex-end;">
        <button class="btn" onclick={() => showImport = false}>Cancel</button>
        <button class="btn btn-primary" onclick={handleImport} disabled={importing}>
          {importing ? "Importing..." : "Import"}
        </button>
      </div>
    </div>
  {/if}

  {#if keys.length === 0}
    <p style="font-size: 0.875rem; color: var(--text-secondary); text-align: center; padding: 2rem 0;">
      No keys yet. Generate or import one to get started.
    </p>
  {:else}
    <div style="display: flex; flex-direction: column; gap: 0.5rem;">
      {#each keys as key (key.fingerprint)}
        <div style="display: flex; align-items: center; justify-content: space-between; padding: 0.75rem; border: 1px solid var(--border); border-radius: 0.5rem;">
          <div style="min-width: 0; flex: 1;">
            <p style="font-size: 0.875rem; font-weight: 500;">
              {key.name ?? "(unnamed)"}
              {#if key.isOwn}
                <span style="font-size: 0.75rem; color: var(--primary); font-weight: normal;">(own key)</span>
              {/if}
            </p>
            <p style="font-size: 0.75rem; color: var(--text-secondary);">
              {key.email ?? ""}
            </p>
            <p style="font-size: 0.625rem; color: var(--text-secondary); font-family: var(--font-mono);">
              {key.fingerprint.slice(-16)}
            </p>
          </div>
          <div style="display: flex; gap: 0.375rem; flex-shrink: 0;">
            <button class="btn" style="font-size: 0.75rem; padding: 0.25rem 0.5rem;" onclick={() => handleExport(key)}>
              Export
            </button>
            <button class="btn btn-danger" style="font-size: 0.75rem; padding: 0.25rem 0.5rem;" onclick={() => handleDelete(key.fingerprint)}>
              Delete
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
