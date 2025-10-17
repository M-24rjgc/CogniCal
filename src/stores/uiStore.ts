import { create } from 'zustand';
import { defaultErrorCopy, type AppError, type AppErrorCode } from '../services/tauriApi';

export type ToastVariant = 'default' | 'success' | 'error' | 'warning';

export interface Toast {
  id: string;
  title: string;
  description?: string;
  variant: ToastVariant;
  createdAt: number;
  duration: number;
}

interface ToastInput {
  id?: string;
  title: string;
  description?: string;
  variant?: ToastVariant;
  duration?: number;
}

interface UIStoreState {
  toasts: Toast[];
  pushToast: (toast: ToastInput) => string;
  dismissToast: (id: string) => void;
  clearToasts: () => void;
  notifySuccess: (message: string, description?: string) => void;
  notifyError: (error: AppError, fallback?: string) => void;
}

const DEFAULT_DURATION = 4000;

const createId = () =>
  typeof crypto !== 'undefined' && 'randomUUID' in crypto
    ? crypto.randomUUID()
    : `toast-${Math.random().toString(36).slice(2, 10)}`;

const errorCopy: Record<AppErrorCode, { title: string; description?: string }> = defaultErrorCopy;

export const useUIStore = create<UIStoreState>((set, get) => ({
  toasts: [],
  pushToast(toast) {
    const id = toast.id ?? createId();
    const variant = toast.variant ?? 'default';
    const duration = toast.duration ?? DEFAULT_DURATION;

    set((state) => ({
      toasts: [
        ...state.toasts,
        {
          id,
          title: toast.title,
          description: toast.description,
          variant,
          duration,
          createdAt: Date.now(),
        },
      ],
    }));

    return id;
  },
  dismissToast(id) {
    set((state) => ({ toasts: state.toasts.filter((toast) => toast.id !== id) }));
  },
  clearToasts() {
    set({ toasts: [] });
  },
  notifySuccess(message, description) {
    get().pushToast({
      title: message,
      description,
      variant: 'success',
    });
  },
  notifyError(error, fallback) {
    const copy = errorCopy[error.code] ?? errorCopy.UNKNOWN;

    const descriptions: string[] = [];

    if (error.details && typeof error.details === 'string') {
      descriptions.push(error.details);
    } else if (error.details && typeof error.details === 'object') {
      try {
        descriptions.push(JSON.stringify(error.details));
      } catch (jsonError) {
        console.debug('[uiStore] 无法序列化错误详情', jsonError);
      }
    } else if (error.message && error.message !== copy.title) {
      descriptions.push(error.message);
    } else if (fallback ?? copy.description) {
      descriptions.push(fallback ?? copy.description ?? '');
    }

    if (error.correlationId) {
      descriptions.push(`诊断 ID: ${error.correlationId}`);
    }

    const description = descriptions
      .map((value) => value?.toString().trim())
      .filter((value) => Boolean(value))
      .join('\n');

    get().pushToast({
      title: copy.title,
      description: description || undefined,
      variant: 'error',
    });
  },
}));

export const selectToasts = () => useUIStore.getState().toasts;
export const pushToast = (toast: ToastInput) => useUIStore.getState().pushToast(toast);
export const dismissToast = (id: string) => useUIStore.getState().dismissToast(id);
export const notifyErrorToast = (error: AppError, fallback?: string) =>
  useUIStore.getState().notifyError(error, fallback);
export const notifySuccessToast = (message: string, description?: string) =>
  useUIStore.getState().notifySuccess(message, description);
