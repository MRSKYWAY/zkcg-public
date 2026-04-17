#![cfg(feature = "zk-halo2-kzg")]

use crate::adapters::halo2_kzg::{HALO2_KZG_SYSTEM_NAME, generate_test_proof};
use crate::{Proof, ProofSystem, Verifier, engine::PublicInputs};

#[test]
fn halo2_kzg_proof_is_accepted_via_custom_system() {
    let proof = Proof::new(
        ProofSystem::custom(HALO2_KZG_SYSTEM_NAME),
        generate_test_proof(39, 40),
    );

    let inputs = PublicInputs::phase1_score(40, [0u8; 32], 1);

    assert!(Verifier::verify(&proof, &inputs).is_ok());
}

#[cfg(feature = "zk-halo2")]
#[test]
fn halo2_standard_verifier_accepts_kzg_proofs_when_both_backends_are_enabled() {
    let proof = Proof::new(ProofSystem::Halo2, generate_test_proof(39, 40));

    let inputs = PublicInputs::phase1_score(40, [0u8; 32], 1);

    assert!(Verifier::verify(&proof, &inputs).is_ok());
}
