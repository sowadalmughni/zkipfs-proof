// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "./interfaces/IZkIPFSVerifier.sol";
import "./libraries/ProofLib.sol";
import "./libraries/SecurityLib.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/security/Pausable.sol";

/**
 * @title ZkIPFSVerifier
 * @author Sowad Al-Mughni
 * @notice Main contract for verifying zero-knowledge proofs of IPFS content
 * @dev Implements efficient on-chain verification with gas optimization and batch processing
 */
contract ZkIPFSVerifier is IZkIPFSVerifier, Ownable, ReentrancyGuard, Pausable {
    using ProofLib for ProofLib.Proof;
    using SecurityLib for SecurityLib.SecurityParams;

    /// @notice Version of the verifier contract
    string public constant VERSION = "1.0.0";

    /// @notice Maximum number of proofs that can be verified in a single batch
    uint256 public constant MAX_BATCH_SIZE = 50;

    /// @notice Minimum security level required for proofs (in bits)
    uint256 public minSecurityLevel = 128;

    /// @notice Maximum age of proofs in seconds (default: 30 days)
    uint256 public maxProofAge = 30 days;

    /// @notice Fee for proof verification (in wei)
    uint256 public verificationFee = 0;

    /// @notice Address that receives verification fees
    address public feeRecipient;

    /// @notice Mapping of proof hashes to verification results
    mapping(bytes32 => VerificationResult) public verificationResults;

    /// @notice Mapping of proof hashes to verification timestamps
    mapping(bytes32 => uint256) public verificationTimestamps;

    /// @notice Mapping of addresses to their verification statistics
    mapping(address => VerificationStats) public userStats;

    /// @notice Total number of proofs verified
    uint256 public totalVerifications;

    /// @notice Total number of successful verifications
    uint256 public successfulVerifications;

    /// @notice Supported proof systems
    mapping(string => bool) public supportedProofSystems;

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

    /// @notice Custom errors
    error InvalidProof();
    error ProofAlreadyVerified();
    error ProofTooOld();
    error InsufficientSecurityLevel();
    error UnsupportedProofSystem();
    error InvalidBatchSize();
    error InsufficientFee();
    error InvalidFeeRecipient();
    error InvalidSecurityLevel();
    error InvalidMaxAge();

    /**
     * @notice Constructor
     * @param _owner Address of the contract owner
     * @param _feeRecipient Address that receives verification fees
     */
    constructor(address _owner, address _feeRecipient) {
        if (_owner == address(0)) revert InvalidFeeRecipient();
        if (_feeRecipient == address(0)) revert InvalidFeeRecipient();

        _transferOwnership(_owner);
        feeRecipient = _feeRecipient;

        // Initialize supported proof systems
        supportedProofSystems["Risc0"] = true;
        supportedProofSystems["Groth16"] = true;
        supportedProofSystems["Plonk"] = true;

        emit FeeRecipientUpdated(address(0), _feeRecipient);
    }

    /**
     * @notice Verify a single zero-knowledge proof
     * @param proof The proof data to verify
     * @return result The verification result
     */
    function verifyProof(ProofLib.Proof calldata proof)
        external
        payable
        nonReentrant
        whenNotPaused
        returns (VerificationResult memory result)
    {
        // Check verification fee
        if (msg.value < verificationFee) revert InsufficientFee();

        // Validate proof structure and parameters
        _validateProof(proof);

        uint256 gasStart = gasleft();
        bytes32 proofHash = proof.getHash();

        // Check if proof was already verified
        if (verificationResults[proofHash].timestamp != 0) {
            revert ProofAlreadyVerified();
        }

        // Perform cryptographic verification
        bool isValid = _performVerification(proof);

        uint256 gasUsed = gasStart - gasleft();

        // Store verification result
        result = VerificationResult({
            isValid: isValid,
            timestamp: block.timestamp,
            verifier: msg.sender,
            contentHash: proof.contentHash,
            rootHash: proof.rootHash,
            securityLevel: proof.securityLevel,
            proofSystem: proof.proofSystem,
            gasUsed: gasUsed
        });

        verificationResults[proofHash] = result;
        verificationTimestamps[proofHash] = block.timestamp;

        // Update statistics
        _updateStats(msg.sender, isValid, gasUsed);

        // Transfer fee to recipient
        if (verificationFee > 0) {
            (bool success, ) = feeRecipient.call{value: verificationFee}("");
            require(success, "Fee transfer failed");
        }

        // Refund excess payment
        if (msg.value > verificationFee) {
            (bool success, ) = msg.sender.call{value: msg.value - verificationFee}("");
            require(success, "Refund failed");
        }

        emit ProofVerified(proofHash, msg.sender, isValid, gasUsed, block.timestamp);

        return result;
    }

    /**
     * @notice Verify multiple proofs in a single transaction
     * @param proofs Array of proofs to verify
     * @return results Array of verification results
     */
    function verifyBatch(ProofLib.Proof[] calldata proofs)
        external
        payable
        nonReentrant
        whenNotPaused
        returns (VerificationResult[] memory results)
    {
        if (proofs.length == 0 || proofs.length > MAX_BATCH_SIZE) {
            revert InvalidBatchSize();
        }

        uint256 totalFee = verificationFee * proofs.length;
        if (msg.value < totalFee) revert InsufficientFee();

        results = new VerificationResult[](proofs.length);
        uint256 totalGasUsed = 0;
        uint256 successfulCount = 0;

        for (uint256 i = 0; i < proofs.length; i++) {
            uint256 gasStart = gasleft();
            
            try this._verifyProofInternal(proofs[i]) returns (bool isValid) {
                uint256 gasUsed = gasStart - gasleft();
                totalGasUsed += gasUsed;

                bytes32 proofHash = proofs[i].getHash();
                
                results[i] = VerificationResult({
                    isValid: isValid,
                    timestamp: block.timestamp,
                    verifier: msg.sender,
                    contentHash: proofs[i].contentHash,
                    rootHash: proofs[i].rootHash,
                    securityLevel: proofs[i].securityLevel,
                    proofSystem: proofs[i].proofSystem,
                    gasUsed: gasUsed
                });

                verificationResults[proofHash] = results[i];
                verificationTimestamps[proofHash] = block.timestamp;

                if (isValid) successfulCount++;

                emit ProofVerified(proofHash, msg.sender, isValid, gasUsed, block.timestamp);
            } catch {
                // Mark as invalid if verification fails
                results[i] = VerificationResult({
                    isValid: false,
                    timestamp: block.timestamp,
                    verifier: msg.sender,
                    contentHash: proofs[i].contentHash,
                    rootHash: proofs[i].rootHash,
                    securityLevel: proofs[i].securityLevel,
                    proofSystem: proofs[i].proofSystem,
                    gasUsed: gasStart - gasleft()
                });
            }
        }

        // Update batch statistics
        userStats[msg.sender].totalVerifications += proofs.length;
        userStats[msg.sender].successfulVerifications += successfulCount;
        userStats[msg.sender].totalGasUsed += totalGasUsed;
        userStats[msg.sender].lastVerificationTime = block.timestamp;

        totalVerifications += proofs.length;
        successfulVerifications += successfulCount;

        // Transfer fee to recipient
        if (totalFee > 0) {
            (bool success, ) = feeRecipient.call{value: totalFee}("");
            require(success, "Fee transfer failed");
        }

        // Refund excess payment
        if (msg.value > totalFee) {
            (bool success, ) = msg.sender.call{value: msg.value - totalFee}("");
            require(success, "Refund failed");
        }

        emit BatchVerificationCompleted(msg.sender, proofs.length, successfulCount, totalGasUsed);

        return results;
    }

    /**
     * @notice Get verification result for a proof
     * @param proofHash Hash of the proof
     * @return result The verification result
     */
    function getVerificationResult(bytes32 proofHash)
        external
        view
        returns (VerificationResult memory result)
    {
        return verificationResults[proofHash];
    }

    /**
     * @notice Check if a proof has been verified
     * @param proofHash Hash of the proof
     * @return verified True if the proof has been verified
     */
    function isProofVerified(bytes32 proofHash) external view returns (bool verified) {
        return verificationResults[proofHash].timestamp != 0;
    }

    /**
     * @notice Get user verification statistics
     * @param user Address of the user
     * @return stats The user's verification statistics
     */
    function getUserStats(address user) external view returns (VerificationStats memory stats) {
        return userStats[user];
    }

    /**
     * @notice Get contract statistics
     * @return total Total verifications
     * @return successful Successful verifications
     * @return successRate Success rate as percentage (scaled by 10000)
     */
    function getContractStats()
        external
        view
        returns (uint256 total, uint256 successful, uint256 successRate)
    {
        total = totalVerifications;
        successful = successfulVerifications;
        successRate = total > 0 ? (successful * 10000) / total : 0;
    }

    /**
     * @notice Internal function to verify a proof (used for batch processing)
     * @param proof The proof to verify
     * @return isValid True if the proof is valid
     */
    function _verifyProofInternal(ProofLib.Proof calldata proof)
        external
        view
        returns (bool isValid)
    {
        require(msg.sender == address(this), "Internal function");
        _validateProof(proof);
        return _performVerification(proof);
    }

    /**
     * @notice Validate proof structure and parameters
     * @param proof The proof to validate
     */
    function _validateProof(ProofLib.Proof calldata proof) internal view {
        // Check proof age
        if (block.timestamp - proof.timestamp > maxProofAge) {
            revert ProofTooOld();
        }

        // Check security level
        if (proof.securityLevel < minSecurityLevel) {
            revert InsufficientSecurityLevel();
        }

        // Check proof system support
        if (!supportedProofSystems[proof.proofSystem]) {
            revert UnsupportedProofSystem();
        }

        // Validate proof structure
        if (!proof.isValid()) {
            revert InvalidProof();
        }
    }

    /**
     * @notice Perform cryptographic verification of the proof
     * @param proof The proof to verify
     * @return isValid True if the proof is cryptographically valid
     */
    function _performVerification(ProofLib.Proof calldata proof)
        internal
        pure
        returns (bool isValid)
    {
        // This would implement the actual cryptographic verification
        // For now, we'll use the proof library's verification function
        return proof.verify();
    }

    /**
     * @notice Update user and global statistics
     * @param user Address of the user
     * @param isValid Whether the verification was successful
     * @param gasUsed Amount of gas used
     */
    function _updateStats(address user, bool isValid, uint256 gasUsed) internal {
        userStats[user].totalVerifications++;
        userStats[user].totalGasUsed += gasUsed;
        userStats[user].lastVerificationTime = block.timestamp;

        totalVerifications++;

        if (isValid) {
            userStats[user].successfulVerifications++;
            successfulVerifications++;
        }
    }

    // Admin functions

    /**
     * @notice Set minimum security level
     * @param _minSecurityLevel New minimum security level
     */
    function setMinSecurityLevel(uint256 _minSecurityLevel) external onlyOwner {
        if (_minSecurityLevel < 80 || _minSecurityLevel > 256) {
            revert InvalidSecurityLevel();
        }

        uint256 oldLevel = minSecurityLevel;
        minSecurityLevel = _minSecurityLevel;
        emit SecurityLevelUpdated(oldLevel, _minSecurityLevel);
    }

    /**
     * @notice Set maximum proof age
     * @param _maxProofAge New maximum proof age in seconds
     */
    function setMaxProofAge(uint256 _maxProofAge) external onlyOwner {
        if (_maxProofAge < 1 hours || _maxProofAge > 365 days) {
            revert InvalidMaxAge();
        }

        uint256 oldAge = maxProofAge;
        maxProofAge = _maxProofAge;
        emit MaxProofAgeUpdated(oldAge, _maxProofAge);
    }

    /**
     * @notice Set verification fee
     * @param _verificationFee New verification fee in wei
     */
    function setVerificationFee(uint256 _verificationFee) external onlyOwner {
        uint256 oldFee = verificationFee;
        verificationFee = _verificationFee;
        emit VerificationFeeUpdated(oldFee, _verificationFee);
    }

    /**
     * @notice Set fee recipient
     * @param _feeRecipient New fee recipient address
     */
    function setFeeRecipient(address _feeRecipient) external onlyOwner {
        if (_feeRecipient == address(0)) revert InvalidFeeRecipient();

        address oldRecipient = feeRecipient;
        feeRecipient = _feeRecipient;
        emit FeeRecipientUpdated(oldRecipient, _feeRecipient);
    }

    /**
     * @notice Update supported proof system
     * @param proofSystem Name of the proof system
     * @param supported Whether the proof system is supported
     */
    function setProofSystemSupport(string calldata proofSystem, bool supported)
        external
        onlyOwner
    {
        supportedProofSystems[proofSystem] = supported;
        emit ProofSystemUpdated(proofSystem, supported);
    }

    /**
     * @notice Pause the contract
     */
    function pause() external onlyOwner {
        _pause();
    }

    /**
     * @notice Unpause the contract
     */
    function unpause() external onlyOwner {
        _unpause();
    }

    /**
     * @notice Emergency withdrawal function
     * @param token Address of the token to withdraw (address(0) for ETH)
     * @param amount Amount to withdraw
     */
    function emergencyWithdraw(address token, uint256 amount) external onlyOwner {
        if (token == address(0)) {
            (bool success, ) = owner().call{value: amount}("");
            require(success, "ETH withdrawal failed");
        } else {
            // Token withdrawal would be implemented here
            revert("Token withdrawal not implemented");
        }
    }

    /**
     * @notice Receive function to accept ETH
     */
    receive() external payable {
        // Allow contract to receive ETH
    }
}

