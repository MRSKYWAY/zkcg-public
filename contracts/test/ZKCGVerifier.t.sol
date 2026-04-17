// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.20;

import "../ZKCGVerifier.sol";
import "../examples/PrivateLoanEligibilityGate.sol";
import "../examples/RiskGatedVault.sol";
import "../examples/RwaTransferGate.sol";
import "../examples/VerifiableOracleWithPolicy.sol";

interface Vm {
    function addr(uint256 privateKey) external returns (address);

    function sign(
        uint256 privateKey,
        bytes32 digest
    ) external returns (uint8 v, bytes32 r, bytes32 s);

    function warp(uint256 newTimestamp) external;
}

contract ZKCGVerifierTest {
    Vm internal constant vm =
        Vm(address(uint160(uint256(keccak256("hevm cheat code")))));

    uint256 internal constant SIGNER_PRIVATE_KEY =
        0x59c6995e998f97a5a0044976f7d8f2f6f57f0e8f4b0fba4b2d6538aacbb44c41;

    bytes32 internal constant LENDING_V1_HASH = keccak256("lending.v1");
    bytes32 internal constant VAULT_V1_HASH = keccak256("risk-gated-vault.v1");
    bytes32 internal constant ORACLE_V1_HASH = keccak256("oracle.sanity.v1");
    bytes32 internal constant RWA_TRANSFER_V1_HASH =
        keccak256("rwa.credit.transfer.v1");
    bytes32 internal constant ZKVM_HASH = keccak256("zkvm");

    ZKCGVerifier internal verifier;
    PrivateLoanEligibilityGate internal loanGate;
    RiskGatedVault internal vault;
    VerifiableOracleWithPolicy internal oracle;
    RwaTransferGate internal rwaTransferGate;

    address internal signer;

    function setUp() public {
        signer = vm.addr(SIGNER_PRIVATE_KEY);
        verifier = new ZKCGVerifier(signer);
        loanGate = new PrivateLoanEligibilityGate(address(verifier));
        vault = new RiskGatedVault(address(verifier));
        oracle = new VerifiableOracleWithPolicy(address(verifier));
        rwaTransferGate = new RwaTransferGate(address(verifier));
        vm.warp(1_800_000_000);
    }

    function testVerifierAcceptsValidLoanAttestation() public {
        address borrower = address(0xB0B);
        uint256 creditLimitCents = 1_000_000;
        uint256 maxAprBps = 1_500;
        uint256 expiresAt = block.timestamp + 1 days;
        bytes32 decisionId = keccak256(bytes("loan:applicant-1"));
        bytes32 payloadHash = keccak256(
            abi.encode(
                address(loanGate),
                borrower,
                creditLimitCents,
                maxAprBps
            )
        );

        (bytes memory proof, bytes memory publicInputs) = signedDecision(
            decisionId,
            LENDING_V1_HASH,
            ZKVM_HASH,
            payloadHash,
            expiresAt
        );

        assertTrue(
            verifier.verifyZKCG(proof, publicInputs),
            "verifier should accept a valid attestation"
        );
    }

    function testLoanGateConsumesVerifierAttestation() public {
        address borrower = address(0xCAFE);
        uint256 creditLimitCents = 2_000_000;
        uint256 maxAprBps = 1_250;
        uint256 expiresAt = block.timestamp + 1 days;
        bytes32 decisionId = keccak256(bytes("loan:applicant-2"));
        bytes32 payloadHash = keccak256(
            abi.encode(
                address(loanGate),
                borrower,
                creditLimitCents,
                maxAprBps
            )
        );

        (bytes memory proof, bytes memory publicInputs) = signedDecision(
            decisionId,
            LENDING_V1_HASH,
            ZKVM_HASH,
            payloadHash,
            expiresAt
        );

        loanGate.approveBorrower(
            proof,
            publicInputs,
            borrower,
            creditLimitCents,
            maxAprBps
        );

        (
            uint256 storedLimit,
            uint256 storedApr,
            bytes32 storedDecisionId
        ) = loanGate.approvals(borrower);

        assertEq(storedLimit, creditLimitCents, "credit limit should be stored");
        assertEq(storedApr, maxAprBps, "max APR should be stored");
        assertEq(
            storedDecisionId,
            decisionId,
            "decision id should be tracked"
        );
    }

    function testRiskGatedVaultAcceptsDepositDecision() public {
        address depositor = address(this);
        uint256 assets = 5 ether;
        uint256 expiresAt = block.timestamp + 1 days;
        bytes32 decisionId = keccak256(bytes("vault:user-1"));
        bytes32 payloadHash = keccak256(
            abi.encode(address(vault), depositor, assets)
        );

        (bytes memory proof, bytes memory publicInputs) = signedDecision(
            decisionId,
            VAULT_V1_HASH,
            ZKVM_HASH,
            payloadHash,
            expiresAt
        );

        vault.depositWithDecision(proof, publicInputs, assets);

        assertEq(
            vault.balances(depositor),
            assets,
            "vault balance should increase"
        );
    }

    function testOracleAcceptsSignedUpdate() public {
        bytes32 feedId = keccak256(bytes("ETH-USD"));
        uint256 price = 3_000e8;
        uint256 validUntil = block.timestamp + 1 hours;
        uint256 expiresAt = validUntil;
        bytes32 decisionId = keccak256(bytes("oracle:ETH-USD:1"));
        bytes32 payloadHash = keccak256(
            abi.encode(address(oracle), feedId, price, validUntil)
        );

        (bytes memory proof, bytes memory publicInputs) = signedDecision(
            decisionId,
            ORACLE_V1_HASH,
            ZKVM_HASH,
            payloadHash,
            expiresAt
        );

        oracle.submitPrice(proof, publicInputs, feedId, price, validUntil);

        assertEq(oracle.readPrice(feedId), price, "oracle price should update");
    }

    function testLoanGateRejectsReplay() public {
        address borrower = address(0x1234);
        uint256 creditLimitCents = 1_500_000;
        uint256 maxAprBps = 1_400;
        uint256 expiresAt = block.timestamp + 1 days;
        bytes32 decisionId = keccak256(bytes("loan:replay"));
        bytes32 payloadHash = keccak256(
            abi.encode(
                address(loanGate),
                borrower,
                creditLimitCents,
                maxAprBps
            )
        );

        (bytes memory proof, bytes memory publicInputs) = signedDecision(
            decisionId,
            LENDING_V1_HASH,
            ZKVM_HASH,
            payloadHash,
            expiresAt
        );

        loanGate.approveBorrower(
            proof,
            publicInputs,
            borrower,
            creditLimitCents,
            maxAprBps
        );

        (bool ok, ) = address(loanGate).call(
            abi.encodeWithSelector(
                loanGate.approveBorrower.selector,
                proof,
                publicInputs,
                borrower,
                creditLimitCents,
                maxAprBps
            )
        );

        assertTrue(!ok, "replayed decision should revert");
    }

    function testRwaTransferGateConsumesTransferApproval() public {
        uint256 expiresAt = block.timestamp + 1 days;
        bytes32 decisionId = keccak256(bytes("rwa:transfer:issuer-a:credit-fund-a:1"));
        bytes32 claimsHash = keccak256(bytes("claims:rwa-transfer-1"));
        bytes32 decisionCommitmentHash =
            keccak256(bytes("decision:rwa-transfer-1"));
        bytes32 payloadHash = keccak256(
            abi.encode(
                address(rwaTransferGate),
                claimsHash,
                decisionCommitmentHash
            )
        );

        (bytes memory proof, bytes memory publicInputs) = signedDecision(
            decisionId,
            RWA_TRANSFER_V1_HASH,
            ZKVM_HASH,
            payloadHash,
            expiresAt
        );

        rwaTransferGate.approveTransfer(
            proof,
            publicInputs,
            claimsHash,
            decisionCommitmentHash
        );

        bytes32 transferKey = rwaTransferGate.transferKeyFor(
            claimsHash,
            decisionCommitmentHash
        );
        assertTrue(
            rwaTransferGate.isTransferApproved(transferKey),
            "transfer approval should be stored"
        );
        assertEq(
            rwaTransferGate.approvedDecisionId(transferKey),
            decisionId,
            "decision id should be tracked"
        );
    }

    function signedDecision(
        bytes32 decisionId,
        bytes32 policyVersionHash,
        bytes32 proofSystemHash,
        bytes32 payloadHash,
        uint256 expiresAt
    ) internal returns (bytes memory proof, bytes memory publicInputs) {
        publicInputs = abi.encode(
            decisionId,
            policyVersionHash,
            proofSystemHash,
            payloadHash,
            expiresAt
        );

        bytes32 digest = verifier.decisionDigest(
            decisionId,
            policyVersionHash,
            proofSystemHash,
            payloadHash,
            expiresAt
        );
        bytes32 ethSignedDigest = keccak256(
            abi.encodePacked("\x19Ethereum Signed Message:\n32", digest)
        );
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(
            SIGNER_PRIVATE_KEY,
            ethSignedDigest
        );
        proof = abi.encodePacked(r, s, v);
    }

    function assertTrue(bool condition, string memory message) internal pure {
        require(condition, message);
    }

    function assertEq(
        uint256 left,
        uint256 right,
        string memory message
    ) internal pure {
        require(left == right, message);
    }

    function assertEq(
        bytes32 left,
        bytes32 right,
        string memory message
    ) internal pure {
        require(left == right, message);
    }

}
