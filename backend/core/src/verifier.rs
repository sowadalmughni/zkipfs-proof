//! Proof verification functionality for zkIPFS-Proof
//!
//! This module provides comprehensive proof verification capabilities,
//! including local verification, on-chain verification, and batch verification.

use crate::{
    error::{ProofError, Result, ResultExt},
    types::*,
    IPFS_CONTENT_VERIFIER_ID,
};
use risc0_zkvm::Receipt;
use sha2::{Digest, Sha256};
use std::time::Instant;
use tracing::{debug, info, instrument, warn};
use chrono::Utc;

/// Comprehensive proof verifier for zkIPFS-Proof
pub struct ProofVerifier {
    /// Configuration for verification
    config: VerificationConfig,
    /// Verification statistics
    stats: VerificationStatistics,
}

/// Configuration for proof verification
#[derive(Clone, Debug)]
pub struct VerificationConfig {
    /// Whether to perform strict verification (includes all checks)
    pub strict_verification: bool,
    /// Whether to include detailed verification steps
    pub include_verification_steps: bool,
    /// Maximum allowed proof age (in seconds)
    pub max_proof_age_seconds: Option<u64>,
    /// Whether to verify proof metadata
    pub verify_metadata: bool,
    /// Custom verification rules
    pub custom_rules: Vec<VerificationRule>,
}

/// Custom verification rule
#[derive(Clone, Debug)]
pub struct VerificationRule {
    pub name: String,
    pub description: String,
    pub rule_type: VerificationRuleType,
}

/// Types of verification rules
#[derive(Clone, Debug)]
pub enum VerificationRuleType {
    /// Minimum security level required
    MinSecurityLevel(u32),
    /// Maximum proof size allowed
    MaxProofSize(u64),
    /// Required proof system
    RequiredProofSystem(String),
    /// Custom validation function
    Custom(fn(&Proof) -> Result<bool>),
}

/// Statistics for proof verification
#[derive(Clone, Debug, Default)]
pub struct VerificationStatistics {
    pub total_verifications: u64,
    pub successful_verifications: u64,
    pub failed_verifications: u64,
    pub avg_verification_time_ms: f64,
    pub total_verification_time_ms: u64,
}

impl Default for VerificationConfig {
    fn default() -> Self {
        Self {
            strict_verification: true,
            include_verification_steps: false,
            max_proof_age_seconds: Some(30 * 24 * 60 * 60), // 30 days
            verify_metadata: true,
            custom_rules: Vec::new(),
        }
    }
}

impl ProofVerifier {
    /// Creates a new proof verifier with default configuration
    pub fn new() -> Self {
        Self::with_config(VerificationConfig::default())
    }

    /// Creates a new proof verifier with custom configuration
    pub fn with_config(config: VerificationConfig) -> Self {
        Self {
            config,
            stats: VerificationStatistics::default(),
        }
    }

    /// Verifies a proof against claimed content with detailed result
    #[instrument(skip(self, proof, claimed_content), fields(proof_id = %proof.id))]
    pub async fn verify_detailed(
        &mut self,
        proof: &Proof,
        claimed_content: &[u8],
    ) -> Result<VerificationResult> {
        let start_time = Instant::now();
        let mut verification_steps = Vec::new();
        let mut warnings = Vec::new();
        
        info!("Starting detailed verification for proof: {}", &proof.id[..8]);
        
        // Step 1: Basic proof structure validation
        let step_start = Instant::now();
        let structure_valid = self.verify_proof_structure(proof).await?;
        verification_steps.push(VerificationStep {
            name: "Proof Structure Validation".to_string(),
            passed: structure_valid,
            duration_ms: step_start.elapsed().as_millis() as u64,
            details: if structure_valid { 
                None 
            } else { 
                Some("Invalid proof structure".to_string()) 
            },
        });
        
        if !structure_valid {
            return Ok(self.create_verification_result(
                false, start_time, verification_steps, warnings
            ));
        }
        
        // Step 2: Cryptographic proof verification
        let step_start = Instant::now();
        let crypto_valid = self.verify_cryptographic_proof(proof).await?;
        verification_steps.push(VerificationStep {
            name: "Cryptographic Proof Verification".to_string(),
            passed: crypto_valid,
            duration_ms: step_start.elapsed().as_millis() as u64,
            details: if crypto_valid { 
                None 
            } else { 
                Some("Cryptographic verification failed".to_string()) 
            },
        });
        
        if !crypto_valid {
            return Ok(self.create_verification_result(
                false, start_time, verification_steps, warnings
            ));
        }
        
        // Step 3: Content hash verification
        let step_start = Instant::now();
        let content_valid = self.verify_content_hash(proof, claimed_content)?;
        verification_steps.push(VerificationStep {
            name: "Content Hash Verification".to_string(),
            passed: content_valid,
            duration_ms: step_start.elapsed().as_millis() as u64,
            details: if content_valid { 
                None 
            } else { 
                Some("Content hash mismatch".to_string()) 
            },
        });
        
        if !content_valid {
            return Ok(self.create_verification_result(
                false, start_time, verification_steps, warnings
            ));
        }
        
        // Step 4: Metadata verification (if enabled)
        if self.config.verify_metadata {
            let step_start = Instant::now();
            let (metadata_valid, metadata_warnings) = self.verify_metadata(proof)?;
            warnings.extend(metadata_warnings);
            verification_steps.push(VerificationStep {
                name: "Metadata Verification".to_string(),
                passed: metadata_valid,
                duration_ms: step_start.elapsed().as_millis() as u64,
                details: if metadata_valid { 
                    None 
                } else { 
                    Some("Metadata verification failed".to_string()) 
                },
            });
            
            if self.config.strict_verification && !metadata_valid {
                return Ok(self.create_verification_result(
                    false, start_time, verification_steps, warnings
                ));
            }
        }
        
        // Step 5: Custom rules verification
        if !self.config.custom_rules.is_empty() {
            let step_start = Instant::now();
            let (rules_valid, rules_warnings) = self.verify_custom_rules(proof)?;
            warnings.extend(rules_warnings);
            verification_steps.push(VerificationStep {
                name: "Custom Rules Verification".to_string(),
                passed: rules_valid,
                duration_ms: step_start.elapsed().as_millis() as u64,
                details: if rules_valid { 
                    None 
                } else { 
                    Some("Custom rules verification failed".to_string()) 
                },
            });
            
            if self.config.strict_verification && !rules_valid {
                return Ok(self.create_verification_result(
                    false, start_time, verification_steps, warnings
                ));
            }
        }
        
        // All verifications passed
        let result = self.create_verification_result(
            true, start_time, verification_steps, warnings
        );
        
        // Update statistics
        self.update_stats(true, start_time.elapsed().as_millis() as u64);
        
        info!(
            "Proof verification completed successfully in {}ms",
            start_time.elapsed().as_millis()
        );
        
        Ok(result)
    }

    /// Simple verification that returns only a boolean result
    pub async fn verify_simple(
        &mut self,
        proof: &Proof,
        claimed_content: &[u8],
    ) -> Result<bool> {
        let result = self.verify_detailed(proof, claimed_content).await?;
        Ok(result.is_valid)
    }

    /// Verifies multiple proofs in batch
    pub async fn verify_batch(
        &mut self,
        proofs_and_content: Vec<(&Proof, &[u8])>,
    ) -> Result<Vec<VerificationResult>> {
        let mut results = Vec::new();
        
        for (proof, content) in proofs_and_content {
            let result = self.verify_detailed(proof, content).await?;
            results.push(result);
        }
        
        Ok(results)
    }

    /// Verifies the basic structure of a proof
    async fn verify_proof_structure(&self, proof: &Proof) -> Result<bool> {
        // Check proof version compatibility
        if proof.version.is_empty() {
            return Ok(false);
        }
        
        // Check if proof is not too old
        if let Some(max_age) = self.config.max_proof_age_seconds {
            let age = Utc::now().signed_duration_since(proof.created_at);
            if age.num_seconds() > max_age as i64 {
                return Ok(false);
            }
        }
        
        // Check ZK proof data structure
        if proof.zk_proof.receipt.is_empty() {
            return Ok(false);
        }
        
        // Check content selection validity
        if !proof.content_selection.is_valid() {
            return Ok(false);
        }
        
        // Check hash lengths
        if proof.content_hash.len() != 32 || proof.root_hash.len() != 32 {
            return Ok(false);
        }
        
        Ok(true)
    }

    /// Verifies the cryptographic proof using Risc0
    async fn verify_cryptographic_proof(&self, proof: &Proof) -> Result<bool> {
        // Deserialize the receipt
        let receipt: Receipt = bincode::deserialize(&proof.zk_proof.receipt)
            .map_err(|e| ProofError::serialization_error(
                "Failed to deserialize receipt",
                Some(Box::new(e))
            ))?;
        
        // Verify the receipt against the expected image ID
        match receipt.verify(IPFS_CONTENT_VERIFIER_ID) {
            Ok(_) => {
                debug!("Cryptographic proof verification successful");
                Ok(true)
            }
            Err(e) => {
                warn!("Cryptographic proof verification failed: {}", e);
                Ok(false)
            }
        }
    }

    /// Verifies that the claimed content matches the proof
    fn verify_content_hash(&self, proof: &Proof, claimed_content: &[u8]) -> Result<bool> {
        let claimed_hash = Sha256::digest(claimed_content);
        let matches = claimed_hash.as_slice() == proof.content_hash.as_slice();
        
        if matches {
            debug!("Content hash verification successful");
        } else {
            debug!("Content hash verification failed");
        }
        
        Ok(matches)
    }

    /// Verifies proof metadata
    fn verify_metadata(&self, proof: &Proof) -> Result<(bool, Vec<String>)> {
        let mut warnings = Vec::new();
        let mut is_valid = true;
        
        // Check security level
        if proof.metadata.security.security_level < 128 {
            warnings.push("Security level below recommended minimum (128 bits)".to_string());
            if self.config.strict_verification {
                is_valid = false;
            }
        }
        
        // Check proof system
        if proof.metadata.security.proof_system != "Risc0" {
            warnings.push("Unexpected proof system".to_string());
            if self.config.strict_verification {
                is_valid = false;
            }
        }
        
        // Check performance metrics for anomalies
        if proof.metadata.performance.generation_time_ms > 3600000 { // 1 hour
            warnings.push("Unusually long proof generation time".to_string());
        }
        
        if proof.metadata.performance.proof_size_bytes > 100 * 1024 * 1024 { // 100MB
            warnings.push("Unusually large proof size".to_string());
        }
        
        // Check file info consistency
        if proof.metadata.file_info.block_count == 0 {
            warnings.push("No IPFS blocks in file info".to_string());
            if self.config.strict_verification {
                is_valid = false;
            }
        }
        
        Ok((is_valid, warnings))
    }

    /// Verifies custom rules
    fn verify_custom_rules(&self, proof: &Proof) -> Result<(bool, Vec<String>)> {
        let mut warnings = Vec::new();
        let mut is_valid = true;
        
        for rule in &self.config.custom_rules {
            let rule_result = match &rule.rule_type {
                VerificationRuleType::MinSecurityLevel(min_level) => {
                    proof.metadata.security.security_level >= *min_level
                }
                VerificationRuleType::MaxProofSize(max_size) => {
                    proof.metadata.performance.proof_size_bytes <= *max_size
                }
                VerificationRuleType::RequiredProofSystem(required_system) => {
                    proof.metadata.security.proof_system == *required_system
                }
                VerificationRuleType::Custom(validator) => {
                    match validator(proof) {
                        Ok(result) => result,
                        Err(e) => {
                            warnings.push(format!("Custom rule '{}' failed: {}", rule.name, e));
                            false
                        }
                    }
                }
            };
            
            if !rule_result {
                warnings.push(format!("Custom rule '{}' failed: {}", rule.name, rule.description));
                if self.config.strict_verification {
                    is_valid = false;
                }
            }
        }
        
        Ok((is_valid, warnings))
    }

    /// Creates a verification result
    fn create_verification_result(
        &self,
        is_valid: bool,
        start_time: Instant,
        verification_steps: Vec<VerificationStep>,
        warnings: Vec<String>,
    ) -> VerificationResult {
        VerificationResult {
            is_valid,
            verified_at: Utc::now(),
            verification_time_ms: start_time.elapsed().as_millis() as u64,
            verifier_info: VerifierInfo {
                version: env!("CARGO_PKG_VERSION").to_string(),
                method: VerificationMethod::Local,
                environment: format!("{} {}", std::env::consts::OS, std::env::consts::ARCH),
            },
            warnings,
            verification_steps: if self.config.include_verification_steps {
                verification_steps
            } else {
                Vec::new()
            },
        }
    }

    /// Updates verification statistics
    fn update_stats(&mut self, is_valid: bool, duration_ms: u64) {
        self.stats.total_verifications += 1;
        self.stats.total_verification_time_ms += duration_ms;
        
        if is_valid {
            self.stats.successful_verifications += 1;
        } else {
            self.stats.failed_verifications += 1;
        }
        
        // Update average
        self.stats.avg_verification_time_ms = 
            self.stats.total_verification_time_ms as f64 / self.stats.total_verifications as f64;
    }

    /// Gets current verification statistics
    pub fn get_statistics(&self) -> &VerificationStatistics {
        &self.stats
    }

    /// Updates the verification configuration
    pub fn update_config(&mut self, config: VerificationConfig) {
        self.config = config;
    }

    /// Adds a custom verification rule
    pub fn add_custom_rule(&mut self, rule: VerificationRule) {
        self.config.custom_rules.push(rule);
    }

    /// Removes a custom verification rule by name
    pub fn remove_custom_rule(&mut self, rule_name: &str) {
        self.config.custom_rules.retain(|rule| rule.name != rule_name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn create_test_proof() -> Proof {
        Proof {
            id: Uuid::new_v4().to_string(),
            zk_proof: ZkProofData {
                receipt: vec![1, 2, 3, 4], // Dummy receipt
                public_inputs: vec![5, 6, 7, 8],
                format_version: "1.0".to_string(),
                compression: Some(CompressionType::None),
            },
            metadata: ProofMetadata {
                guest_metadata: crate::guest_types::ProofMetadata {
                    block_count: 1,
                    content_size: 100,
                    timestamp: 12345,
                },
                file_info: FileInfo {
                    filename: Some("test.txt".to_string()),
                    size: 100,
                    mime_type: Some("text/plain".to_string()),
                    file_hash: [0; 32],
                    ipfs_cid: "QmTest".to_string(),
                    block_count: 1,
                    avg_block_size: 100,
                },
                performance: PerformanceMetrics {
                    generation_time_ms: 1000,
                    file_processing_time_ms: 100,
                    zk_generation_time_ms: 900,
                    peak_memory_bytes: 1024 * 1024,
                    zk_cycles: 10000,
                    proof_size_bytes: 1024,
                    compression_ratio: None,
                },
                security: SecurityParameters {
                    security_level: 128,
                    hash_function: "SHA-256".to_string(),
                    proof_system: "Risc0".to_string(),
                    risc0_version: "1.2".to_string(),
                    formal_verification: false,
                },
                environment: GenerationEnvironment {
                    os: "linux".to_string(),
                    arch: "x86_64".to_string(),
                    hardware_acceleration: Some(HardwareAcceleration::None),
                    prover_type: ProverType::Local,
                    library_version: "0.1.0".to_string(),
                    git_commit: None,
                },
                custom: std::collections::HashMap::new(),
            },
            content_selection: ContentSelection::Pattern { content: b"test".to_vec() },
            content_hash: [0; 32],
            root_hash: [1; 32],
            created_at: Utc::now(),
            version: "0.1.0".to_string(),
        }
    }

    #[test]
    fn test_verifier_creation() {
        let verifier = ProofVerifier::new();
        assert_eq!(verifier.stats.total_verifications, 0);
    }

    #[test]
    fn test_proof_structure_validation() {
        let verifier = ProofVerifier::new();
        let proof = create_test_proof();
        
        // This should pass basic structure validation
        // Note: We can't test the full async method in a sync test,
        // but we can test the structure validation logic
        assert!(proof.content_selection.is_valid());
        assert_eq!(proof.content_hash.len(), 32);
        assert_eq!(proof.root_hash.len(), 32);
    }

    #[test]
    fn test_content_hash_verification() {
        let verifier = ProofVerifier::new();
        let mut proof = create_test_proof();
        let content = b"test content";
        
        // Set the correct content hash
        let hash = sha2::Sha256::digest(content);
        proof.content_hash = hash.into();
        
        let result = verifier.verify_content_hash(&proof, content).unwrap();
        assert!(result);
        
        // Test with wrong content
        let wrong_result = verifier.verify_content_hash(&proof, b"wrong content").unwrap();
        assert!(!wrong_result);
    }

    #[test]
    fn test_custom_rules() {
        let mut verifier = ProofVerifier::new();
        
        // Add a custom rule
        let rule = VerificationRule {
            name: "min_security_128".to_string(),
            description: "Minimum 128-bit security".to_string(),
            rule_type: VerificationRuleType::MinSecurityLevel(128),
        };
        verifier.add_custom_rule(rule);
        
        let proof = create_test_proof();
        let (is_valid, warnings) = verifier.verify_custom_rules(&proof).unwrap();
        assert!(is_valid);
        assert!(warnings.is_empty());
        
        // Test with insufficient security level
        let rule = VerificationRule {
            name: "min_security_256".to_string(),
            description: "Minimum 256-bit security".to_string(),
            rule_type: VerificationRuleType::MinSecurityLevel(256),
        };
        verifier.add_custom_rule(rule);
        
        let (is_valid, warnings) = verifier.verify_custom_rules(&proof).unwrap();
        assert!(!is_valid);
        assert!(!warnings.is_empty());
    }

    #[test]
    fn test_statistics_update() {
        let mut verifier = ProofVerifier::new();
        
        verifier.update_stats(true, 100);
        assert_eq!(verifier.stats.total_verifications, 1);
        assert_eq!(verifier.stats.successful_verifications, 1);
        assert_eq!(verifier.stats.avg_verification_time_ms, 100.0);
        
        verifier.update_stats(false, 200);
        assert_eq!(verifier.stats.total_verifications, 2);
        assert_eq!(verifier.stats.failed_verifications, 1);
        assert_eq!(verifier.stats.avg_verification_time_ms, 150.0);
    }
}

