use serde::{Deserialize, Serialize};
use zkcg_common::types::Hash;

#[derive(Debug, Deserialize)]
pub struct SubmitProofRequest {
    pub proof: String,
    pub public_inputs: PublicInputsDto,
    pub new_state_commitment: Hash,
}

#[derive(Debug, Deserialize)]
pub struct PublicInputsDto {
    pub threshold: u64,
    pub old_state_root: Hash,
    pub nonce: u64,
}

#[derive(Debug, Serialize)]
pub struct SubmitProofResponse {
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProveRequest {
    pub secret_value: u64,
    pub threshold: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProveResponse {
    pub proof: String,
    pub public_inputs: ProvePublicInputs,
    pub commitment: [u8; 32],
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProvePublicInputs {
    pub threshold: u64,
}

#[derive(Debug, Deserialize)]
pub struct DemoProveRequest {
    pub score: u64,
    pub threshold: u64,
}

#[derive(Debug, Serialize)]
pub struct DemoProveResponse {
    pub proof: String,
    pub proof_size_bytes: usize,
    pub note: &'static str,
}

#[derive(Debug, Deserialize)]
pub struct DemoVerifyRequest {
    pub proof: String,
    pub threshold: u64,
}

#[derive(Debug, Serialize)]
pub struct DemoVerifyResponse {
    pub verified: bool,
}

#[derive(Debug, Deserialize)]
pub struct ComplianceEvaluateRequest {
    pub applicant_id: String,
    pub risk_score: u64,
    pub threshold: u64,
    pub monthly_income_cents: u64,
    pub monthly_debt_cents: u64,
    pub requested_credit_cents: u64,
}

#[derive(Debug, Serialize)]
pub struct ComplianceEvaluateResponse {
    pub application_id: String,
    pub decision: &'static str,
    pub policy_passed: bool,
    pub risk_band: &'static str,
    pub reasons: Vec<String>,
    pub proof_verified: bool,
    pub proof: String,
}