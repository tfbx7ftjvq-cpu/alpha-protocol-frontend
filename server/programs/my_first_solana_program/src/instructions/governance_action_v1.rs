use anchor_lang::prelude::*;

use crate::error::CustomError;
use crate::instructions::contributor_v1::hash_contributor_payload;
use crate::state::{
    ActionType, GovernanceActionRequestV1, GovernanceActionTypeV1, GovernancePayloadV1,
    GovernanceProposalTypeV1, ProtocolModuleIdV1,
};

pub const GOVERNANCE_PAYLOAD_V1_SCHEMA_VERSION: u8 = 1;
pub const GOVERNANCE_PROPOSAL_ACTION_V1_SCHEMA_VERSION: u16 = 1;
pub const GOVERNANCE_PAYLOAD_V1_DOMAIN_SEPARATOR: &[u8] = b"alpha_governance_payload_v1";
pub const GOVERNANCE_PAYLOAD_V1_DOMAIN_SEPARATOR_BYTES: [u8; 27] = *b"alpha_governance_payload_v1";

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct GovernancePayloadHashEnvelopeV1 {
    pub domain_separator: [u8; 27],
    pub payload: GovernancePayloadV1,
}

pub fn governance_action_stable_code_v1(action_type: GovernanceActionTypeV1) -> u8 {
    match action_type {
        GovernanceActionTypeV1::TreasuryUpdateRevenueSplit => 0,
        GovernanceActionTypeV1::TreasuryApproveSpending => 1,
        GovernanceActionTypeV1::TreasuryApproveBuilderPayout => 2,
        GovernanceActionTypeV1::GreenLabelApproveCertification => 3,
        GovernanceActionTypeV1::GreenLabelRejectCertification => 4,
        GovernanceActionTypeV1::GreenLabelRevokeCertification => 5,
        GovernanceActionTypeV1::GreenLabelRefundBond => 6,
        GovernanceActionTypeV1::GreenLabelSlashBond => 7,
        GovernanceActionTypeV1::VictimReliefApproveCompensation => 8,
        GovernanceActionTypeV1::VictimReliefRejectClaim => 9,
        GovernanceActionTypeV1::VictimReliefUpdatePolicy => 10,
        GovernanceActionTypeV1::ScamRegistryPublishReport => 11,
        GovernanceActionTypeV1::ScamRegistryRemoveReport => 12,
        GovernanceActionTypeV1::ScamRegistryAppealDecision => 13,
        GovernanceActionTypeV1::ContributorAdd => 14,
        GovernanceActionTypeV1::ContributorRemove => 15,
        GovernanceActionTypeV1::ContributorUpdateRole => 16,
        GovernanceActionTypeV1::ContributorApproveMilestone => 17,
        GovernanceActionTypeV1::ContributorApprovePayout => 18,
        GovernanceActionTypeV1::ProtocolUpdateParameter => 19,
        GovernanceActionTypeV1::ProtocolUpgradeProgram => 20,
        GovernanceActionTypeV1::ProtocolEmergencyAction => 21,
        GovernanceActionTypeV1::VictimReliefUpholdAppeal => 22,
        GovernanceActionTypeV1::VictimReliefOverturnAppeal => 23,
        GovernanceActionTypeV1::VictimReliefCancelPayout => 24,
    }
}

pub fn governance_action_from_stable_code_v1(code: u8) -> Result<GovernanceActionTypeV1> {
    match code {
        0 => Ok(GovernanceActionTypeV1::TreasuryUpdateRevenueSplit),
        1 => Ok(GovernanceActionTypeV1::TreasuryApproveSpending),
        2 => Ok(GovernanceActionTypeV1::TreasuryApproveBuilderPayout),
        3 => Ok(GovernanceActionTypeV1::GreenLabelApproveCertification),
        4 => Ok(GovernanceActionTypeV1::GreenLabelRejectCertification),
        5 => Ok(GovernanceActionTypeV1::GreenLabelRevokeCertification),
        6 => Ok(GovernanceActionTypeV1::GreenLabelRefundBond),
        7 => Ok(GovernanceActionTypeV1::GreenLabelSlashBond),
        8 => Ok(GovernanceActionTypeV1::VictimReliefApproveCompensation),
        9 => Ok(GovernanceActionTypeV1::VictimReliefRejectClaim),
        10 => Ok(GovernanceActionTypeV1::VictimReliefUpdatePolicy),
        11 => Ok(GovernanceActionTypeV1::ScamRegistryPublishReport),
        12 => Ok(GovernanceActionTypeV1::ScamRegistryRemoveReport),
        13 => Ok(GovernanceActionTypeV1::ScamRegistryAppealDecision),
        14 => Ok(GovernanceActionTypeV1::ContributorAdd),
        15 => Ok(GovernanceActionTypeV1::ContributorRemove),
        16 => Ok(GovernanceActionTypeV1::ContributorUpdateRole),
        17 => Ok(GovernanceActionTypeV1::ContributorApproveMilestone),
        18 => Ok(GovernanceActionTypeV1::ContributorApprovePayout),
        19 => Ok(GovernanceActionTypeV1::ProtocolUpdateParameter),
        20 => Ok(GovernanceActionTypeV1::ProtocolUpgradeProgram),
        21 => Ok(GovernanceActionTypeV1::ProtocolEmergencyAction),
        22 => Ok(GovernanceActionTypeV1::VictimReliefUpholdAppeal),
        23 => Ok(GovernanceActionTypeV1::VictimReliefOverturnAppeal),
        24 => Ok(GovernanceActionTypeV1::VictimReliefCancelPayout),
        _ => err!(CustomError::InvalidGovernanceActionCode),
    }
}

pub fn map_governance_action_to_security_action(
    action_type: GovernanceActionTypeV1,
) -> Result<ActionType> {
    match action_type {
        GovernanceActionTypeV1::TreasuryUpdateRevenueSplit => {
            Ok(ActionType::TreasuryUpdateRevenueSplit)
        }
        GovernanceActionTypeV1::TreasuryApproveSpending => Ok(ActionType::TreasuryApproveSpending),
        GovernanceActionTypeV1::TreasuryApproveBuilderPayout => {
            Ok(ActionType::TreasuryApproveBuilderPayout)
        }
        GovernanceActionTypeV1::GreenLabelApproveCertification => {
            Ok(ActionType::GreenLabelApproveCertification)
        }
        GovernanceActionTypeV1::GreenLabelRejectCertification => {
            Ok(ActionType::GreenLabelRejectCertification)
        }
        GovernanceActionTypeV1::GreenLabelRevokeCertification => {
            Ok(ActionType::GreenLabelRevokeCertification)
        }
        GovernanceActionTypeV1::GreenLabelRefundBond => Ok(ActionType::GreenLabelRefund),
        GovernanceActionTypeV1::GreenLabelSlashBond => Ok(ActionType::GreenLabelSlash),
        GovernanceActionTypeV1::VictimReliefApproveCompensation => {
            Ok(ActionType::VictimReliefApproveCompensation)
        }
        GovernanceActionTypeV1::VictimReliefRejectClaim => Ok(ActionType::VictimReliefRejectClaim),
        GovernanceActionTypeV1::VictimReliefUpdatePolicy => {
            Ok(ActionType::VictimReliefUpdatePolicy)
        }
        GovernanceActionTypeV1::ScamRegistryPublishReport => {
            Ok(ActionType::ScamRegistryPublishReport)
        }
        GovernanceActionTypeV1::ScamRegistryRemoveReport => {
            Ok(ActionType::ScamRegistryRemoveReport)
        }
        GovernanceActionTypeV1::ScamRegistryAppealDecision => {
            Ok(ActionType::ScamRegistryAppealDecision)
        }
        GovernanceActionTypeV1::ContributorAdd => Ok(ActionType::ContributorAddContributor),
        GovernanceActionTypeV1::ContributorRemove => Ok(ActionType::ContributorRemoveContributor),
        GovernanceActionTypeV1::ContributorUpdateRole => Ok(ActionType::ContributorUpdateRole),
        GovernanceActionTypeV1::ContributorApproveMilestone => {
            Ok(ActionType::ContributorApproveMilestone)
        }
        GovernanceActionTypeV1::ContributorApprovePayout => {
            Ok(ActionType::ContributorApproveBuilderPayout)
        }
        GovernanceActionTypeV1::ProtocolUpdateParameter => Ok(ActionType::ProtocolUpdateParameter),
        GovernanceActionTypeV1::ProtocolUpgradeProgram => Ok(ActionType::ProtocolUpgradeProgram),
        GovernanceActionTypeV1::ProtocolEmergencyAction => Ok(ActionType::ProtocolEmergencyAction),
        GovernanceActionTypeV1::VictimReliefUpholdAppeal => {
            Ok(ActionType::VictimReliefUpholdAppeal)
        }
        GovernanceActionTypeV1::VictimReliefOverturnAppeal => {
            Ok(ActionType::VictimReliefOverturnAppeal)
        }
        GovernanceActionTypeV1::VictimReliefCancelPayout => {
            Ok(ActionType::VictimReliefCancelPayout)
        }
    }
}

pub fn map_governance_action_to_governance_proposal_type_v1(
    action_type: GovernanceActionTypeV1,
) -> GovernanceProposalTypeV1 {
    match action_type {
        GovernanceActionTypeV1::TreasuryUpdateRevenueSplit
        | GovernanceActionTypeV1::TreasuryApproveSpending
        | GovernanceActionTypeV1::TreasuryApproveBuilderPayout => {
            GovernanceProposalTypeV1::Treasury
        }
        GovernanceActionTypeV1::GreenLabelApproveCertification
        | GovernanceActionTypeV1::GreenLabelRejectCertification
        | GovernanceActionTypeV1::GreenLabelRevokeCertification
        | GovernanceActionTypeV1::GreenLabelRefundBond
        | GovernanceActionTypeV1::GreenLabelSlashBond => GovernanceProposalTypeV1::GreenLabel,
        GovernanceActionTypeV1::VictimReliefApproveCompensation
        | GovernanceActionTypeV1::VictimReliefRejectClaim
        | GovernanceActionTypeV1::VictimReliefUpdatePolicy
        | GovernanceActionTypeV1::VictimReliefUpholdAppeal
        | GovernanceActionTypeV1::VictimReliefOverturnAppeal
        | GovernanceActionTypeV1::VictimReliefCancelPayout => {
            GovernanceProposalTypeV1::VictimRelief
        }
        GovernanceActionTypeV1::ScamRegistryPublishReport
        | GovernanceActionTypeV1::ScamRegistryRemoveReport
        | GovernanceActionTypeV1::ScamRegistryAppealDecision => {
            GovernanceProposalTypeV1::ScamRegistry
        }
        GovernanceActionTypeV1::ContributorAdd
        | GovernanceActionTypeV1::ContributorRemove
        | GovernanceActionTypeV1::ContributorUpdateRole
        | GovernanceActionTypeV1::ContributorApproveMilestone
        | GovernanceActionTypeV1::ContributorApprovePayout => GovernanceProposalTypeV1::Contributor,
        GovernanceActionTypeV1::ProtocolUpdateParameter => GovernanceProposalTypeV1::Parameter,
        GovernanceActionTypeV1::ProtocolUpgradeProgram => GovernanceProposalTypeV1::Upgrade,
        GovernanceActionTypeV1::ProtocolEmergencyAction => GovernanceProposalTypeV1::Emergency,
    }
}

pub fn map_governance_action_to_module(action_type: GovernanceActionTypeV1) -> ProtocolModuleIdV1 {
    match action_type {
        GovernanceActionTypeV1::TreasuryUpdateRevenueSplit
        | GovernanceActionTypeV1::TreasuryApproveSpending
        | GovernanceActionTypeV1::TreasuryApproveBuilderPayout => ProtocolModuleIdV1::Treasury,
        GovernanceActionTypeV1::GreenLabelApproveCertification
        | GovernanceActionTypeV1::GreenLabelRejectCertification
        | GovernanceActionTypeV1::GreenLabelRevokeCertification
        | GovernanceActionTypeV1::GreenLabelRefundBond
        | GovernanceActionTypeV1::GreenLabelSlashBond => ProtocolModuleIdV1::GreenLabel,
        GovernanceActionTypeV1::VictimReliefApproveCompensation
        | GovernanceActionTypeV1::VictimReliefRejectClaim
        | GovernanceActionTypeV1::VictimReliefUpdatePolicy
        | GovernanceActionTypeV1::VictimReliefUpholdAppeal
        | GovernanceActionTypeV1::VictimReliefOverturnAppeal
        | GovernanceActionTypeV1::VictimReliefCancelPayout => ProtocolModuleIdV1::VictimRelief,
        GovernanceActionTypeV1::ScamRegistryPublishReport
        | GovernanceActionTypeV1::ScamRegistryRemoveReport
        | GovernanceActionTypeV1::ScamRegistryAppealDecision => ProtocolModuleIdV1::ScamRegistry,
        GovernanceActionTypeV1::ContributorAdd
        | GovernanceActionTypeV1::ContributorRemove
        | GovernanceActionTypeV1::ContributorUpdateRole
        | GovernanceActionTypeV1::ContributorApproveMilestone
        | GovernanceActionTypeV1::ContributorApprovePayout => ProtocolModuleIdV1::Contributor,
        GovernanceActionTypeV1::ProtocolUpdateParameter
        | GovernanceActionTypeV1::ProtocolUpgradeProgram
        | GovernanceActionTypeV1::ProtocolEmergencyAction => ProtocolModuleIdV1::Protocol,
    }
}

pub fn governance_payload_from_action_request_v1(
    request: &GovernanceActionRequestV1,
    created_at: i64,
) -> Result<GovernancePayloadV1> {
    require!(
        request.schema_version == GOVERNANCE_PROPOSAL_ACTION_V1_SCHEMA_VERSION,
        CustomError::InvalidGovernancePayloadSchema
    );
    require!(
        request.module_id == map_governance_action_to_module(request.action_type),
        CustomError::GovernanceActionModuleMismatch
    );
    require_keys_eq!(
        request.target_program,
        crate::ID,
        CustomError::GovernanceActionTargetMismatch
    );
    require!(
        request.target_account != Pubkey::default(),
        CustomError::GovernanceActionTargetMismatch
    );
    require!(
        request.parameters_hash != [0u8; 32],
        CustomError::InvalidGovernanceProposal
    );

    Ok(GovernancePayloadV1 {
        schema_version: GOVERNANCE_PAYLOAD_V1_SCHEMA_VERSION,
        action_type: request.action_type,
        module_id: request.module_id,
        target_program: request.target_program,
        target_account: request.target_account,
        parameters_hash: request.parameters_hash,
        evidence_hash: request.evidence_hash,
        created_at,
    })
}

pub fn validate_governance_action_target(
    action_type: GovernanceActionTypeV1,
    module_id: ProtocolModuleIdV1,
    target_program: Pubkey,
    target_account: Pubkey,
    payload_hash: [u8; 32],
) -> Result<()> {
    require!(
        module_id == map_governance_action_to_module(action_type),
        CustomError::InvalidGovernanceProposal
    );
    require!(
        target_program != Pubkey::default(),
        CustomError::InvalidGovernanceProposal
    );
    require!(
        target_account != Pubkey::default(),
        CustomError::InvalidGovernanceProposal
    );
    require!(
        payload_hash != [0u8; 32],
        CustomError::InvalidGovernanceProposal
    );

    Ok(())
}

pub fn hash_governance_payload_v1(payload: &GovernancePayloadV1) -> Result<[u8; 32]> {
    require!(
        payload.schema_version == GOVERNANCE_PAYLOAD_V1_SCHEMA_VERSION,
        CustomError::InvalidGovernanceProposal
    );
    require!(
        payload.module_id == map_governance_action_to_module(payload.action_type),
        CustomError::InvalidGovernanceProposal
    );
    require!(
        payload.target_program != Pubkey::default(),
        CustomError::InvalidGovernanceProposal
    );
    require!(
        payload.target_account != Pubkey::default(),
        CustomError::InvalidGovernanceProposal
    );

    let envelope = GovernancePayloadHashEnvelopeV1 {
        domain_separator: GOVERNANCE_PAYLOAD_V1_DOMAIN_SEPARATOR_BYTES,
        payload: *payload,
    };

    hash_contributor_payload(&envelope)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    const PROGRAM: Pubkey = Pubkey::new_from_array([1; 32]);
    const ACCOUNT: Pubkey = Pubkey::new_from_array([2; 32]);
    const PARAMETERS_HASH: [u8; 32] = [3; 32];
    const EVIDENCE_HASH: [u8; 32] = [4; 32];

    const ALL_ACTIONS: [GovernanceActionTypeV1; 25] = [
        GovernanceActionTypeV1::TreasuryUpdateRevenueSplit,
        GovernanceActionTypeV1::TreasuryApproveSpending,
        GovernanceActionTypeV1::TreasuryApproveBuilderPayout,
        GovernanceActionTypeV1::GreenLabelApproveCertification,
        GovernanceActionTypeV1::GreenLabelRejectCertification,
        GovernanceActionTypeV1::GreenLabelRevokeCertification,
        GovernanceActionTypeV1::GreenLabelRefundBond,
        GovernanceActionTypeV1::GreenLabelSlashBond,
        GovernanceActionTypeV1::VictimReliefApproveCompensation,
        GovernanceActionTypeV1::VictimReliefRejectClaim,
        GovernanceActionTypeV1::VictimReliefUpdatePolicy,
        GovernanceActionTypeV1::ScamRegistryPublishReport,
        GovernanceActionTypeV1::ScamRegistryRemoveReport,
        GovernanceActionTypeV1::ScamRegistryAppealDecision,
        GovernanceActionTypeV1::ContributorAdd,
        GovernanceActionTypeV1::ContributorRemove,
        GovernanceActionTypeV1::ContributorUpdateRole,
        GovernanceActionTypeV1::ContributorApproveMilestone,
        GovernanceActionTypeV1::ContributorApprovePayout,
        GovernanceActionTypeV1::ProtocolUpdateParameter,
        GovernanceActionTypeV1::ProtocolUpgradeProgram,
        GovernanceActionTypeV1::ProtocolEmergencyAction,
        GovernanceActionTypeV1::VictimReliefUpholdAppeal,
        GovernanceActionTypeV1::VictimReliefOverturnAppeal,
        GovernanceActionTypeV1::VictimReliefCancelPayout,
    ];

    fn payload(action_type: GovernanceActionTypeV1) -> GovernancePayloadV1 {
        GovernancePayloadV1 {
            schema_version: GOVERNANCE_PAYLOAD_V1_SCHEMA_VERSION,
            action_type,
            module_id: map_governance_action_to_module(action_type),
            target_program: PROGRAM,
            target_account: ACCOUNT,
            parameters_hash: PARAMETERS_HASH,
            evidence_hash: EVIDENCE_HASH,
            created_at: 100,
        }
    }

    #[test]
    fn governance_action_stable_codes_are_unique_and_append_only_snapshot() {
        let mut codes = BTreeSet::new();
        for action_type in ALL_ACTIONS {
            assert!(codes.insert(governance_action_stable_code_v1(action_type)));
        }
        assert_eq!(codes.len(), ALL_ACTIONS.len());
        assert_eq!(
            ALL_ACTIONS.map(governance_action_stable_code_v1),
            [
                0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22,
                23, 24,
            ]
        );
    }

    #[test]
    fn governance_action_stable_code_roundtrips() {
        for action_type in ALL_ACTIONS {
            let code = governance_action_stable_code_v1(action_type);
            assert_eq!(
                governance_action_from_stable_code_v1(code).unwrap(),
                action_type
            );
        }
    }

    #[test]
    fn unknown_governance_action_stable_code_is_rejected() {
        let err = governance_action_from_stable_code_v1(200).unwrap_err();
        assert_eq!(err, CustomError::InvalidGovernanceActionCode.into());
    }

    #[test]
    fn all_governance_actions_have_module_mapping() {
        for action_type in ALL_ACTIONS {
            let module_id = map_governance_action_to_module(action_type);
            validate_governance_action_target(action_type, module_id, PROGRAM, ACCOUNT, [9; 32])
                .unwrap();
        }
    }

    #[test]
    fn governance_action_maps_to_governance_proposal_category() {
        assert_eq!(
            map_governance_action_to_governance_proposal_type_v1(
                GovernanceActionTypeV1::TreasuryApproveSpending
            ),
            GovernanceProposalTypeV1::Treasury
        );
        assert_eq!(
            map_governance_action_to_governance_proposal_type_v1(
                GovernanceActionTypeV1::ContributorApprovePayout
            ),
            GovernanceProposalTypeV1::Contributor
        );
        assert_eq!(
            map_governance_action_to_governance_proposal_type_v1(
                GovernanceActionTypeV1::GreenLabelSlashBond
            ),
            GovernanceProposalTypeV1::GreenLabel
        );
        assert_eq!(
            map_governance_action_to_governance_proposal_type_v1(
                GovernanceActionTypeV1::VictimReliefApproveCompensation
            ),
            GovernanceProposalTypeV1::VictimRelief
        );
        assert_eq!(
            map_governance_action_to_governance_proposal_type_v1(
                GovernanceActionTypeV1::VictimReliefUpholdAppeal
            ),
            GovernanceProposalTypeV1::VictimRelief
        );
        assert_eq!(
            map_governance_action_to_governance_proposal_type_v1(
                GovernanceActionTypeV1::VictimReliefOverturnAppeal
            ),
            GovernanceProposalTypeV1::VictimRelief
        );
        assert_eq!(
            map_governance_action_to_governance_proposal_type_v1(
                GovernanceActionTypeV1::VictimReliefCancelPayout
            ),
            GovernanceProposalTypeV1::VictimRelief
        );
        assert_eq!(
            map_governance_action_to_governance_proposal_type_v1(
                GovernanceActionTypeV1::ScamRegistryPublishReport
            ),
            GovernanceProposalTypeV1::ScamRegistry
        );
        assert_eq!(
            map_governance_action_to_governance_proposal_type_v1(
                GovernanceActionTypeV1::ProtocolUpdateParameter
            ),
            GovernanceProposalTypeV1::Parameter
        );
        assert_eq!(
            map_governance_action_to_governance_proposal_type_v1(
                GovernanceActionTypeV1::ProtocolUpgradeProgram
            ),
            GovernanceProposalTypeV1::Upgrade
        );
        assert_eq!(
            map_governance_action_to_governance_proposal_type_v1(
                GovernanceActionTypeV1::ProtocolEmergencyAction
            ),
            GovernanceProposalTypeV1::Emergency
        );
    }

    #[test]
    fn module_mapping_matches_expected_domains() {
        assert_eq!(
            map_governance_action_to_module(GovernanceActionTypeV1::TreasuryApproveSpending),
            ProtocolModuleIdV1::Treasury
        );
        assert_eq!(
            map_governance_action_to_module(GovernanceActionTypeV1::GreenLabelSlashBond),
            ProtocolModuleIdV1::GreenLabel
        );
        assert_eq!(
            map_governance_action_to_module(GovernanceActionTypeV1::VictimReliefRejectClaim),
            ProtocolModuleIdV1::VictimRelief
        );
        assert_eq!(
            map_governance_action_to_module(GovernanceActionTypeV1::ScamRegistryAppealDecision),
            ProtocolModuleIdV1::ScamRegistry
        );
        assert_eq!(
            map_governance_action_to_module(GovernanceActionTypeV1::ContributorApprovePayout),
            ProtocolModuleIdV1::Contributor
        );
        assert_eq!(
            map_governance_action_to_module(GovernanceActionTypeV1::ProtocolEmergencyAction),
            ProtocolModuleIdV1::Protocol
        );
    }

    #[test]
    fn all_governance_actions_map_to_security_actions() {
        for action_type in ALL_ACTIONS {
            map_governance_action_to_security_action(action_type).unwrap();
        }
    }

    #[test]
    fn currently_executable_actions_keep_legacy_security_mappings() {
        assert_eq!(
            map_governance_action_to_security_action(GovernanceActionTypeV1::GreenLabelRefundBond)
                .unwrap(),
            ActionType::GreenLabelRefund
        );
        assert_eq!(
            map_governance_action_to_security_action(GovernanceActionTypeV1::GreenLabelSlashBond)
                .unwrap(),
            ActionType::GreenLabelSlash
        );
        assert_eq!(
            map_governance_action_to_security_action(GovernanceActionTypeV1::ContributorAdd)
                .unwrap(),
            ActionType::ContributorAddContributor
        );
        assert_eq!(
            map_governance_action_to_security_action(
                GovernanceActionTypeV1::ContributorApprovePayout
            )
            .unwrap(),
            ActionType::ContributorApproveBuilderPayout
        );
    }

    #[test]
    fn future_actions_map_to_distinct_security_placeholders() {
        assert_eq!(
            map_governance_action_to_security_action(
                GovernanceActionTypeV1::VictimReliefApproveCompensation
            )
            .unwrap(),
            ActionType::VictimReliefApproveCompensation
        );
        assert_eq!(
            map_governance_action_to_security_action(
                GovernanceActionTypeV1::ScamRegistryPublishReport
            )
            .unwrap(),
            ActionType::ScamRegistryPublishReport
        );
        assert_eq!(
            map_governance_action_to_security_action(
                GovernanceActionTypeV1::ProtocolUpgradeProgram
            )
            .unwrap(),
            ActionType::ProtocolUpgradeProgram
        );
    }

    #[test]
    fn payload_hash_is_deterministic_for_same_input() {
        let payload = payload(GovernanceActionTypeV1::ContributorAdd);
        assert_eq!(
            hash_governance_payload_v1(&payload).unwrap(),
            hash_governance_payload_v1(&payload).unwrap()
        );
    }

    #[test]
    fn payload_hash_changes_when_any_field_changes() {
        let base = payload(GovernanceActionTypeV1::ContributorAdd);
        let base_hash = hash_governance_payload_v1(&base).unwrap();

        let mut changed_action = base;
        changed_action.action_type = GovernanceActionTypeV1::ContributorRemove;
        changed_action.module_id = map_governance_action_to_module(changed_action.action_type);
        assert_ne!(
            base_hash,
            hash_governance_payload_v1(&changed_action).unwrap()
        );

        let mut changed_program = base;
        changed_program.target_program = Pubkey::new_from_array([5; 32]);
        assert_ne!(
            base_hash,
            hash_governance_payload_v1(&changed_program).unwrap()
        );

        let mut changed_account = base;
        changed_account.target_account = Pubkey::new_from_array([6; 32]);
        assert_ne!(
            base_hash,
            hash_governance_payload_v1(&changed_account).unwrap()
        );

        let mut changed_parameters = base;
        changed_parameters.parameters_hash = [7; 32];
        assert_ne!(
            base_hash,
            hash_governance_payload_v1(&changed_parameters).unwrap()
        );

        let mut changed_evidence = base;
        changed_evidence.evidence_hash = [8; 32];
        assert_ne!(
            base_hash,
            hash_governance_payload_v1(&changed_evidence).unwrap()
        );

        let mut changed_time = base;
        changed_time.created_at = 101;
        assert_ne!(
            base_hash,
            hash_governance_payload_v1(&changed_time).unwrap()
        );
    }

    #[test]
    fn domain_separator_affects_payload_hash() {
        let payload = payload(GovernanceActionTypeV1::ContributorAdd);
        let domain_hash = hash_governance_payload_v1(&payload).unwrap();

        let raw_hash = hash_contributor_payload(&payload).unwrap();

        assert_ne!(domain_hash, raw_hash);
    }
}
