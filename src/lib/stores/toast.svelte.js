/** @type {Array<{id: number, message: string, type: string}>} */
let toasts = $state([]);
let nextId = 0;

function show(message, type = "info", duration = 3000) {
  const id = nextId++;
  toasts = [...toasts, { id, message, type }];
  if (duration > 0) {
    setTimeout(() => dismiss(id), duration);
  }
}

function dismiss(id) {
  toasts = toasts.filter(t => t.id !== id);
}

export const toastStore = {
  get toasts() { return toasts; },
  show,
  dismiss,
  success: (msg) => show(msg, "success", 3000),
  error: (msg) => show(msg, "error", 5000),
  info: (msg) => show(msg, "info", 3000),
  warning: (msg) => show(msg, "warning", 4000),
};
