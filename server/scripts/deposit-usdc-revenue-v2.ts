import * as anchor from '@coral-xyz/anchor';
import { BN, Program, type Idl } from '@coral-xyz/anchor';
import { getAssociatedTokenAddressSync, TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { PublicKey } from '@solana/web3.js';
import * as fs from 'fs';
import * as path from 'path';

const PROGRAM_ID = new PublicKey('HrLBQxUD3XHkB3KABjHXTiBHuAe6jVP2UPqiwmpmH8EY');
const DEFAULT_AMOUNT = '100000000';

const SEEDS = {
  treasuryConfigV2: 'treasury_config_v2',
  treasuryUsdcStateV2: 'treasury_usdc_state_v2',
  reliefUsdcVault: 'relief_usdc_vault',
  buybackUsdcVault: 'buyback_usdc_vault',
  buildersUsdcVault: 'builders_usdc_vault',
  stakingUsdcVault: 'staking_usdc_vault',
  vaultAuthorityV2: 'vault_authority_v2',
} as const;

function readRequiredPublicKey(name: string): PublicKey {
  const value = process.env[name];
  if (!value) {
    throw new Error(`Missing required env var: ${name}`);
  }

  return new PublicKey(value);
}

function readAmount(): BN {
  const raw = process.env.AMOUNT ?? DEFAULT_AMOUNT;
  if (!/^\d+$/.test(raw)) {
    throw new Error('AMOUNT must be a positive integer string');
  }

  const amount = new BN(raw);
  if (amount.lte(new BN(0))) {
    throw new Error('AMOUNT must be greater than 0');
  }

  return amount;
}

function derivePda(seed: string): PublicKey {
  const [pda] = PublicKey.findProgramAddressSync([Buffer.from(seed)], PROGRAM_ID);
  return pda;
}

function loadProgram(provider: anchor.AnchorProvider): Program<Idl> {
  const idlPath = path.resolve(__dirname, '../target/idl/my_first_solana_program.json');
  const idl = JSON.parse(fs.readFileSync(idlPath, 'utf8')) as Idl;

  return new Program(idl, provider);
}

function splitAmount(amount: BN, numerator: number): BN {
  return amount.mul(new BN(numerator)).div(new BN(100));
}

async function main() {
  const usdcMint = readRequiredPublicKey('USDC_MINT');
  const amount = readAmount();

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = loadProgram(provider);
  const depositor = provider.wallet.publicKey;
  const depositorUsdcTokenAccount = getAssociatedTokenAddressSync(usdcMint, depositor);

  const treasuryConfig = derivePda(SEEDS.treasuryConfigV2);
  const treasuryUsdcState = derivePda(SEEDS.treasuryUsdcStateV2);
  const reliefUsdcVault = derivePda(SEEDS.reliefUsdcVault);
  const buybackUsdcVault = derivePda(SEEDS.buybackUsdcVault);
  const buildersUsdcVault = derivePda(SEEDS.buildersUsdcVault);
  const stakingUsdcVault = derivePda(SEEDS.stakingUsdcVault);
  const vaultAuthority = derivePda(SEEDS.vaultAuthorityV2);

  const expectedRelief = splitAmount(amount, 50);
  const expectedBuyback = splitAmount(amount, 20);
  const expectedBuilders = splitAmount(amount, 20);
  const expectedStaking = amount.sub(expectedRelief).sub(expectedBuyback).sub(expectedBuilders);

  console.log('Program ID:', PROGRAM_ID.toBase58());
  console.log('Depositor:', depositor.toBase58());
  console.log('USDC Mint:', usdcMint.toBase58());
  console.log('depositor_usdc_token_account:', depositorUsdcTokenAccount.toBase58());
  console.log('treasury_config_v2:', treasuryConfig.toBase58());
  console.log('treasury_usdc_state_v2:', treasuryUsdcState.toBase58());
  console.log('relief_usdc_vault:', reliefUsdcVault.toBase58());
  console.log('buyback_usdc_vault:', buybackUsdcVault.toBase58());
  console.log('builders_usdc_vault:', buildersUsdcVault.toBase58());
  console.log('staking_usdc_vault:', stakingUsdcVault.toBase58());
  console.log('vault_authority_v2:', vaultAuthority.toBase58());
  console.log('Amount:', amount.toString());
  console.log('Expected split:');
  console.log('  relief 50%:', expectedRelief.toString());
  console.log('  buyback 20%:', expectedBuyback.toString());
  console.log('  builders 20%:', expectedBuilders.toString());
  console.log('  staking 10%:', expectedStaking.toString());

  const signature = await (program.methods as any)
    .depositUsdcRevenue(amount)
    .accountsStrict({
      depositor,
      depositorUsdcTokenAccount,
      treasuryConfig,
      treasuryUsdcState,
      usdcMint,
      vaultAuthority,
      reliefUsdcVault,
      buybackUsdcVault,
      buildersUsdcVault,
      stakingUsdcVault,
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .rpc();

  console.log('Transaction signature:', signature);
  console.log('USDC revenue deposit V2 完成');
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
