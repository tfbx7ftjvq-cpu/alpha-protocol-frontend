use anchor_lang::prelude::*;

use crate::constants::{GOVERNANCE_CONFIG_V1_SEED, PROTOCOL_MODULE_REGISTRY_V1_SEED};
use crate::error::CustomError;
use crate::state::{GovernanceConfigV1, ProtocolModuleIdV1, ProtocolModuleRegistryV1};

pub const PROTOCOL_MODULE_REGISTRY_V1_SCHEMA_VERSION: u16 = 1;

#[derive(Accounts)]
#[instruction(module_id: ProtocolModuleIdV1)]
pub struct InitializeProtocolModuleRegistryV1<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub bootstrap_authority: Signer<'info>,

    #[account(
        seeds = [GOVERNANCE_CONFIG_V1_SEED],
        bump = security_governance_config.bump
    )]
    pub security_governance_config: Account<'info, GovernanceConfigV1>,

    #[account(
        init,
        payer = payer,
        space = 8 + ProtocolModuleRegistryV1::INIT_SPACE,
        seeds = [
            PROTOCOL_MODULE_REGISTRY_V1_SEED,
            &[protocol_module_stable_code_v1(module_id)]
        ],
        bump
    )]
    pub protocol_module_registry: Account<'info, ProtocolModuleRegistryV1>,

    pub system_program: Program<'info, System>,
}

pub fn initialize_protocol_module_registry_v1_handler(
    ctx: Context<InitializeProtocolModuleRegistryV1>,
    module_id: ProtocolModuleIdV1,
    schema_version: u16,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let registry_key = ctx.accounts.protocol_module_registry.key();
    record_protocol_module_registry_init(
        &mut ctx.accounts.protocol_module_registry,
        registry_key,
        ctx.accounts.security_governance_config.key(),
        ctx.accounts.security_governance_config.authority,
        ctx.accounts.bootstrap_authority.key(),
        module_id,
        schema_version,
        now,
        ctx.bumps.protocol_module_registry,
    )
}

pub fn protocol_module_stable_code_v1(module_id: ProtocolModuleIdV1) -> u8 {
    match module_id {
        ProtocolModuleIdV1::Treasury => 1,
        ProtocolModuleIdV1::GreenLabel => 2,
        ProtocolModuleIdV1::VictimRelief => 3,
        ProtocolModuleIdV1::ScamRegistry => 4,
        ProtocolModuleIdV1::Contributor => 5,
        ProtocolModuleIdV1::Protocol => 6,
    }
}

pub fn protocol_module_from_stable_code_v1(code: u8) -> Result<ProtocolModuleIdV1> {
    match code {
        1 => Ok(ProtocolModuleIdV1::Treasury),
        2 => Ok(ProtocolModuleIdV1::GreenLabel),
        3 => Ok(ProtocolModuleIdV1::VictimRelief),
        4 => Ok(ProtocolModuleIdV1::ScamRegistry),
        5 => Ok(ProtocolModuleIdV1::Contributor),
        6 => Ok(ProtocolModuleIdV1::Protocol),
        _ => err!(CustomError::InvalidProtocolModuleCode),
    }
}

pub fn expected_protocol_module_registry_key_and_bump(
    module_id: ProtocolModuleIdV1,
) -> (Pubkey, u8) {
    let module_code = protocol_module_stable_code_v1(module_id);
    Pubkey::find_program_address(
        &[PROTOCOL_MODULE_REGISTRY_V1_SEED, &[module_code]],
        &crate::ID,
    )
}

pub fn expected_security_governance_config_key() -> Pubkey {
    Pubkey::find_program_address(&[GOVERNANCE_CONFIG_V1_SEED], &crate::ID).0
}

#[allow(clippy::too_many_arguments)]
pub fn record_protocol_module_registry_init(
    registry: &mut ProtocolModuleRegistryV1,
    registry_key: Pubkey,
    security_governance_config: Pubkey,
    security_governance_authority: Pubkey,
    bootstrap_authority: Pubkey,
    module_id: ProtocolModuleIdV1,
    schema_version: u16,
    now_ts: i64,
    bump: u8,
) -> Result<()> {
    require!(
        registry.security_governance_config == Pubkey::default(),
        CustomError::ProtocolModuleRegistryMismatch
    );
    require_keys_eq!(
        bootstrap_authority,
        security_governance_authority,
        CustomError::UnauthorizedSecurityAuthority
    );
    require!(
        schema_version == PROTOCOL_MODULE_REGISTRY_V1_SCHEMA_VERSION,
        CustomError::InvalidProtocolModuleRegistrySchema
    );

    let module_code = protocol_module_stable_code_v1(module_id);
    let (expected_registry_key, expected_bump) =
        expected_protocol_module_registry_key_and_bump(module_id);
    require_keys_eq!(
        registry_key,
        expected_registry_key,
        CustomError::ProtocolModuleRegistryMismatch
    );
    require!(
        bump == expected_bump,
        CustomError::ProtocolModuleRegistryMismatch
    );

    registry.security_governance_config = security_governance_config;
    registry.module_id = module_id;
    registry.module_code = module_code;
    registry.program_id = crate::ID;
    registry.enabled = true;
    registry.schema_version = schema_version;
    registry.created_at = now_ts;
    registry.updated_at = now_ts;
    registry.bump = bump;

    Ok(())
}

pub fn validate_protocol_module_registry_v1(
    registry: &ProtocolModuleRegistryV1,
    registry_key: Pubkey,
    expected_security_governance_config: Pubkey,
    expected_module: ProtocolModuleIdV1,
    expected_target_program: Pubkey,
) -> Result<()> {
    require_keys_eq!(
        registry.security_governance_config,
        expected_security_governance_config,
        CustomError::ProtocolModuleGovernanceConfigMismatch
    );
    require!(
        registry.module_id == expected_module,
        CustomError::ProtocolModuleRegistryMismatch
    );
    require!(
        registry.module_code == protocol_module_stable_code_v1(expected_module),
        CustomError::InvalidProtocolModuleCode
    );
    require_keys_eq!(
        registry.program_id,
        expected_target_program,
        CustomError::ProtocolModuleProgramMismatch
    );
    require_keys_eq!(
        registry.program_id,
        crate::ID,
        CustomError::ProtocolModuleProgramMismatch
    );
    require!(registry.enabled, CustomError::ProtocolModuleDisabled);
    require!(
        registry.schema_version == PROTOCOL_MODULE_REGISTRY_V1_SCHEMA_VERSION,
        CustomError::InvalidProtocolModuleRegistrySchema
    );

    let (expected_registry_key, expected_bump) =
        expected_protocol_module_registry_key_and_bump(expected_module);
    require_keys_eq!(
        registry_key,
        expected_registry_key,
        CustomError::ProtocolModuleRegistryMismatch
    );
    require!(
        registry.bump == expected_bump,
        CustomError::ProtocolModuleRegistryMismatch
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    const SECURITY_CONFIG: Pubkey = Pubkey::new_from_array([1; 32]);
    const AUTHORITY: Pubkey = Pubkey::new_from_array([2; 32]);
    const WRONG_AUTHORITY: Pubkey = Pubkey::new_from_array([3; 32]);

    const ALL_MODULES: [ProtocolModuleIdV1; 6] = [
        ProtocolModuleIdV1::Treasury,
        ProtocolModuleIdV1::GreenLabel,
        ProtocolModuleIdV1::VictimRelief,
        ProtocolModuleIdV1::ScamRegistry,
        ProtocolModuleIdV1::Contributor,
        ProtocolModuleIdV1::Protocol,
    ];

    fn blank_registry() -> ProtocolModuleRegistryV1 {
        ProtocolModuleRegistryV1 {
            security_governance_config: Pubkey::default(),
            module_id: ProtocolModuleIdV1::Treasury,
            module_code: 0,
            program_id: Pubkey::default(),
            enabled: false,
            schema_version: 0,
            created_at: 0,
            updated_at: 0,
            bump: 0,
        }
    }

    fn initialized_registry(module_id: ProtocolModuleIdV1) -> (ProtocolModuleRegistryV1, Pubkey) {
        let mut registry = blank_registry();
        let (registry_key, bump) = expected_protocol_module_registry_key_and_bump(module_id);
        record_protocol_module_registry_init(
            &mut registry,
            registry_key,
            SECURITY_CONFIG,
            AUTHORITY,
            AUTHORITY,
            module_id,
            PROTOCOL_MODULE_REGISTRY_V1_SCHEMA_VERSION,
            100,
            bump,
        )
        .unwrap();
        (registry, registry_key)
    }

    #[test]
    fn module_stable_codes_are_unique() {
        let mut codes = BTreeSet::new();
        for module_id in ALL_MODULES {
            assert!(codes.insert(protocol_module_stable_code_v1(module_id)));
        }
        assert_eq!(codes.len(), ALL_MODULES.len());
    }

    #[test]
    fn module_stable_code_roundtrips() {
        for module_id in ALL_MODULES {
            let code = protocol_module_stable_code_v1(module_id);
            assert_eq!(
                protocol_module_from_stable_code_v1(code).unwrap(),
                module_id
            );
        }
    }

    #[test]
    fn unknown_module_code_fails() {
        let err = protocol_module_from_stable_code_v1(99).unwrap_err();
        assert_eq!(err, CustomError::InvalidProtocolModuleCode.into());
    }

    #[test]
    fn module_code_snapshot_is_fixed() {
        assert_eq!(
            ALL_MODULES.map(protocol_module_stable_code_v1),
            [1, 2, 3, 4, 5, 6]
        );
    }

    #[test]
    fn all_modules_initialize_with_expected_pda() {
        for module_id in ALL_MODULES {
            let (registry, registry_key) = initialized_registry(module_id);
            assert_eq!(registry.module_id, module_id);
            assert_eq!(
                registry.module_code,
                protocol_module_stable_code_v1(module_id)
            );
            assert_eq!(registry.program_id, crate::ID);
            assert!(registry.enabled);
            validate_protocol_module_registry_v1(
                &registry,
                registry_key,
                SECURITY_CONFIG,
                module_id,
                crate::ID,
            )
            .unwrap();
        }
    }

    #[test]
    fn wrong_bootstrap_authority_fails() {
        let module_id = ProtocolModuleIdV1::Treasury;
        let (registry_key, bump) = expected_protocol_module_registry_key_and_bump(module_id);
        let mut registry = blank_registry();
        let err = record_protocol_module_registry_init(
            &mut registry,
            registry_key,
            SECURITY_CONFIG,
            AUTHORITY,
            WRONG_AUTHORITY,
            module_id,
            PROTOCOL_MODULE_REGISTRY_V1_SCHEMA_VERSION,
            100,
            bump,
        )
        .unwrap_err();

        assert_eq!(err, CustomError::UnauthorizedSecurityAuthority.into());
    }

    #[test]
    fn invalid_schema_fails() {
        let module_id = ProtocolModuleIdV1::Treasury;
        let (registry_key, bump) = expected_protocol_module_registry_key_and_bump(module_id);
        let mut registry = blank_registry();
        let err = record_protocol_module_registry_init(
            &mut registry,
            registry_key,
            SECURITY_CONFIG,
            AUTHORITY,
            AUTHORITY,
            module_id,
            2,
            100,
            bump,
        )
        .unwrap_err();

        assert_eq!(err, CustomError::InvalidProtocolModuleRegistrySchema.into());
    }

    #[test]
    fn duplicate_module_registry_fails() {
        let module_id = ProtocolModuleIdV1::Treasury;
        let (mut registry, registry_key) = initialized_registry(module_id);
        let bump = registry.bump;
        let err = record_protocol_module_registry_init(
            &mut registry,
            registry_key,
            SECURITY_CONFIG,
            AUTHORITY,
            AUTHORITY,
            module_id,
            PROTOCOL_MODULE_REGISTRY_V1_SCHEMA_VERSION,
            100,
            bump,
        )
        .unwrap_err();

        assert_eq!(err, CustomError::ProtocolModuleRegistryMismatch.into());
    }

    #[test]
    fn validation_rejects_wrong_module() {
        let (registry, registry_key) = initialized_registry(ProtocolModuleIdV1::Treasury);
        let err = validate_protocol_module_registry_v1(
            &registry,
            registry_key,
            SECURITY_CONFIG,
            ProtocolModuleIdV1::Contributor,
            crate::ID,
        )
        .unwrap_err();

        assert_eq!(err, CustomError::ProtocolModuleRegistryMismatch.into());
    }

    #[test]
    fn validation_rejects_wrong_module_code() {
        let (mut registry, registry_key) = initialized_registry(ProtocolModuleIdV1::Treasury);
        registry.module_code = 99;
        let err = validate_protocol_module_registry_v1(
            &registry,
            registry_key,
            SECURITY_CONFIG,
            ProtocolModuleIdV1::Treasury,
            crate::ID,
        )
        .unwrap_err();

        assert_eq!(err, CustomError::InvalidProtocolModuleCode.into());
    }

    #[test]
    fn validation_rejects_wrong_program() {
        let (mut registry, registry_key) = initialized_registry(ProtocolModuleIdV1::Treasury);
        registry.program_id = Pubkey::new_from_array([9; 32]);
        let err = validate_protocol_module_registry_v1(
            &registry,
            registry_key,
            SECURITY_CONFIG,
            ProtocolModuleIdV1::Treasury,
            crate::ID,
        )
        .unwrap_err();

        assert_eq!(err, CustomError::ProtocolModuleProgramMismatch.into());
    }

    #[test]
    fn validation_rejects_wrong_governance_config() {
        let (registry, registry_key) = initialized_registry(ProtocolModuleIdV1::Treasury);
        let err = validate_protocol_module_registry_v1(
            &registry,
            registry_key,
            Pubkey::new_from_array([8; 32]),
            ProtocolModuleIdV1::Treasury,
            crate::ID,
        )
        .unwrap_err();

        assert_eq!(
            err,
            CustomError::ProtocolModuleGovernanceConfigMismatch.into()
        );
    }

    #[test]
    fn validation_rejects_disabled_registry() {
        let (mut registry, registry_key) = initialized_registry(ProtocolModuleIdV1::Treasury);
        registry.enabled = false;
        let err = validate_protocol_module_registry_v1(
            &registry,
            registry_key,
            SECURITY_CONFIG,
            ProtocolModuleIdV1::Treasury,
            crate::ID,
        )
        .unwrap_err();

        assert_eq!(err, CustomError::ProtocolModuleDisabled.into());
    }

    #[test]
    fn validation_rejects_wrong_schema() {
        let (mut registry, registry_key) = initialized_registry(ProtocolModuleIdV1::Treasury);
        registry.schema_version = 2;
        let err = validate_protocol_module_registry_v1(
            &registry,
            registry_key,
            SECURITY_CONFIG,
            ProtocolModuleIdV1::Treasury,
            crate::ID,
        )
        .unwrap_err();

        assert_eq!(err, CustomError::InvalidProtocolModuleRegistrySchema.into());
    }

    #[test]
    fn validation_rejects_wrong_pda_or_bump() {
        let (mut registry, registry_key) = initialized_registry(ProtocolModuleIdV1::Treasury);
        registry.bump = registry.bump.wrapping_add(1);
        let err = validate_protocol_module_registry_v1(
            &registry,
            registry_key,
            SECURITY_CONFIG,
            ProtocolModuleIdV1::Treasury,
            crate::ID,
        )
        .unwrap_err();

        assert_eq!(err, CustomError::ProtocolModuleRegistryMismatch.into());

        let (registry, _) = initialized_registry(ProtocolModuleIdV1::Treasury);
        let err = validate_protocol_module_registry_v1(
            &registry,
            Pubkey::new_from_array([7; 32]),
            SECURITY_CONFIG,
            ProtocolModuleIdV1::Treasury,
            crate::ID,
        )
        .unwrap_err();

        assert_eq!(err, CustomError::ProtocolModuleRegistryMismatch.into());
    }
}
