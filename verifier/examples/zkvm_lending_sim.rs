use zkcg_common::{errors::ProtocolError, state::ProtocolState, types::Commitment};

use zkcg_verifier::{
    Proof, ProofSystem, Verifier,
    backend_zkvm::ZkVmBackend,
    engine::{PublicInputs, VerifierEngine},
};

use zkcg_zkvm_host::{ZkVmProverError, prove};

fn main() {
    println!("zkVM DeFi Lending Simulation");

    let state = ProtocolState::genesis();
    let mut engine = VerifierEngine::new(state.clone(), Box::new(ZkVmBackend));

    let threshold = 600u64;
    let score = 550u64;

    let inputs = PublicInputs::phase1_score(threshold, state.state_root, state.nonce + 1);

    let commitment = Commitment([42u8; 32]);

    let proof_bytes = match prove(score, threshold, state.state_root, state.nonce + 1) {
        Ok(p) => p,
        Err(ZkVmProverError::PolicyViolation) => {
            println!("Loan rejected: policy violation");
            return;
        }
        Err(e) => {
            println!("Prover error: {:?}", e);
            return;
        }
    };

    let proof = Proof::new(ProofSystem::ZkVm, proof_bytes.clone());

    match Verifier::verify(&proof, &inputs) {
        Ok(()) => {
            println!("Universal verifier accepted the zkVM proof");
        }
        Err(ProtocolError::InvalidProof) => {
            println!("Invalid zkVM proof");
            return;
        }
        Err(e) => {
            println!("Verification failed: {:?}", e);
            return;
        }
    }

    match engine.process_transition(&proof_bytes, inputs, commitment) {
        Ok(new_state) => {
            println!("zkVM proof accepted");
            println!("New protocol state: {:?}", new_state);
        }
        Err(ProtocolError::InvalidProof) => {
            println!("Invalid zkVM proof");
        }
        Err(e) => {
            println!("Verification failed: {:?}", e);
        }
    }
}
