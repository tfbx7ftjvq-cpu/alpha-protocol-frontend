# Alpha Protocol Supabase Solana Web3 Identity Compatibility V1

Status: local fix verified; remote migration and deployment not performed
Baseline commit: `cf8245bf0ba531d34e1ec9b0b9f4371522bd1da0`
Phase: `2E-6B-4K`
Date: `2026-07-31`

## 1. Observed Staging result

A real browser flow completed:

1. Cloudflare Turnstile returned a valid challenge;
2. Phantom displayed the reviewed off-chain authentication statement;
3. the connected wallet signed the message;
4. Supabase created one `Web3` Auth user;
5. a read-only query confirmed the identity belongs to the connected wallet.

The dedicated Staging project stores the identity as:

```text
provider       = web3
provider_id    = web3:solana:<wallet-address>
identity_data.sub = web3:solana:<wallet-address>
```

The inspected row did not contain separate `identity_data.chain`,
`identity_data.address`, or `identity_data.wallet_address` values.

The earlier client and database parsers expected those separate properties.
They therefore rejected the valid identity. This was a fail-closed
compatibility defect, not a failed Phantom signature and not an RLS bypass.

## 2. Client correction

The browser parser now requires:

- exactly one identity with provider `web3`;
- an exact, case-sensitive `identity_data.sub` prefix
  `web3:solana:`;
- a suffix that decodes to exactly 32 bytes under the Base58 alphabet;
- any optional legacy `chain`, `address`, or `wallet_address` property to
  agree with the canonical subject.

Email identities, legacy-only fields, Ethereum subjects, wrong prefixes,
invalid Base58 values, contradictory optional fields, and multiple Web3
identities all return no verified wallet.

The connected Phantom address must still equal the resolved address before
the application accepts the session.

## 3. Database correction

The new migration is:

```text
supabase/migrations/202607310001_web3_solana_identity_subject_compatibility.sql
```

It replaces only:

```text
public.current_verified_solana_wallet()
```

The security-definer resolver now:

1. selects every `web3` identity for `auth.uid()`;
2. requires exactly one such identity;
3. requires `provider_id` and `identity_data.sub` to match exactly;
4. requires the exact `web3:solana:` prefix;
5. validates the suffix against the Base58 alphabet;
6. decodes its numeric length and requires exactly 32 bytes;
7. otherwise returns `NULL`.

The migration explicitly does not update
`public.operations_intake_control`. Applying it cannot open intake.
Existing owner RLS policies continue to compare every submitted
`wallet_address` with this resolver.

## 4. Adversarial coverage

Local tests cover:

- the exact observed Supabase subject;
- missing or legacy-only identity fields;
- email and non-Solana identities;
- prefix case changes and extra suffix data;
- invalid Base58 and incorrect decoded byte length;
- disagreement between `provider_id` and `identity_data.sub`;
- contradictory optional client fields;
- multiple Web3 identities;
- a connected-wallet switch;
- successful RLS insertion for the matching identity;
- rejection of switched-wallet and email-only insertion;
- the database gate remaining disabled after every migration.

Local verification result:

```text
operations tests: 56 passed, 0 failed
operations tooling TypeScript: passed
application TypeScript: passed
ESLint: passed
Vite production build: passed
```

The complete `npm run operations:verify` command passed.

## 5. Remote state and next gate

At the end of this phase:

- Cloudflare Pages remains deployed at
  `https://alpha-protocol-frontend.pages.dev/`;
- Turnstile, Supabase CAPTCHA, and the Solana Web3 provider are configured;
- Supabase Anonymous Sign-Ins remain disabled;
- frontend mode is `wallet-staging`;
- database `operations_intake_control.mode` remains `disabled`;
- migration `202607310001` is not applied remotely;
- no operations intake row has been submitted;
- no Solana transaction or treasury action has occurred.

The reviewed order is:

1. commit and deploy this parser fix;
2. confirm the Pages build uses that commit;
3. inspect migration parity and run a dry-run;
4. separately confirm and apply migration `202607310001`;
5. run remote schema lint and the read-only preflight;
6. repeat Turnstile and Phantom authentication while the database gate stays
   disabled;
7. verify the Auth identity and resolver result;
8. only after a separate explicit decision, consider activating the database
   intake gate and running a controlled submission E2E.

Do not activate the database gate as part of applying this compatibility
migration.

## 6. Boundaries

This phase:

- does not change a Solana program;
- does not deploy or upgrade Devnet;
- does not send a Devnet or Mainnet transaction;
- does not authorize treasury spending;
- does not mark any intake wallet as payout-verified;
- does not activate public intake;
- is not a professional Mainnet security audit or legal review.
