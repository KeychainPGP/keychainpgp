<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  type View = "home" | "keys" | "settings";

  let currentView: View = $state("home");
  let clipboardContent: string | null = $state(null);
  let statusMessage: string = $state("Ready");

  async function readClipboard() {
    try {
      clipboardContent = await invoke("read_clipboard");
    } catch (e) {
      clipboardContent = null;
    }
  }

  async function handleEncrypt() {
    statusMessage = "Encrypting...";
    // TODO: open recipient selection dialog
    statusMessage = "Select recipients to encrypt (coming soon)";
  }

  async function handleDecrypt() {
    statusMessage = "Decrypting...";
    try {
      const result: any = await invoke("decrypt_clipboard", { passphrase: null });
      if (result.success) {
        statusMessage = result.message;
        // TODO: open decrypted message viewer
      } else {
        statusMessage = result.message;
      }
    } catch (e) {
      statusMessage = String(e);
    }
  }
</script>

<main class="flex flex-col h-screen">
  <!-- Navigation -->
  <nav class="flex border-b border-[var(--color-border)] px-4">
    <button
      class="px-4 py-3 text-sm font-medium transition-colors"
      class:text-[var(--color-primary)]={currentView === "home"}
      class:border-b-2={currentView === "home"}
      class:border-[var(--color-primary)]={currentView === "home"}
      onclick={() => (currentView = "home")}
    >
      Encrypt / Decrypt
    </button>
    <button
      class="px-4 py-3 text-sm font-medium transition-colors"
      class:text-[var(--color-primary)]={currentView === "keys"}
      class:border-b-2={currentView === "keys"}
      class:border-[var(--color-primary)]={currentView === "keys"}
      onclick={() => (currentView = "keys")}
    >
      Keys
    </button>
    <button
      class="px-4 py-3 text-sm font-medium transition-colors"
      class:text-[var(--color-primary)]={currentView === "settings"}
      class:border-b-2={currentView === "settings"}
      class:border-[var(--color-primary)]={currentView === "settings"}
      onclick={() => (currentView = "settings")}
    >
      Settings
    </button>
  </nav>

  <!-- Content -->
  <div class="flex-1 overflow-auto p-6">
    {#if currentView === "home"}
      <div class="max-w-2xl mx-auto space-y-6">
        <div class="text-center space-y-2">
          <h1 class="text-2xl font-bold">KeychainPGP</h1>
          <p class="text-[var(--color-text-secondary)]">
            Copy text, then click Encrypt or Decrypt.
          </p>
        </div>

        <!-- Clipboard preview -->
        <div
          class="rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-secondary)] p-4 min-h-32"
        >
          {#if clipboardContent}
            <p class="text-sm font-mono whitespace-pre-wrap break-all">
              {clipboardContent.slice(0, 500)}{clipboardContent.length > 500 ? "..." : ""}
            </p>
          {:else}
            <p class="text-[var(--color-text-secondary)] text-sm italic">
              Your clipboard content will appear here. Copy some text, then click Encrypt or
              Decrypt.
            </p>
          {/if}
        </div>

        <!-- Action buttons -->
        <div class="flex gap-4 justify-center">
          <button
            class="flex-1 max-w-48 py-4 rounded-lg bg-[var(--color-primary)] text-white font-semibold
                   hover:bg-[var(--color-primary-hover)] transition-colors"
            onclick={handleEncrypt}
          >
            ENCRYPT
            <span class="block text-xs font-normal opacity-75 mt-1">Ctrl+Shift+E</span>
          </button>
          <button
            class="flex-1 max-w-48 py-4 rounded-lg bg-[var(--color-primary)] text-white font-semibold
                   hover:bg-[var(--color-primary-hover)] transition-colors"
            onclick={handleDecrypt}
          >
            DECRYPT
            <span class="block text-xs font-normal opacity-75 mt-1">Ctrl+Shift+D</span>
          </button>
        </div>
      </div>
    {:else if currentView === "keys"}
      <div class="max-w-2xl mx-auto">
        <h2 class="text-xl font-bold mb-4">Key Manager</h2>
        <p class="text-[var(--color-text-secondary)]">
          Key management interface will be implemented here.
        </p>
      </div>
    {:else if currentView === "settings"}
      <div class="max-w-2xl mx-auto">
        <h2 class="text-xl font-bold mb-4">Settings</h2>
        <p class="text-[var(--color-text-secondary)]">
          Settings panel will be implemented here.
        </p>
      </div>
    {/if}
  </div>

  <!-- Status bar -->
  <footer class="border-t border-[var(--color-border)] px-4 py-2 text-xs text-[var(--color-text-secondary)] flex justify-between">
    <span>{statusMessage}</span>
    <span>Auto-clear: 30s</span>
  </footer>
</main>
