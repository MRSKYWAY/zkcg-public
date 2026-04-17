use crate::types::{
    ACCREDITATION_MISSING_BIT, AML_FAILED_BIT, ATTESTATION_EXPIRED_BIT,
    CONCENTRATION_LIMIT_EXCEEDED_BIT, DecisionCode, EligibilityClass, HOLDING_PERIOD_NOT_MET_BIT,
    InvestorTypeCode, JURISDICTION_BLOCKED_BIT, KYC_MISSING_BIT, POSITION_LIMIT_EXCEEDED_BIT,
    RwaCreditOnboardingDecisionCommitment, RwaCreditOnboardingFacts,
    RwaCreditTransferDecisionCommitment, RwaCreditTransferFacts, SANCTIONS_HIT_BIT,
    WALLET_REVOKED_BIT,
};

pub fn evaluate_rwa_credit_onboarding_v1(
    facts: &RwaCreditOnboardingFacts,
) -> RwaCreditOnboardingDecisionCommitment {
    let institutional = matches!(facts.investor_type, InvestorTypeCode::Institutional);

    let mut reason_bits = 0u32;
    if facts.attestation_expired {
        reason_bits |= ATTESTATION_EXPIRED_BIT;
    }
    if !facts.kyc_passed {
        reason_bits |= KYC_MISSING_BIT;
    }
    if !facts.aml_cleared {
        reason_bits |= AML_FAILED_BIT;
    }
    if !facts.sanctions_clear {
        reason_bits |= SANCTIONS_HIT_BIT;
    }
    if !facts.jurisdiction_allowed || !facts.residency_allowed {
        reason_bits |= JURISDICTION_BLOCKED_BIT;
    }
    if !institutional && !facts.accredited {
        reason_bits |= ACCREDITATION_MISSING_BIT;
    }
    if facts.wallet_revoked {
        reason_bits |= WALLET_REVOKED_BIT;
    }

    let decision = if reason_bits == 0 {
        DecisionCode::Eligible
    } else {
        DecisionCode::Ineligible
    };
    let eligibility_class = if reason_bits == 0 {
        if institutional {
            EligibilityClass::Institutional
        } else {
            EligibilityClass::Accredited
        }
    } else {
        EligibilityClass::None
    };

    RwaCreditOnboardingDecisionCommitment {
        decision,
        eligibility_class,
        reason_bits,
        expires_at: facts.expires_at,
        issuer_id_hash: facts.issuer_id_hash,
        asset_id_hash: facts.asset_id_hash,
        wallet_address: facts.wallet_address,
    }
}

pub fn evaluate_rwa_credit_transfer_v1(
    facts: &RwaCreditTransferFacts,
) -> RwaCreditTransferDecisionCommitment {
    let onboarding = evaluate_rwa_credit_onboarding_v1(&RwaCreditOnboardingFacts {
        issuer_id_hash: facts.issuer_id_hash,
        asset_id_hash: facts.asset_id_hash,
        wallet_address: facts.receiver_wallet,
        investor_type: facts.receiver_investor_type,
        attestation_expired: facts.attestation_expired,
        accredited: facts.receiver_accredited,
        kyc_passed: facts.receiver_kyc_passed,
        aml_cleared: facts.receiver_aml_cleared,
        sanctions_clear: facts.receiver_sanctions_clear,
        jurisdiction_code: facts.receiver_jurisdiction_code,
        jurisdiction_allowed: facts.receiver_jurisdiction_allowed,
        residency_allowed: facts.receiver_residency_allowed,
        wallet_revoked: facts.receiver_revoked,
        expires_at: facts.expires_at,
        evaluation_time: facts.evaluation_time,
    });

    let mut reason_bits = onboarding.reason_bits;
    if !facts.holding_period_met {
        reason_bits |= HOLDING_PERIOD_NOT_MET_BIT;
    }
    if facts.position_limit_exceeded {
        reason_bits |= POSITION_LIMIT_EXCEEDED_BIT;
    }
    if facts.concentration_limit_exceeded {
        reason_bits |= CONCENTRATION_LIMIT_EXCEEDED_BIT;
    }
    if facts.sender_revoked {
        reason_bits |= WALLET_REVOKED_BIT;
    }

    let decision = if reason_bits == 0 {
        DecisionCode::Approved
    } else {
        DecisionCode::Denied
    };

    RwaCreditTransferDecisionCommitment {
        decision,
        eligibility_class: onboarding.eligibility_class,
        reason_bits,
        expires_at: facts.expires_at,
        issuer_id_hash: facts.issuer_id_hash,
        asset_id_hash: facts.asset_id_hash,
        sender_wallet: facts.sender_wallet,
        receiver_wallet: facts.receiver_wallet,
        transfer_amount_units: facts.transfer_amount_units,
    }
}
