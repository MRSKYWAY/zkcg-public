use crate::{
    core::{
        proof::{Proof, ProofSystem},
        verifier::ProofVerifier,
    },
    engine::PublicInputs,
};
use std::{collections::HashMap, sync::Arc};
use zkcg_common::errors::ProtocolError;

#[derive(Default)]
pub struct VerifierRegistry {
    verifiers: HashMap<ProofSystem, Arc<dyn ProofVerifier>>,
}

impl VerifierRegistry {
    pub fn new() -> Self {
        Self {
            verifiers: HashMap::new(),
        }
    }

    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        register_halo2(&mut registry);
        register_halo2_kzg(&mut registry);
        register_zkvm(&mut registry);
        registry
    }

    pub fn register<V>(
        &mut self,
        system: ProofSystem,
        verifier: V,
    ) -> Option<Arc<dyn ProofVerifier>>
    where
        V: ProofVerifier + 'static,
    {
        self.register_arc(system, Arc::new(verifier))
    }

    pub fn register_arc(
        &mut self,
        system: ProofSystem,
        verifier: Arc<dyn ProofVerifier>,
    ) -> Option<Arc<dyn ProofVerifier>> {
        self.verifiers.insert(system, verifier)
    }

    pub fn verify(&self, proof: &Proof, public_inputs: &PublicInputs) -> Result<(), ProtocolError> {
        self.verifiers
            .get(&proof.system)
            .ok_or(ProtocolError::InvalidProof)?
            .verify(&proof.data, public_inputs)
    }
}

#[cfg(feature = "zk-halo2")]
fn register_halo2(registry: &mut VerifierRegistry) {
    use crate::adapters::halo2::Halo2Verifier;

    registry.register(ProofSystem::Halo2, Halo2Verifier::bundled());
}

#[cfg(not(feature = "zk-halo2"))]
fn register_halo2(_registry: &mut VerifierRegistry) {}

#[cfg(feature = "zk-halo2-kzg")]
fn register_halo2_kzg(registry: &mut VerifierRegistry) {
    use crate::adapters::halo2_kzg::{HALO2_KZG_SYSTEM_NAME, Halo2KzgVerifier};

    registry.register(
        ProofSystem::custom(HALO2_KZG_SYSTEM_NAME),
        Halo2KzgVerifier::bundled(),
    );
}

#[cfg(not(feature = "zk-halo2-kzg"))]
fn register_halo2_kzg(_registry: &mut VerifierRegistry) {}

#[cfg(feature = "zk-vm")]
fn register_zkvm(registry: &mut VerifierRegistry) {
    use crate::adapters::zkvm::ZkVmVerifier;

    registry.register(ProofSystem::ZkVm, ZkVmVerifier);
}

#[cfg(not(feature = "zk-vm"))]
fn register_zkvm(_registry: &mut VerifierRegistry) {}

pub(crate) fn default_registry() -> &'static VerifierRegistry {
    use std::sync::OnceLock;

    static DEFAULT_REGISTRY: OnceLock<VerifierRegistry> = OnceLock::new();

    DEFAULT_REGISTRY.get_or_init(VerifierRegistry::with_defaults)
}

pub(crate) fn verify(proof: &Proof, public_inputs: &PublicInputs) -> Result<(), ProtocolError> {
    default_registry().verify(proof, public_inputs)
}
