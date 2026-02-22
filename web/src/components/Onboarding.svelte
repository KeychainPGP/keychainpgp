<script lang="ts">
  import { completeOnboarding } from "../lib/preferences";
  import { t } from "../lib/i18n.svelte";

  let { onDismiss }: { onDismiss: () => void } = $props();

  function dismiss() {
    completeOnboarding();
    onDismiss();
  }
</script>

<div class="card onboarding-banner">
  <h2>{t("onboarding_title")}</h2>
  <p>{t("onboarding_desc")}</p>
  <button class="btn btn-primary" onclick={dismiss}>{t("onboarding_dismiss")}</button>
</div>

<style>
  .onboarding-banner {
    margin-bottom: 1.5rem;
    text-align: center;
    border-color: var(--color-primary);
    animation: fadeIn 0.3s ease-in;
  }
  .onboarding-banner h2 {
    font-size: 1.125rem;
    font-weight: 600;
    margin-bottom: 0.5rem;
  }
  .onboarding-banner p {
    font-size: 0.875rem;
    color: var(--color-text-secondary);
    margin-bottom: 1rem;
    line-height: 1.6;
  }
  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(-8px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
