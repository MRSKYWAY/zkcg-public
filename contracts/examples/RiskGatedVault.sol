// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.20;

import "./BaseZKCGDecisionConsumer.sol";
import "../libraries/ZKCGDecisionLib.sol";

contract RiskGatedVault is BaseZKCGDecisionConsumer {
    mapping(address => uint256) public balances;

    event DepositAccepted(
        bytes32 indexed decisionId,
        address indexed depositor,
        uint256 assets
    );

    constructor(address verifierAddress)
        BaseZKCGDecisionConsumer(verifierAddress)
    {}

    function depositWithDecision(
        bytes calldata proof,
        bytes calldata publicInputs,
        uint256 assets
    ) external {
        ZKCGDecisionLib.DecisionInputs memory inputs = _consumeVerifiedDecision(
            proof,
            publicInputs,
            ZKCGDecisionLib.RISK_GATED_VAULT_V1_HASH,
            ZKCGDecisionLib.hashVaultDeposit(address(this), msg.sender, assets)
        );

        balances[msg.sender] += assets;

        emit DepositAccepted(inputs.decisionId, msg.sender, assets);
    }
}
