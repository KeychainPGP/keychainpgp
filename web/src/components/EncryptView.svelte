<script lang="ts">
  import { encrypt } from "../lib/wasm";
  import { listKeys, type StoredKey } from "../lib/keystore";

  let plaintext = $state("");
  let output = $state("");
  let error = $state("");
  let keys: StoredKey[] = $state([]);
  let selected: Set<string> = $state(new Set());

  async function loadKeys() {
    keys = await listKeys();
  }

  loadKeys();

  function toggleKey(fp: string) {
    if (selected.has(fp)) {
      selected.delete(fp);
    } else {
      selected.add(fp);
    }
    selected = new Set(selected);
  }

  function handleEncrypt() {
    error = "";
    output = "";
    if (!plaintext.trim()) {
      error = "Enter a message to encrypt.";
      return;
    }
    if (selected.size === 0) {
      error = "Select at least one recipient key.";
      return;
    }

    try {
      const recipientKeys = keys
        .filter((k) => selected.has(k.fingerprint))
        .map((k) => k.publicKey);
      output = encrypt(plaintext, recipientKeys);
    } catch (e) {
      error = String(e);
    }
  }

  async function copyOutput() {
    await navigator.clipboard.writeText(output);
  }
</script>

<div class="card" style="display: flex; flex-direction: column; gap: 1rem;">
  <h2 style="font-size: 1rem; font-weight: 600;">Encrypt Message</h2>

  <textarea
    class="textarea"
    placeholder="Type your message here..."
    bind:value={plaintext}
    rows="5"
  ></textarea>

  <div>
    <p style="font-size: 0.875rem; font-weight: 500; margin-bottom: 0.5rem;">Recipients</p>
    {#if keys.length === 0}
      <p style="font-size: 0.875rem; color: var(--text-secondary);">No keys available. Add keys in the Keys tab.</p>
    {:else}
      <div style="display: flex; flex-direction: column; gap: 0.375rem; max-height: 200px; overflow-y: auto;">
        {#each keys as key (key.fingerprint)}
          <label style="display: flex; align-items: center; gap: 0.5rem; padding: 0.5rem; border-radius: 0.375rem; border: 1px solid var(--border); cursor: pointer; font-size: 0.875rem;"
            class:selected={selected.has(key.fingerprint)}
          >
            <input
              type="checkbox"
              checked={selected.has(key.fingerprint)}
              onchange={() => toggleKey(key.fingerprint)}
            />
            <span>{key.name ?? key.email ?? key.fingerprint.slice(-16)}</span>
            {#if key.isOwn}
              <span style="font-size: 0.75rem; color: var(--primary);">(own)</span>
            {/if}
          </label>
        {/each}
      </div>
    {/if}
  </div>

  <button class="btn btn-primary" onclick={handleEncrypt} disabled={!plaintext.trim() || selected.size === 0}>
    Encrypt
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

<style>
  .selected {
    border-color: var(--primary) !important;
    background: rgba(59, 130, 246, 0.1);
  }
</style>
