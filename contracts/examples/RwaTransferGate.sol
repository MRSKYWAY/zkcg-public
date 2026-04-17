// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.20;

import "./BaseZKCGDecisionConsumer.sol";
import "../libraries/ZKCGDecisionLib.sol";

contract RwaTransferGate is BaseZKCGDecisionConsumer {
    struct ApprovedTransfer {
        bytes32 claimsHash;
        bytes32 decisionCommitmentHash;
        bytes32 decisionId;
    }

    mapping(bytes32 => ApprovedTransfer) public approvedTransfers;

    event TransferApproved(
        bytes32 indexed transferKey,
        bytes32 indexed decisionId,
        bytes32 claimsHash,
        bytes32 decisionCommitmentHash
    );

    constructor(address verifierAddress)
        BaseZKCGDecisionConsumer(verifierAddress)
    {}

    function approveTransfer(
        bytes calldata proof,
        bytes calldata publicInputs,
        bytes32 claimsHash,
        bytes32 decisionCommitmentHash
    ) external {
        ZKCGDecisionLib.DecisionInputs memory inputs = _consumeVerifiedDecision(
            proof,
            publicInputs,
            ZKCGDecisionLib.RWA_CREDIT_TRANSFER_V1_HASH,
            ZKCGDecisionLib.hashRwaTransfer(address(this), claimsHash, decisionCommitmentHash)
        );

        bytes32 transferKey = transferKeyFor(claimsHash, decisionCommitmentHash);
        approvedTransfers[transferKey] = ApprovedTransfer({
            claimsHash: claimsHash,
            decisionCommitmentHash: decisionCommitmentHash,
            decisionId: inputs.decisionId
        });

        emit TransferApproved(
            transferKey,
            inputs.decisionId,
            claimsHash,
            decisionCommitmentHash
        );
    }

    function transferKeyFor(
        bytes32 claimsHash,
        bytes32 decisionCommitmentHash
    ) public pure returns (bytes32) {
        return keccak256(abi.encode(claimsHash, decisionCommitmentHash));
    }

    function approvedDecisionId(
        bytes32 transferKey
    ) external view returns (bytes32) {
        return approvedTransfers[transferKey].decisionId;
    }

    function isTransferApproved(
        bytes32 transferKey
    ) external view returns (bool) {
        return approvedTransfers[transferKey].decisionId != bytes32(0);
    }
}
