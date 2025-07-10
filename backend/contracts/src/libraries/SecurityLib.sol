// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/**
 * @title SecurityLib
 * @author Sowad Al-Mughni
 * @notice Library for handling security parameters and validation
 */
library SecurityLib {
    /// @notice Minimum allowed security level (80 bits)
    uint256 public constant MIN_SECURITY_LEVEL = 80;

    /// @notice Maximum allowed security level (256 bits)
    uint256 public constant MAX_SECURITY_LEVEL = 256;

    /// @notice Recommended minimum security level (128 bits)
    uint256 public constant RECOMMENDED_MIN_SECURITY_LEVEL = 128;

    /// @notice Maximum proof age (1 year)
    uint256 public constant MAX_PROOF_AGE = 365 days;

    /// @notice Security parameters structure
    struct SecurityParams {
        uint256 securityLevel;         // Security level in bits
        uint256 maxProofAge;           // Maximum age for proofs
        bool requireFreshness;         // Whether to require fresh proofs
        bool allowWeakSecurity;        // Whether to allow security levels below 128 bits
        uint256 challengeWindow;       // Time window for challenge-response
        bytes32 trustedSetupHash;      // Hash of trusted setup (for systems that require it)
    }

    /// @notice Security validation result
    struct ValidationResult {
        bool isValid;
        string[] warnings;
        string[] errors;
        uint256 riskScore;             // Risk score from 0-100
    }

    /// @notice Custom errors
    error InvalidSecurityLevel();
    error SecurityLevelTooLow();
    error ProofTooOld();
    error InvalidTrustedSetup();
    error ChallengeWindowExpired();

    /**
     * @notice Validate security parameters
     * @param params The security parameters to validate
     * @return result Validation result with warnings and errors
     */
    function validateSecurityParams(SecurityParams memory params)
        internal
        pure
        returns (ValidationResult memory result)
    {
        result.warnings = new string[](0);
        result.errors = new string[](0);
        result.riskScore = 0;
        result.isValid = true;

        // Check security level bounds
        if (params.securityLevel < MIN_SECURITY_LEVEL || params.securityLevel > MAX_SECURITY_LEVEL) {
            result.isValid = false;
            result.errors = _addError(result.errors, "Security level out of bounds");
            result.riskScore += 50;
        }

        // Check if security level is below recommended minimum
        if (params.securityLevel < RECOMMENDED_MIN_SECURITY_LEVEL && !params.allowWeakSecurity) {
            result.isValid = false;
            result.errors = _addError(result.errors, "Security level below recommended minimum");
            result.riskScore += 30;
        } else if (params.securityLevel < RECOMMENDED_MIN_SECURITY_LEVEL) {
            result.warnings = _addWarning(result.warnings, "Security level below recommended minimum");
            result.riskScore += 20;
        }

        // Check proof age limits
        if (params.maxProofAge > MAX_PROOF_AGE) {
            result.warnings = _addWarning(result.warnings, "Proof age limit very high");
            result.riskScore += 10;
        }

        // Check challenge window
        if (params.challengeWindow > 0 && params.challengeWindow < 1 hours) {
            result.warnings = _addWarning(result.warnings, "Challenge window very short");
            result.riskScore += 5;
        }

        return result;
    }

    /**
     * @notice Check if a proof meets security requirements
     * @param proofSecurityLevel Security level of the proof
     * @param proofTimestamp Timestamp when proof was created
     * @param params Security parameters to check against
     * @return meets True if the proof meets security requirements
     */
    function meetsSecurityRequirements(
        uint256 proofSecurityLevel,
        uint256 proofTimestamp,
        SecurityParams memory params
    ) internal view returns (bool meets) {
        // Check security level
        if (proofSecurityLevel < params.securityLevel) {
            return false;
        }

        // Check proof age
        if (params.requireFreshness && params.maxProofAge > 0) {
            if (block.timestamp - proofTimestamp > params.maxProofAge) {
                return false;
            }
        }

        return true;
    }

    /**
     * @notice Calculate risk score for a proof
     * @param proofSecurityLevel Security level of the proof
     * @param proofAge Age of the proof in seconds
     * @param params Security parameters
     * @return riskScore Risk score from 0-100 (lower is better)
     */
    function calculateRiskScore(
        uint256 proofSecurityLevel,
        uint256 proofAge,
        SecurityParams memory params
    ) internal pure returns (uint256 riskScore) {
        riskScore = 0;

        // Security level risk
        if (proofSecurityLevel < 128) {
            riskScore += 40;
        } else if (proofSecurityLevel < 192) {
            riskScore += 20;
        } else if (proofSecurityLevel < 256) {
            riskScore += 10;
        }

        // Age risk
        if (proofAge > 30 days) {
            riskScore += 30;
        } else if (proofAge > 7 days) {
            riskScore += 15;
        } else if (proofAge > 1 days) {
            riskScore += 5;
        }

        // Cap at 100
        if (riskScore > 100) {
            riskScore = 100;
        }

        return riskScore;
    }

    /**
     * @notice Get security level category
     * @param securityLevel Security level in bits
     * @return category Security category string
     */
    function getSecurityCategory(uint256 securityLevel)
        internal
        pure
        returns (string memory category)
    {
        if (securityLevel >= 256) {
            return "Quantum-Resistant";
        } else if (securityLevel >= 192) {
            return "Very High";
        } else if (securityLevel >= 128) {
            return "High";
        } else if (securityLevel >= 112) {
            return "Medium";
        } else if (securityLevel >= 80) {
            return "Low";
        } else {
            return "Insufficient";
        }
    }

    /**
     * @notice Check if security level is quantum-resistant
     * @param securityLevel Security level in bits
     * @return resistant True if quantum-resistant
     */
    function isQuantumResistant(uint256 securityLevel) internal pure returns (bool resistant) {
        return securityLevel >= 256;
    }

    /**
     * @notice Validate trusted setup hash
     * @param trustedSetupHash Hash to validate
     * @param expectedHash Expected hash value
     * @return valid True if the trusted setup is valid
     */
    function validateTrustedSetup(
        bytes32 trustedSetupHash,
        bytes32 expectedHash
    ) internal pure returns (bool valid) {
        if (expectedHash == bytes32(0)) {
            // No trusted setup required
            return true;
        }
        return trustedSetupHash == expectedHash;
    }

    /**
     * @notice Check if challenge window is still valid
     * @param challengeTimestamp When the challenge was issued
     * @param windowDuration Duration of the challenge window
     * @return valid True if still within the challenge window
     */
    function isChallengeWindowValid(
        uint256 challengeTimestamp,
        uint256 windowDuration
    ) internal view returns (bool valid) {
        if (windowDuration == 0) return true; // No challenge window
        return block.timestamp <= challengeTimestamp + windowDuration;
    }

    /**
     * @notice Generate a challenge hash for proof verification
     * @param verifier Address of the verifier
     * @param proofHash Hash of the proof being verified
     * @param nonce Random nonce
     * @return challengeHash The generated challenge hash
     */
    function generateChallenge(
        address verifier,
        bytes32 proofHash,
        uint256 nonce
    ) internal view returns (bytes32 challengeHash) {
        return keccak256(abi.encodePacked(
            verifier,
            proofHash,
            nonce,
            block.timestamp,
            block.difficulty
        ));
    }

    /**
     * @notice Verify challenge response
     * @param challenge Original challenge hash
     * @param response Response to the challenge
     * @param expectedResponse Expected response hash
     * @return valid True if the response is valid
     */
    function verifyChallengeResponse(
        bytes32 challenge,
        bytes32 response,
        bytes32 expectedResponse
    ) internal pure returns (bool valid) {
        bytes32 computedResponse = keccak256(abi.encodePacked(challenge, response));
        return computedResponse == expectedResponse;
    }

    /**
     * @notice Create default security parameters
     * @return params Default security parameters
     */
    function createDefaultParams() internal pure returns (SecurityParams memory params) {
        params.securityLevel = RECOMMENDED_MIN_SECURITY_LEVEL;
        params.maxProofAge = 30 days;
        params.requireFreshness = true;
        params.allowWeakSecurity = false;
        params.challengeWindow = 24 hours;
        params.trustedSetupHash = bytes32(0); // No trusted setup required
    }

    /**
     * @notice Create strict security parameters
     * @return params Strict security parameters
     */
    function createStrictParams() internal pure returns (SecurityParams memory params) {
        params.securityLevel = 192; // Very high security
        params.maxProofAge = 7 days; // Short proof lifetime
        params.requireFreshness = true;
        params.allowWeakSecurity = false;
        params.challengeWindow = 1 hours; // Short challenge window
        params.trustedSetupHash = bytes32(0);
    }

    /**
     * @notice Create permissive security parameters
     * @return params Permissive security parameters
     */
    function createPermissiveParams() internal pure returns (SecurityParams memory params) {
        params.securityLevel = MIN_SECURITY_LEVEL; // Minimum allowed
        params.maxProofAge = MAX_PROOF_AGE; // Maximum allowed age
        params.requireFreshness = false;
        params.allowWeakSecurity = true;
        params.challengeWindow = 0; // No challenge window
        params.trustedSetupHash = bytes32(0);
    }

    // Internal helper functions

    /**
     * @notice Add an error to the errors array
     * @param errors Current errors array
     * @param error Error message to add
     * @return newErrors Updated errors array
     */
    function _addError(string[] memory errors, string memory error)
        private
        pure
        returns (string[] memory newErrors)
    {
        newErrors = new string[](errors.length + 1);
        for (uint256 i = 0; i < errors.length; i++) {
            newErrors[i] = errors[i];
        }
        newErrors[errors.length] = error;
    }

    /**
     * @notice Add a warning to the warnings array
     * @param warnings Current warnings array
     * @param warning Warning message to add
     * @return newWarnings Updated warnings array
     */
    function _addWarning(string[] memory warnings, string memory warning)
        private
        pure
        returns (string[] memory newWarnings)
    {
        newWarnings = new string[](warnings.length + 1);
        for (uint256 i = 0; i < warnings.length; i++) {
            newWarnings[i] = warnings[i];
        }
        newWarnings[warnings.length] = warning;
    }
}

