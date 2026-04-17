#![cfg(all(feature = "zk-halo2", feature = "zk-vm"))]

use crate::{Proof, ProofSystem, Verifier, engine::PublicInputs};
use halo2_proofs::{
    circuit::Value,
    plonk::{VerifyingKey, create_proof, keygen_pk},
    poly::commitment::Params,
    transcript::{Blake2bWrite, Challenge255},
};
use halo2curves::bn256::{Fr, G1Affine};
use rand::rngs::OsRng;
use zkcg_circuits::{halo2_artifacts::verifier_artifacts, score_circuit::ScoreCircuit};
use zkcg_common::errors::ProtocolError;
use zkcg_zkvm_host::prove;

#[derive(Copy, Clone)]
enum Expectation {
    Accept,
    Reject,
}

struct TestScenario {
    score: u64,
    threshold: u64,
    expected: Expectation,
    desc: &'static str,
}

fn scenarios() -> Vec<TestScenario> {
    vec![
        TestScenario {
            score: 39,
            threshold: 40,
            expected: Expectation::Accept,
            desc: "Valid transition",
        },
        TestScenario {
            score: 41,
            threshold: 40,
            expected: Expectation::Reject,
            desc: "Policy violation",
        },
        TestScenario {
            score: 0,
            threshold: 0,
            expected: Expectation::Accept,
            desc: "Zero boundary",
        },
    ]
}

fn matches_expectation(result: Result<(), ProtocolError>, expected: Expectation) -> bool {
    match expected {
        Expectation::Accept => result.is_ok(),
        Expectation::Reject => result.is_err(),
    }
}

fn halo2_prove(
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

    let instances = vec![vec![Fr::from(threshold)]];
    let instance_refs: Vec<&[Fr]> = instances.iter().map(|v| v.as_slice()).collect();
    let all_instances = vec![instance_refs.as_slice()];

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

fn zkvm_prove(score: u64, threshold: u64) -> Result<Proof, ProtocolError> {
    prove(score, threshold, [0u8; 32], 1)
        .map(|bytes| Proof::new(ProofSystem::ZkVm, bytes))
        .map_err(|_| ProtocolError::InvalidProof)
}

fn rust_only(score: u64, threshold: u64) -> Result<(), ProtocolError> {
    if score > threshold {
        Err(ProtocolError::PolicyViolation)
    } else {
        Ok(())
    }
}

#[test]
fn cross_backend_equivalence() {
    let artifacts = verifier_artifacts();

    for s in scenarios() {
        let inputs = PublicInputs::phase1_score(s.threshold, [0u8; 32], 1);

        let halo2_proof = halo2_prove(s.score, s.threshold, &artifacts.params, &artifacts.vk);
        let halo2_result = Verifier::verify(&halo2_proof, &inputs);

        assert!(
            matches_expectation(halo2_result, s.expected),
            "Halo2 failed: {}",
            s.desc
        );

        let zkvm_result =
            zkvm_prove(s.score, s.threshold).and_then(|proof| Verifier::verify(&proof, &inputs));

        assert!(
            matches_expectation(zkvm_result, s.expected),
            "zkVM failed: {}",
            s.desc
        );

        let rust_result = rust_only(s.score, s.threshold);
        assert!(
            matches_expectation(rust_result, s.expected),
            "Rust failed: {}",
            s.desc
        );
    }
}
