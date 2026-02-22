export type View = "home" | "keys" | "settings";
export type InputMode = "clipboard" | "compose";

export type ModalType =
  | "recipient-selector"
  | "decrypted-viewer"
  | "passphrase"
  | "key-import"
  | "key-details"
  | "confirm"
  | "error"
  | "verify-result"
  | "qr-export"
  | "key-discovery"
  | "key-sync-export"
  | "key-sync-import"
  | "donate";

export interface ModalProps {
  /** For decrypted-viewer */
  plaintext?: string;
  /** For key-details */
  fingerprint?: string;
  /** For confirm dialog */
  title?: string;
  message?: string;
  onConfirm?: () => void;
  /** For error dialog */
  error?: string;
  suggestion?: string;
  /** For passphrase dialog */
  onSubmit?: (passphrase: string) => void;
  /** For verify-result modal */
  verifyResult?: import("$lib/tauri").VerifyResultInfo;
  /** For recipient-selector: text to encrypt (compose mode) */
  text?: string;
}

export const TRUST_LABELS: Record<number, string> = {
  0: "Unknown",
  1: "Imported",
  2: "Verified",
};

export const TRUST_COLORS: Record<number, string> = {
  0: "text-gray-400",
  1: "text-yellow-500",
  2: "text-green-500",
};
