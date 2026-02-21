import type { View, ModalType, ModalProps, InputMode } from "$lib/types";
import * as m from "$lib/paraglide/messages.js";

export type AppAction = "encrypt" | "decrypt" | "sign" | "verify" | null;

let currentView: View = $state("home");
let activeModal: ModalType | null = $state(null);
let modalProps: ModalProps = $state({});
let statusMessage: string = $state(m.ready());
let statusTimeout: ReturnType<typeof setTimeout> | null = null;
let pendingAction: AppAction = $state(null);
let inputMode: InputMode = $state("clipboard");
let composeText: string = $state("");

export const appStore = {
  get currentView() { return currentView; },
  set currentView(v: View) { currentView = v; },

  get activeModal() { return activeModal; },
  get modalProps() { return modalProps; },

  get statusMessage() { return statusMessage; },

  get pendingAction() { return pendingAction; },

  get inputMode() { return inputMode; },
  set inputMode(v: InputMode) { inputMode = v; },

  get composeText() { return composeText; },
  set composeText(v: string) { composeText = v; },

  openModal(type: ModalType, props: ModalProps = {}) {
    activeModal = type;
    modalProps = props;
  },

  closeModal() {
    activeModal = null;
    modalProps = {};
  },

  setStatus(msg: string, durationMs = 5000) {
    statusMessage = msg;
    if (statusTimeout) clearTimeout(statusTimeout);
    if (durationMs > 0) {
      statusTimeout = setTimeout(() => {
        statusMessage = m.ready();
      }, durationMs);
    }
  },

  /** Dispatch an action (from hotkey, tray, etc.). HomeView will consume it. */
  dispatchAction(action: AppAction) {
    currentView = "home";
    pendingAction = action;
  },

  /** Clear the pending action after it has been consumed. */
  clearAction() {
    pendingAction = null;
  },
};
