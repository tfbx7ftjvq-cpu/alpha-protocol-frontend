import * as anchor from '@coral-xyz/anchor';
import { Program, type Idl } from '@coral-xyz/anchor';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { PublicKey, SYSVAR_RENT_PUBKEY, SystemProgram } from '@solana/web3.js';
import * as fs from 'fs';
import * as path from 'path';

const PROGRAM_ID = new PublicKey('HrLBQxUD3XHkB3KABjHXTiBHuAe6jVP2UPqiwmpmH8EY');

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

function derivePda(seed: string): PublicKey {
  const [pda] = PublicKey.findProgramAddressSync([Buffer.from(seed)], PROGRAM_ID);
  return pda;
}

function loadProgram(provider: anchor.AnchorProvider): Program<Idl> {
  const idlPath = path.resolve(__dirname, '../target/idl/my_first_solana_program.json');
  const idl = JSON.parse(fs.readFileSync(idlPath, 'utf8')) as Idl;

  return new Program(idl, provider);
}

async function main() {
  const usdcMint = readRequiredPublicKey('USDC_MINT');
  const alphaMint = readRequiredPublicKey('ALPHA_MINT');

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = loadProgram(provider);
  const authority = provider.wallet.publicKey;

  const treasuryConfig = derivePda(SEEDS.treasuryConfigV2);
  const treasuryUsdcState = derivePda(SEEDS.treasuryUsdcStateV2);
  const reliefUsdcVault = derivePda(SEEDS.reliefUsdcVault);
  const buybackUsdcVault = derivePda(SEEDS.buybackUsdcVault);
  const buildersUsdcVault = derivePda(SEEDS.buildersUsdcVault);
  const stakingUsdcVault = derivePda(SEEDS.stakingUsdcVault);
  const vaultAuthority = derivePda(SEEDS.vaultAuthorityV2);

  console.log('Program ID:', PROGRAM_ID.toBase58());
  console.log('Authority:', authority.toBase58());
  console.log('USDC Mint:', usdcMint.toBase58());
  console.log('ALPHA Mint:', alphaMint.toBase58());
  console.log('treasury_config_v2:', treasuryConfig.toBase58());
  console.log('treasury_usdc_state_v2:', treasuryUsdcState.toBase58());
  console.log('relief_usdc_vault:', reliefUsdcVault.toBase58());
  console.log('buyback_usdc_vault:', buybackUsdcVault.toBase58());
  console.log('builders_usdc_vault:', buildersUsdcVault.toBase58());
  console.log('staking_usdc_vault:', stakingUsdcVault.toBase58());
  console.log('vault_authority_v2:', vaultAuthority.toBase58());

  const signature = await (program.methods as any)
    .initializeUsdcTreasury(usdcMint, alphaMint)
    .accountsStrict({
      treasuryConfig,
      treasuryUsdcState,
      usdcMintAccount: usdcMint,
      vaultAuthority,
      reliefUsdcVault,
      buybackUsdcVault,
      buildersUsdcVault,
      stakingUsdcVault,
      authority,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
      rent: SYSVAR_RENT_PUBKEY,
    })
    .rpc();

  console.log('Transaction signature:', signature);
  console.log('USDC Treasury V2 初始化完成');
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
