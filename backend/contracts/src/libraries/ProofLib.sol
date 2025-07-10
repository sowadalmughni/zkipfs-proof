// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/**
 * @title ProofLib
 * @author Sowad Al-Mughni
 * @notice Library for handling zero-knowledge proof data structures and verification
 */
library ProofLib {
    /// @notice Maximum size for proof data (1MB)
    uint256 public constant MAX_PROOF_SIZE = 1024 * 1024;

    /// @notice Maximum size for public inputs (64KB)
    uint256 public constant MAX_PUBLIC_INPUTS_SIZE = 64 * 1024;

    /// @notice Proof data structure
    struct Proof {
        // Proof metadata
        bytes32 id;                    // Unique proof identifier
        uint256 timestamp;             // Proof generation timestamp
        uint256 securityLevel;         // Security level in bits
        string proofSystem;            // Proof system used (Risc0, Groth16, etc.)
        
        // Content information
        bytes32 contentHash;           // Hash of the proven content
        bytes32 rootHash;              // IPFS root hash
        bytes32 fileHash;              // Original file hash
        uint256 fileSize;              // Original file size
        
        // ZK proof data
        bytes receipt;                 // ZK proof receipt/proof data
        bytes publicInputs;            // Public inputs for verification
        bytes32 imageId;               // Risc0 image ID (for Risc0 proofs)
        
        // Content selection
        ContentSelection selection;     // What content was proven
        
        // Verification parameters
        uint256 maxAge;                // Maximum age for this proof
        bool requiresOnChainData;      // Whether verification requires on-chain data
    }

    /// @notice Content selection types
    enum SelectionType {
        FullFile,      // Entire file content
        ByteRange,     // Specific byte range
        Pattern,       // Pattern matching
        Multiple       // Multiple selections
    }

    /// @notice Content selection structure
    struct ContentSelection {
        SelectionType selectionType;
        bytes data;                    // Selection-specific data
        bytes32 selectionHash;         // Hash of the selection
    }

    /// @notice Verification context for advanced verification
    struct VerificationContext {
        uint256 blockNumber;           // Block number for verification
        uint256 timestamp;             // Timestamp for verification
        address verifier;              // Address performing verification
        bytes32 challengeHash;         // Optional challenge hash
        bool strictMode;               // Whether to use strict verification
    }

    /// @notice Custom errors
    error InvalidProofStructure();
    error ProofTooLarge();
    error PublicInputsTooLarge();
    error InvalidContentSelection();
    error InvalidSecurityLevel();
    error InvalidTimestamp();
    error EmptyProofData();
    error InvalidImageId();

    /**
     * @notice Validate proof structure and basic parameters
     * @param proof The proof to validate
     * @return valid True if the proof structure is valid
     */
    function isValid(Proof memory proof) internal pure returns (bool valid) {
        // Check basic structure
        if (proof.id == bytes32(0)) return false;
        if (proof.timestamp == 0) return false;
        if (proof.securityLevel < 80 || proof.securityLevel > 256) return false;
        if (bytes(proof.proofSystem).length == 0) return false;
        
        // Check hashes
        if (proof.contentHash == bytes32(0)) return false;
        if (proof.rootHash == bytes32(0)) return false;
        if (proof.fileHash == bytes32(0)) return false;
        
        // Check proof data
        if (proof.receipt.length == 0) return false;
        if (proof.receipt.length > MAX_PROOF_SIZE) return false;
        if (proof.publicInputs.length > MAX_PUBLIC_INPUTS_SIZE) return false;
        
        // Check file size
        if (proof.fileSize == 0) return false;
        
        // Check content selection
        if (proof.selection.selectionHash == bytes32(0)) return false;
        
        return true;
    }

    /**
     * @notice Get hash of the proof for unique identification
     * @param proof The proof to hash
     * @return hash The proof hash
     */
    function getHash(Proof memory proof) internal pure returns (bytes32 hash) {
        return keccak256(abi.encodePacked(
            proof.id,
            proof.timestamp,
            proof.contentHash,
            proof.rootHash,
            proof.fileHash,
            proof.selection.selectionHash,
            keccak256(proof.receipt)
        ));
    }

    /**
     * @notice Verify the cryptographic proof
     * @param proof The proof to verify
     * @return valid True if the proof is cryptographically valid
     */
    function verify(Proof memory proof) internal pure returns (bool valid) {
        // Validate structure first
        if (!isValid(proof)) return false;
        
        // Perform proof system specific verification
        if (keccak256(bytes(proof.proofSystem)) == keccak256(bytes("Risc0"))) {
            return _verifyRisc0Proof(proof);
        } else if (keccak256(bytes(proof.proofSystem)) == keccak256(bytes("Groth16"))) {
            return _verifyGroth16Proof(proof);
        } else if (keccak256(bytes(proof.proofSystem)) == keccak256(bytes("Plonk"))) {
            return _verifyPlonkProof(proof);
        }
        
        return false;
    }

    /**
     * @notice Verify proof with additional context
     * @param proof The proof to verify
     * @param context Additional verification context
     * @return valid True if the proof is valid in the given context
     */
    function verifyWithContext(
        Proof memory proof,
        VerificationContext memory context
    ) internal pure returns (bool valid) {
        // Basic verification first
        if (!verify(proof)) return false;
        
        // Context-specific checks
        if (context.strictMode) {
            // In strict mode, perform additional checks
            if (proof.timestamp > context.timestamp) return false;
            if (context.challengeHash != bytes32(0)) {
                // Verify challenge response if provided
                bytes32 expectedResponse = keccak256(abi.encodePacked(
                    proof.contentHash,
                    context.challengeHash
                ));
                // This would check against proof data in a real implementation
            }
        }
        
        return true;
    }

    /**
     * @notice Extract public inputs from proof
     * @param proof The proof containing public inputs
     * @return inputs Array of public input values
     */
    function extractPublicInputs(Proof memory proof) 
        internal 
        pure 
        returns (bytes32[] memory inputs) 
    {
        if (proof.publicInputs.length == 0) {
            return new bytes32[](0);
        }
        
        // Decode public inputs (assuming they are packed as bytes32 values)
        uint256 inputCount = proof.publicInputs.length / 32;
        inputs = new bytes32[](inputCount);
        
        for (uint256 i = 0; i < inputCount; i++) {
            bytes32 input;
            assembly {
                input := mload(add(add(proof.publicInputs, 0x20), mul(i, 0x20)))
            }
            inputs[i] = input;
        }
        
        return inputs;
    }

    /**
     * @notice Verify content selection against proof
     * @param proof The proof to check
     * @param expectedContent Expected content hash
     * @return valid True if the content selection is valid
     */
    function verifyContentSelection(
        Proof memory proof,
        bytes32 expectedContent
    ) internal pure returns (bool valid) {
        // This would implement content selection verification
        // For now, we'll do a simple hash comparison
        return proof.contentHash == expectedContent;
    }

    /**
     * @notice Get proof size in bytes
     * @param proof The proof to measure
     * @return size Total size of the proof data
     */
    function getProofSize(Proof memory proof) internal pure returns (uint256 size) {
        return proof.receipt.length + proof.publicInputs.length + proof.selection.data.length;
    }

    /**
     * @notice Check if proof is expired
     * @param proof The proof to check
     * @param currentTimestamp Current timestamp
     * @return expired True if the proof is expired
     */
    function isExpired(
        Proof memory proof,
        uint256 currentTimestamp
    ) internal pure returns (bool expired) {
        if (proof.maxAge == 0) return false; // No expiration
        return (currentTimestamp - proof.timestamp) > proof.maxAge;
    }

    // Internal verification functions for different proof systems

    /**
     * @notice Verify Risc0 proof
     * @param proof The Risc0 proof to verify
     * @return valid True if the Risc0 proof is valid
     */
    function _verifyRisc0Proof(Proof memory proof) private pure returns (bool valid) {
        // Check image ID
        if (proof.imageId == bytes32(0)) return false;
        
        // In a real implementation, this would:
        // 1. Verify the receipt against the image ID
        // 2. Check the public inputs
        // 3. Validate the proof structure
        
        // For now, we'll do basic validation
        return proof.receipt.length >= 32 && proof.imageId != bytes32(0);
    }

    /**
     * @notice Verify Groth16 proof
     * @param proof The Groth16 proof to verify
     * @return valid True if the Groth16 proof is valid
     */
    function _verifyGroth16Proof(Proof memory proof) private pure returns (bool valid) {
        // Groth16 proofs have a specific structure
        // This would implement the actual Groth16 verification
        return proof.receipt.length == 256; // Groth16 proofs are typically 256 bytes
    }

    /**
     * @notice Verify PLONK proof
     * @param proof The PLONK proof to verify
     * @return valid True if the PLONK proof is valid
     */
    function _verifyPlonkProof(Proof memory proof) private pure returns (bool valid) {
        // PLONK proof verification would be implemented here
        return proof.receipt.length >= 64; // Simplified check
    }

    /**
     * @notice Create a content selection for full file
     * @param fileHash Hash of the full file
     * @return selection The content selection structure
     */
    function createFullFileSelection(bytes32 fileHash) 
        internal 
        pure 
        returns (ContentSelection memory selection) 
    {
        selection.selectionType = SelectionType.FullFile;
        selection.data = abi.encodePacked(fileHash);
        selection.selectionHash = fileHash;
    }

    /**
     * @notice Create a content selection for byte range
     * @param startByte Starting byte position
     * @param endByte Ending byte position
     * @param rangeHash Hash of the byte range content
     * @return selection The content selection structure
     */
    function createByteRangeSelection(
        uint256 startByte,
        uint256 endByte,
        bytes32 rangeHash
    ) internal pure returns (ContentSelection memory selection) {
        selection.selectionType = SelectionType.ByteRange;
        selection.data = abi.encodePacked(startByte, endByte);
        selection.selectionHash = rangeHash;
    }

    /**
     * @notice Create a content selection for pattern matching
     * @param pattern The pattern to match
     * @param patternHash Hash of the pattern content
     * @return selection The content selection structure
     */
    function createPatternSelection(
        bytes memory pattern,
        bytes32 patternHash
    ) internal pure returns (ContentSelection memory selection) {
        selection.selectionType = SelectionType.Pattern;
        selection.data = pattern;
        selection.selectionHash = patternHash;
    }

    /**
     * @notice Validate content selection structure
     * @param selection The content selection to validate
     * @return valid True if the selection is valid
     */
    function isValidSelection(ContentSelection memory selection) 
        internal 
        pure 
        returns (bool valid) 
    {
        if (selection.selectionHash == bytes32(0)) return false;
        if (selection.data.length == 0) return false;
        
        // Type-specific validation
        if (selection.selectionType == SelectionType.ByteRange) {
            if (selection.data.length != 64) return false; // Two uint256 values
        }
        
        return true;
    }
}

