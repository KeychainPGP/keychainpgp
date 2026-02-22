<script lang="ts">
  import { initWasm } from "./lib/wasm";
  import EncryptView from "./components/EncryptView.svelte";
  import DecryptView from "./components/DecryptView.svelte";
  import SignView from "./components/SignView.svelte";
  import VerifyView from "./components/VerifyView.svelte";
  import KeyManager from "./components/KeyManager.svelte";

  let ready = $state(false);
  let error = $state("");
  let tab: "encrypt" | "decrypt" | "sign" | "verify" | "keys" = $state("encrypt");

  async function load() {
    try {
      await initWasm();
      ready = true;
    } catch (e) {
      error = `Failed to load WASM module: ${e}`;
    }
  }

  load();
</script>

<header style="text-align: center; margin-bottom: 2rem;">
  <h1 style="font-size: 1.5rem; font-weight: 700; display: inline-flex; align-items: center; gap: 0.5rem; justify-content: center;">
    <img src="./icon.png" alt="" style="width: 32px; height: 32px; border-radius: 6px;" />
    KeychainPGP <span style="color: var(--text-secondary); font-weight: 400; font-size: 0.875rem;">Web</span>
  </h1>
  <p style="color: var(--text-secondary); font-size: 0.875rem; margin-top: 0.25rem;">
    Browser-based PGP encryption powered by WebAssembly
  </p>
</header>

{#if error}
  <div class="card" style="text-align: center;">
    <p class="error">{error}</p>
  </div>
{:else if !ready}
  <div class="card" style="text-align: center;">
    <p style="color: var(--text-secondary);">Loading cryptographic engine...</p>
  </div>
{:else}
  <div class="tabs">
    <button class="tab" class:active={tab === "encrypt"} onclick={() => tab = "encrypt"}>Encrypt</button>
    <button class="tab" class:active={tab === "decrypt"} onclick={() => tab = "decrypt"}>Decrypt</button>
    <button class="tab" class:active={tab === "sign"} onclick={() => tab = "sign"}>Sign</button>
    <button class="tab" class:active={tab === "verify"} onclick={() => tab = "verify"}>Verify</button>
    <button class="tab" class:active={tab === "keys"} onclick={() => tab = "keys"}>Keys</button>
  </div>

  {#if tab === "encrypt"}
    <EncryptView />
  {:else if tab === "decrypt"}
    <DecryptView />
  {:else if tab === "sign"}
    <SignView />
  {:else if tab === "verify"}
    <VerifyView />
  {:else if tab === "keys"}
    <KeyManager />
  {/if}
{/if}

<footer style="text-align: center; margin-top: 3rem; color: var(--text-secondary); font-size: 0.75rem;">
  <p>All cryptography runs locally in your browser via WebAssembly. No data is sent to any server.</p>
</footer>
