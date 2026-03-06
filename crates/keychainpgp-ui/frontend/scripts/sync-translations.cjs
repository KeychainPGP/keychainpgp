/**
 * sync-translations.js
 *
 * Ensures all locale files have the same keys as en.json.
 * - Missing keys are filled with the English value as fallback.
 * - Stale keys (present in locale but not in en.json) are removed.
 *
 * Run: node scripts/sync-translations.js
 */

const fs = require("fs");
const path = require("path");

const MESSAGES_DIR = path.join(__dirname, "..", "messages");
const SOURCE_LOCALE = "en";
const SOURCE_FILE = path.join(MESSAGES_DIR, `${SOURCE_LOCALE}.json`);

const source = JSON.parse(fs.readFileSync(SOURCE_FILE, "utf-8"));
const sourceKeys = Object.keys(source).filter((k) => !k.startsWith("$"));

const localeFiles = fs
  .readdirSync(MESSAGES_DIR)
  .filter((f) => f.endsWith(".json") && f !== `${SOURCE_LOCALE}.json`);

let totalAdded = 0;
let totalRemoved = 0;

for (const file of localeFiles) {
  const filePath = path.join(MESSAGES_DIR, file);
  const locale = JSON.parse(fs.readFileSync(filePath, "utf-8"));
  const localeName = path.basename(file, ".json");

  let added = 0;
  let removed = 0;

  // Add missing keys with English fallback
  for (const key of sourceKeys) {
    if (!(key in locale)) {
      locale[key] = source[key];
      added++;
    }
  }

  // Remove stale keys not in en.json
  for (const key of Object.keys(locale)) {
    if (key.startsWith("$")) continue;
    if (!(key in source)) {
      delete locale[key];
      removed++;
    }
  }

  if (added > 0 || removed > 0) {
    // Rebuild object with $schema first, then keys in en.json order
    const ordered = {};
    if (locale["$schema"]) {
      ordered["$schema"] = locale["$schema"];
    }
    for (const key of sourceKeys) {
      if (key in locale) {
        ordered[key] = locale[key];
      }
    }

    fs.writeFileSync(filePath, JSON.stringify(ordered, null, 2) + "\n", "utf-8");
    console.log(`  ${localeName}: +${added} added, -${removed} removed`);
    totalAdded += added;
    totalRemoved += removed;
  }
}

if (totalAdded === 0 && totalRemoved === 0) {
  console.log("sync-translations: all locales are in sync.");
} else {
  console.log(
    `sync-translations: ${totalAdded} key(s) added, ${totalRemoved} key(s) removed across ${localeFiles.length} locales.`
  );
}
