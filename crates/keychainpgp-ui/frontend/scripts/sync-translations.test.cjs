/**
 * Tests for sync-translations.cjs
 *
 * Run: node scripts/sync-translations.test.cjs
 */

const fs = require("fs");
const path = require("path");
const { execSync } = require("child_process");

const MESSAGES_DIR = path.join(__dirname, "..", "messages");
const SCRIPT = path.join(__dirname, "sync-translations.cjs");

let passed = 0;
let failed = 0;

function assert(condition, name) {
  if (condition) {
    console.log(`  PASS: ${name}`);
    passed++;
  } else {
    console.log(`  FAIL: ${name}`);
    failed++;
  }
}

function readJson(file) {
  return JSON.parse(fs.readFileSync(path.join(MESSAGES_DIR, file), "utf-8"));
}

function writeJson(file, data) {
  fs.writeFileSync(
    path.join(MESSAGES_DIR, file),
    JSON.stringify(data, null, 2) + "\n",
    "utf-8"
  );
}

function run() {
  return execSync(`node "${SCRIPT}"`, { encoding: "utf-8" });
}

// Backup originals
const enBackup = readJson("en.json");
const frBackup = readJson("fr.json");

try {
  // --- Test 1: New key in en.json is propagated to all locales ---
  console.log("Test 1: Missing key added to all locales");
  const en = readJson("en.json");
  en["__test_new_key"] = "Test value";
  writeJson("en.json", en);

  run();

  const fr1 = readJson("fr.json");
  const de1 = readJson("de.json");
  assert(fr1["__test_new_key"] === "Test value", "key added to fr.json");
  assert(de1["__test_new_key"] === "Test value", "key added to de.json");

  // --- Test 2: Stale key in locale is removed ---
  console.log("Test 2: Stale key removed from locale");
  const fr2 = readJson("fr.json");
  fr2["__stale_key"] = "Should be removed";
  writeJson("fr.json", fr2);

  run();

  const fr3 = readJson("fr.json");
  assert(!("__stale_key" in fr3), "stale key removed from fr.json");

  // --- Test 3: Existing translations are preserved ---
  console.log("Test 3: Existing translations preserved");
  assert(fr3["loading"] === frBackup["loading"], "fr loading unchanged");

  // --- Test 4: Key order follows en.json ---
  console.log("Test 4: Key order follows en.json");
  const frKeys = Object.keys(fr3).filter((k) => !k.startsWith("$"));
  const enKeys = Object.keys(readJson("en.json")).filter((k) => !k.startsWith("$"));
  const frOrdered = enKeys.filter((k) => frKeys.includes(k));
  assert(
    JSON.stringify(frKeys) === JSON.stringify(frOrdered),
    "fr.json key order matches en.json"
  );

  // --- Test 5: Idempotent — running twice changes nothing ---
  console.log("Test 5: Idempotent run");
  const output = run();
  assert(
    output.includes("all locales are in sync"),
    "second run reports all in sync"
  );
} finally {
  // Restore originals
  writeJson("en.json", enBackup);
  writeJson("fr.json", frBackup);
  // Re-run to restore all locales
  run();
}

console.log(`\nResults: ${passed} passed, ${failed} failed`);
process.exit(failed > 0 ? 1 : 0);
