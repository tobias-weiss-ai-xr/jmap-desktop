/**
 * Toast notification system — provides user-visible feedback for errors and success.
 */

export type ToastType = 'error' | 'success' | 'info';

export interface Toast {
  id: number;
  message: string;
  type: ToastType;
  expiresAt: number;
}

let toasts: Toast[] = [];
let listeners: Array<() => void> = [];
let nextId = 0;

export function addToast(message: string, type: ToastType = 'info', durationMs: number = 5000): number {
  const id = nextId++;
  const toast: Toast = { id, message, type, expiresAt: Date.now() + durationMs };
  toasts.push(toast);
  notifyListeners();

  if (durationMs > 0) {
    setTimeout(() => removeToast(id), durationMs);
  }

  return id;
}

export function addError(message: string, durationMs?: number): number {
  return addToast(message, 'error', durationMs);
}

export function addSuccess(message: string, durationMs?: number): number {
  return addToast(message, 'success', durationMs);
}

export function removeToast(id: number) {
  toasts = toasts.filter((t) => t.id !== id);
  notifyListeners();
}

export function getToasts(): readonly Toast[] {
  return toasts.filter((t) => t.expiresAt > Date.now());
}

export function subscribe(listener: () => void): () => void {
  listeners.push(listener);
  return () => {
    listeners = listeners.filter((l) => l !== listener);
  };
}

function notifyListeners() {
  for (const fn of listeners) fn();
}
