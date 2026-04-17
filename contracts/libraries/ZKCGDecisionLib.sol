// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.20;

library ZKCGDecisionLib {
    uint256 internal constant DECISION_INPUTS_LENGTH = 160;

    bytes32 internal constant LENDING_V1_HASH = keccak256("lending.v1");
    bytes32 internal constant RISK_GATED_VAULT_V1_HASH =
        keccak256("risk-gated-vault.v1");
    bytes32 internal constant ORACLE_SANITY_V1_HASH =
        keccak256("oracle.sanity.v1");
    bytes32 internal constant RWA_CREDIT_ONBOARDING_V1_HASH =
        keccak256("rwa.credit.onboarding.v1");
    bytes32 internal constant RWA_CREDIT_TRANSFER_V1_HASH =
        keccak256("rwa.credit.transfer.v1");

    bytes32 internal constant ZKVM_HASH = keccak256("zkvm");
    bytes32 internal constant HALO2_HASH = keccak256("halo2");

    struct DecisionInputs {
        bytes32 decisionId;
        bytes32 policyVersionHash;
        bytes32 proofSystemHash;
        bytes32 payloadHash;
        uint256 expiresAt;
    }

    function decodeDecisionInputs(
        bytes calldata publicInputs
    ) internal pure returns (DecisionInputs memory inputs) {
        require(
            publicInputs.length == DECISION_INPUTS_LENGTH,
            "zkcg: malformed public inputs"
        );

        (
            bytes32 decisionId,
            bytes32 policyVersionHash,
            bytes32 proofSystemHash,
            bytes32 payloadHash,
            uint256 expiresAt
        ) = abi.decode(
                publicInputs,
                (bytes32, bytes32, bytes32, bytes32, uint256)
            );

        inputs = DecisionInputs({
            decisionId: decisionId,
            policyVersionHash: policyVersionHash,
            proofSystemHash: proofSystemHash,
            payloadHash: payloadHash,
            expiresAt: expiresAt
        });
    }

    function hashLoanTerms(
        address creditLine,
        address borrower,
        uint256 creditLimitCents,
        uint256 maxAprBps
    ) internal pure returns (bytes32) {
        return
            keccak256(
                abi.encode(
                    creditLine,
                    borrower,
                    creditLimitCents,
                    maxAprBps
                )
            );
    }

    function hashVaultDeposit(
        address vault,
        address depositor,
        uint256 assets
    ) internal pure returns (bytes32) {
        return keccak256(abi.encode(vault, depositor, assets));
    }

    function hashOracleUpdate(
        address oracleConsumer,
        bytes32 feedId,
        uint256 price,
        uint256 validUntil
    ) internal pure returns (bytes32) {
        return
            keccak256(
                abi.encode(oracleConsumer, feedId, price, validUntil)
            );
    }

    function hashRwaOnboarding(
        address rwaGate,
        bytes32 claimsHash,
        bytes32 decisionCommitmentHash
    ) internal pure returns (bytes32) {
        return keccak256(abi.encode(rwaGate, claimsHash, decisionCommitmentHash));
    }

    function hashRwaTransfer(
        address rwaGate,
        bytes32 claimsHash,
        bytes32 decisionCommitmentHash
    ) internal pure returns (bytes32) {
        return keccak256(abi.encode(rwaGate, claimsHash, decisionCommitmentHash));
    }
}
