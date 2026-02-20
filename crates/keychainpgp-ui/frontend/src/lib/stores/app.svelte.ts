import type { View, ModalType, ModalProps } from "$lib/types";

export type AppAction = "encrypt" | "decrypt" | "sign" | "verify" | null;

let currentView: View = $state("home");
let activeModal: ModalType | null = $state(null);
let modalProps: ModalProps = $state({});
let statusMessage: string = $state("Ready");
let statusTimeout: ReturnType<typeof setTimeout> | null = null;
let pendingAction: AppAction = $state(null);

export const appStore = {
  get currentView() { return currentView; },
  set currentView(v: View) { currentView = v; },

  get activeModal() { return activeModal; },
  get modalProps() { return modalProps; },

  get statusMessage() { return statusMessage; },

  get pendingAction() { return pendingAction; },

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
        statusMessage = "Ready";
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
