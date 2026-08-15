# Phase 2E-6G Isolated Supabase Recovery Rehearsal

This is a preparation-only runbook. It never authorizes a restore over current
Staging, a production project, or a Mainnet system.

1. Inventory the source project, Dashboard backup/PITR capability, migration
   list through `202608110002`, release commit, gate mode, and role inventory.
   Do not put credentials, dumps, or user data in Git.
2. If a backup is required without PITR, create an encrypted `pg_dump` only on
   an approved workstation. Do not place a password in a command line and do
   not commit the dump.
3. Restore only into a newly created target explicitly classified
   `isolated_restore`. The local validator rejects the current Staging ref
   `neevswvhndkalxkainxo` and every target not carrying that classification.
4. Before any functional test, keep the restored intake gate `disabled`; verify
   migration parity, linked lint, Auth health, seven public reads, and seven
   private anonymous denials.
5. Record only the evidence schema fields: isolated target classification,
   backup/PITR inventory date, parity through `202608110002`, RLS probe counts,
   disabled gate, and no-mutation attestation. Validate locally with:

   ```bash
   cd project
   npm run operations:recovery:rehearsal:validate
   ```

The validator does not connect to Supabase and cannot restore, create users,
grant roles, move funds, or send a Solana transaction. Any real rehearsal is a
separately approved human operation.
