// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Test.sol";
import "../src/ZkIPFSVerifier.sol";
import "../src/libraries/ProofLib.sol";

/**
 * @title ZkIPFSVerifierTest
 * @author Sowad Al-Mughni
 * @notice Comprehensive tests for the ZkIPFS verifier contract
 */
contract ZkIPFSVerifierTest is Test {
    ZkIPFSVerifier public verifier;
    
    address public owner = address(0x1);
    address public feeRecipient = address(0x2);
    address public user1 = address(0x3);
    address public user2 = address(0x4);

    // Test proof data
    ProofLib.Proof public testProof;
    bytes32 public constant TEST_CONTENT_HASH = keccak256("test content");
    bytes32 public constant TEST_ROOT_HASH = keccak256("test root");
    bytes32 public constant TEST_FILE_HASH = keccak256("test file");

    event ProofVerified(
        bytes32 indexed proofHash,
        address indexed verifier,
        bool isValid,
        uint256 gasUsed,
        uint256 timestamp
    );

    function setUp() public {
        // Deploy contract
        verifier = new ZkIPFSVerifier(owner, feeRecipient);

        // Create test proof
        testProof = ProofLib.Proof({
            id: keccak256("test-proof-1"),
            timestamp: block.timestamp,
            securityLevel: 128,
            proofSystem: "Risc0",
            contentHash: TEST_CONTENT_HASH,
            rootHash: TEST_ROOT_HASH,
            fileHash: TEST_FILE_HASH,
            fileSize: 1024,
            receipt: abi.encodePacked(keccak256("test receipt data")),
            publicInputs: abi.encodePacked(TEST_CONTENT_HASH, TEST_ROOT_HASH),
            imageId: keccak256("test image id"),
            selection: ProofLib.createFullFileSelection(TEST_FILE_HASH),
            maxAge: 30 days,
            requiresOnChainData: false
        });

        // Fund test accounts
        vm.deal(user1, 10 ether);
        vm.deal(user2, 10 ether);
    }

    function testDeployment() public {
        assertEq(verifier.owner(), owner);
        assertEq(verifier.feeRecipient(), feeRecipient);
        assertEq(verifier.minSecurityLevel(), 128);
        assertEq(verifier.maxProofAge(), 30 days);
        assertEq(verifier.verificationFee(), 0);
        assertTrue(verifier.supportedProofSystems("Risc0"));
        assertTrue(verifier.supportedProofSystems("Groth16"));
        assertTrue(verifier.supportedProofSystems("Plonk"));
    }

    function testVerifyValidProof() public {
        vm.startPrank(user1);
        
        // Verify the proof
        ZkIPFSVerifier.VerificationResult memory result = verifier.verifyProof(testProof);
        
        // Check result
        assertTrue(result.isValid);
        assertEq(result.verifier, user1);
        assertEq(result.contentHash, TEST_CONTENT_HASH);
        assertEq(result.rootHash, TEST_ROOT_HASH);
        assertEq(result.securityLevel, 128);
        assertEq(result.proofSystem, "Risc0");
        assertGt(result.gasUsed, 0);
        assertEq(result.timestamp, block.timestamp);

        vm.stopPrank();
    }

    function testVerifyProofWithFee() public {
        // Set verification fee
        vm.prank(owner);
        verifier.setVerificationFee(0.01 ether);

        vm.startPrank(user1);
        
        // Verify with correct fee
        ZkIPFSVerifier.VerificationResult memory result = verifier.verifyProof{value: 0.01 ether}(testProof);
        assertTrue(result.isValid);

        vm.stopPrank();

        // Check fee was transferred
        assertEq(feeRecipient.balance, 0.01 ether);
    }

    function testVerifyProofInsufficientFee() public {
        // Set verification fee
        vm.prank(owner);
        verifier.setVerificationFee(0.01 ether);

        vm.startPrank(user1);
        
        // Try to verify with insufficient fee
        vm.expectRevert(ZkIPFSVerifier.InsufficientFee.selector);
        verifier.verifyProof{value: 0.005 ether}(testProof);

        vm.stopPrank();
    }

    function testVerifyProofExcessFeeRefund() public {
        // Set verification fee
        vm.prank(owner);
        verifier.setVerificationFee(0.01 ether);

        uint256 initialBalance = user1.balance;

        vm.startPrank(user1);
        
        // Verify with excess fee
        verifier.verifyProof{value: 0.02 ether}(testProof);

        vm.stopPrank();

        // Check refund
        assertEq(user1.balance, initialBalance - 0.01 ether);
        assertEq(feeRecipient.balance, 0.01 ether);
    }

    function testVerifyProofAlreadyVerified() public {
        vm.startPrank(user1);
        
        // Verify the proof first time
        verifier.verifyProof(testProof);
        
        // Try to verify again
        vm.expectRevert(ZkIPFSVerifier.ProofAlreadyVerified.selector);
        verifier.verifyProof(testProof);

        vm.stopPrank();
    }

    function testVerifyExpiredProof() public {
        // Create expired proof
        ProofLib.Proof memory expiredProof = testProof;
        expiredProof.timestamp = block.timestamp - 31 days; // Older than maxProofAge

        vm.startPrank(user1);
        
        vm.expectRevert(ZkIPFSVerifier.ProofTooOld.selector);
        verifier.verifyProof(expiredProof);

        vm.stopPrank();
    }

    function testVerifyLowSecurityProof() public {
        // Create low security proof
        ProofLib.Proof memory lowSecProof = testProof;
        lowSecProof.securityLevel = 64; // Below minimum

        vm.startPrank(user1);
        
        vm.expectRevert(ZkIPFSVerifier.InsufficientSecurityLevel.selector);
        verifier.verifyProof(lowSecProof);

        vm.stopPrank();
    }

    function testVerifyUnsupportedProofSystem() public {
        // Create proof with unsupported system
        ProofLib.Proof memory unsupportedProof = testProof;
        unsupportedProof.proofSystem = "UnsupportedSystem";

        vm.startPrank(user1);
        
        vm.expectRevert(ZkIPFSVerifier.UnsupportedProofSystem.selector);
        verifier.verifyProof(unsupportedProof);

        vm.stopPrank();
    }

    function testBatchVerification() public {
        // Create multiple proofs
        ProofLib.Proof[] memory proofs = new ProofLib.Proof[](3);
        
        for (uint256 i = 0; i < 3; i++) {
            proofs[i] = testProof;
            proofs[i].id = keccak256(abi.encodePacked("test-proof-", i));
            proofs[i].contentHash = keccak256(abi.encodePacked("content-", i));
        }

        vm.startPrank(user1);
        
        // Verify batch
        ZkIPFSVerifier.VerificationResult[] memory results = verifier.verifyBatch(proofs);
        
        // Check results
        assertEq(results.length, 3);
        for (uint256 i = 0; i < 3; i++) {
            assertTrue(results[i].isValid);
            assertEq(results[i].verifier, user1);
        }

        vm.stopPrank();
    }

    function testBatchVerificationTooLarge() public {
        // Create batch that's too large
        ProofLib.Proof[] memory proofs = new ProofLib.Proof[](51); // Exceeds MAX_BATCH_SIZE

        vm.startPrank(user1);
        
        vm.expectRevert(ZkIPFSVerifier.InvalidBatchSize.selector);
        verifier.verifyBatch(proofs);

        vm.stopPrank();
    }

    function testBatchVerificationEmpty() public {
        // Create empty batch
        ProofLib.Proof[] memory proofs = new ProofLib.Proof[](0);

        vm.startPrank(user1);
        
        vm.expectRevert(ZkIPFSVerifier.InvalidBatchSize.selector);
        verifier.verifyBatch(proofs);

        vm.stopPrank();
    }

    function testGetVerificationResult() public {
        vm.startPrank(user1);
        
        // Verify proof
        verifier.verifyProof(testProof);
        
        // Get result
        bytes32 proofHash = testProof.getHash();
        ZkIPFSVerifier.VerificationResult memory result = verifier.getVerificationResult(proofHash);
        
        assertTrue(result.isValid);
        assertEq(result.verifier, user1);

        vm.stopPrank();
    }

    function testIsProofVerified() public {
        bytes32 proofHash = testProof.getHash();
        
        // Initially not verified
        assertFalse(verifier.isProofVerified(proofHash));
        
        vm.startPrank(user1);
        
        // Verify proof
        verifier.verifyProof(testProof);
        
        // Now verified
        assertTrue(verifier.isProofVerified(proofHash));

        vm.stopPrank();
    }

    function testGetUserStats() public {
        vm.startPrank(user1);
        
        // Initially empty stats
        ZkIPFSVerifier.VerificationStats memory stats = verifier.getUserStats(user1);
        assertEq(stats.totalVerifications, 0);
        assertEq(stats.successfulVerifications, 0);
        
        // Verify proof
        verifier.verifyProof(testProof);
        
        // Check updated stats
        stats = verifier.getUserStats(user1);
        assertEq(stats.totalVerifications, 1);
        assertEq(stats.successfulVerifications, 1);
        assertGt(stats.totalGasUsed, 0);
        assertEq(stats.lastVerificationTime, block.timestamp);

        vm.stopPrank();
    }

    function testGetContractStats() public {
        // Initially empty
        (uint256 total, uint256 successful, uint256 successRate) = verifier.getContractStats();
        assertEq(total, 0);
        assertEq(successful, 0);
        assertEq(successRate, 0);

        vm.startPrank(user1);
        
        // Verify proof
        verifier.verifyProof(testProof);
        
        // Check updated stats
        (total, successful, successRate) = verifier.getContractStats();
        assertEq(total, 1);
        assertEq(successful, 1);
        assertEq(successRate, 10000); // 100% * 10000

        vm.stopPrank();
    }

    // Admin function tests

    function testSetMinSecurityLevel() public {
        vm.startPrank(owner);
        
        vm.expectEmit(true, true, false, true);
        emit ZkIPFSVerifier.SecurityLevelUpdated(128, 192);
        
        verifier.setMinSecurityLevel(192);
        assertEq(verifier.minSecurityLevel(), 192);

        vm.stopPrank();
    }

    function testSetMinSecurityLevelInvalid() public {
        vm.startPrank(owner);
        
        // Too low
        vm.expectRevert(ZkIPFSVerifier.InvalidSecurityLevel.selector);
        verifier.setMinSecurityLevel(50);
        
        // Too high
        vm.expectRevert(ZkIPFSVerifier.InvalidSecurityLevel.selector);
        verifier.setMinSecurityLevel(300);

        vm.stopPrank();
    }

    function testSetMinSecurityLevelUnauthorized() public {
        vm.startPrank(user1);
        
        vm.expectRevert("Ownable: caller is not the owner");
        verifier.setMinSecurityLevel(192);

        vm.stopPrank();
    }

    function testSetMaxProofAge() public {
        vm.startPrank(owner);
        
        vm.expectEmit(true, true, false, true);
        emit ZkIPFSVerifier.MaxProofAgeUpdated(30 days, 7 days);
        
        verifier.setMaxProofAge(7 days);
        assertEq(verifier.maxProofAge(), 7 days);

        vm.stopPrank();
    }

    function testSetMaxProofAgeInvalid() public {
        vm.startPrank(owner);
        
        // Too short
        vm.expectRevert(ZkIPFSVerifier.InvalidMaxAge.selector);
        verifier.setMaxProofAge(30 minutes);
        
        // Too long
        vm.expectRevert(ZkIPFSVerifier.InvalidMaxAge.selector);
        verifier.setMaxProofAge(400 days);

        vm.stopPrank();
    }

    function testSetVerificationFee() public {
        vm.startPrank(owner);
        
        vm.expectEmit(true, true, false, true);
        emit ZkIPFSVerifier.VerificationFeeUpdated(0, 0.01 ether);
        
        verifier.setVerificationFee(0.01 ether);
        assertEq(verifier.verificationFee(), 0.01 ether);

        vm.stopPrank();
    }

    function testSetFeeRecipient() public {
        address newRecipient = address(0x5);
        
        vm.startPrank(owner);
        
        vm.expectEmit(true, true, false, true);
        emit ZkIPFSVerifier.FeeRecipientUpdated(feeRecipient, newRecipient);
        
        verifier.setFeeRecipient(newRecipient);
        assertEq(verifier.feeRecipient(), newRecipient);

        vm.stopPrank();
    }

    function testSetFeeRecipientZeroAddress() public {
        vm.startPrank(owner);
        
        vm.expectRevert(ZkIPFSVerifier.InvalidFeeRecipient.selector);
        verifier.setFeeRecipient(address(0));

        vm.stopPrank();
    }

    function testSetProofSystemSupport() public {
        vm.startPrank(owner);
        
        vm.expectEmit(false, false, false, true);
        emit ZkIPFSVerifier.ProofSystemUpdated("NewSystem", true);
        
        verifier.setProofSystemSupport("NewSystem", true);
        assertTrue(verifier.supportedProofSystems("NewSystem"));
        
        verifier.setProofSystemSupport("NewSystem", false);
        assertFalse(verifier.supportedProofSystems("NewSystem"));

        vm.stopPrank();
    }

    function testPauseUnpause() public {
        vm.startPrank(owner);
        
        // Pause contract
        verifier.pause();
        assertTrue(verifier.paused());
        
        // Unpause contract
        verifier.unpause();
        assertFalse(verifier.paused());

        vm.stopPrank();
    }

    function testVerifyWhenPaused() public {
        vm.prank(owner);
        verifier.pause();

        vm.startPrank(user1);
        
        vm.expectRevert("Pausable: paused");
        verifier.verifyProof(testProof);

        vm.stopPrank();
    }

    function testEmergencyWithdraw() public {
        // Send some ETH to contract
        vm.deal(address(verifier), 1 ether);
        
        uint256 ownerBalanceBefore = owner.balance;
        
        vm.startPrank(owner);
        
        verifier.emergencyWithdraw(address(0), 0.5 ether);
        
        vm.stopPrank();
        
        assertEq(owner.balance, ownerBalanceBefore + 0.5 ether);
    }

    function testReceiveEther() public {
        // Send ETH to contract
        (bool success,) = address(verifier).call{value: 1 ether}("");
        assertTrue(success);
        assertEq(address(verifier).balance, 1 ether);
    }

    // Fuzz testing

    function testFuzzVerificationFee(uint256 fee) public {
        vm.assume(fee <= 10 ether); // Reasonable upper bound
        
        vm.prank(owner);
        verifier.setVerificationFee(fee);
        
        assertEq(verifier.verificationFee(), fee);
    }

    function testFuzzSecurityLevel(uint256 level) public {
        vm.assume(level >= 80 && level <= 256);
        
        vm.prank(owner);
        verifier.setMinSecurityLevel(level);
        
        assertEq(verifier.minSecurityLevel(), level);
    }

    function testFuzzMaxProofAge(uint256 age) public {
        vm.assume(age >= 1 hours && age <= 365 days);
        
        vm.prank(owner);
        verifier.setMaxProofAge(age);
        
        assertEq(verifier.maxProofAge(), age);
    }
}

