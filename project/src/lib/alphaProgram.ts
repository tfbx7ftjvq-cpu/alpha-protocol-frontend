import { AnchorProvider, Program, type Idl, type Wallet } from '@coral-xyz/anchor';
import { PublicKey, type Connection } from '@solana/web3.js';
import idl from '../idl/my_first_solana_program.json';

export const PROGRAM_ID = new PublicKey('HrLBQxUD3XHkB3KABjHXTiBHuAe6jVP2UPqiwmpmH8EY');
export const TREASURY_STATE_SEED = 'treasury_state';

export function getTreasuryStatePda(): PublicKey {
  const [pda] = PublicKey.findProgramAddressSync(
    [new TextEncoder().encode(TREASURY_STATE_SEED)],
    PROGRAM_ID,
  );

  return pda;
}

export function createAlphaProgram(connection: Connection, wallet: Wallet): Program<Idl> {
  const provider = new AnchorProvider(connection, wallet, AnchorProvider.defaultOptions());

  return new Program(idl as Idl, provider);
}
