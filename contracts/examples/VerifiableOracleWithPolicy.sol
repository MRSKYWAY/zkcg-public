// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.20;

import "./BaseZKCGDecisionConsumer.sol";
import "../libraries/ZKCGDecisionLib.sol";

contract VerifiableOracleWithPolicy is BaseZKCGDecisionConsumer {
    struct PriceRecord {
        uint256 price;
        uint256 validUntil;
        bytes32 decisionId;
    }

    mapping(bytes32 => PriceRecord) public latestPrice;

    event PriceAccepted(
        bytes32 indexed feedId,
        bytes32 indexed decisionId,
        uint256 price,
        uint256 validUntil
    );

    constructor(address verifierAddress)
        BaseZKCGDecisionConsumer(verifierAddress)
    {}

    function submitPrice(
        bytes calldata proof,
        bytes calldata publicInputs,
        bytes32 feedId,
        uint256 price,
        uint256 validUntil
    ) external {
        ZKCGDecisionLib.DecisionInputs memory inputs = _consumeVerifiedDecision(
            proof,
            publicInputs,
            ZKCGDecisionLib.ORACLE_SANITY_V1_HASH,
            ZKCGDecisionLib.hashOracleUpdate(
                address(this),
                feedId,
                price,
                validUntil
            )
        );

        latestPrice[feedId] = PriceRecord({
            price: price,
            validUntil: validUntil,
            decisionId: inputs.decisionId
        });

        emit PriceAccepted(feedId, inputs.decisionId, price, validUntil);
    }

    function readPrice(bytes32 feedId) external view returns (uint256 price) {
        PriceRecord memory record = latestPrice[feedId];
        require(record.validUntil >= block.timestamp, "zkcg: stale price");
        return record.price;
    }
}
