import assert from "assert";
import { PublicKey } from "@solana/web3.js";
import {
  DAO_CONTROL_ACTIVATION_CONFIRMATION_ENV,
  DAO_CONTROL_ACTIVATION_CONFIRMATION_VALUE,
  RuntimeIdl,
  assertInstructionAccountSchema,
  assertInstructionKeysMatchSchema,
  buildIx,
  meta,
  readU64,
  readU64Default,
  requireDaoControlActivationConfirmation,
} from "./common";

const TEST_KEY = new PublicKey("11111111111111111111111111111111");

function withEnv<T>(name: string, value: string | undefined, fn: () => T): T {
  const previous = process.env[name];
  if (value === undefined) {
    delete process.env[name];
  } else {
    process.env[name] = value;
  }
  try {
    return fn();
  } finally {
    if (previous === undefined) {
      delete process.env[name];
    } else {
      process.env[name] = previous;
    }
  }
}

function assertThrowsMessage(fn: () => unknown, expected: string): void {
  assert.throws(fn, (error) => error instanceof Error && error.message.includes(expected));
}

const fakeIdl = {
  instructions: [
    {
      name: "demo",
      discriminator: [1, 2, 3, 4, 5, 6, 7, 8],
      accounts: [
        { name: "payer", signer: true, writable: true },
        { name: "state", writable: true },
        { name: "system_program" },
      ],
      args: [],
    },
    {
      name: "nested_demo",
      discriminator: [8, 7, 6, 5, 4, 3, 2, 1],
      accounts: [
        {
          name: "group",
          accounts: [
            { name: "inner_signer", signer: true },
            { name: "inner_state", writable: true },
          ],
        },
      ],
      args: [],
    },
  ],
} as unknown as RuntimeIdl;

function testReadU64(): void {
  assert.strictEqual(withEnv("TEST_U64", "0", () => readU64("TEST_U64")), 0n);
  assert.strictEqual(withEnv("TEST_U64", "1", () => readU64("TEST_U64")), 1n);
  assert.strictEqual(withEnv("TEST_U64", undefined, () => readU64Default("TEST_U64", 7n)), 7n);
  assertThrowsMessage(() => withEnv("TEST_U64", undefined, () => readU64("TEST_U64")), "non-negative integer");
  assertThrowsMessage(() => withEnv("TEST_U64", "-1", () => readU64("TEST_U64")), "non-negative integer");
  assertThrowsMessage(() => withEnv("TEST_U64", "1.5", () => readU64("TEST_U64")), "non-negative integer");
  assertThrowsMessage(() => withEnv("TEST_U64", "abc", () => readU64("TEST_U64")), "non-negative integer");
  assertThrowsMessage(() => withEnv("TEST_U64", "18446744073709551616", () => readU64("TEST_U64")), "fit in u64");
}

function testDaoActivationConfirmation(): void {
  assertThrowsMessage(() => requireDaoControlActivationConfirmation({}), DAO_CONTROL_ACTIVATION_CONFIRMATION_ENV);
  assertThrowsMessage(
    () => requireDaoControlActivationConfirmation({ [DAO_CONTROL_ACTIVATION_CONFIRMATION_ENV]: "yes" }),
    DAO_CONTROL_ACTIVATION_CONFIRMATION_VALUE,
  );
  assert.doesNotThrow(() =>
    requireDaoControlActivationConfirmation({
      [DAO_CONTROL_ACTIVATION_CONFIRMATION_ENV]: DAO_CONTROL_ACTIVATION_CONFIRMATION_VALUE,
    }),
  );
}

function testIdlSchema(): void {
  const goodKeys = [
    meta("payer", TEST_KEY, true, true),
    meta("state", TEST_KEY, false, true),
    meta("system_program", TEST_KEY),
  ];
  assert.doesNotThrow(() => assertInstructionKeysMatchSchema(fakeIdl, "demo", goodKeys));
  assert.doesNotThrow(() => buildIx(fakeIdl, "demo", goodKeys));
  assert.doesNotThrow(() =>
    assertInstructionAccountSchema(fakeIdl, "nested_demo", [
      { name: "inner_signer", signer: true },
      { name: "inner_state", writable: true },
    ]),
  );
  assertThrowsMessage(() => assertInstructionKeysMatchSchema(fakeIdl, "demo", goodKeys.slice(0, 2)), "account count mismatch");
  assertThrowsMessage(
    () =>
      assertInstructionKeysMatchSchema(fakeIdl, "demo", [
        meta("state", TEST_KEY, false, true),
        meta("payer", TEST_KEY, true, true),
        meta("system_program", TEST_KEY),
      ]),
    "account order/name mismatch",
  );
  assertThrowsMessage(
    () =>
      assertInstructionKeysMatchSchema(fakeIdl, "demo", [
        meta("payer", TEST_KEY, false, true),
        meta("state", TEST_KEY, false, true),
        meta("system_program", TEST_KEY),
      ]),
    "signer mismatch",
  );
  assertThrowsMessage(
    () =>
      assertInstructionKeysMatchSchema(fakeIdl, "demo", [
        meta("payer", TEST_KEY, true, false),
        meta("state", TEST_KEY, false, true),
        meta("system_program", TEST_KEY),
      ]),
    "writable mismatch",
  );
  assertThrowsMessage(
    () =>
      assertInstructionKeysMatchSchema(fakeIdl, "demo", [
        meta(TEST_KEY, true, true),
        meta("state", TEST_KEY, false, true),
        meta("system_program", TEST_KEY),
      ]),
    "missing script account name",
  );
}

testReadU64();
testDaoActivationConfirmation();
testIdlSchema();
console.log("alpha-v1 common tooling tests passed");
