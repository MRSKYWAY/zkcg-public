// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.20;

contract ZKCGSignedDecisionGate {
    bytes32 public constant DECISION_TYPEHASH =
        keccak256(
            "ZKCGApprovedDecision(uint256 chainId,address verifyingContract,bytes32 decisionId,bytes32 policyVersionHash,bytes32 proofSystemHash,bytes32 proofHash,uint256 expiresAt,address beneficiary,uint256 amount)"
        );

    bytes32 public constant LENDING_V1_HASH = keccak256("lending.v1");

    address public owner;
    address public zkcgSigner;
    bytes32 public acceptedPolicyVersionHash;
    bool public paused;
    mapping(bytes32 => bool) public consumedDecision;

    event SettledWithAttestation(bytes32 indexed decisionId, address indexed beneficiary, uint256 amount);
    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);
    event SignerUpdated(address indexed previousSigner, address indexed newSigner);
    event AcceptedPolicyVersionUpdated(bytes32 indexed previousPolicyVersionHash, bytes32 indexed newPolicyVersionHash);
    event PauseUpdated(bool paused);

    constructor(address signer) {
        owner = msg.sender;
        zkcgSigner = signer;
        acceptedPolicyVersionHash = LENDING_V1_HASH;
    }

    function settleApprovedDecision(
        bytes32 decisionId,
        bytes32 policyVersionHash,
        bytes32 proofSystemHash,
        bytes32 proofHash,
        uint256 expiresAt,
        bytes calldata signature,
        address beneficiary,
        uint256 amount
    ) external {
        require(!paused, "zkcg: paused");
        require(!consumedDecision[decisionId], "zkcg: decision already consumed");
        require(block.timestamp <= expiresAt, "zkcg: decision expired");
        require(policyVersionHash == acceptedPolicyVersionHash, "zkcg: unexpected policy version");

        bytes32 digest = keccak256(
            abi.encode(
                DECISION_TYPEHASH,
                block.chainid,
                address(this),
                decisionId,
                policyVersionHash,
                proofSystemHash,
                proofHash,
                expiresAt,
                beneficiary,
                amount
            )
        );

        require(_recoverSigner(digest, signature) == zkcgSigner, "zkcg: invalid attestation");

        consumedDecision[decisionId] = true;
        _release(beneficiary, amount);
        emit SettledWithAttestation(decisionId, beneficiary, amount);
    }

    function decisionDigest(
        bytes32 decisionId,
        bytes32 policyVersionHash,
        bytes32 proofSystemHash,
        bytes32 proofHash,
        uint256 expiresAt,
        address beneficiary,
        uint256 amount
    ) external view returns (bytes32) {
        return keccak256(
            abi.encode(
                DECISION_TYPEHASH,
                block.chainid,
                address(this),
                decisionId,
                policyVersionHash,
                proofSystemHash,
                proofHash,
                expiresAt,
                beneficiary,
                amount
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

    function updateAcceptedPolicyVersion(bytes32 newPolicyVersionHash) external onlyOwner {
        require(newPolicyVersionHash != bytes32(0), "zkcg: zero policy version");
        emit AcceptedPolicyVersionUpdated(
            acceptedPolicyVersionHash,
            newPolicyVersionHash
        );
        acceptedPolicyVersionHash = newPolicyVersionHash;
    }

    function setPaused(bool newPaused) external onlyOwner {
        paused = newPaused;
        emit PauseUpdated(newPaused);
    }

    function _release(address beneficiary, uint256 amount) internal virtual {
        beneficiary;
        amount;
    }

    modifier onlyOwner() {
        require(msg.sender == owner, "zkcg: not owner");
        _;
    }

    function _recoverSigner(
        bytes32 digest,
        bytes calldata signature
    ) internal pure returns (address signer) {
        require(signature.length == 65, "zkcg: invalid signature length");

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

        bytes32 ethSignedDigest = keccak256(
            abi.encodePacked("\x19Ethereum Signed Message:\n32", digest)
        );

        signer = ecrecover(ethSignedDigest, v, r, s);
        require(signer != address(0), "zkcg: invalid signer");
    }
}
