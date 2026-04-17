// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.20;

import "../interfaces/IZKCGVerifier.sol";
import "../libraries/ZKCGDecisionLib.sol";

abstract contract BaseZKCGDecisionConsumer {
    IZKCGVerifier public immutable verifier;

    mapping(bytes32 => bool) public consumedDecision;

    event DecisionConsumed(bytes32 indexed decisionId);

    error InvalidZKCGProof();
    error DecisionAlreadyConsumed(bytes32 decisionId);
    error UnexpectedPolicyVersion(bytes32 expected, bytes32 actual);
    error UnexpectedPayloadHash(bytes32 expected, bytes32 actual);

    constructor(address verifierAddress) {
        require(verifierAddress != address(0), "zkcg: zero verifier");
        verifier = IZKCGVerifier(verifierAddress);
    }

    function _consumeVerifiedDecision(
        bytes calldata proof,
        bytes calldata publicInputs,
        bytes32 expectedPolicyVersionHash,
        bytes32 expectedPayloadHash
    ) internal returns (ZKCGDecisionLib.DecisionInputs memory inputs) {
        if (!verifier.verifyZKCG(proof, publicInputs)) {
            revert InvalidZKCGProof();
        }

        inputs = ZKCGDecisionLib.decodeDecisionInputs(publicInputs);

        if (consumedDecision[inputs.decisionId]) {
            revert DecisionAlreadyConsumed(inputs.decisionId);
        }

        if (inputs.policyVersionHash != expectedPolicyVersionHash) {
            revert UnexpectedPolicyVersion(
                expectedPolicyVersionHash,
                inputs.policyVersionHash
            );
        }

        if (inputs.payloadHash != expectedPayloadHash) {
            revert UnexpectedPayloadHash(
                expectedPayloadHash,
                inputs.payloadHash
            );
        }

        consumedDecision[inputs.decisionId] = true;
        emit DecisionConsumed(inputs.decisionId);
    }
}
