#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ProofSystem {
    Halo2,
    ZkVm,
    Groth16,
    Stark,
    Custom(&'static str),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Proof {
    pub system: ProofSystem,
    pub data: Vec<u8>,
}

impl Proof {
    pub fn new(system: ProofSystem, data: impl Into<Vec<u8>>) -> Self {
        Self {
            system,
            data: data.into(),
        }
    }
}

impl ProofSystem {
    pub const fn custom(name: &'static str) -> Self {
        Self::Custom(name)
    }
}
