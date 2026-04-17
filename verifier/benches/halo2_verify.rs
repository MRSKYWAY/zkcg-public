#![cfg(feature = "zk-halo2")]

use criterion::{Criterion, criterion_group, criterion_main};
use std::time::Duration;

use halo2_proofs::{
    circuit::Value,
    plonk::{create_proof, keygen_pk},
    poly::commitment::Params,
    transcript::{Blake2bWrite, Challenge255},
};
use halo2curves::bn256::{Fr as Fp, G1Affine};
use rand::rngs::OsRng;

use zkcg_circuits::halo2_artifacts::verifier_artifacts;
use zkcg_circuits::score_circuit::ScoreCircuit;
use zkcg_verifier::engine::PublicInputs;
use zkcg_verifier::{Proof, ProofSystem, Verifier};

fn generate_halo2_proof(
    params: &Params<G1Affine>,
    pk: &halo2_proofs::plonk::ProvingKey<G1Affine>,
) -> Vec<u8> {
    let circuit = ScoreCircuit::<Fp> {
        score: Value::known(Fp::from(500)),
        threshold: Value::known(Fp::from(600)),
    };

    let public_inputs = vec![vec![Fp::from(600)]];
    let instances: Vec<&[Fp]> = public_inputs.iter().map(|v| v.as_slice()).collect();
    let all_instances = vec![instances.as_slice()];

    let mut proof_bytes = Vec::new();
    let mut transcript =
        Blake2bWrite::<_, G1Affine, Challenge255<G1Affine>>::init(&mut proof_bytes);

    create_proof(
        params,
        pk,
        &[circuit],
        &all_instances,
        OsRng,
        &mut transcript,
    )
    .unwrap();

    transcript.finalize().to_vec()
}

fn setup() -> (Params<G1Affine>, halo2_proofs::plonk::ProvingKey<G1Affine>) {
    let artifacts = verifier_artifacts();
    let params = artifacts.params;

    let circuit = ScoreCircuit::<Fp> {
        score: Value::known(Fp::from(500)),
        threshold: Value::known(Fp::from(600)),
    };

    let pk = keygen_pk(&params, artifacts.vk, &circuit).unwrap();

    (params, pk)
}

fn bench_halo2_verify(c: &mut Criterion) {
    let mut group = c.benchmark_group("halo2_verify");

    let (params, pk) = setup();
    let proof = Proof::new(ProofSystem::Halo2, generate_halo2_proof(&params, &pk));

    let public_inputs = PublicInputs::phase1_score(600, [0u8; 32], 1);

    group.bench_function("verify", |b| {
        b.iter(|| {
            Verifier::verify(&proof, &public_inputs).expect("halo2 verification failed");
        });
    });

    group.finish();
}

fn bench_halo2_prove(c: &mut Criterion) {
    let mut group = c.benchmark_group("halo2_prove");

    group.sample_size(10);
    group.warm_up_time(Duration::from_secs(2));
    group.measurement_time(Duration::from_secs(30));

    let (params, pk) = setup();

    group.bench_function("prove", |b| {
        b.iter(|| {
            let _proof = generate_halo2_proof(&params, &pk);
        });
    });

    group.finish();
}

fn bench_halo2_prove_and_verify(c: &mut Criterion) {
    let mut group = c.benchmark_group("halo2_prove_and_verify");

    group.sample_size(10);
    group.warm_up_time(Duration::from_secs(2));
    group.measurement_time(Duration::from_secs(30));

    let (params, pk) = setup();

    let public_inputs = PublicInputs::phase1_score(600, [0u8; 32], 1);

    group.bench_function("prove_and_verify", |b| {
        b.iter(|| {
            let proof = Proof::new(ProofSystem::Halo2, generate_halo2_proof(&params, &pk));
            Verifier::verify(&proof, &public_inputs).expect("halo2 verification failed");
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_halo2_verify,
    bench_halo2_prove,
    bench_halo2_prove_and_verify
);

criterion_main!(benches);
