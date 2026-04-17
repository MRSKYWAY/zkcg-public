// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.20;

import "./BaseZKCGDecisionConsumer.sol";
import "../libraries/ZKCGDecisionLib.sol";

contract PrivateLoanEligibilityGate is BaseZKCGDecisionConsumer {
    struct CreditApproval {
        uint256 creditLimitCents;
        uint256 maxAprBps;
        bytes32 decisionId;
    }

    mapping(address => CreditApproval) public approvals;

    event BorrowerApproved(
        bytes32 indexed decisionId,
        address indexed borrower,
        uint256 creditLimitCents,
        uint256 maxAprBps
    );

    constructor(address verifierAddress)
        BaseZKCGDecisionConsumer(verifierAddress)
    {}

    function approveBorrower(
        bytes calldata proof,
        bytes calldata publicInputs,
        address borrower,
        uint256 creditLimitCents,
        uint256 maxAprBps
    ) external {
        ZKCGDecisionLib.DecisionInputs memory inputs = _consumeVerifiedDecision(
            proof,
            publicInputs,
            ZKCGDecisionLib.LENDING_V1_HASH,
            ZKCGDecisionLib.hashLoanTerms(
                address(this),
                borrower,
                creditLimitCents,
                maxAprBps
            )
        );

        approvals[borrower] = CreditApproval({
            creditLimitCents: creditLimitCents,
            maxAprBps: maxAprBps,
            decisionId: inputs.decisionId
        });

        emit BorrowerApproved(
            inputs.decisionId,
            borrower,
            creditLimitCents,
            maxAprBps
        );
    }
}
