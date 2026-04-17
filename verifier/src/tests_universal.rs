use crate::{
    Proof, ProofSystem, ProofVerifier, VerificationRequest, Verifier, VerifierRegistry,
    engine::PublicInputs,
    proof::{self, ProofInput},
};
use zkcg_common::errors::ProtocolError;

#[derive(Clone, Copy, Debug, Default)]
struct AcceptVerifier;

impl ProofVerifier for AcceptVerifier {
    fn verify(&self, _proof: &[u8], _public_inputs: &PublicInputs) -> Result<(), ProtocolError> {
        Ok(())
    }
}

fn inputs(threshold: u64) -> PublicInputs {
    PublicInputs::phase1_score(threshold, [0u8; 32], 1)
}

#[cfg(not(feature = "zk-halo2"))]
#[test]
fn unavailable_halo2_system_is_rejected() {
    let proof = Proof::new(ProofSystem::Halo2, b"halo2-proof".to_vec());
    let err = Verifier::verify(&proof, &inputs(40)).unwrap_err();

    assert!(matches!(err, ProtocolError::InvalidProof));
}

#[cfg(not(feature = "zk-halo2"))]
#[test]
fn legacy_halo2_wrapper_is_rejected_when_feature_is_disabled() {
    let err = proof::verify(ProofInput {
        proof_bytes: b"halo2-proof",
        public_inputs: &inputs(40),
    })
    .unwrap_err();

    assert!(matches!(err, ProtocolError::InvalidProof));
}

#[cfg(not(feature = "zk-vm"))]
#[test]
fn unavailable_zkvm_system_is_rejected() {
    let proof = Proof::new(ProofSystem::ZkVm, b"zkvm-proof".to_vec());
    let err = Verifier::verify(&proof, &inputs(40)).unwrap_err();

    assert!(matches!(err, ProtocolError::InvalidProof));
}

#[test]
fn empty_batch_is_accepted() {
    let batch: Vec<VerificationRequest> = Vec::new();

    assert!(Verifier::verify_batch(&batch).is_ok());
}

#[test]
fn empty_parallel_batch_is_accepted() {
    let batch: Vec<VerificationRequest> = Vec::new();

    assert!(Verifier::verify_batch_parallel(&batch).is_ok());
    assert!(Verifier::verify_batch_parallel_results(&batch).is_empty());
}

#[test]
fn custom_registry_routes_registered_systems() {
    let mut registry = VerifierRegistry::new();
    let proof = Proof::new(ProofSystem::custom("groth16-demo"), b"proof".to_vec());

    registry.register(ProofSystem::custom("groth16-demo"), AcceptVerifier);

    assert!(Verifier::verify_with_registry(&registry, &proof, &inputs(40)).is_ok());
}

#[test]
fn custom_registry_rejects_unregistered_systems() {
    let registry = VerifierRegistry::new();
    let proof = Proof::new(ProofSystem::Groth16, b"proof".to_vec());
    let err = Verifier::verify_with_registry(&registry, &proof, &inputs(40)).unwrap_err();

    assert!(matches!(err, ProtocolError::InvalidProof));
}

#[cfg(feature = "zk-halo2")]
mod halo2 {
    use super::*;
    use crate::{backend::ProofBackend, backend_halo2::Halo2Backend};
    use halo2_proofs::{
        arithmetic::Field,
        circuit::Value,
        plonk::{create_proof, keygen_pk, keygen_vk},
        poly::commitment::Params,
        transcript::{Blake2bWrite, Challenge255},
    };
    use halo2curves::bn256::{Fr, G1Affine};
    use rand::rngs::OsRng;
    use zkcg_circuits::halo2_artifacts::verifier_artifacts;
    use zkcg_circuits::score_circuit::ScoreCircuit;

    fn generate_valid_proof_with_params(
        score: u64,
        threshold: u64,
        params: &Params<G1Affine>,
    ) -> Vec<u8> {
        let circuit = ScoreCircuit::<Fr> {
            score: Value::known(Fr::from(score)),
            threshold: Value::known(Fr::from(threshold)),
        };

        let vk = keygen_vk(params, &circuit).unwrap();
        let pk = keygen_pk(params, vk, &circuit).unwrap();

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

        transcript.finalize()
    }

    fn backend(params: Params<G1Affine>) -> Halo2Backend {
        let dummy = ScoreCircuit::<Fr> {
            score: Value::known(Fr::ZERO),
            threshold: Value::known(Fr::ZERO),
        };

        let vk = keygen_vk(&params, &dummy).unwrap();
        Halo2Backend::from_cpu_parts(vk, params)
    }

    fn generate_bundled_proof(score: u64, threshold: u64) -> Vec<u8> {
        let artifacts = verifier_artifacts();
        let circuit = ScoreCircuit::<Fr> {
            score: Value::known(Fr::from(score)),
            threshold: Value::known(Fr::from(threshold)),
        };

        let pk = keygen_pk(&artifacts.params, artifacts.vk.clone(), &circuit).unwrap();

        let public_inputs = vec![vec![Fr::from(threshold)]];
        let instance_slices: Vec<&[Fr]> = public_inputs.iter().map(|v| v.as_slice()).collect();
        let all_instances: Vec<&[&[Fr]]> = vec![instance_slices.as_slice()];

        let mut transcript = Blake2bWrite::<_, G1Affine, Challenge255<G1Affine>>::init(Vec::new());

        create_proof(
            &artifacts.params,
            &pk,
            &[circuit],
            &all_instances,
            OsRng,
            &mut transcript,
        )
        .unwrap();

        transcript.finalize()
    }

    #[test]
    fn universal_verifier_routes_halo2_proofs() {
        let proof_bytes = generate_bundled_proof(39, 40);
        let proof = Proof::new(ProofSystem::Halo2, proof_bytes);

        assert!(Verifier::verify(&proof, &inputs(40)).is_ok());
    }

    #[test]
    fn halo2_backend_alias_still_verifies() {
        let params: Params<G1Affine> = Params::new(9);
        let proof_bytes = generate_valid_proof_with_params(39, 40, &params);
        let backend = backend(params);

        assert!(backend.verify(&proof_bytes, &inputs(40)).is_ok());
    }

    #[test]
    fn legacy_halo2_wrapper_uses_universal_dispatch() {
        let proof_bytes = generate_bundled_proof(39, 40);

        assert!(
            proof::verify(ProofInput {
                proof_bytes: &proof_bytes,
                public_inputs: &inputs(40),
            })
            .is_ok()
        );
    }

    #[test]
    fn verify_batch_routes_multiple_halo2_requests() {
        let proof = Proof::new(ProofSystem::Halo2, generate_bundled_proof(39, 40));
        let request = VerificationRequest::new(proof, inputs(40));
        let batch = vec![request.clone(), request];

        assert!(Verifier::verify_batch(&batch).is_ok());
        assert!(
            Verifier::verify_batch_results(&batch)
                .into_iter()
                .all(|result| result.is_ok())
        );
    }

    #[test]
    fn verify_batch_parallel_routes_multiple_halo2_requests() {
        let proof = Proof::new(ProofSystem::Halo2, generate_bundled_proof(39, 40));
        let request = VerificationRequest::new(proof, inputs(40));
        let batch = vec![request.clone(), request];

        assert!(Verifier::verify_batch_parallel(&batch).is_ok());
        assert!(
            Verifier::verify_batch_parallel_results(&batch)
                .into_iter()
                .all(|result| result.is_ok())
        );
    }
}

#[cfg(feature = "zk-vm")]
mod zkvm {
    use super::*;
    use crate::{backend::ProofBackend, backend_zkvm::ZkVmBackend};
    use zkcg_zkvm_host::prove;

    #[test]
    fn universal_verifier_routes_zkvm_proofs() {
        let public_inputs = inputs(10);
        let phase1 = public_inputs.phase1().unwrap();
        let proof_bytes =
            prove(5, 10, phase1.old_state_root, phase1.nonce).expect("valid zkvm proof");
        let proof = Proof::new(ProofSystem::ZkVm, proof_bytes);

        assert!(Verifier::verify(&proof, &public_inputs).is_ok());
    }

    #[test]
    fn zkvm_backend_alias_still_verifies() {
        let public_inputs = inputs(10);
        let phase1 = public_inputs.phase1().unwrap();
        let proof_bytes =
            prove(5, 10, phase1.old_state_root, phase1.nonce).expect("valid zkvm proof");
        let backend = ZkVmBackend;

        assert!(backend.verify(&proof_bytes, &public_inputs).is_ok());
    }

    #[test]
    fn verify_batch_routes_multiple_zkvm_requests() {
        let public_inputs = inputs(10);
        let phase1 = public_inputs.phase1().unwrap();
        let proof_bytes =
            prove(5, 10, phase1.old_state_root, phase1.nonce).expect("valid zkvm proof");
        let proof = Proof::new(ProofSystem::ZkVm, proof_bytes);
        let request = VerificationRequest::new(proof, public_inputs);
        let batch = vec![request.clone(), request];

        assert!(Verifier::verify_batch(&batch).is_ok());
        assert!(
            Verifier::verify_batch_results(&batch)
                .into_iter()
                .all(|result| result.is_ok())
        );
    }

    #[test]
    fn verify_batch_parallel_routes_multiple_zkvm_requests() {
        let public_inputs = inputs(10);
        let phase1 = public_inputs.phase1().unwrap();
        let proof_bytes =
            prove(5, 10, phase1.old_state_root, phase1.nonce).expect("valid zkvm proof");
        let proof = Proof::new(ProofSystem::ZkVm, proof_bytes);
        let request = VerificationRequest::new(proof, public_inputs);
        let batch = vec![request.clone(), request];

        assert!(Verifier::verify_batch_parallel(&batch).is_ok());
        assert!(
            Verifier::verify_batch_parallel_results(&batch)
                .into_iter()
                .all(|result| result.is_ok())
        );
    }

    #[test]
    fn verify_batch_parallel_results_preserve_failures() {
        let public_inputs = inputs(10);
        let phase1 = public_inputs.phase1().unwrap();
        let proof_bytes =
            prove(5, 10, phase1.old_state_root, phase1.nonce).expect("valid zkvm proof");
        let valid = VerificationRequest::new(
            Proof::new(ProofSystem::ZkVm, proof_bytes.clone()),
            public_inputs,
        );
        let invalid =
            VerificationRequest::new(Proof::new(ProofSystem::ZkVm, proof_bytes), inputs(11));
        let results = Verifier::verify_batch_parallel_results(&[valid, invalid]);

        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(matches!(
            results[1],
            Err(zkcg_common::errors::ProtocolError::InvalidProof)
        ));
    }
}
