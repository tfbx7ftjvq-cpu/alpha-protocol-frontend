# Alpha Protocol Turnstile-Protected Wallet Auth Staging V1

Status: remote challenge and Web3 authentication reached Supabase; intake locked
Baseline commit: `f125230873d97e05b5cd432ea40a194ee8832f15`
Phase: `2E-6B-4J`
Date: `2026-07-31`

## 1. Purpose

This phase closes the remaining client-side anti-abuse gap in the
wallet-authenticated operations intake design. A Cloudflare Turnstile token is
now required before the frontend may ask Supabase to authenticate a Solana
wallet.

This phase does not open intake. The deployed frontend remains read-only until
all independent gates are reviewed and explicitly activated.

The architecture boundary remains:

> Smart contract = constitution + vault + cashier.
> Operations database = intake + review + discussion + publication.

Turnstile is an authentication abuse control. It does not verify a payout
recipient, cast a DAO vote, authorize a token, sign a Solana transaction, or
move SOL, USDC, or ALPHA.

## 2. Implemented authentication path

When `wallet-staging` is eventually enabled, the browser path is:

1. load the official Cloudflare Turnstile script in explicit-render mode;
2. render a Managed challenge with action `operations_wallet_auth`;
3. receive one short-lived challenge token;
4. require a connected Solana wallet with message-signing support;
5. pass the exact page URL and challenge token to Supabase
   `signInWithWeb3`;
6. let Supabase validate CAPTCHA and the signed Web3 authentication message;
7. compare the returned Solana identity with the currently connected wallet;
8. query the database-side identity and intake gate;
9. permit form submission only when all identity and gate checks agree.

Failure at any step leaves public reads available and mutations locked.

## 3. Turnstile browser behavior

The component uses:

```text
https://challenges.cloudflare.com/turnstile/v0/api.js?render=explicit
```

Reviewed widget options are:

- action: `operations_wallet_auth`;
- theme: dark;
- size: flexible;
- appearance: always;
- execution: render;
- automatic retry;
- automatic refresh after expiry or timeout;
- hidden response-field creation disabled.

The component:

- loads the script once;
- tolerates React remounts;
- removes its widget during cleanup;
- clears tokens on error, expiry, and timeout;
- resets the widget after every sign-in attempt;
- never persists or logs a challenge token.

For the explicitly confirmed Staging E2E only, the container exposes the
public Turnstile widget id through `data-turnstile-widget-id`. This is not a
response token or secret. It lets a trusted operator use the official
`turnstile.getResponse(widgetId)` browser API to copy one already completed
challenge directly into the current test process without adding the token to
application state, storage, configuration files, or Git.

Turnstile tokens are treated as single-use values. The UI clears the token
before calling wallet authentication and resets the widget in a `finally`
path, whether authentication succeeds or fails.

## 4. Browser configuration

The frontend now recognizes:

```dotenv
VITE_TURNSTILE_SITE_KEY=<browser-safe-site-key>
```

This is a public site key and may be present in the compiled browser bundle.
The configuration still fails closed unless all wallet-staging requirements
are valid:

```dotenv
VITE_SUPABASE_URL=https://<project-ref>.supabase.co
VITE_SUPABASE_ANON_KEY=<browser-safe-publishable-or-anon-key>
VITE_OPERATIONS_PROJECT_REF=<exact-20-character-project-ref>
VITE_OPERATIONS_WEB3_URL=https://<exact-staging-page>/
VITE_TURNSTILE_SITE_KEY=<browser-safe-site-key>
VITE_OPERATIONS_INTAKE_MODE=wallet-staging
```

If the site key is absent or malformed, the backend remains configured for
public reads but intake is forced back to disabled.

Never place any of these values in a `VITE_*` variable:

- Turnstile secret key;
- Supabase service-role key;
- Supabase secret key;
- database password;
- wallet private key, seed phrase, or upgrade authority.

The Turnstile secret belongs only in the Supabase CAPTCHA provider
configuration.

## 5. Token validation and forwarding

Before a wallet signature is requested, the application rejects a challenge
token that:

- is missing;
- contains whitespace;
- is shorter than the reviewed minimum;
- exceeds the Turnstile maximum of 2048 characters.

The validated value is forwarded only as:

```ts
options: {
  url: exactWeb3Url,
  captchaToken,
}
```

There is no fallback path that signs in without the token.

## 6. Tests

The phase adds checks for:

- missing and malformed Turnstile site keys;
- public-read-only fallback when the site key is invalid;
- accepted site-key configuration;
- challenge-token lower and upper boundaries;
- whitespace and oversized-token rejection;
- exact forwarding into Supabase Web3 Auth options;
- official explicit-render script URL;
- challenge action and lifecycle callbacks;
- widget removal and reset behavior;
- sign-in button gating on a challenge token;
- token clearing before use and reset after every attempt;
- absence of a browser-side Turnstile secret variable.

Expected full local verification:

```text
operations tests: 53 passed, 0 failed
operations tooling TypeScript: passed
frontend TypeScript: passed
ESLint: passed
production Vite build: passed
```

This phase adds no npm dependency and should not modify `package.json` or
`package-lock.json`.

## 7. Current remote state

Already complete:

- Supabase Staging project:
  `neevswvhndkalxkainxo`;
- migrations through `202608020001` applied;
- local/remote migration parity confirmed;
- linked schema lint reports no issues;
- read-only preflight passed;
- Cloudflare Pages deployment:
  `https://alpha-protocol-frontend.pages.dev/`;
- public database reads confirmed from the deployment;
- exact Auth page configuration and conservative Web3 Auth rate limit;
- one Cloudflare Turnstile Managed widget bound to the Pages hostname;
- the public Turnstile site key in Cloudflare Pages;
- the Turnstile secret in Supabase CAPTCHA configuration only;
- Supabase CAPTCHA and the Solana Web3 Wallet provider enabled;
- frontend `wallet-staging` mode enabled;
- a real Turnstile challenge and Phantom message-signature flow reached
  Supabase Auth;
- the resulting Auth identity was read back as matching
  `provider_id = identity_data.sub = web3:solana:<connected-wallet>`;
- the Phase 4K parser is deployed from commit
  `da21920783d5e56ea0da14f44511009b5bc1db09`;
- the Phase 4K compatibility migration is active and the read-only preflight
  still passes.
- lint-cleanup migration `202608020001` is applied remotely.

Still closed:

- database operations intake gate remains `disabled`;
- Supabase Anonymous Sign-Ins remain disabled;
- Phase 4L gate-audit migration `202608020002` is local only and not applied
  remotely;
- no public intake mutation has run.

The first browser authentication was intentionally conducted while the
database gate was closed. It revealed a fail-closed compatibility defect that
Phase `2E-6B-4K` has now corrected without opening intake. Its remote schema
lint follow-up removes only a redundant PL/pgSQL declaration. See
`docs/supabase-web3-solana-identity-compatibility-v1.md`.

Phase `2E-6B-4L` subsequently separates authenticated-session state from the
independent database write gate. A valid wallet session may read its private
history while the gate remains disabled; new submissions still require both
the verified wallet and the enabled gate. See
`docs/wallet-session-intake-gate-separation-and-controlled-staging-activation-v1.md`.

## 8. Reviewed remote activation order

Do not skip or reorder these gates:

1. commit and deploy this code while frontend intake remains `disabled`;
2. create one Turnstile Managed widget for hostname
   `alpha-protocol-frontend.pages.dev`;
3. keep pre-clearance disabled;
4. add only the public site key to Cloudflare Pages as
   `VITE_TURNSTILE_SITE_KEY`;
5. redeploy and verify that read-only mode still does not render the widget;
6. place the Turnstile secret only in Supabase CAPTCHA configuration;
7. enable only the Supabase Solana Web3 Wallet provider;
8. keep Anonymous Sign-Ins disabled;
9. create a reviewed temporary deployment with frontend mode
   `wallet-staging`, while the database gate remains disabled;
10. complete Turnstile and wallet-signature browser authentication;
11. confirm the expected database-gate rejection and confirm no operation row
    was inserted;
12. review Auth logs and challenge behavior;
13. explicitly activate the database intake gate with a non-empty auditable
    reference;
14. complete a fresh challenge, copy its response into the current operator
    process, explicitly confirm, and run the mutating wallet RLS E2E within
    the token lifetime;
15. verify cleanup and zero temporary residue;
16. only then decide whether the reviewed Staging deployment may retain
    `wallet-staging`.

If activation pauses at any point, restore or retain:

```dotenv
VITE_OPERATIONS_INTAKE_MODE=disabled
```

and leave the database intake gate disabled.

## 9. Operational limits

Turnstile and Auth rate limits reduce automated abuse. They do not:

- prevent an attacker from controlling multiple real wallets;
- validate the truth of submitted evidence;
- replace moderation;
- replace database RLS or rate-limit triggers;
- make a wallet a verified payout destination;
- authorize treasury spending.

The browser smoke test is required because a command-line E2E runner cannot
legitimately mint a production Turnstile token. The controlled runner may
consume one response produced by a human-completed challenge on the reviewed
Pages hostname. Cloudflare response tokens expire after five minutes and are
single-use, so the value must be copied immediately before the run, supplied
only through `OPERATIONS_STAGING_E2E_CAPTCHA_TOKEN`, and cleared afterward.
Production CAPTCHA must not be disabled, bypassed, hard-coded, or persisted in
test tooling.

## 10. Deployment boundaries

This phase:

- does not change a Solana program;
- does not upgrade the Devnet program;
- does not close or modify the uploaded Devnet buffer;
- does not initialize or activate protocol authority control;
- does not send a Devnet transaction;
- does not enter Mainnet;
- does not send a Mainnet transaction;
- does not activate public operations intake.

This is not a professional Mainnet security audit or legal review.

## 11. Official references

- [Cloudflare Turnstile widget management](https://developers.cloudflare.com/turnstile/get-started/widget-management/dashboard/)
- [Cloudflare explicit rendering](https://developers.cloudflare.com/turnstile/get-started/client-side-rendering/widget-configurations/)
- [Cloudflare Turnstile token validation](https://developers.cloudflare.com/turnstile/get-started/server-side-validation/)
- [Supabase CAPTCHA](https://supabase.com/docs/guides/auth/auth-captcha)
- [Supabase Web3 authentication](https://supabase.com/docs/guides/auth/auth-web3)
- [Supabase JavaScript `signInWithWeb3`](https://supabase.com/docs/reference/javascript/auth-signinwithweb3)
