#![cfg(feature = "zk-vm")]

pub use crate::adapters::zkvm::ZkVmVerifier as ZkVmBackend;
pub use zkcg_common::types::ZkVmJournal;
