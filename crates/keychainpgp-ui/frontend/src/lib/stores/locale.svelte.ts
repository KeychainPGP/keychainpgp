import {
  getLocale,
  setLocale as paraglideSetLocale,
  overwriteGetLocale,
  isLocale,
  locales,
  baseLocale,
  extractLocaleFromNavigator,
} from "$lib/paraglide/runtime.js";

type Locale = (typeof locales)[number];

const RTL_LOCALES: string[] = ["ar", "he"];

let current: Locale = $state(baseLocale as Locale);

function applyLocale(tag: Locale) {
  current = tag;
  document.documentElement.lang = tag;
  document.documentElement.dir = RTL_LOCALES.includes(tag) ? "rtl" : "ltr";
}

/**
 * Resolve the "auto" setting to an actual locale tag.
 * Priority: navigator.languages exact match → base tag match → "en"
 */
function resolveAuto(): Locale {
  const detected = extractLocaleFromNavigator();
  if (detected && isLocale(detected)) {
    return detected as Locale;
  }
  return baseLocale as Locale;
}

/**
 * Initialize locale on app startup.
 * @param savedLocale The locale from persisted settings ("auto" or a specific tag)
 */
export function initLocale(savedLocale: string) {
  const tag = savedLocale === "auto" ? resolveAuto() : savedLocale;
  const resolved = isLocale(tag) ? (tag as Locale) : (baseLocale as Locale);

  // Overwrite paraglide's getLocale to always return our reactive value
  overwriteGetLocale(() => current);

  // Set the locale in paraglide (cookie, globalVariable, etc.) without reload
  paraglideSetLocale(resolved, { reload: false });
  applyLocale(resolved);
}

/**
 * Change locale at runtime (e.g., from the settings dropdown).
 * @param tag The locale tag to switch to, or "auto" for OS detection
 */
export function changeLocale(tag: string) {
  const resolved = tag === "auto" ? resolveAuto() : tag;
  if (!isLocale(resolved)) return;
  const locale = resolved as Locale;

  paraglideSetLocale(locale, { reload: false });
  applyLocale(locale);
}

export const localeStore = {
  get current() { return current; },
  get locales() { return locales; },
};
