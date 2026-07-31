import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';

const componentSource = readFileSync(
  new URL('../src/components/TurnstileChallenge.tsx', import.meta.url),
  'utf8',
);
const dashboardSource = readFileSync(
  new URL('../src/components/OperationsDashboard.tsx', import.meta.url),
  'utf8',
);
const browserConfigSource = readFileSync(
  new URL('../src/lib/operationsSupabase.ts', import.meta.url),
  'utf8',
);

test('Turnstile uses explicit rendering and handles the full token lifecycle', () => {
  assert.match(
    componentSource,
    /https:\/\/challenges\.cloudflare\.com\/turnstile\/v0\/api\.js\?render=explicit/,
  );
  assert.match(componentSource, /action = 'operations_wallet_auth'/);
  assert.match(componentSource, /'response-field': false/);
  assert.match(componentSource, /'expired-callback'/);
  assert.match(componentSource, /'timeout-callback'/);
  assert.match(componentSource, /\.remove\(widgetId\)/);
  assert.match(componentSource, /\.reset\(widgetId\)/);
});

test('wallet sign-in is disabled until a challenge token exists and resets it after use', () => {
  assert.match(dashboardSource, /Boolean\(turnstileToken\)/);
  assert.match(dashboardSource, /const singleUseToken = turnstileToken/);
  assert.match(dashboardSource, /setTurnstileToken\(null\)/);
  assert.match(dashboardSource, /setTurnstileResetKey/);
  assert.match(dashboardSource, /await auth\.signIn\(singleUseToken\)/);
});

test('browser code never defines a Turnstile secret variable', () => {
  const browserSources = [
    componentSource,
    dashboardSource,
    browserConfigSource,
  ].join('\n');

  assert.doesNotMatch(browserSources, /VITE_[A-Z0-9_]*TURNSTILE[A-Z0-9_]*SECRET/i);
  assert.doesNotMatch(browserSources, /TURNSTILE_SECRET_KEY/i);
});
