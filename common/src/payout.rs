use crate::types::{
    BulkPayoutRoundDecisionCommitment, BulkPayoutRoundFacts, CHUNK_COUNT_LIMIT_EXCEEDED_BIT,
    DecisionCode, MAX_RECIPIENT_LIMIT_EXCEEDED_BIT, PayoutReleaseDecisionCommitment,
    PayoutReleaseFacts, RECIPIENT_AML_FAILED_BIT, RECIPIENT_APPROVAL_MISSING_BIT,
    RECIPIENT_KYC_MISSING_BIT, RECIPIENT_SANCTIONS_HIT_BIT, RECIPIENT_SNAPSHOT_EXPIRED_BIT,
    RELEASE_WINDOW_EXPIRED_BIT, ROUND_REPLAYED_BIT, ROW_LIMIT_EXCEEDED_BIT,
    TOTAL_AMOUNT_LIMIT_EXCEEDED_BIT,
};

pub const BULK_PAYOUT_REASON_CODE_MAP: [(u32, &str); 6] = [
    (RELEASE_WINDOW_EXPIRED_BIT, "release_window_expired"),
    (ROW_LIMIT_EXCEEDED_BIT, "row_limit_exceeded"),
    (
        TOTAL_AMOUNT_LIMIT_EXCEEDED_BIT,
        "total_amount_limit_exceeded",
    ),
    (
        MAX_RECIPIENT_LIMIT_EXCEEDED_BIT,
        "max_recipient_limit_exceeded",
    ),
    (CHUNK_COUNT_LIMIT_EXCEEDED_BIT, "chunk_count_limit_exceeded"),
    (ROUND_REPLAYED_BIT, "round_replayed"),
];

pub const PAYOUT_RELEASE_REASON_CODE_MAP: [(u32, &str); 11] = [
    (RELEASE_WINDOW_EXPIRED_BIT, "release_window_expired"),
    (ROW_LIMIT_EXCEEDED_BIT, "row_limit_exceeded"),
    (
        TOTAL_AMOUNT_LIMIT_EXCEEDED_BIT,
        "total_amount_limit_exceeded",
    ),
    (
        MAX_RECIPIENT_LIMIT_EXCEEDED_BIT,
        "max_recipient_limit_exceeded",
    ),
    (CHUNK_COUNT_LIMIT_EXCEEDED_BIT, "chunk_count_limit_exceeded"),
    (ROUND_REPLAYED_BIT, "round_replayed"),
    (
        RECIPIENT_SNAPSHOT_EXPIRED_BIT,
        "recipient_snapshot_expired",
    ),
    (
        RECIPIENT_APPROVAL_MISSING_BIT,
        "recipient_approval_missing",
    ),
    (RECIPIENT_KYC_MISSING_BIT, "recipient_kyc_missing"),
    (RECIPIENT_AML_FAILED_BIT, "recipient_aml_failed"),
    (RECIPIENT_SANCTIONS_HIT_BIT, "recipient_sanctions_hit"),
];

pub fn evaluate_bulk_payout_round_v1(
    facts: &BulkPayoutRoundFacts,
) -> BulkPayoutRoundDecisionCommitment {
    let mut reason_bits = 0u32;

    if facts.release_window_expired {
        reason_bits |= RELEASE_WINDOW_EXPIRED_BIT;
    }
    if facts.row_limit_exceeded {
        reason_bits |= ROW_LIMIT_EXCEEDED_BIT;
    }
    if facts.total_amount_limit_exceeded {
        reason_bits |= TOTAL_AMOUNT_LIMIT_EXCEEDED_BIT;
    }
    if facts.max_recipient_limit_exceeded {
        reason_bits |= MAX_RECIPIENT_LIMIT_EXCEEDED_BIT;
    }
    if facts.chunk_count_limit_exceeded {
        reason_bits |= CHUNK_COUNT_LIMIT_EXCEEDED_BIT;
    }
    if facts.round_replayed {
        reason_bits |= ROUND_REPLAYED_BIT;
    }

    BulkPayoutRoundDecisionCommitment {
        decision: if reason_bits == 0 {
            DecisionCode::Approved
        } else {
            DecisionCode::Denied
        },
        reason_bits,
        operator_id_hash: facts.operator_id_hash,
        program_id_hash: facts.program_id_hash,
        asset_id_hash: facts.asset_id_hash,
        round_id_hash: facts.round_id_hash,
        manifest_root: facts.manifest_root,
        row_count: facts.row_count,
        chunk_count: facts.chunk_count,
        total_amount_units: facts.total_amount_units,
        max_recipient_amount_units: facts.max_recipient_amount_units,
        round_nonce: facts.round_nonce,
        release_window_ends_at: facts.release_window_ends_at,
    }
}

pub fn evaluate_payout_release_v1(
    facts: &PayoutReleaseFacts,
) -> PayoutReleaseDecisionCommitment {
    let mut reason_bits = 0u32;

    if facts.release_window_expired {
        reason_bits |= RELEASE_WINDOW_EXPIRED_BIT;
    }
    if facts.row_limit_exceeded {
        reason_bits |= ROW_LIMIT_EXCEEDED_BIT;
    }
    if facts.total_amount_limit_exceeded {
        reason_bits |= TOTAL_AMOUNT_LIMIT_EXCEEDED_BIT;
    }
    if facts.max_recipient_limit_exceeded {
        reason_bits |= MAX_RECIPIENT_LIMIT_EXCEEDED_BIT;
    }
    if facts.chunk_count_limit_exceeded {
        reason_bits |= CHUNK_COUNT_LIMIT_EXCEEDED_BIT;
    }
    if facts.round_replayed {
        reason_bits |= ROUND_REPLAYED_BIT;
    }
    if facts.recipient_snapshot_expired {
        reason_bits |= RECIPIENT_SNAPSHOT_EXPIRED_BIT;
    }
    if !facts.recipient_set_complete || !facts.all_recipients_approved {
        reason_bits |= RECIPIENT_APPROVAL_MISSING_BIT;
    }
    if !facts.all_recipients_kyc_passed {
        reason_bits |= RECIPIENT_KYC_MISSING_BIT;
    }
    if !facts.all_recipients_aml_cleared {
        reason_bits |= RECIPIENT_AML_FAILED_BIT;
    }
    if !facts.all_recipients_sanctions_clear {
        reason_bits |= RECIPIENT_SANCTIONS_HIT_BIT;
    }

    PayoutReleaseDecisionCommitment {
        decision: if reason_bits == 0 {
            DecisionCode::Approved
        } else {
            DecisionCode::Denied
        },
        reason_bits,
        operator_id_hash: facts.operator_id_hash,
        program_id_hash: facts.program_id_hash,
        asset_id_hash: facts.asset_id_hash,
        round_id_hash: facts.round_id_hash,
        manifest_root: facts.manifest_root,
        recipient_snapshot_hash: facts.recipient_snapshot_hash,
        row_count: facts.row_count,
        chunk_count: facts.chunk_count,
        total_amount_units: facts.total_amount_units,
        max_recipient_amount_units: facts.max_recipient_amount_units,
        round_nonce: facts.round_nonce,
        release_window_ends_at: facts.release_window_ends_at,
        recipient_snapshot_expires_at: facts.recipient_snapshot_expires_at,
    }
}
