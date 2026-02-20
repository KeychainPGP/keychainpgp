/** Format a full fingerprint as grouped hex: "ABCD 1234 ..." */
export function formatFingerprint(fp: string): string {
  return fp.replace(/(.{4})/g, "$1 ").trim();
}

/** Short fingerprint: last 8 chars. */
export function shortFingerprint(fp: string): string {
  return fp.slice(-8).toUpperCase();
}

/** Format an ISO 8601 date string to a readable date. */
export function formatDate(iso: string | null | undefined): string {
  if (!iso) return "N/A";
  const date = new Date(iso);
  if (isNaN(date.getTime())) return iso.split("T")[0] ?? iso;
  return date.toLocaleDateString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
  });
}

/** Truncate text to maxLen, appending "..." if truncated. */
export function truncate(text: string, maxLen: number): string {
  if (text.length <= maxLen) return text;
  return text.slice(0, maxLen) + "...";
}

/** Check if text looks like a PGP message. */
export function isPgpMessage(text: string): boolean {
  return text.trimStart().startsWith("-----BEGIN PGP MESSAGE-----");
}

/** Check if text looks like a PGP signed message. */
export function isPgpSignedMessage(text: string): boolean {
  return text.trimStart().startsWith("-----BEGIN PGP SIGNED MESSAGE-----");
}

/** Check if text looks like a PGP public key. */
export function isPgpPublicKey(text: string): boolean {
  return text.trimStart().startsWith("-----BEGIN PGP PUBLIC KEY BLOCK-----");
}
