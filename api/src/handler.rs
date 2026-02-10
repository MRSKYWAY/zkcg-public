use axum::{Extension, Json, http::StatusCode};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use std::sync::{Arc, Mutex};
use zkcg_common::{errors::ProtocolError, types::Commitment};
#[cfg(feature = "zk-vm")]
use zkcg_verifier::backend::ProofBackend;
#[cfg(feature = "zk-vm")]
use zkcg_verifier::backend_zkvm::ZkVmBackend;
use zkcg_verifier::engine::{PublicInputs, VerifierEngine};

#[cfg(feature = "zk-vm")]
use zkcg_zkvm_host::{ZkVmProverError, prove as zkvm_prove};

#[cfg(feature = "zk-vm")]
use crate::models::ProvePublicInputs;
use crate::models::{
    ComplianceEvaluateRequest, ComplianceEvaluateResponse, DemoProveRequest, DemoProveResponse,
    DemoVerifyRequest, DemoVerifyResponse, ProveRequest, ProveResponse, SubmitProofRequest,
    SubmitProofResponse,
};

#[derive(Clone)]
pub struct AppState {
    pub engine: Arc<Mutex<VerifierEngine>>,
}

pub async fn submit_proof(
    Extension(state): Extension<AppState>,
    Json(req): Json<SubmitProofRequest>,
) -> Result<Json<SubmitProofResponse>, (StatusCode, String)> {
    let mut engine = state.engine.lock().unwrap();
    let proof_bytes = STANDARD
        .decode(&req.proof)
        .map_err(|_| (StatusCode::BAD_REQUEST, "invalid base64 proof".to_string()))?;

    let inputs = PublicInputs {
        threshold: req.public_inputs.threshold,
        old_state_root: req.public_inputs.old_state_root,
        nonce: req.public_inputs.nonce,
    };

    let commitment = Commitment(req.new_state_commitment);
    engine
        .process_transition(&proof_bytes, inputs, commitment)
        .map_err(map_error)?;

    Ok(Json(SubmitProofResponse {
        status: "accepted".to_string(),
    }))
}

fn map_error(err: ProtocolError) -> (StatusCode, String) {
    use ProtocolError::*;

    match err {
        InvalidFormat => (StatusCode::BAD_REQUEST, err.to_string()),
        InvalidNonce => (StatusCode::CONFLICT, err.to_string()),
        StateMismatch => (StatusCode::CONFLICT, err.to_string()),
        PolicyViolation => (StatusCode::UNPROCESSABLE_ENTITY, err.to_string()),
        InvalidProof => (StatusCode::BAD_REQUEST, err.to_string()),
        CommitmentMismatch => (StatusCode::BAD_REQUEST, err.to_string()),
    }
}

#[cfg(feature = "zk-vm")]
fn map_prover_error(err: ZkVmProverError) -> (StatusCode, String) {
    match err {
        ZkVmProverError::PolicyViolation => (
            StatusCode::UNPROCESSABLE_ENTITY,
            "policy violation".to_string(),
        ),
        ZkVmProverError::ExecutionFailed => {
            (StatusCode::BAD_REQUEST, "zkvm execution failed".to_string())
        }
    }
}

#[cfg(feature = "zk-vm")]
pub async fn prove(
    Extension(_state): Extension<AppState>,
    Json(req): Json<ProveRequest>,
) -> Result<Json<ProveResponse>, (StatusCode, String)> {
    if std::env::var("ZKCG_ENABLE_PROVER").is_err() {
        return Err((StatusCode::FORBIDDEN, "prover disabled".into()));
    }

    let old_state_root = [0u8; 32];
    let nonce = 1;

    let proof = zkvm_prove(req.secret_value, req.threshold, old_state_root, nonce)
        .map_err(map_prover_error)?;

    Ok(Json(ProveResponse {
        proof: STANDARD.encode(&proof),
        public_inputs: ProvePublicInputs {
            threshold: req.threshold,
        },
        commitment: {
            let mut c = [0u8; 32];
            c[0] = (req.secret_value % 256) as u8;
            c
        },
    }))
}

#[cfg(not(feature = "zk-vm"))]
pub async fn prove(
    Extension(_state): Extension<AppState>,
    Json(_req): Json<ProveRequest>,
) -> Result<Json<ProveResponse>, (StatusCode, String)> {
    Err((
        StatusCode::NOT_IMPLEMENTED,
        "zk-vm feature is disabled".to_string(),
    ))
}

#[cfg(not(feature = "zk-vm"))]
pub fn demo_prove(score: u64, threshold: u64) -> Result<Vec<u8>, ProtocolError> {
    if score > 100 || threshold > 100 {
        return Err(ProtocolError::PolicyViolation);
    }

    Ok(format!("demo:{score}:{threshold}").into_bytes())
}

#[cfg(feature = "zk-vm")]
pub fn demo_prove(score: u64, threshold: u64) -> Result<Vec<u8>, ProtocolError> {
    if score > 100 || threshold > 100 {
        return Err(ProtocolError::PolicyViolation);
    }

    let old_state_root = [0u8; 32];
    let nonce = 1;

    let proof = zkvm_prove(score, threshold, old_state_root, nonce)
        .map_err(|_| ProtocolError::InvalidProof)?;

    Ok(proof)
}

#[cfg(not(feature = "zk-vm"))]
pub fn demo_verify(proof_b64: &str, threshold: u64) -> Result<bool, ProtocolError> {
    let proof_bytes = STANDARD
        .decode(proof_b64)
        .map_err(|_| ProtocolError::InvalidFormat)?;

    let decoded = String::from_utf8(proof_bytes).map_err(|_| ProtocolError::InvalidFormat)?;

    let parts: Vec<&str> = decoded.split(':').collect();
    if parts.len() != 3 || parts[0] != "demo" {
        return Ok(false);
    }

    let encoded_threshold = parts[2]
        .parse::<u64>()
        .map_err(|_| ProtocolError::InvalidFormat)?;

    Ok(encoded_threshold == threshold)
}

#[cfg(feature = "zk-vm")]
pub fn demo_verify(proof_b64: &str, threshold: u64) -> Result<bool, ProtocolError> {
    let proof_bytes = STANDARD
        .decode(proof_b64)
        .map_err(|_| ProtocolError::InvalidFormat)?;

    let public_inputs = PublicInputs {
        threshold,
        old_state_root: [0u8; 32],
        nonce: 1,
    };

    let backend = ZkVmBackend;
    match backend.verify(&proof_bytes, &public_inputs) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

pub async fn demo_prove_handler(
    Json(req): Json<DemoProveRequest>,
) -> Result<Json<DemoProveResponse>, (StatusCode, String)> {
    let proof = demo_prove(req.score, req.threshold).map_err(map_error)?;

    Ok(Json(DemoProveResponse {
        proof: STANDARD.encode(&proof),
        proof_size_bytes: proof.len(),
        note: "Demo-only stateless proof",
    }))
}

pub async fn demo_verify_handler(
    Json(req): Json<DemoVerifyRequest>,
) -> Result<Json<DemoVerifyResponse>, (StatusCode, String)> {
    let verified = demo_verify(&req.proof, req.threshold).map_err(map_error)?;

    Ok(Json(DemoVerifyResponse { verified }))
}

pub async fn compliance_evaluate_handler(
    Json(req): Json<ComplianceEvaluateRequest>,
) -> Result<Json<ComplianceEvaluateResponse>, (StatusCode, String)> {
    if req.applicant_id.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "applicant_id must not be empty".to_string(),
        ));
    }

    if req.risk_score > 100 || req.threshold > 100 {
        return Err((
            StatusCode::BAD_REQUEST,
            "risk_score and threshold must be <= 100".to_string(),
        ));
    }

    if req.monthly_income_cents == 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            "monthly_income_cents must be > 0".to_string(),
        ));
    }

    let dti_bps = req.monthly_debt_cents.saturating_mul(10_000) / req.monthly_income_cents;
    let credit_to_income_bps =
        req.requested_credit_cents.saturating_mul(10_000) / req.monthly_income_cents;

    let mut reasons = Vec::new();

    let policy_score = req.risk_score <= req.threshold;
    if !policy_score {
        reasons.push(format!(
            "risk_score {} exceeds threshold {}",
            req.risk_score, req.threshold
        ));
    }

    let policy_dti = dti_bps <= 4_500;
    if !policy_dti {
        reasons.push(format!("debt_to_income_bps {} exceeds max 4500", dti_bps));
    }

    let policy_credit = credit_to_income_bps <= 30_000;
    if !policy_credit {
        reasons.push(format!(
            "credit_to_income_bps {} exceeds max 30000",
            credit_to_income_bps
        ));
    }

    let policy_passed = policy_score && policy_dti && policy_credit;

    let proof = demo_prove(req.risk_score, req.threshold).map_err(map_error)?;
    let proof_b64 = STANDARD.encode(&proof);
    let proof_verified = demo_verify(&proof_b64, req.threshold).map_err(map_error)?;

    let risk_band = if req.risk_score <= 20 {
        "low"
    } else if req.risk_score <= 50 {
        "medium"
    } else {
        "high"
    };

    let decision = if policy_passed && proof_verified {
        "approved"
    } else {
        if !proof_verified {
            reasons.push("proof verification failed".to_string());
        }
        "denied"
    };

    Ok(Json(ComplianceEvaluateResponse {
        application_id: format!("{}-{}", req.applicant_id, req.risk_score),
        decision,
        policy_passed,
        risk_band,
        reasons,
        proof_verified,
        proof: proof_b64,
    }))
}