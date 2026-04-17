use std::time::Instant;

use rand::thread_rng;
use rand_distr::{Distribution, Normal};

use halo2_proofs::{
    circuit::Value,
    plonk::{VerifyingKey, create_proof, keygen_pk},
    poly::commitment::Params,
    transcript::{Blake2bWrite, Challenge255},
};
use halo2curves::bn256::{Fr, G1Affine};
use rand::rngs::OsRng;

use zkcg_circuits::halo2_artifacts::verifier_artifacts;
use zkcg_circuits::score_circuit::ScoreCircuit;
use zkcg_verifier::{Proof, ProofSystem, Verifier, engine::PublicInputs};

fn main() {
    let num_loans = 1_000;
    let threshold = 600u64;

    let mut rng = thread_rng();
    let normal = Normal::new(715.0, 100.0).unwrap();

    let scores: Vec<u64> = (0..num_loans)
        .map(|_| {
            let s: f64 = normal.sample(&mut rng);
            s.clamp(300.0, 850.0) as u64
        })
        .collect();

    let artifacts = verifier_artifacts();
    let params: Params<G1Affine> = artifacts.params;
    let vk = artifacts.vk;

    println!("DeFi Lending Simulation (Halo2 via universal verifier)");
    println!("Loans evaluated: {}", num_loans);

    let prove_start = Instant::now();

    let proofs: Vec<Proof> = scores
        .iter()
        .map(|&score| generate_proof(score, threshold, &params, &vk))
        .collect();

    let prove_time = prove_start.elapsed();

    let verify_start = Instant::now();

    let approvals = proofs
        .iter()
        .filter(|proof| {
            let inputs = PublicInputs::phase1_score(threshold, [0u8; 32], 1);

            Verifier::verify(proof, &inputs).is_ok()
        })
        .count();

    let verify_time = verify_start.elapsed();
    let total_time = prove_time + verify_time;

    println!(
        "Approvals: {} ({:.1}%)",
        approvals,
        approvals as f64 / num_loans as f64 * 100.0
    );

    println!("\nHalo2 Performance (real execution):");
    println!("- Prove total:  {:?}", prove_time);
    println!("- Verify total: {:?}", verify_time);
    println!(
        "- Throughput:   {:.2} TPS",
        num_loans as f64 / total_time.as_secs_f64()
    );
}

fn generate_proof(
    score: u64,
    threshold: u64,
    params: &Params<G1Affine>,
    vk: &VerifyingKey<G1Affine>,
) -> Proof {
    let circuit = ScoreCircuit::<Fr> {
        score: Value::known(Fr::from(score)),
        threshold: Value::known(Fr::from(threshold)),
    };

    let pk = keygen_pk(params, vk.clone(), &circuit).unwrap();

    let public_inputs = vec![vec![Fr::from(threshold)]];
    let instance_slices: Vec<&[Fr]> = public_inputs.iter().map(|v| v.as_slice()).collect();
    let all_instances: Vec<&[&[Fr]]> = vec![instance_slices.as_slice()];

    let mut transcript = Blake2bWrite::<_, G1Affine, Challenge255<G1Affine>>::init(Vec::new());

    create_proof(
        params,
        &pk,
        &[circuit],
        &all_instances,
        OsRng,
        &mut transcript,
    )
    .unwrap();

    Proof::new(ProofSystem::Halo2, transcript.finalize())
}
