// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.20;

import "./interfaces/IZKCGVerifier.sol";
import "./libraries/ZKCGDecisionLib.sol";

contract ZKCGVerifier is IZKCGVerifier {
    bytes32 public constant DECISION_TYPEHASH =
        keccak256(
            "ZKCGDecision(uint256 chainId,address verifyingContract,bytes32 decisionId,bytes32 policyVersionHash,bytes32 proofSystemHash,bytes32 payloadHash,uint256 expiresAt)"
        );

    uint256 private constant SECP256K1_HALF_N =
        0x7fffffffffffffffffffffffffffffff5d576e7357a4501ddfe92f46681b20a0;

    address public owner;
    address public zkcgSigner;
    bool public paused;

    mapping(bytes32 => bool) public acceptedPolicyVersion;
    mapping(bytes32 => bool) public acceptedProofSystem;

    event OwnershipTransferred(
        address indexed previousOwner,
        address indexed newOwner
    );
    event SignerUpdated(
        address indexed previousSigner,
        address indexed newSigner
    );
    event PolicyVersionUpdated(bytes32 indexed policyVersionHash, bool accepted);
    event ProofSystemUpdated(bytes32 indexed proofSystemHash, bool accepted);
    event PauseUpdated(bool paused);

    constructor(address signer) {
        require(signer != address(0), "zkcg: zero signer");

        owner = msg.sender;
        zkcgSigner = signer;

        _setAcceptedPolicyVersion(ZKCGDecisionLib.LENDING_V1_HASH, true);
        _setAcceptedPolicyVersion(
            ZKCGDecisionLib.RISK_GATED_VAULT_V1_HASH,
            true
        );
        _setAcceptedPolicyVersion(
            ZKCGDecisionLib.ORACLE_SANITY_V1_HASH,
            true
        );
        _setAcceptedPolicyVersion(
            ZKCGDecisionLib.RWA_CREDIT_ONBOARDING_V1_HASH,
            true
        );
        _setAcceptedPolicyVersion(
            ZKCGDecisionLib.RWA_CREDIT_TRANSFER_V1_HASH,
            true
        );

        _setAcceptedProofSystem(ZKCGDecisionLib.ZKVM_HASH, true);
        _setAcceptedProofSystem(ZKCGDecisionLib.HALO2_HASH, true);
    }

    function verifyZKCG(
        bytes calldata proof,
        bytes calldata publicInputs
    ) external view override returns (bool) {
        return _verify(proof, publicInputs);
    }

    function decodeDecisionInputs(
        bytes calldata publicInputs
    )
        external
        pure
        returns (
            bytes32 decisionId,
            bytes32 policyVersionHash,
            bytes32 proofSystemHash,
            bytes32 payloadHash,
            uint256 expiresAt
        )
    {
        if (publicInputs.length != ZKCGDecisionLib.DECISION_INPUTS_LENGTH) {
            return (bytes32(0), bytes32(0), bytes32(0), bytes32(0), 0);
        }

        ZKCGDecisionLib.DecisionInputs memory inputs = ZKCGDecisionLib
            .decodeDecisionInputs(publicInputs);

        return (
            inputs.decisionId,
            inputs.policyVersionHash,
            inputs.proofSystemHash,
            inputs.payloadHash,
            inputs.expiresAt
        );
    }

    function decisionDigest(
        bytes32 decisionId,
        bytes32 policyVersionHash,
        bytes32 proofSystemHash,
        bytes32 payloadHash,
        uint256 expiresAt
    ) public view returns (bytes32) {
        return
            keccak256(
                abi.encode(
                    DECISION_TYPEHASH,
                    block.chainid,
                    address(this),
                    decisionId,
                    policyVersionHash,
                    proofSystemHash,
                    payloadHash,
                    expiresAt
                )
            );
    }

    function transferOwnership(address newOwner) external onlyOwner {
        require(newOwner != address(0), "zkcg: zero owner");
        emit OwnershipTransferred(owner, newOwner);
        owner = newOwner;
    }

    function updateSigner(address newSigner) external onlyOwner {
        require(newSigner != address(0), "zkcg: zero signer");
        emit SignerUpdated(zkcgSigner, newSigner);
        zkcgSigner = newSigner;
    }

    function setAcceptedPolicyVersion(
        bytes32 policyVersionHash,
        bool accepted
    ) external onlyOwner {
        require(policyVersionHash != bytes32(0), "zkcg: zero policy version");
        _setAcceptedPolicyVersion(policyVersionHash, accepted);
    }

    function setAcceptedProofSystem(
        bytes32 proofSystemHash,
        bool accepted
    ) external onlyOwner {
        require(proofSystemHash != bytes32(0), "zkcg: zero proof system");
        _setAcceptedProofSystem(proofSystemHash, accepted);
    }

    function setPaused(bool newPaused) external onlyOwner {
        paused = newPaused;
        emit PauseUpdated(newPaused);
    }

    function _verify(
        bytes calldata proof,
        bytes calldata publicInputs
    ) internal view returns (bool) {
        if (paused) {
            return false;
        }

        if (
            proof.length != 65 ||
            publicInputs.length != ZKCGDecisionLib.DECISION_INPUTS_LENGTH
        ) {
            return false;
        }

        ZKCGDecisionLib.DecisionInputs memory inputs = ZKCGDecisionLib
            .decodeDecisionInputs(publicInputs);

        if (!acceptedPolicyVersion[inputs.policyVersionHash]) {
            return false;
        }

        if (!acceptedProofSystem[inputs.proofSystemHash]) {
            return false;
        }

        if (block.timestamp > inputs.expiresAt) {
            return false;
        }

        return
            _recoverSigner(
                decisionDigest(
                    inputs.decisionId,
                    inputs.policyVersionHash,
                    inputs.proofSystemHash,
                    inputs.payloadHash,
                    inputs.expiresAt
                ),
                proof
            ) == zkcgSigner;
    }

    function _setAcceptedPolicyVersion(
        bytes32 policyVersionHash,
        bool accepted
    ) internal {
        acceptedPolicyVersion[policyVersionHash] = accepted;
        emit PolicyVersionUpdated(policyVersionHash, accepted);
    }

    function _setAcceptedProofSystem(
        bytes32 proofSystemHash,
        bool accepted
    ) internal {
        acceptedProofSystem[proofSystemHash] = accepted;
        emit ProofSystemUpdated(proofSystemHash, accepted);
    }

    function _recoverSigner(
        bytes32 digest,
        bytes calldata signature
    ) internal pure returns (address signer) {
        if (signature.length != 65) {
            return address(0);
        }

        bytes32 r;
        bytes32 s;
        uint8 v;

        assembly {
            r := calldataload(signature.offset)
            s := calldataload(add(signature.offset, 32))
            v := byte(0, calldataload(add(signature.offset, 64)))
        }

        if (v < 27) {
            v += 27;
        }

        if (v != 27 && v != 28) {
            return address(0);
        }

        if (uint256(s) > SECP256K1_HALF_N) {
            return address(0);
        }

        bytes32 ethSignedDigest = keccak256(
            abi.encodePacked("\x19Ethereum Signed Message:\n32", digest)
        );

        signer = ecrecover(ethSignedDigest, v, r, s);
    }

    modifier onlyOwner() {
        require(msg.sender == owner, "zkcg: not owner");
        _;
    }
}
