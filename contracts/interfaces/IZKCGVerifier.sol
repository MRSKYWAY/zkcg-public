// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.20;

interface IZKCGVerifier {
    function verifyZKCG(
        bytes calldata proof,
        bytes calldata publicInputs
    ) external view returns (bool);
}
