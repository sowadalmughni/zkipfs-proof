// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "../libraries/ProofLib.sol";

/**
 * @title IZkIPFSVerifier
 * @author Sowad Al-Mughni
 * @notice Interface for the ZkIPFS proof verifier contract
 */
interface IZkIPFSVerifier {
    /// @notice Verification result structure
    struct VerificationResult {
        bool isValid;
        uint256 timestamp;
        address verifier;
        bytes32 contentHash;
        bytes32 rootHash;
        uint256 securityLevel;
        string proofSystem;
        uint256 gasUsed;
    }

    /// @notice User verification statistics
    struct VerificationStats {
        uint256 totalVerifications;
        uint256 successfulVerifications;
        uint256 totalGasUsed;
        uint256 lastVerificationTime;
    }

    /// @notice Events
    event ProofVerified(
        bytes32 indexed proofHash,
        address indexed verifier,
        bool isValid,
        uint256 gasUsed,
        uint256 timestamp
    );

    event BatchVerificationCompleted(
        address indexed verifier,
        uint256 totalProofs,
        uint256 successfulProofs,
        uint256 totalGasUsed
    );

    event SecurityLevelUpdated(uint256 oldLevel, uint256 newLevel);
    event MaxProofAgeUpdated(uint256 oldAge, uint256 newAge);
    event VerificationFeeUpdated(uint256 oldFee, uint256 newFee);
    event FeeRecipientUpdated(address oldRecipient, address newRecipient);
    event ProofSystemUpdated(string proofSystem, bool supported);

    /// @notice Verify a single zero-knowledge proof
    function verifyProof(ProofLib.Proof calldata proof)
        external
        payable
        returns (VerificationResult memory result);

    /// @notice Verify multiple proofs in a single transaction
    function verifyBatch(ProofLib.Proof[] calldata proofs)
        external
        payable
        returns (VerificationResult[] memory results);

    /// @notice Get verification result for a proof
    function getVerificationResult(bytes32 proofHash)
        external
        view
        returns (VerificationResult memory result);

    /// @notice Check if a proof has been verified
    function isProofVerified(bytes32 proofHash) external view returns (bool verified);

    /// @notice Get user verification statistics
    function getUserStats(address user) external view returns (VerificationStats memory stats);

    /// @notice Get contract statistics
    function getContractStats()
        external
        view
        returns (uint256 total, uint256 successful, uint256 successRate);
}

