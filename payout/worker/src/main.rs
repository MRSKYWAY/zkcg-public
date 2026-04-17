use anyhow::{anyhow, ensure, Context, Result};
use clap::{Parser, Subcommand};
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use sha3::{Digest, Keccak256};
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use zkcg_common::types::{
    DecisionCode, PAYOUT_RELEASE_V1_POLICY_VERSION, PayoutReleaseClaims, ProofClaims,
};
use zkcg_halo2_prover::{
    Halo2PayoutContext, PayoutPolicy, PayoutRecipientSnapshot, PayoutRow, build_payout_release,
    recipient_snapshot_covers_manifest,
};
use zkcg_verifier::{Proof, ProofSystem, Verifier};

#[derive(Debug, Parser)]
#[command(name = "zkcg-payout-worker")]
#[command(about = "Self-hosted bulk payout proving worker for ZKCG")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    ProveRound {
        #[arg(long)]
        manifest: PathBuf,
        #[arg(long)]
        policy: PathBuf,
        #[arg(long)]
        recipient_snapshot: PathBuf,
        #[arg(long)]
        state_db: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
    VerifyRound {
        #[arg(long)]
        proof: PathBuf,
        #[arg(long)]
        claims: PathBuf,
    },
    AuthorizeRelease {
        #[arg(long)]
        proof: PathBuf,
        #[arg(long)]
        claims: PathBuf,
        #[arg(long)]
        state_db: PathBuf,
    },
}

#[derive(Debug, Serialize)]
struct DecisionArtifact {
    policy_version: &'static str,
    proof_system: &'static str,
    decision: &'static str,
    reason_bits: u32,
    reason_codes: Vec<String>,
    claims_hash: String,
    decision_commitment_hash: String,
    manifest_root: String,
    recipient_snapshot_hash: String,
    row_count: u64,
    chunk_count: u64,
    total_amount_units: u64,
    max_recipient_amount_units: u64,
    round_nonce: u64,
    release_window_ends_at: u64,
}

#[derive(Debug, Serialize)]
struct ProofRoundSummary {
    status: &'static str,
    out_dir: String,
    decision: &'static str,
    proof_bytes: usize,
    claims_hash: String,
    decision_commitment_hash: String,
    manifest_root: String,
    recipient_snapshot_hash: String,
    reason_codes: Vec<String>,
}

#[derive(Debug, Serialize)]
struct VerifyRoundSummary {
    verified: bool,
    system: &'static str,
    policy_version: &'static str,
}

#[derive(Debug, Serialize)]
struct ReleaseEnvelope {
    status: &'static str,
    decision: &'static str,
    policy_version: &'static str,
    proof_system: &'static str,
    operator_id_hash: String,
    program_id_hash: String,
    asset_id_hash: String,
    round_id_hash: String,
    manifest_root: String,
    recipient_snapshot_hash: String,
    row_count: u64,
    chunk_count: u64,
    total_amount_units: u64,
    max_recipient_amount_units: u64,
    round_nonce: u64,
    release_window_ends_at: u64,
    claims_hash: String,
    decision_commitment_hash: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::ProveRound {
            manifest,
            policy,
            recipient_snapshot,
            state_db,
            out,
        } => prove_round(&manifest, &policy, &recipient_snapshot, &state_db, &out),
        Commands::VerifyRound { proof, claims } => verify_round(&proof, &claims),
        Commands::AuthorizeRelease {
            proof,
            claims,
            state_db,
        } => authorize_release(&proof, &claims, &state_db),
    }
}

fn prove_round(
    manifest: &Path,
    policy: &Path,
    recipient_snapshot: &Path,
    state_db: &Path,
    out: &Path,
) -> Result<()> {
    let rows = read_manifest(manifest)?;
    let policy = read_policy(policy)?;
    let recipient_snapshot = read_recipient_snapshot(recipient_snapshot)?;
    let evaluation_time = current_unix_seconds()?;

    ensure!(
        recipient_snapshot_covers_manifest(&rows, &recipient_snapshot)?,
        "recipient snapshot must cover every manifest recipient"
    );

    let conn = open_store(state_db)?;
    let provisional =
        build_payout_release(&rows, &policy, &recipient_snapshot, evaluation_time, false)?;
    let already_released = release_exists(&conn, &provisional.claims)?;
    let context = Halo2PayoutContext::new();
    let artifact = if already_released {
        context.prove_release(&rows, &policy, &recipient_snapshot, evaluation_time, true)?
    } else {
        context.prove_release(&rows, &policy, &recipient_snapshot, evaluation_time, false)?
    };
    let build = artifact.build;

    let claims_hash = canonical_hash_hex(&build.claims)?;
    let decision_commitment_hash = canonical_hash_hex(&build.claims.expected)?;
    let reason_codes = build.reason_codes.clone();
    let proof = artifact.proof;

    fs::create_dir_all(out)
        .with_context(|| format!("failed to create output directory {}", out.display()))?;
    fs::write(out.join("proof.bin"), &proof)
        .with_context(|| format!("failed to write {}", out.join("proof.bin").display()))?;
    fs::write(
        out.join("claims.json"),
        serde_json::to_vec_pretty(&build.claims).context("failed to serialize claims")?,
    )
    .with_context(|| format!("failed to write {}", out.join("claims.json").display()))?;

    let decision = DecisionArtifact {
        policy_version: PAYOUT_RELEASE_V1_POLICY_VERSION,
        proof_system: "halo2",
        decision: decision_label(build.claims.expected.decision),
        reason_bits: build.claims.expected.reason_bits,
        reason_codes: reason_codes.clone(),
        claims_hash: claims_hash.clone(),
        decision_commitment_hash: decision_commitment_hash.clone(),
        manifest_root: hash_to_hex(&build.claims.facts.manifest_root),
        recipient_snapshot_hash: hash_to_hex(&build.claims.facts.recipient_snapshot_hash),
        row_count: build.claims.facts.row_count,
        chunk_count: build.claims.facts.chunk_count,
        total_amount_units: build.claims.facts.total_amount_units,
        max_recipient_amount_units: build.claims.facts.max_recipient_amount_units,
        round_nonce: build.claims.facts.round_nonce,
        release_window_ends_at: build.claims.facts.release_window_ends_at,
    };
    fs::write(
        out.join("decision.json"),
        serde_json::to_vec_pretty(&decision).context("failed to serialize decision artifact")?,
    )
    .with_context(|| format!("failed to write {}", out.join("decision.json").display()))?;

    record_audit(
        &conn,
        &build.claims,
        &claims_hash,
        &decision_commitment_hash,
        false,
        evaluation_time,
    )?;

    print_json(&ProofRoundSummary {
        status: "ok",
        out_dir: out.display().to_string(),
        decision: decision_label(build.claims.expected.decision),
        proof_bytes: proof.len(),
        claims_hash,
        decision_commitment_hash,
        manifest_root: hash_to_hex(&build.claims.facts.manifest_root),
        recipient_snapshot_hash: hash_to_hex(&build.claims.facts.recipient_snapshot_hash),
        reason_codes,
    })
}

fn verify_round(proof_path: &Path, claims_path: &Path) -> Result<()> {
    let claims = read_claims(claims_path)?;
    let proof_bytes =
        fs::read(proof_path).with_context(|| format!("failed to read {}", proof_path.display()))?;

    verify_with_registry(&proof_bytes, &claims)?;

    print_json(&VerifyRoundSummary {
        verified: true,
        system: "halo2",
        policy_version: PAYOUT_RELEASE_V1_POLICY_VERSION,
    })
}

fn authorize_release(proof_path: &Path, claims_path: &Path, state_db: &Path) -> Result<()> {
    let claims = read_claims(claims_path)?;
    let proof_bytes =
        fs::read(proof_path).with_context(|| format!("failed to read {}", proof_path.display()))?;
    verify_with_registry(&proof_bytes, &claims)?;

    ensure!(
        claims.expected.decision == DecisionCode::Approved,
        "round proof is not approved for release"
    );
    ensure!(
        !claims.facts.round_replayed,
        "round was already marked as replayed at prove time"
    );

    let conn = open_store(state_db)?;
    ensure!(
        !release_exists(&conn, &claims)?,
        "round release has already been authorized"
    );

    let now = current_unix_seconds()?;
    let claims_hash = canonical_hash_hex(&claims)?;
    let decision_commitment_hash = canonical_hash_hex(&claims.expected)?;

    record_release(&conn, &claims, &decision_commitment_hash, now)?;
    record_audit(
        &conn,
        &claims,
        &claims_hash,
        &decision_commitment_hash,
        true,
        now,
    )?;

    print_json(&ReleaseEnvelope {
        status: "authorized",
        decision: decision_label(claims.expected.decision),
        policy_version: PAYOUT_RELEASE_V1_POLICY_VERSION,
        proof_system: "halo2",
        operator_id_hash: hash_to_hex(&claims.facts.operator_id_hash),
        program_id_hash: hash_to_hex(&claims.facts.program_id_hash),
        asset_id_hash: hash_to_hex(&claims.facts.asset_id_hash),
        round_id_hash: hash_to_hex(&claims.facts.round_id_hash),
        manifest_root: hash_to_hex(&claims.facts.manifest_root),
        recipient_snapshot_hash: hash_to_hex(&claims.facts.recipient_snapshot_hash),
        row_count: claims.facts.row_count,
        chunk_count: claims.facts.chunk_count,
        total_amount_units: claims.facts.total_amount_units,
        max_recipient_amount_units: claims.facts.max_recipient_amount_units,
        round_nonce: claims.facts.round_nonce,
        release_window_ends_at: claims.facts.release_window_ends_at,
        claims_hash,
        decision_commitment_hash,
    })
}

fn read_manifest(path: &Path) -> Result<Vec<PayoutRow>> {
    let file = File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut rows = Vec::new();

    for (line_no, line) in reader.lines().enumerate() {
        let line = line.with_context(|| format!("failed to read line {}", line_no + 1))?;
        if line.trim().is_empty() {
            continue;
        }
        let row = serde_json::from_str::<PayoutRow>(&line)
            .with_context(|| format!("invalid payout manifest line {}", line_no + 1))?;
        rows.push(row);
    }

    ensure!(!rows.is_empty(), "payout manifest must not be empty");
    Ok(rows)
}

fn read_policy(path: &Path) -> Result<PayoutPolicy> {
    let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_slice(&bytes)
        .with_context(|| format!("invalid policy JSON at {}", path.display()))
}

fn read_recipient_snapshot(path: &Path) -> Result<PayoutRecipientSnapshot> {
    let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_slice(&bytes)
        .with_context(|| format!("invalid recipient snapshot JSON at {}", path.display()))
}

fn read_claims(path: &Path) -> Result<PayoutReleaseClaims> {
    let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_slice(&bytes)
        .with_context(|| format!("invalid claims JSON at {}", path.display()))
}

fn verify_with_registry(proof_bytes: &[u8], claims: &PayoutReleaseClaims) -> Result<()> {
    let public_inputs = ProofClaims::PayoutReleaseV1(*claims);
    let proof = Proof::new(ProofSystem::Halo2, proof_bytes.to_vec());
    Verifier::verify(&proof, &public_inputs).map_err(|err| anyhow!(err.to_string()))
}

fn open_store(path: &Path) -> Result<Connection> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let conn = Connection::open(path)
        .with_context(|| format!("failed to open sqlite database {}", path.display()))?;
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS released_rounds (
            operator_id_hash TEXT NOT NULL,
            program_id_hash TEXT NOT NULL,
            asset_id_hash TEXT NOT NULL,
            round_id_hash TEXT NOT NULL,
            round_nonce INTEGER NOT NULL,
            decision_commitment_hash TEXT NOT NULL,
            manifest_root TEXT NOT NULL,
            recipient_snapshot_hash TEXT NOT NULL DEFAULT '',
            released_at INTEGER NOT NULL,
            PRIMARY KEY (operator_id_hash, program_id_hash, asset_id_hash, round_id_hash, round_nonce)
        );

        CREATE TABLE IF NOT EXISTS decision_audit (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            operator_id_hash TEXT NOT NULL,
            program_id_hash TEXT NOT NULL,
            asset_id_hash TEXT NOT NULL,
            round_id_hash TEXT NOT NULL,
            round_nonce INTEGER NOT NULL,
            policy_version TEXT NOT NULL,
            decision TEXT NOT NULL,
            reason_bits INTEGER NOT NULL,
            claims_hash TEXT NOT NULL,
            decision_commitment_hash TEXT NOT NULL,
            manifest_root TEXT NOT NULL DEFAULT '',
            recipient_snapshot_hash TEXT NOT NULL DEFAULT '',
            authorized INTEGER NOT NULL,
            created_at INTEGER NOT NULL
        );
        ",
    )?;
    ensure_column(
        &conn,
        "released_rounds",
        "recipient_snapshot_hash",
        "ALTER TABLE released_rounds ADD COLUMN recipient_snapshot_hash TEXT NOT NULL DEFAULT ''",
    )?;
    ensure_column(
        &conn,
        "decision_audit",
        "manifest_root",
        "ALTER TABLE decision_audit ADD COLUMN manifest_root TEXT NOT NULL DEFAULT ''",
    )?;
    ensure_column(
        &conn,
        "decision_audit",
        "recipient_snapshot_hash",
        "ALTER TABLE decision_audit ADD COLUMN recipient_snapshot_hash TEXT NOT NULL DEFAULT ''",
    )?;
    Ok(conn)
}

fn ensure_column(conn: &Connection, table: &str, column: &str, alter_sql: &str) -> Result<()> {
    let mut statement = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let columns = statement.query_map([], |row| row.get::<_, String>(1))?;
    let existing = columns.collect::<Result<Vec<_>, _>>()?;

    if !existing.iter().any(|name| name == column) {
        conn.execute(alter_sql, [])?;
    }

    Ok(())
}

fn release_exists(conn: &Connection, claims: &PayoutReleaseClaims) -> Result<bool> {
    let exists = conn
        .query_row(
            "
            SELECT 1
            FROM released_rounds
            WHERE operator_id_hash = ?1
              AND program_id_hash = ?2
              AND asset_id_hash = ?3
              AND round_id_hash = ?4
              AND round_nonce = ?5
            LIMIT 1
            ",
            params![
                hash_to_hex(&claims.facts.operator_id_hash),
                hash_to_hex(&claims.facts.program_id_hash),
                hash_to_hex(&claims.facts.asset_id_hash),
                hash_to_hex(&claims.facts.round_id_hash),
                claims.facts.round_nonce as i64,
            ],
            |_row| Ok(()),
        )
        .optional()?;
    Ok(exists.is_some())
}

fn record_release(
    conn: &Connection,
    claims: &PayoutReleaseClaims,
    decision_commitment_hash: &str,
    released_at: u64,
) -> Result<()> {
    conn.execute(
        "
        INSERT INTO released_rounds (
            operator_id_hash,
            program_id_hash,
            asset_id_hash,
            round_id_hash,
            round_nonce,
            decision_commitment_hash,
            manifest_root,
            recipient_snapshot_hash,
            released_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        ",
        params![
            hash_to_hex(&claims.facts.operator_id_hash),
            hash_to_hex(&claims.facts.program_id_hash),
            hash_to_hex(&claims.facts.asset_id_hash),
            hash_to_hex(&claims.facts.round_id_hash),
            claims.facts.round_nonce as i64,
            decision_commitment_hash,
            hash_to_hex(&claims.facts.manifest_root),
            hash_to_hex(&claims.facts.recipient_snapshot_hash),
            released_at as i64,
        ],
    )?;
    Ok(())
}

fn record_audit(
    conn: &Connection,
    claims: &PayoutReleaseClaims,
    claims_hash: &str,
    decision_commitment_hash: &str,
    authorized: bool,
    created_at: u64,
) -> Result<()> {
    conn.execute(
        "
        INSERT INTO decision_audit (
            operator_id_hash,
            program_id_hash,
            asset_id_hash,
            round_id_hash,
            round_nonce,
            policy_version,
            decision,
            reason_bits,
            claims_hash,
            decision_commitment_hash,
            manifest_root,
            recipient_snapshot_hash,
            authorized,
            created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
        ",
        params![
            hash_to_hex(&claims.facts.operator_id_hash),
            hash_to_hex(&claims.facts.program_id_hash),
            hash_to_hex(&claims.facts.asset_id_hash),
            hash_to_hex(&claims.facts.round_id_hash),
            claims.facts.round_nonce as i64,
            PAYOUT_RELEASE_V1_POLICY_VERSION,
            decision_label(claims.expected.decision),
            claims.expected.reason_bits as i64,
            claims_hash,
            decision_commitment_hash,
            hash_to_hex(&claims.facts.manifest_root),
            hash_to_hex(&claims.facts.recipient_snapshot_hash),
            if authorized { 1i64 } else { 0i64 },
            created_at as i64,
        ],
    )?;
    Ok(())
}

fn canonical_hash_hex<T: Serialize>(value: &T) -> Result<String> {
    let encoded = bincode::serialize(value).context("failed to serialize canonical payload")?;
    Ok(format!("0x{}", hex::encode(Keccak256::digest(encoded))))
}

fn hash_to_hex(hash: &[u8; 32]) -> String {
    format!("0x{}", hex::encode(hash))
}

fn decision_label(decision: DecisionCode) -> &'static str {
    match decision {
        DecisionCode::Approved => "approved",
        DecisionCode::Denied => "denied",
        DecisionCode::Eligible => "eligible",
        DecisionCode::Ineligible => "ineligible",
    }
}

fn current_unix_seconds() -> Result<u64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before unix epoch")?
        .as_secs())
}

fn print_json<T: Serialize>(value: &T) -> Result<()> {
    println!(
        "{}",
        serde_json::to_string_pretty(value).context("failed to serialize JSON output")?
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};
    use zkcg_common::payout::PAYOUT_RELEASE_REASON_CODE_MAP;

    fn temp_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be valid")
            .as_nanos();
        std::env::temp_dir().join(format!("zkcg-payout-worker-{name}-{nanos}"))
    }

    fn sample_manifest(path: &Path) {
        fs::write(
            path,
            concat!(
                r#"{"recipient_address":"0x1111111111111111111111111111111111111111","amount_units":10}"#,
                "\n",
                r#"{"recipient_address":"0x2222222222222222222222222222222222222222","amount_units":25}"#,
                "\n"
            ),
        )
        .expect("manifest should write");
    }

    fn sample_policy(path: &Path) {
        fs::write(
            path,
            serde_json::to_vec_pretty(&PayoutPolicy {
                operator_id: "miner-a".to_string(),
                program_id: "pool-main".to_string(),
                asset_id: "btc".to_string(),
                round_id: "round-42".to_string(),
                round_cap_units: 1_000,
                per_recipient_cap_units: 100,
                max_rows_per_round: 10,
                max_chunks_per_round: 4,
                round_nonce: 42,
                release_window_ends_at: 4_000_000_000,
            })
            .expect("policy should serialize"),
        )
        .expect("policy should write");
    }

    fn sample_snapshot(path: &Path) {
        fs::write(
            path,
            serde_json::to_vec_pretty(&PayoutRecipientSnapshot {
                expires_at: 4_000_000_000,
                recipients: vec![
                    zkcg_halo2_prover::PayoutRecipientStatus {
                        recipient_address: "0x1111111111111111111111111111111111111111"
                            .to_string(),
                        approved: true,
                        kyc_passed: true,
                        aml_cleared: true,
                        sanctions_clear: true,
                    },
                    zkcg_halo2_prover::PayoutRecipientStatus {
                        recipient_address: "0x2222222222222222222222222222222222222222"
                            .to_string(),
                        approved: true,
                        kyc_passed: true,
                        aml_cleared: true,
                        sanctions_clear: true,
                    },
                ],
            })
            .expect("snapshot should serialize"),
        )
        .expect("snapshot should write");
    }

    #[test]
    fn prove_and_verify_round_artifacts() {
        let base = temp_path("prove-verify");
        let manifest = base.join("manifest.ndjson");
        let policy = base.join("policy.json");
        let snapshot = base.join("snapshot.json");
        let db = base.join("state.sqlite");
        let out = base.join("out");
        fs::create_dir_all(&base).expect("base dir should exist");
        sample_manifest(&manifest);
        sample_policy(&policy);
        sample_snapshot(&snapshot);

        prove_round(&manifest, &policy, &snapshot, &db, &out).expect("prove should succeed");
        verify_round(&out.join("proof.bin"), &out.join("claims.json"))
            .expect("verify should succeed");
    }

    #[test]
    fn authorize_release_rejects_replay() {
        let base = temp_path("authorize");
        let manifest = base.join("manifest.ndjson");
        let policy = base.join("policy.json");
        let snapshot = base.join("snapshot.json");
        let db = base.join("state.sqlite");
        let out = base.join("out");
        fs::create_dir_all(&base).expect("base dir should exist");
        sample_manifest(&manifest);
        sample_policy(&policy);
        sample_snapshot(&snapshot);

        prove_round(&manifest, &policy, &snapshot, &db, &out).expect("prove should succeed");
        authorize_release(&out.join("proof.bin"), &out.join("claims.json"), &db)
            .expect("first authorization should succeed");
        let err = authorize_release(&out.join("proof.bin"), &out.join("claims.json"), &db)
            .expect_err("second authorization should be rejected");

        assert!(err.to_string().contains("already been authorized"));
    }

    #[test]
    fn prove_round_rejects_incomplete_recipient_snapshot() {
        let base = temp_path("snapshot-incomplete");
        let manifest = base.join("manifest.ndjson");
        let policy = base.join("policy.json");
        let snapshot = base.join("snapshot.json");
        let db = base.join("state.sqlite");
        let out = base.join("out");
        fs::create_dir_all(&base).expect("base dir should exist");
        sample_manifest(&manifest);
        sample_policy(&policy);
        fs::write(
            &snapshot,
            serde_json::json!({
                "expires_at": 4_000_000_000u64,
                "recipients": [{
                    "recipient_address": "0x1111111111111111111111111111111111111111",
                    "approved": true,
                    "kyc_passed": true,
                    "aml_cleared": true,
                    "sanctions_clear": true
                }]
            })
            .to_string(),
        )
        .expect("snapshot should write");

        let err = prove_round(&manifest, &policy, &snapshot, &db, &out)
            .expect_err("missing recipient should be rejected");

        assert!(err.to_string().contains("recipient snapshot"));
    }

    #[test]
    fn reason_code_map_contains_expected_codes() {
        let codes = PAYOUT_RELEASE_REASON_CODE_MAP
            .iter()
            .map(|(_, code)| *code)
            .collect::<Vec<_>>();
        assert!(codes.contains(&"round_replayed"));
        assert!(codes.contains(&"total_amount_limit_exceeded"));
        assert!(codes.contains(&"recipient_sanctions_hit"));
    }
}
