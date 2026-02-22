import type { Locale, MessageKey, Messages } from "./i18n-types";
import en from "../locales/en.json";

export interface LocaleInfo {
  code: Locale;
  name: string;
  dir: "ltr" | "rtl";
}

export const LOCALES: LocaleInfo[] = [
  { code: "en", name: "English", dir: "ltr" },
  { code: "fr", name: "Fran\u00e7ais", dir: "ltr" },
  { code: "de", name: "Deutsch", dir: "ltr" },
  { code: "es", name: "Espa\u00f1ol", dir: "ltr" },
  { code: "it", name: "Italiano", dir: "ltr" },
  { code: "nl", name: "Nederlands", dir: "ltr" },
  { code: "pt-BR", name: "Portugu\u00eas (Brasil)", dir: "ltr" },
  { code: "pt-PT", name: "Portugu\u00eas (Portugal)", dir: "ltr" },
  { code: "ru", name: "\u0420\u0443\u0441\u0441\u043a\u0438\u0439", dir: "ltr" },
  { code: "uk", name: "\u0423\u043a\u0440\u0430\u0457\u043d\u0441\u044c\u043a\u0430", dir: "ltr" },
  { code: "pl", name: "Polski", dir: "ltr" },
  { code: "tr", name: "T\u00fcrk\u00e7e", dir: "ltr" },
  { code: "zh-CN", name: "\u7b80\u4f53\u4e2d\u6587", dir: "ltr" },
  { code: "zh-TW", name: "\u7e41\u9ad4\u4e2d\u6587", dir: "ltr" },
  { code: "ja", name: "\u65e5\u672c\u8a9e", dir: "ltr" },
  { code: "ko", name: "\ud55c\uad6d\uc5b4", dir: "ltr" },
  { code: "ar", name: "\u0627\u0644\u0639\u0631\u0628\u064a\u0629", dir: "rtl" },
  { code: "he", name: "\u05e2\u05d1\u05e8\u05d9\u05ea", dir: "rtl" },
  { code: "hi", name: "\u0939\u093f\u0928\u094d\u0926\u0940", dir: "ltr" },
  { code: "th", name: "\u0e44\u0e17\u0e22", dir: "ltr" },
];

const STORAGE_KEY = "keychainpgp-locale";
const fallback = en as Messages;
const cache = new Map<string, Messages>();
cache.set("en", fallback);

let currentMessages: Messages = fallback;
let currentLocale: Locale = "en";

/** Explicit locale loader map for Vite code-splitting. */
const loaders: Record<string, () => Promise<{ default: Messages }>> = {
  fr: () => import("../locales/fr.json") as Promise<{ default: Messages }>,
  de: () => import("../locales/de.json") as Promise<{ default: Messages }>,
  es: () => import("../locales/es.json") as Promise<{ default: Messages }>,
  it: () => import("../locales/it.json") as Promise<{ default: Messages }>,
  nl: () => import("../locales/nl.json") as Promise<{ default: Messages }>,
  "pt-BR": () => import("../locales/pt-BR.json") as Promise<{ default: Messages }>,
  "pt-PT": () => import("../locales/pt-PT.json") as Promise<{ default: Messages }>,
  ru: () => import("../locales/ru.json") as Promise<{ default: Messages }>,
  uk: () => import("../locales/uk.json") as Promise<{ default: Messages }>,
  pl: () => import("../locales/pl.json") as Promise<{ default: Messages }>,
  tr: () => import("../locales/tr.json") as Promise<{ default: Messages }>,
  "zh-CN": () => import("../locales/zh-CN.json") as Promise<{ default: Messages }>,
  "zh-TW": () => import("../locales/zh-TW.json") as Promise<{ default: Messages }>,
  ja: () => import("../locales/ja.json") as Promise<{ default: Messages }>,
  ko: () => import("../locales/ko.json") as Promise<{ default: Messages }>,
  ar: () => import("../locales/ar.json") as Promise<{ default: Messages }>,
  he: () => import("../locales/he.json") as Promise<{ default: Messages }>,
  hi: () => import("../locales/hi.json") as Promise<{ default: Messages }>,
  th: () => import("../locales/th.json") as Promise<{ default: Messages }>,
};

export function getLocale(): Locale {
  return currentLocale;
}

export function detectLocale(): Locale {
  const stored = localStorage.getItem(STORAGE_KEY);
  if (stored && LOCALES.some((l) => l.code === stored)) return stored as Locale;

  const lang = navigator.language;
  const exact = LOCALES.find((l) => l.code === lang);
  if (exact) return exact.code;
  const prefix = LOCALES.find((l) => lang.startsWith(l.code.split("-")[0]!));
  if (prefix) return prefix.code;
  return "en";
}

export async function setLocale(locale: Locale): Promise<void> {
  localStorage.setItem(STORAGE_KEY, locale);

  if (!cache.has(locale)) {
    const loader = loaders[locale];
    if (loader) {
      const mod = await loader();
      cache.set(locale, mod.default);
    }
  }

  currentMessages = cache.get(locale) ?? fallback;
  currentLocale = locale;

  const info = LOCALES.find((l) => l.code === locale);
  document.documentElement.setAttribute("dir", info?.dir ?? "ltr");
  document.documentElement.setAttribute("lang", locale);
}

export function t(key: MessageKey, params?: Record<string, string | number>): string {
  let msg = currentMessages[key] ?? fallback[key] ?? key;
  if (params) {
    for (const [k, v] of Object.entries(params)) {
      msg = msg.replace(new RegExp(`\\{${k}\\}`, "g"), String(v));
    }
  }
  return msg;
}
