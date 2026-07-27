/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_MAINNET_RPC_ENDPOINT?: string;
  readonly VITE_ALPHA_LAUNCH_STATUS?: 'prelaunch' | 'live';
  readonly VITE_ALPHA_MAINNET_MINT?: string;
  readonly VITE_REVENUE_WALLET?: string;
  readonly VITE_ALPHA_PUMP_URL?: string;
  readonly VITE_REVENUE_DISTRIBUTION_THRESHOLD_USDC?: string;
  readonly VITE_SUPABASE_URL?: string;
  readonly VITE_SUPABASE_ANON_KEY?: string;
  readonly VITE_OPERATIONS_INTAKE_MODE?: 'disabled' | 'anonymous';
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
