use zkcg_halo2_test::{
    Acceptance, AdversarialCase, AdversarialHarness, BindingCase, CircuitMutationCase,
    CircuitMutationRunner, DifferentialCase, DifferentialHarness, MalformedProofMutation,
    MutationCase, MutationHarness, ReplayHarness, SecurityOracle, VerificationBackend,
    mutate_proof,
};

struct Oracle;

impl SecurityOracle<u64> for Oracle {
    type Expected = bool;

    fn expected(&self, input: &u64) -> Self::Expected {
        *input < 10
    }
}

struct Backend {
    accept_at: u64,
}

impl VerificationBackend<u64> for Backend {
    type Error = String;

    fn verify(&self, input: &u64) -> Result<Acceptance, Self::Error> {
        Ok(if *input == self.accept_at {
            Acceptance::Accepted
        } else {
            Acceptance::Rejected
        })
    }
}

#[test]
fn malformed_mutations_are_deterministic() {
    let proof = vec![0xAA, 0xBB, 0xCC];
    assert_eq!(
        mutate_proof(&proof, MalformedProofMutation::Empty),
        Vec::<u8>::new()
    );
    assert_eq!(
        mutate_proof(&proof, MalformedProofMutation::Truncate(2)),
        vec![0xAA, 0xBB]
    );
    assert_eq!(
        mutate_proof(&proof, MalformedProofMutation::Append(vec![0xDD])),
        vec![0xAA, 0xBB, 0xCC, 0xDD]
    );
}

#[test]
fn adversarial_case_can_assert_rejection() {
    let harness = AdversarialHarness::new(|input: &u64| {
        Ok::<_, String>(if *input > 100 {
            Acceptance::Rejected
        } else {
            Acceptance::Accepted
        })
    });

    let result = harness
        .run(AdversarialCase {
            name: "too-large".into(),
            input: 101,
            expected: Acceptance::Rejected,
        })
        .unwrap();
    assert_eq!(result.observed, Acceptance::Rejected);
}

#[test]
fn semantic_mutation_requires_rejection() {
    let harness = MutationHarness::new(|_: &u64, _: &bool| Ok::<_, String>(Acceptance::Rejected));

    let result = harness
        .run(MutationCase {
            name: "flip decision".into(),
            input: 42_u64,
            expected: true,
        })
        .unwrap();
    assert!(result.expected_rejection);
}

#[test]
fn replay_requires_original_acceptance_and_mutated_rejection() {
    let harness = ReplayHarness::new(|input: &u64| {
        Ok::<_, String>(if *input == 1 {
            Acceptance::Accepted
        } else {
            Acceptance::Rejected
        })
    });

    let result = harness
        .run(BindingCase {
            name: "wallet substitution".into(),
            original: 1,
            mutated: 2,
        })
        .unwrap();
    assert!(result.original_accepted);
    assert!(result.mutated_rejected);
}

#[test]
fn differential_testing_requires_oracle_agreement() {
    let harness = DifferentialHarness::new(
        Backend { accept_at: 5 },
        Backend { accept_at: 5 },
        |input: &u64| {
            Ok::<_, String>(if *input == 5 {
                Acceptance::Accepted
            } else {
                Acceptance::Rejected
            })
        },
    );

    let result = harness
        .run(DifferentialCase {
            name: "same result".into(),
            input: 5,
        })
        .unwrap();
    assert_eq!(result.oracle, Acceptance::Accepted);
    assert_eq!(result.backend_a, Acceptance::Accepted);
    assert_eq!(result.backend_b, Acceptance::Accepted);
}

#[test]
fn circuit_mutation_requires_intact_rejection_and_mutated_acceptance() {
    let runner = CircuitMutationRunner::new(|input: &u64, mutated: bool| {
        Ok::<_, String>(if mutated && *input == 9 {
            Acceptance::Accepted
        } else {
            Acceptance::Rejected
        })
    });

    let result = runner
        .run(CircuitMutationCase {
            name: "missing binding".into(),
            malicious_input: 9,
        })
        .unwrap();
    assert!(result.intact_rejected);
    assert!(result.mutated_accepted);
}

#[test]
fn oracle_can_be_used_independently() {
    assert!(Oracle.expected(&3));
    assert!(!Oracle.expected(&13));
}
