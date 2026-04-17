use std::time::Instant;

use rayon::prelude::*;

use zkcg_common::{state::ProtocolState, types::Commitment};
use zkcg_verifier::{
    Proof, ProofSystem, Verifier,
    backend_zkvm::ZkVmBackend,
    engine::{PublicInputs, VerifierEngine},
};

use zkcg_zkvm_host::prove as zkvm_prove;

use halo2_proofs::{
    circuit::Value,
    plonk::{create_proof, keygen_pk, keygen_vk},
    poly::commitment::Params,
    transcript::{Blake2bWrite, Challenge255},
};
use halo2curves::bn256::{Fr, G1Affine};
use rand::rngs::OsRng;
use zkcg_circuits::halo2_artifacts::verifier_artifacts;
use zkcg_circuits::score_circuit::ScoreCircuit;

fn main() {
    println!("Halo2 -> zkVM Attestation Pipeline\n");

    let threshold = 600u64;
    let scores = vec![550, 720, 480, 810, 590];

    println!("Phase 1: Halo2 policy verification");

    let phase1_start = Instant::now();

    let params: Params<G1Affine> = verifier_artifacts().params;

    let approved: Vec<u64> = scores
        .par_iter()
        .copied()
        .filter(|&score| {
            let proof = Proof::new(ProofSystem::Halo2, halo2_prove(score, threshold, &params));
            let inputs = PublicInputs::phase1_score(threshold, [0u8; 32], 1);

            Verifier::verify(&proof, &inputs).is_ok()
        })
        .collect();

    let phase1_time = phase1_start.elapsed();

    println!(
        "Approved {} / {} loans in {:.2?}",
        approved.len(),
        scores.len(),
        phase1_time
    );

    println!("\nPhase 2a: zkVM proof generation (parallel)");

    let phase2a_start = Instant::now();

    let zkvm_proofs: Vec<Vec<u8>> = approved
        .par_iter()
        .map(|&score| {
            zkvm_prove(score, threshold, [0u8; 32], 0).expect("zkVM proof generation failed")
        })
        .collect();

    let phase2a_time = phase2a_start.elapsed();

    println!(
        "Generated {} zkVM proofs in {:.2?} ({:.2} proofs/sec)",
        zkvm_proofs.len(),
        phase2a_time,
        zkvm_proofs.len() as f64 / phase2a_time.as_secs_f64()
    );

    println!("\nPhase 2b: zkVM protocol attestation (sequential)");

    let phase2b_start = Instant::now();

    let mut engine = VerifierEngine::new(ProtocolState::genesis(), Box::new(ZkVmBackend));

    for (i, proof) in zkvm_proofs.iter().enumerate() {
        let state = engine.state().clone();

        let inputs = PublicInputs::phase1_score(threshold, state.state_root, state.nonce + 1);

        engine
            .process_transition(proof, inputs, Commitment([i as u8; 32]))
            .expect("zkVM attestation failed");
    }

    let phase2b_time = phase2b_start.elapsed();

    println!(
        "Attested {} loans in {:.2?} ({:.2} tx/sec)",
        zkvm_proofs.len(),
        phase2b_time,
        zkvm_proofs.len() as f64 / phase2b_time.as_secs_f64()
    );

    let total_time = phase1_time + phase2a_time + phase2b_time;

    println!("\n========== SUMMARY ==========");
    println!("Total loans evaluated: {}", scores.len());
    println!("Loans approved:       {}", approved.len());
    println!("Halo2 phase:          {:.2?}", phase1_time);
    println!("zkVM prove phase:     {:.2?}", phase2a_time);
    println!("zkVM attest phase:    {:.2?}", phase2b_time);
    println!("-----------------------------");
    println!("Total pipeline time:  {:.2?}", total_time);
    println!(
        "End-to-end throughput: {:.2} loans/sec",
        approved.len() as f64 / total_time.as_secs_f64()
    );

    println!("\nHalo2 -> zkVM attestation pipeline completed successfully");
}

fn halo2_prove(score: u64, threshold: u64, params: &Params<G1Affine>) -> Vec<u8> {
    let circuit = ScoreCircuit::<Fr> {
        score: Value::known(Fr::from(score)),
        threshold: Value::known(Fr::from(threshold)),
    };

    let vk = keygen_vk(params, &circuit).unwrap();
    let pk = keygen_pk(params, vk, &circuit).unwrap();

    let public_inputs = vec![vec![Fr::from(threshold)]];
    let instance_slices: Vec<&[Fr]> = public_inputs.iter().map(|v| v.as_slice()).collect();

    let mut transcript = Blake2bWrite::<_, G1Affine, Challenge255<G1Affine>>::init(Vec::new());

    create_proof(
        params,
        &pk,
        &[circuit],
        &[instance_slices.as_slice()],
        OsRng,
        &mut transcript,
    )
    .unwrap();

    transcript.finalize()
}
