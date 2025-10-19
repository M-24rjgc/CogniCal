import { useCallback, useEffect, useMemo, useRef } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';
import {
  getFirstOnboardingStepId,
  ONBOARDING_REPLAY_EVENT,
  ONBOARDING_TOUR_STEPS,
  getOnboardingStepById,
  OnboardingErrorType,
  type OnboardingStepDefinition,
  type OnboardingStepId,
} from '../../utils/onboarding';
import { useOnboardingStore } from '../../stores/onboardingStore';
import { pushToast } from '../../stores/uiStore';

type DriverFactory = (config?: DriverConfig) => DriverInstance;

interface DriverEventContext {
  config: Record<string, unknown>;
  state: Record<string, unknown>;
  driver: DriverInstance;
}

type DriverEventHandler = (
  element: Element | undefined,
  step: DriverStep | undefined,
  context: DriverEventContext,
) => void | Promise<void>;

interface DriverConfig extends Record<string, unknown> {
  steps: DriverStep[];
  allowClose?: boolean;
  animate?: boolean;
  overlayClickBehavior?: 'close' | 'none' | 'nextStep';
  closeBtnText?: string;
  doneBtnText?: string;
  nextBtnText?: string;
  prevBtnText?: string;
  overlayOpacity?: number;
  showProgress?: boolean;
  showButtons?: string[];
  progressText?: string;
  onHighlighted?: DriverEventHandler;
  onHighlightStarted?: DriverEventHandler;
  onDeselected?: DriverEventHandler;
  onDestroyStarted?: DriverEventHandler;
  onDestroyed?: DriverEventHandler;
  onNextClick?: DriverEventHandler;
  onPrevClick?: DriverEventHandler;
  onCloseClick?: DriverEventHandler;
}

interface DriverInstance {
  drive: (startIndex?: number) => void;
  destroy: () => void;
  getActiveIndex: () => number | undefined;
  isLastStep: () => boolean;
  setConfig: (config: DriverConfig) => void;
  setSteps: (steps: DriverStep[]) => void;
  refresh?: () => void;
}

interface DriverStep {
  element: string;
  popover?: {
    title?: string;
    description?: string;
    position?: string;
    nextBtnText?: string;
    prevBtnText?: string;
    doneBtnText?: string;
  };
  onboardingStepId: OnboardingStepId;
}

type CleanupReason = 'completed' | 'dismissed' | 'forced';

let driverCssLoaded = false;

const DRIVER_TEXT = {
  close: '跳过',
  done: '完成',
  next: '下一步',
  prev: '上一步',
} as const;

const DRIVER_BASE_OPTIONS: Omit<DriverConfig, 'steps'> = {
  allowClose: true,
  overlayClickBehavior: 'close',
  closeBtnText: DRIVER_TEXT.close,
  doneBtnText: DRIVER_TEXT.done,
  nextBtnText: DRIVER_TEXT.next,
  prevBtnText: DRIVER_TEXT.prev,
  overlayOpacity: 0.35,
  showProgress: true,
  showButtons: ['next', 'previous', 'close'],
};

const handleOnboardingError = (errorType: OnboardingErrorType, stepId?: OnboardingStepId) => {
  switch (errorType) {
    case OnboardingErrorType.ELEMENT_NOT_FOUND:
      console.warn(`[onboarding] Element not found for step: ${stepId}`);
      pushToast({
        title: '引导目标未找到',
        description: '请确认页面已加载相关模块，或稍后重试。',
        variant: 'warning',
      });
      break;
    case OnboardingErrorType.DRIVER_LOAD_FAILED:
      console.error('[onboarding] Driver.js failed to load');
      pushToast({
        title: '引导模块加载失败',
        description: '请刷新页面后重试，或联系技术支持。',
        variant: 'error',
      });
      break;
    case OnboardingErrorType.NAVIGATION_FAILED:
      console.warn(`[onboarding] Navigation failed for step: ${stepId}`);
      pushToast({
        title: '页面导航失败',
        description: '无法跳转到引导目标页面，请手动导航后重试。',
        variant: 'warning',
      });
      break;
    case OnboardingErrorType.STATE_CORRUPTION:
      console.error('[onboarding] State corruption detected');
      pushToast({
        title: '引导状态异常',
        description: '引导状态出现异常，已自动重置。',
        variant: 'error',
      });
      break;
  }
};

const loadDriver = async (): Promise<DriverFactory | null> => {
  if (typeof window === 'undefined') {
    return null;
  }

  try {
    const [driverModule] = await Promise.all([
      import('driver.js'),
      driverCssLoaded ? Promise.resolve() : import('driver.js/dist/driver.css'),
    ]);

    if (!driverCssLoaded) {
      driverCssLoaded = true;
    }

    const factoryCandidate =
      typeof driverModule.driver === 'function'
        ? driverModule.driver
        : typeof driverModule.default === 'function'
          ? driverModule.default
          : null;

    return (factoryCandidate as DriverFactory | null) ?? null;
  } catch (error) {
    console.error('[onboarding] Failed to load driver.js', error);
    handleOnboardingError(OnboardingErrorType.DRIVER_LOAD_FAILED);
    return null;
  }
};

const asDriverStep = (definition: OnboardingStepDefinition): DriverStep => {
  const elementAvailable =
    typeof document !== 'undefined' && Boolean(document.querySelector(definition.selector));

  if (!elementAvailable) {
    console.warn('[onboarding] Missing element for step', definition.id);
  }

  return {
    element: definition.selector,
    onboardingStepId: definition.id,
    popover: {
      title: definition.title,
      description: definition.description,
      position: definition.placement ?? 'auto',
      nextBtnText: DRIVER_TEXT.next,
      prevBtnText: DRIVER_TEXT.prev,
      doneBtnText: DRIVER_TEXT.done,
    },
  };
};

const buildTourFrom = (startStepId: OnboardingStepId): DriverStep[] | null => {
  const startIndex = ONBOARDING_TOUR_STEPS.findIndex((step) => step.id === startStepId);
  if (startIndex < 0) {
    return null;
  }

  const orderedSteps = ONBOARDING_TOUR_STEPS.slice(startIndex);
  if (orderedSteps.length === 0) {
    return null;
  }

  return orderedSteps.map(asDriverStep);
};

const OnboardingOrchestrator = () => {
  const location = useLocation();
  const navigate = useNavigate();

  const progress = useOnboardingStore((state) => state.progress);
  const markStepComplete = useOnboardingStore((state) => state.markStepComplete);
  const setHasCompletedTour = useOnboardingStore((state) => state.setHasCompletedTour);
  const recordDismissal = useOnboardingStore((state) => state.recordDismissal);
  const consumeReplayRequest = useOnboardingStore((state) => state.consumeReplayRequest);

  const driverInstanceRef = useRef<DriverInstance | null>(null);
  const driverFactoryRef = useRef<DriverFactory | null>(null);
  const driverLoaderRef = useRef<Promise<DriverFactory | null> | null>(null);
  const isRunningRef = useRef(false);
  const activeStepIdRef = useRef<OnboardingStepId | null>(null);
  const completionRef = useRef(false);
  const teardownReasonRef = useRef<CleanupReason | null>(null);
  const missingStepReportRef = useRef(new Set<OnboardingStepId>());
  const tourLockRef = useRef<Promise<void> | null>(null);
  const navigationCancelRef = useRef<(() => void) | null>(null);

  const waitForElement = useCallback(
    async (
      selector: string,
      timeout = 5000,
      interval = 100,
      retryCount = 2,
    ): Promise<Element | null> => {
      if (typeof document === 'undefined' || typeof window === 'undefined') {
        return null;
      }

      for (let attempt = 0; attempt <= retryCount; attempt++) {
        const existing = document.querySelector(selector);
        if (existing) {
          return existing;
        }

        const startedAt = Date.now();

        const result = await new Promise<Element | null>((resolve) => {
          let timer: number | null = null;

          const clear = () => {
            if (timer !== null) {
              window.clearInterval(timer);
              timer = null;
            }
          };

          const check = () => {
            const found = document.querySelector(selector);
            if (found) {
              clear();
              resolve(found);
              return;
            }

            if (Date.now() - startedAt >= timeout) {
              clear();
              resolve(null);
            }
          };

          timer = window.setInterval(check, interval);
          check();
        });

        if (result) {
          return result;
        }

        if (attempt < retryCount) {
          await new Promise((resolve) => setTimeout(resolve, 1000)); // 重试间隔
        }
      }

      handleOnboardingError(OnboardingErrorType.ELEMENT_NOT_FOUND);
      return null;
    },
    [],
  );

  const ensureStepContext = useCallback(
    async (stepId: OnboardingStepId) => {
      if (typeof document === 'undefined') {
        return;
      }

      // 取消之前的导航
      if (navigationCancelRef.current) {
        navigationCancelRef.current();
      }

      let cancelled = false;
      navigationCancelRef.current = () => {
        cancelled = true;
      };

      try {
        switch (stepId) {
          case 'dashboard-overview': {
            if (location.pathname !== '/') {
              navigate('/');
            }
            if (cancelled) return;
            await waitForElement('[data-onboarding="dashboard-overview"]', 5000);
            break;
          }
          case 'task-quick-create': {
            const selector =
              '[data-onboarding="task-quick-create"], [data-action-id="create-task"]';
            if (!document.querySelector(selector)) {
              if (!location.pathname.startsWith('/tasks')) {
                navigate('/tasks');
              }
              if (cancelled) return;
              await waitForElement(selector, 5000);
            }
            break;
          }
          case 'ai-parse-panel': {
            const selector = '[data-onboarding="ai-parse-panel"]';
            if (!document.querySelector(selector)) {
              navigate('/tasks', { state: { intent: 'create-task', source: 'onboarding' } });
              if (cancelled) return;
              await waitForElement(selector, 5000);
            }
            break;
          }
          case 'planning-center': {
            const selector = '[data-onboarding="planning-center"]';
            if (!document.querySelector(selector)) {
              navigate('/tasks', { state: { intent: 'open-planning', source: 'onboarding' } });
              if (cancelled) return;
              await waitForElement(selector, 5000);
            }
            break;
          }
          case 'settings-api-key': {
            const selector = '[data-onboarding="settings-api-key"]';
            if (!document.querySelector(selector)) {
              if (!location.pathname.startsWith('/settings')) {
                navigate('/settings');
              }
              if (cancelled) return;
              await waitForElement(selector, 5000);
            }
            break;
          }
          default:
            break;
        }
      } finally {
        if (navigationCancelRef.current && !cancelled) {
          navigationCancelRef.current = null;
        }
      }
    },
    [location.pathname, navigate, waitForElement],
  );

  const shouldAutoLaunch = useMemo(() => {
    if (progress.hasCompletedTour) {
      return false;
    }
    return !progress.dismissedAt;
  }, [progress.dismissedAt, progress.hasCompletedTour]);

  const pendingStepId = useMemo<OnboardingStepId | null>(() => {
    if (progress.hasCompletedTour) {
      return null;
    }

    const completed = new Set(progress.completedStepIds);
    for (const step of ONBOARDING_TOUR_STEPS) {
      if (!completed.has(step.id)) {
        return step.id;
      }
    }

    return null;
  }, [progress.completedStepIds, progress.hasCompletedTour]);

  const cleanupDriver = useCallback(
    (reason?: CleanupReason) => {
      const instance = driverInstanceRef.current;
      const wasRunning = isRunningRef.current;

      const finalReason = reason ?? teardownReasonRef.current ?? null;

      if (!instance && !wasRunning && !reason && finalReason === teardownReasonRef.current) {
        return;
      }

      if (finalReason === 'completed') {
        const state = useOnboardingStore.getState().progress;
        if (!state.hasCompletedTour) {
          setHasCompletedTour(true);
        }
      } else if (finalReason === 'dismissed') {
        recordDismissal(activeStepIdRef.current ?? undefined);
      }

      driverInstanceRef.current = null;
      activeStepIdRef.current = null;
      isRunningRef.current = false;
      completionRef.current = false;
      teardownReasonRef.current = finalReason;
      missingStepReportRef.current.clear();
    },
    [recordDismissal, setHasCompletedTour],
  );

  const forceDestroyDriver = useCallback(() => {
    const instance = driverInstanceRef.current;
    if (!instance) {
      return;
    }

    teardownReasonRef.current = 'forced';
    try {
      instance.destroy();
    } catch (error) {
      console.warn('[onboarding] Failed to force destroy driver instance', error);
      cleanupDriver('forced');
    }
  }, [cleanupDriver]);

  const ensureDriverFactory = useCallback(async () => {
    if (driverFactoryRef.current) {
      return driverFactoryRef.current;
    }

    if (!driverLoaderRef.current) {
      driverLoaderRef.current = loadDriver();
    }

    const factory = await driverLoaderRef.current;
    driverLoaderRef.current = null;
    driverFactoryRef.current = factory;
    return factory;
  }, []);

  const startTour = useCallback(
    async (stepId: OnboardingStepId | null, options?: { forceRestart?: boolean }) => {
      if (!stepId) {
        return;
      }

      // 异步锁机制，防止并发启动
      if (tourLockRef.current) {
        await tourLockRef.current;
      }

      const tourPromise = (async () => {
        if (isRunningRef.current) {
          if (options?.forceRestart) {
            forceDestroyDriver();
          } else {
            return;
          }
        }

        const factory = await ensureDriverFactory();
        if (!factory) {
          pushToast({
            title: '引导暂不可用',
            description: '加载引导模块失败，可稍后在帮助中心重试。',
            variant: 'warning',
          });
          return;
        }

        const steps = buildTourFrom(stepId);
        if (!steps || steps.length === 0) {
          return;
        }

        missingStepReportRef.current.clear();
        await ensureStepContext(stepId);

        teardownReasonRef.current = null;
        completionRef.current = useOnboardingStore.getState().progress.hasCompletedTour;

        const config: DriverConfig = {
          ...DRIVER_BASE_OPTIONS,
          steps,
          onHighlightStarted: async (_element, step, context) => {
            teardownReasonRef.current = null;

            const onboardingStep = step as DriverStep | undefined;
            const stepIdFromEvent = onboardingStep?.onboardingStepId;

            if (!stepIdFromEvent) {
              return;
            }

            await ensureStepContext(stepIdFromEvent);

            const definition = getOnboardingStepById(stepIdFromEvent);
            const selector = definition?.selector;
            const preparedElement =
              selector && typeof document !== 'undefined' ? document.querySelector(selector) : null;

            if (!preparedElement) {
              if (!missingStepReportRef.current.has(stepIdFromEvent)) {
                missingStepReportRef.current.add(stepIdFromEvent);
                pushToast({
                  title: '暂未定位到引导目标',
                  description: '请确认页面已加载相关模块，或稍后在帮助中心重新播放引导。',
                  variant: 'warning',
                });
              }
              return;
            }

            if (missingStepReportRef.current.has(stepIdFromEvent)) {
              missingStepReportRef.current.delete(stepIdFromEvent);
            }

            context.driver.refresh?.();
          },
          onHighlighted: (_element, step, context) => {
            const onboardingStep = step as DriverStep | undefined;
            const stepIdFromEvent = onboardingStep?.onboardingStepId;
            const activeIndex = context.driver.getActiveIndex();
            const fallbackStepId =
              typeof activeIndex === 'number' ? steps[activeIndex]?.onboardingStepId : undefined;
            const resolvedStepId = stepIdFromEvent ?? fallbackStepId;

            if (resolvedStepId) {
              activeStepIdRef.current = resolvedStepId;
              markStepComplete(resolvedStepId);
              const state = useOnboardingStore.getState().progress;
              completionRef.current = state.hasCompletedTour;
            }
          },
          onDestroyStarted: () => {
            if (teardownReasonRef.current === 'forced') {
              return;
            }

            const state = useOnboardingStore.getState().progress;
            teardownReasonRef.current = state.hasCompletedTour ? 'completed' : 'dismissed';
          },
          onDestroyed: () => {
            const reason =
              teardownReasonRef.current ?? (completionRef.current ? 'completed' : undefined);
            cleanupDriver(reason);
          },
          onCloseClick: (_element, _step, context) => {
            if (teardownReasonRef.current !== 'forced') {
              teardownReasonRef.current = 'dismissed';
            }
            try {
              context.driver.destroy();
            } catch (error) {
              console.warn('[onboarding] Failed to destroy driver on close click', error);
              cleanupDriver('forced');
            }
          },
        };

        try {
          const instance = factory(config);
          driverInstanceRef.current = instance;
          isRunningRef.current = true;
          instance.setSteps?.(steps);
          instance.drive();
        } catch (error) {
          console.error('[onboarding] Failed to start tour', error);
          teardownReasonRef.current = 'forced';
          cleanupDriver('forced');
          pushToast({
            title: '引导启动失败',
            description: '系统已记录错误，可稍后在帮助中心重新播放。',
            variant: 'error',
          });
        }
      })();

      tourLockRef.current = tourPromise;
      try {
        await tourPromise;
      } finally {
        if (tourLockRef.current === tourPromise) {
          tourLockRef.current = null;
        }
      }
    },
    [cleanupDriver, ensureDriverFactory, ensureStepContext, forceDestroyDriver, markStepComplete],
  );

  useEffect(() => {
    const handleReplay = () => {
      const token = consumeReplayRequest();
      if (!token) {
        return;
      }

      startTour(getFirstOnboardingStepId(), { forceRestart: true }).catch((error) => {
        console.error('[onboarding] Failed to replay tour', error);
      });
    };

    const handleUserInteraction = (event: Event) => {
      if (!isRunningRef.current) return;

      // 检查是否是引导相关的交互
      const target = event.target as Element;
      if (target.closest('.driver-popover')) return;

      // 处理非引导交互 - 可以在这里添加日志或警告
      console.warn('[onboarding] User interaction during tour detected');
    };

    window.addEventListener(ONBOARDING_REPLAY_EVENT, handleReplay);
    document.addEventListener('click', handleUserInteraction, { capture: true });

    return () => {
      window.removeEventListener(ONBOARDING_REPLAY_EVENT, handleReplay);
      document.removeEventListener('click', handleUserInteraction, { capture: true });
      forceDestroyDriver();
    };
  }, [consumeReplayRequest, forceDestroyDriver, startTour]);

  useEffect(() => {
    if (!shouldAutoLaunch) {
      return;
    }

    startTour(pendingStepId).catch((error) => {
      console.error('[onboarding] Failed to auto launch tour', error);
    });
  }, [location.key, pendingStepId, shouldAutoLaunch, startTour]);

  return null;
};

export default OnboardingOrchestrator;
