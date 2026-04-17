// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.20;

import "../interfaces/IZKCGVerifier.sol";

contract ZKCGVerifierConsumer {
    IZKCGVerifier public immutable verifier;

    event SettledWithProof(address indexed beneficiary, uint256 amount);

    constructor(address verifierAddress) {
        verifier = IZKCGVerifier(verifierAddress);
    }

    function settleWithProof(
        bytes calldata proof,
        bytes calldata publicInputs,
        address beneficiary,
        uint256 amount
    ) external {
        require(
            verifier.verifyZKCG(proof, publicInputs),
            "zkcg: invalid proof"
        );

        _release(beneficiary, amount);
        emit SettledWithProof(beneficiary, amount);
    }

    function _release(address beneficiary, uint256 amount) internal virtual {
        beneficiary;
        amount;
    }
}
