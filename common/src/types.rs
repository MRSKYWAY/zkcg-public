use serde::{Deserialize, Serialize};

pub type Hash = [u8; 32];
pub type Address = [u8; 20];

pub const ATTESTATION_EXPIRED_BIT: u32 = 1 << 0;
pub const KYC_MISSING_BIT: u32 = 1 << 1;
pub const AML_FAILED_BIT: u32 = 1 << 2;
pub const SANCTIONS_HIT_BIT: u32 = 1 << 3;
pub const JURISDICTION_BLOCKED_BIT: u32 = 1 << 4;
pub const ACCREDITATION_MISSING_BIT: u32 = 1 << 5;
pub const WALLET_REVOKED_BIT: u32 = 1 << 6;
pub const HOLDING_PERIOD_NOT_MET_BIT: u32 = 1 << 7;
pub const POSITION_LIMIT_EXCEEDED_BIT: u32 = 1 << 8;
pub const CONCENTRATION_LIMIT_EXCEEDED_BIT: u32 = 1 << 9;
pub const RELEASE_WINDOW_EXPIRED_BIT: u32 = 1 << 10;
pub const ROW_LIMIT_EXCEEDED_BIT: u32 = 1 << 11;
pub const TOTAL_AMOUNT_LIMIT_EXCEEDED_BIT: u32 = 1 << 12;
pub const MAX_RECIPIENT_LIMIT_EXCEEDED_BIT: u32 = 1 << 13;
pub const CHUNK_COUNT_LIMIT_EXCEEDED_BIT: u32 = 1 << 14;
pub const ROUND_REPLAYED_BIT: u32 = 1 << 15;
pub const RECIPIENT_SNAPSHOT_EXPIRED_BIT: u32 = 1 << 16;
pub const RECIPIENT_APPROVAL_MISSING_BIT: u32 = 1 << 17;
pub const RECIPIENT_KYC_MISSING_BIT: u32 = 1 << 18;
pub const RECIPIENT_AML_FAILED_BIT: u32 = 1 << 19;
pub const RECIPIENT_SANCTIONS_HIT_BIT: u32 = 1 << 20;

pub const RWA_REASON_CODE_MAP: [(u32, &str); 10] = [
    (ATTESTATION_EXPIRED_BIT, "attestation_expired"),
    (KYC_MISSING_BIT, "kyc_missing"),
    (AML_FAILED_BIT, "aml_failed"),
    (SANCTIONS_HIT_BIT, "sanctions_hit"),
    (JURISDICTION_BLOCKED_BIT, "jurisdiction_blocked"),
    (ACCREDITATION_MISSING_BIT, "accreditation_missing"),
    (WALLET_REVOKED_BIT, "wallet_revoked"),
    (HOLDING_PERIOD_NOT_MET_BIT, "holding_period_not_met"),
    (POSITION_LIMIT_EXCEEDED_BIT, "position_limit_exceeded"),
    (
        CONCENTRATION_LIMIT_EXCEEDED_BIT,
        "concentration_limit_exceeded",
    ),
];

pub const PHASE1_SCORE_V1_POLICY_VERSION: &str = "phase1.score.v1";
pub const RWA_CREDIT_ONBOARDING_V1_POLICY_VERSION: &str = "rwa.credit.onboarding.v1";
pub const RWA_CREDIT_TRANSFER_V1_POLICY_VERSION: &str = "rwa.credit.transfer.v1";
pub const BULK_PAYOUT_ROUND_V1_POLICY_VERSION: &str = "bulk.payout.round.v1";
pub const PAYOUT_RELEASE_V1_POLICY_VERSION: &str = "payout.release.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Commitment(pub Hash);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum InvestorTypeCode {
    Retail = 0,
    Accredited = 1,
    Institutional = 2,
}

impl InvestorTypeCode {
    pub const fn as_u8(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum DecisionCode {
    Eligible = 0,
    Ineligible = 1,
    Approved = 2,
    Denied = 3,
}

impl DecisionCode {
    pub const fn as_u8(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum EligibilityClass {
    None = 0,
    Accredited = 1,
    Institutional = 2,
}

impl EligibilityClass {
    pub const fn as_u8(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Phase1ScoreClaims {
    pub threshold: u64,
    pub old_state_root: Hash,
    pub nonce: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RwaCreditOnboardingFacts {
    pub issuer_id_hash: Hash,
    pub asset_id_hash: Hash,
    pub wallet_address: Address,
    pub investor_type: InvestorTypeCode,
    pub attestation_expired: bool,
    pub accredited: bool,
    pub kyc_passed: bool,
    pub aml_cleared: bool,
    pub sanctions_clear: bool,
    pub jurisdiction_code: u16,
    pub jurisdiction_allowed: bool,
    pub residency_allowed: bool,
    pub wallet_revoked: bool,
    pub expires_at: u64,
    pub evaluation_time: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RwaCreditTransferFacts {
    pub issuer_id_hash: Hash,
    pub asset_id_hash: Hash,
    pub sender_wallet: Address,
    pub receiver_wallet: Address,
    pub receiver_investor_type: InvestorTypeCode,
    pub attestation_expired: bool,
    pub receiver_accredited: bool,
    pub receiver_kyc_passed: bool,
    pub receiver_aml_cleared: bool,
    pub receiver_sanctions_clear: bool,
    pub receiver_jurisdiction_code: u16,
    pub receiver_jurisdiction_allowed: bool,
    pub receiver_residency_allowed: bool,
    pub sender_revoked: bool,
    pub receiver_revoked: bool,
    pub holding_period_met: bool,
    pub position_limit_exceeded: bool,
    pub concentration_limit_exceeded: bool,
    pub transfer_amount_units: u64,
    pub post_transfer_position_units: u64,
    pub wallet_position_limit_units: u64,
    pub post_transfer_concentration_bps: u64,
    pub concentration_limit_bps: u64,
    pub expires_at: u64,
    pub evaluation_time: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RwaCreditOnboardingDecisionCommitment {
    pub decision: DecisionCode,
    pub eligibility_class: EligibilityClass,
    pub reason_bits: u32,
    pub expires_at: u64,
    pub issuer_id_hash: Hash,
    pub asset_id_hash: Hash,
    pub wallet_address: Address,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RwaCreditTransferDecisionCommitment {
    pub decision: DecisionCode,
    pub eligibility_class: EligibilityClass,
    pub reason_bits: u32,
    pub expires_at: u64,
    pub issuer_id_hash: Hash,
    pub asset_id_hash: Hash,
    pub sender_wallet: Address,
    pub receiver_wallet: Address,
    pub transfer_amount_units: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RwaCreditOnboardingClaims {
    pub facts: RwaCreditOnboardingFacts,
    pub expected: RwaCreditOnboardingDecisionCommitment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RwaCreditTransferClaims {
    pub facts: RwaCreditTransferFacts,
    pub expected: RwaCreditTransferDecisionCommitment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BulkPayoutRoundFacts {
    pub operator_id_hash: Hash,
    pub program_id_hash: Hash,
    pub asset_id_hash: Hash,
    pub round_id_hash: Hash,
    pub manifest_root: Hash,
    pub row_count: u64,
    pub chunk_count: u64,
    pub total_amount_units: u64,
    pub max_recipient_amount_units: u64,
    pub round_cap_units: u64,
    pub per_recipient_cap_units: u64,
    pub max_rows_per_round: u64,
    pub max_chunks_per_round: u64,
    pub round_nonce: u64,
    pub release_window_ends_at: u64,
    pub evaluation_time: u64,
    pub release_window_expired: bool,
    pub row_limit_exceeded: bool,
    pub total_amount_limit_exceeded: bool,
    pub max_recipient_limit_exceeded: bool,
    pub chunk_count_limit_exceeded: bool,
    pub round_replayed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BulkPayoutRoundDecisionCommitment {
    pub decision: DecisionCode,
    pub reason_bits: u32,
    pub operator_id_hash: Hash,
    pub program_id_hash: Hash,
    pub asset_id_hash: Hash,
    pub round_id_hash: Hash,
    pub manifest_root: Hash,
    pub row_count: u64,
    pub chunk_count: u64,
    pub total_amount_units: u64,
    pub max_recipient_amount_units: u64,
    pub round_nonce: u64,
    pub release_window_ends_at: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BulkPayoutRoundClaims {
    pub facts: BulkPayoutRoundFacts,
    pub expected: BulkPayoutRoundDecisionCommitment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PayoutReleaseFacts {
    pub operator_id_hash: Hash,
    pub program_id_hash: Hash,
    pub asset_id_hash: Hash,
    pub round_id_hash: Hash,
    pub manifest_root: Hash,
    pub recipient_snapshot_hash: Hash,
    pub row_count: u64,
    pub chunk_count: u64,
    pub total_amount_units: u64,
    pub max_recipient_amount_units: u64,
    pub round_cap_units: u64,
    pub per_recipient_cap_units: u64,
    pub max_rows_per_round: u64,
    pub max_chunks_per_round: u64,
    pub round_nonce: u64,
    pub release_window_ends_at: u64,
    pub recipient_snapshot_expires_at: u64,
    pub evaluation_time: u64,
    pub release_window_expired: bool,
    pub row_limit_exceeded: bool,
    pub total_amount_limit_exceeded: bool,
    pub max_recipient_limit_exceeded: bool,
    pub chunk_count_limit_exceeded: bool,
    pub round_replayed: bool,
    pub recipient_snapshot_expired: bool,
    pub recipient_set_complete: bool,
    pub all_recipients_approved: bool,
    pub all_recipients_kyc_passed: bool,
    pub all_recipients_aml_cleared: bool,
    pub all_recipients_sanctions_clear: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PayoutReleaseDecisionCommitment {
    pub decision: DecisionCode,
    pub reason_bits: u32,
    pub operator_id_hash: Hash,
    pub program_id_hash: Hash,
    pub asset_id_hash: Hash,
    pub round_id_hash: Hash,
    pub manifest_root: Hash,
    pub recipient_snapshot_hash: Hash,
    pub row_count: u64,
    pub chunk_count: u64,
    pub total_amount_units: u64,
    pub max_recipient_amount_units: u64,
    pub round_nonce: u64,
    pub release_window_ends_at: u64,
    pub recipient_snapshot_expires_at: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PayoutReleaseClaims {
    pub facts: PayoutReleaseFacts,
    pub expected: PayoutReleaseDecisionCommitment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProofClaims {
    Phase1ScoreV1(Phase1ScoreClaims),
    RwaCreditOnboardingV1(RwaCreditOnboardingClaims),
    RwaCreditTransferV1(RwaCreditTransferClaims),
    BulkPayoutRoundV1(BulkPayoutRoundClaims),
    PayoutReleaseV1(PayoutReleaseClaims),
}

impl ProofClaims {
    pub const fn phase1_score(threshold: u64, old_state_root: Hash, nonce: u64) -> Self {
        Self::Phase1ScoreV1(Phase1ScoreClaims {
            threshold,
            old_state_root,
            nonce,
        })
    }

    pub const fn phase1(self) -> Option<Phase1ScoreClaims> {
        match self {
            Self::Phase1ScoreV1(claims) => Some(claims),
            _ => None,
        }
    }
}

pub fn split_hash_u128(value: &Hash) -> [u128; 2] {
    let mut high = [0u8; 16];
    let mut low = [0u8; 16];
    high.copy_from_slice(&value[..16]);
    low.copy_from_slice(&value[16..]);
    [u128::from_be_bytes(high), u128::from_be_bytes(low)]
}

pub fn split_address_u128(value: &Address) -> [u128; 2] {
    let mut high = [0u8; 16];
    let mut low = [0u8; 16];
    high[12..].copy_from_slice(&value[..4]);
    low.copy_from_slice(&value[4..]);
    [u128::from_be_bytes(high), u128::from_be_bytes(low)]
}

impl RwaCreditOnboardingClaims {
    pub fn identifiers_match(&self) -> bool {
        self.expected.issuer_id_hash == self.facts.issuer_id_hash
            && self.expected.asset_id_hash == self.facts.asset_id_hash
            && self.expected.wallet_address == self.facts.wallet_address
            && self.expected.expires_at == self.facts.expires_at
    }
}

impl RwaCreditTransferClaims {
    pub fn identifiers_match(&self) -> bool {
        self.expected.issuer_id_hash == self.facts.issuer_id_hash
            && self.expected.asset_id_hash == self.facts.asset_id_hash
            && self.expected.sender_wallet == self.facts.sender_wallet
            && self.expected.receiver_wallet == self.facts.receiver_wallet
            && self.expected.transfer_amount_units == self.facts.transfer_amount_units
            && self.expected.expires_at == self.facts.expires_at
    }
}

impl BulkPayoutRoundClaims {
    pub fn identifiers_match(&self) -> bool {
        self.expected.operator_id_hash == self.facts.operator_id_hash
            && self.expected.program_id_hash == self.facts.program_id_hash
            && self.expected.asset_id_hash == self.facts.asset_id_hash
            && self.expected.round_id_hash == self.facts.round_id_hash
            && self.expected.manifest_root == self.facts.manifest_root
            && self.expected.row_count == self.facts.row_count
            && self.expected.chunk_count == self.facts.chunk_count
            && self.expected.total_amount_units == self.facts.total_amount_units
            && self.expected.max_recipient_amount_units == self.facts.max_recipient_amount_units
            && self.expected.round_nonce == self.facts.round_nonce
            && self.expected.release_window_ends_at == self.facts.release_window_ends_at
    }
}

impl PayoutReleaseClaims {
    pub fn identifiers_match(&self) -> bool {
        self.expected.operator_id_hash == self.facts.operator_id_hash
            && self.expected.program_id_hash == self.facts.program_id_hash
            && self.expected.asset_id_hash == self.facts.asset_id_hash
            && self.expected.round_id_hash == self.facts.round_id_hash
            && self.expected.manifest_root == self.facts.manifest_root
            && self.expected.recipient_snapshot_hash == self.facts.recipient_snapshot_hash
            && self.expected.row_count == self.facts.row_count
            && self.expected.chunk_count == self.facts.chunk_count
            && self.expected.total_amount_units == self.facts.total_amount_units
            && self.expected.max_recipient_amount_units == self.facts.max_recipient_amount_units
            && self.expected.round_nonce == self.facts.round_nonce
            && self.expected.release_window_ends_at == self.facts.release_window_ends_at
            && self.expected.recipient_snapshot_expires_at
                == self.facts.recipient_snapshot_expires_at
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZkVmInput {
    Phase1ScoreV1 {
        score: u64,
        claims: Phase1ScoreClaims,
    },
    RwaCreditOnboardingV1(RwaCreditOnboardingClaims),
    RwaCreditTransferV1(RwaCreditTransferClaims),
}

pub type ZkVmJournal = ProofClaims;
