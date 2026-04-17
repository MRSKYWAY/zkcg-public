#![cfg(feature = "zk-vm")]

use criterion::{Criterion, criterion_group, criterion_main};
use std::time::Instant;

use zkcg_verifier::engine::PublicInputs;
use zkcg_verifier::{Proof, ProofSystem, Verifier};

use zkcg_zkvm_host::prove;

fn measure_prove_once() {
    println!("Measuring zkVM prove() once (not a benchmark)...");
    let start = Instant::now();

    let proof = prove(500, 600, [0u8; 32], 1).expect("zkvm proof generation failed");

    let elapsed = start.elapsed();
    println!(
        "prove() took {:?} (proof size = {} bytes)",
        elapsed,
        proof.len()
    );
}

fn bench_zkvm_verify(c: &mut Criterion) {
    let mut group = c.benchmark_group("zkvm");

    let public_inputs = PublicInputs::phase1_score(600, [0u8; 32], 1);

    let proof = Proof::new(
        ProofSystem::ZkVm,
        prove(500, 600, [0u8; 32], 1).expect("zkvm proof generation failed"),
    );

    group.bench_function("verify", |b| {
        b.iter(|| {
            Verifier::verify(&proof, &public_inputs).expect("zkvm verification failed");
        });
    });

    group.finish();
}

#[cfg(feature = "zk-vm")]
pub fn debug_double_prove() {
    use std::time::Instant;

    println!("---- zkVM double prove test ----");

    let t1 = Instant::now();
    let _ = prove(500, 600, [0u8; 32], 1).unwrap();
    println!("first prove took {:?}", t1.elapsed());

    let t2 = Instant::now();
    let _ = prove(500, 600, [0u8; 32], 2).unwrap();
    println!("second prove took {:?}", t2.elapsed());
}

fn benches(c: &mut Criterion) {
    measure_prove_once();
    bench_zkvm_verify(c);
}

criterion_group!(zkvm_benches, benches);
criterion_main!(zkvm_benches);
