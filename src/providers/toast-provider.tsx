import { createContext, useCallback, useContext } from 'react';
import { ToastViewport } from '../components/common/ToastProvider';
import { useUIStore, type ToastVariant } from '../stores/uiStore';

interface ToastPayload {
  title: string;
  description?: string;
  variant?: ToastVariant;
  duration?: number;
}

interface ToastContextValue {
  notify: (toast: ToastPayload) => void;
}

const ToastContext = createContext<ToastContextValue>({
  notify: () => undefined,
});

export function ToastProvider({ children }: { children: React.ReactNode }) {
  const pushToast = useUIStore((state) => state.pushToast);

  const notify = useCallback(
    (toast: ToastPayload) => {
      pushToast({
        title: toast.title,
        description: toast.description,
        variant: toast.variant,
        duration: toast.duration,
      });
    },
    [pushToast],
  );

  return (
    <ToastContext.Provider value={{ notify }}>
      {children}
      <ToastViewport />
    </ToastContext.Provider>
  );
}

export function useToast() {
  return useContext(ToastContext);
}
