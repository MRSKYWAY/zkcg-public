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