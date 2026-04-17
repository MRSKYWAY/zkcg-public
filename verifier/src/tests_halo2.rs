#![cfg(feature = "zk-halo2")]

use rand::rngs::OsRng;

use halo2_proofs::{
    circuit::Value,
    plonk::{VerifyingKey, create_proof, keygen_pk},
    poly::commitment::Params,
    transcript::{Blake2bWrite, Challenge255},
};
use halo2curves::bn256::{Fr, G1Affine};

use crate::{Proof, ProofSystem, Verifier, engine::PublicInputs};
use zkcg_circuits::{halo2_artifacts::verifier_artifacts, score_circuit::ScoreCircuit};

fn generate_valid_proof_with_params(
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

#[test]
fn valid_halo2_proof_is_accepted() {
    let artifacts = verifier_artifacts();

    let proof = generate_valid_proof_with_params(39, 40, &artifacts.params, &artifacts.vk);

    let inputs = PublicInputs::phase1_score(40, [0u8; 32], 1);

    assert!(Verifier::verify(&proof, &inputs).is_ok());
}

#[test]
fn modified_proof_is_rejected() {
    let artifacts = verifier_artifacts();
    let mut proof = generate_valid_proof_with_params(39, 40, &artifacts.params, &artifacts.vk);
    proof.data[10] ^= 0xFF;

    let inputs = PublicInputs::phase1_score(40, [0u8; 32], 1);

    assert!(Verifier::verify(&proof, &inputs).is_err());
}

#[test]
fn wrong_public_input_is_rejected() {
    let artifacts = verifier_artifacts();
    let proof = generate_valid_proof_with_params(39, 40, &artifacts.params, &artifacts.vk);

    let wrong_inputs = PublicInputs::phase1_score(41, [0u8; 32], 1);

    assert!(Verifier::verify(&proof, &wrong_inputs).is_err());
}

#[test]
fn empty_proof_is_rejected() {
    let inputs = PublicInputs::phase1_score(40, [0u8; 32], 1);

    let proof = Proof::new(ProofSystem::Halo2, Vec::<u8>::new());

    assert!(Verifier::verify(&proof, &inputs).is_err());
}
