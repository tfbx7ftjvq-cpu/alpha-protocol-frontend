import { useEffect, useRef, useState } from 'react';

const TURNSTILE_SCRIPT_ID = 'alpha-protocol-turnstile-script';
const TURNSTILE_SCRIPT_URL =
  'https://challenges.cloudflare.com/turnstile/v0/api.js?render=explicit';

type TurnstileWidgetId = string;
type ChallengeStatus = 'loading' | 'ready' | 'verified' | 'error';

interface TurnstileApi {
  render: (
    container: HTMLElement,
    options: {
      sitekey: string;
      action: string;
      theme: 'dark';
      size: 'flexible';
      appearance: 'always';
      execution: 'render';
      retry: 'auto';
      'refresh-expired': 'auto';
      'refresh-timeout': 'auto';
      'response-field': false;
      callback: (token: string) => void;
      'error-callback': () => void;
      'expired-callback': () => void;
      'timeout-callback': () => void;
    },
  ) => TurnstileWidgetId;
  getResponse: (widgetId: TurnstileWidgetId) => string;
  remove: (widgetId: TurnstileWidgetId) => void;
  reset: (widgetId: TurnstileWidgetId) => void;
}

declare global {
  interface Window {
    turnstile?: TurnstileApi;
  }
}

interface TurnstileChallengeProps {
  siteKey: string;
  action?: string;
  resetKey: number;
  onToken: (token: string | null) => void;
  onError: (message: string | null) => void;
}

let turnstileLoader: Promise<TurnstileApi> | null = null;

export default function TurnstileChallenge({
  siteKey,
  action = 'operations_wallet_auth',
  resetKey,
  onToken,
  onError,
}: TurnstileChallengeProps) {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const widgetIdRef = useRef<TurnstileWidgetId | null>(null);
  const onTokenRef = useRef(onToken);
  const onErrorRef = useRef(onError);
  const [status, setStatus] = useState<ChallengeStatus>('loading');

  onTokenRef.current = onToken;
  onErrorRef.current = onError;

  useEffect(() => {
    let cancelled = false;
    let renderedContainer: HTMLDivElement | null = null;

    setStatus('loading');
    onTokenRef.current(null);
    onErrorRef.current(null);

    void loadTurnstile()
      .then((turnstile) => {
        if (cancelled || !containerRef.current) {
          return;
        }

        renderedContainer = containerRef.current;
        const widgetId = turnstile.render(renderedContainer, {
          sitekey: siteKey,
          action,
          theme: 'dark',
          size: 'flexible',
          appearance: 'always',
          execution: 'render',
          retry: 'auto',
          'refresh-expired': 'auto',
          'refresh-timeout': 'auto',
          'response-field': false,
          callback: (token) => {
            setStatus('verified');
            onErrorRef.current(null);
            onTokenRef.current(token);
          },
          'error-callback': () => invalidateChallenge(
            'Turnstile 验证加载失败，请稍后重试',
          ),
          'expired-callback': () => invalidateChallenge(
            'Turnstile 验证已过期，请重新验证',
          ),
          'timeout-callback': () => invalidateChallenge(
            'Turnstile 验证超时，请重新验证',
          ),
        });
        widgetIdRef.current = widgetId;
        renderedContainer.dataset.turnstileWidgetId = widgetId;
        setStatus('ready');
      })
      .catch(() => {
        if (!cancelled) {
          setStatus('error');
          onTokenRef.current(null);
          onErrorRef.current('Turnstile 脚本加载失败，请检查网络后重试');
        }
      });

    return () => {
      cancelled = true;
      renderedContainer?.removeAttribute('data-turnstile-widget-id');
      const widgetId = widgetIdRef.current;
      widgetIdRef.current = null;
      if (widgetId && window.turnstile) {
        window.turnstile.remove(widgetId);
      }
    };

    function invalidateChallenge(message: string) {
      if (cancelled) {
        return;
      }
      setStatus('error');
      onTokenRef.current(null);
      onErrorRef.current(message);
    }
  }, [action, siteKey]);

  useEffect(() => {
    const widgetId = widgetIdRef.current;
    if (!widgetId || !window.turnstile) {
      return;
    }

    window.turnstile.reset(widgetId);
    setStatus('ready');
    onTokenRef.current(null);
    onErrorRef.current(null);
  }, [resetKey]);

  return (
    <div className="space-y-2">
      <div
        ref={containerRef}
        className="min-h-[65px] w-full overflow-hidden rounded border border-zinc-800 bg-zinc-950"
      />
      <p className="text-[10px] text-zinc-600" role="status" aria-live="polite">
        {challengeStatusLabel(status)}
      </p>
    </div>
  );
}

function challengeStatusLabel(status: ChallengeStatus): string {
  switch (status) {
    case 'loading':
      return '正在加载 Turnstile 安全验证…';
    case 'verified':
      return 'Turnstile 验证已完成；验证结果仅用于本次钱包认证。';
    case 'error':
      return 'Turnstile 验证当前不可用。';
    default:
      return '请完成 Turnstile 安全验证后再签名。';
  }
}

function loadTurnstile(): Promise<TurnstileApi> {
  if (window.turnstile) {
    return Promise.resolve(window.turnstile);
  }

  if (turnstileLoader) {
    return turnstileLoader;
  }

  turnstileLoader = new Promise<TurnstileApi>((resolve, reject) => {
    let script = document.getElementById(TURNSTILE_SCRIPT_ID) as HTMLScriptElement | null;

    const handleLoad = () => {
      cleanupListeners();
      if (window.turnstile) {
        resolve(window.turnstile);
        return;
      }
      turnstileLoader = null;
      reject(new Error('Turnstile API unavailable after script load'));
    };

    const handleError = () => {
      cleanupListeners();
      script?.remove();
      turnstileLoader = null;
      reject(new Error('Turnstile script failed to load'));
    };

    const cleanupListeners = () => {
      script?.removeEventListener('load', handleLoad);
      script?.removeEventListener('error', handleError);
    };

    if (!script) {
      script = document.createElement('script');
      script.id = TURNSTILE_SCRIPT_ID;
      script.src = TURNSTILE_SCRIPT_URL;
      script.async = true;
      script.defer = true;
      document.head.appendChild(script);
    }

    script.addEventListener('load', handleLoad, { once: true });
    script.addEventListener('error', handleError, { once: true });
  });

  return turnstileLoader;
}
