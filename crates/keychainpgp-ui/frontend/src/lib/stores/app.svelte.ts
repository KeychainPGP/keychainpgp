import type { View, ModalType, ModalProps } from "$lib/types";

let currentView: View = $state("home");
let activeModal: ModalType | null = $state(null);
let modalProps: ModalProps = $state({});
let statusMessage: string = $state("Ready");
let statusTimeout: ReturnType<typeof setTimeout> | null = null;

export const appStore = {
  get currentView() { return currentView; },
  set currentView(v: View) { currentView = v; },

  get activeModal() { return activeModal; },
  get modalProps() { return modalProps; },

  get statusMessage() { return statusMessage; },

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
};
