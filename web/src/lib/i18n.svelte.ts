/**
 * Svelte 5 reactive wrapper for i18n.
 *
 * Components import `t` from this module and call `t("key")` in templates.
 * When the locale changes, `version` is bumped, causing all `t()` calls to
 * re-evaluate.
 */
import {
  t as _t,
  setLocale as _setLocale,
  detectLocale,
  getLocale as _getLocale,
  LOCALES,
} from "./i18n";
import type { Locale, MessageKey } from "./i18n-types";

let version = $state(0);

export async function initI18n(): Promise<void> {
  const locale = detectLocale();
  await _setLocale(locale);
  version++;
}

export async function setLocale(locale: Locale): Promise<void> {
  await _setLocale(locale);
  version++;
}

export function getLocale(): Locale {
  void version;
  return _getLocale();
}

export function t(key: MessageKey, params?: Record<string, string | number>): string {
  void version;
  return _t(key, params);
}

export { LOCALES };
export type { Locale };
