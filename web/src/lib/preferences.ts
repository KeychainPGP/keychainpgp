const PREFIX = "keychainpgp-";

export function getPref(key: string): string | null {
  return localStorage.getItem(PREFIX + key);
}

export function setPref(key: string, value: string): void {
  localStorage.setItem(PREFIX + key, value);
}

export function hasCompletedOnboarding(): boolean {
  return getPref("onboarded") === "true";
}

export function completeOnboarding(): void {
  setPref("onboarded", "true");
}
