//! Proof generation functionality for zkIPFS-Proof
//!
//! This module provides the main `ProofGenerator` struct and associated functionality
//! for generating zero-knowledge proofs of IPFS content.

use crate::{
    error::{ProofError, Result, ResultExt},
    types::*,
    ipfs::IpfsProcessor,
    IPFS_CONTENT_VERIFIER_ELF,
    IPFS_CONTENT_VERIFIER_ID,
};
use risc0_zkvm::{default_prover, ExecutorEnv, ProverOpts, Receipt};
use sha2::{Digest, Sha256};
use std::path::Path;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tracing::{debug, info, warn, instrument};
use uuid::Uuid;
use chrono::Utc;

/// Main proof generator for zkIPFS-Proof
pub struct ProofGenerator {
    /// Configuration for proof generation
    config: ProofConfig,
    /// IPFS processor for handling file operations
    ipfs_processor: IpfsProcessor,
    /// Performance statistics
    stats: ProofStatistics,
}

impl ProofGenerator {
    /// Creates a new proof generator with default configuration
    pub async fn new() -> Result<Self> {
        Self::with_config(ProofConfig::default()).await
    }

    /// Creates a new proof generator with custom configuration
    pub async fn with_config(config: ProofConfig) -> Result<Self> {
        let ipfs_processor = IpfsProcessor::new().await
            .context("Failed to initialize IPFS processor")?;

        Ok(Self {
            config,
            ipfs_processor,
            stats: ProofStatistics {
                total_proofs_generated: 0,
                total_proofs_verified: 0,
                avg_generation_time_ms: 0.0,
                avg_verification_time_ms: 0.0,
                generation_success_rate: 1.0,
                verification_success_rate: 1.0,
                total_data_processed_bytes: 0,
                common_file_types: std::collections::HashMap::new(),
            },
        })
    }

    /// Generates a zero-knowledge proof for the specified content selection
    #[instrument(skip(self, file_path), fields(file = %file_path.display()))]
    pub async fn generate_proof(
        &mut self,
        file_path: &Path,
        content_selection: ContentSelection,
    ) -> Result<Proof> {
        let start_time = Instant::now();
        
        info!("Starting proof generation for file: {}", file_path.display());
        
        // Validate inputs
        self.validate_inputs(file_path, &content_selection)?;
        
        // Process the file and extract IPFS blocks
        let file_processing_start = Instant::now();
        let (blocks, file_info) = self.ipfs_processor
            .process_file(file_path)
            .await
            .context("Failed to process file into IPFS blocks")?;
        let file_processing_time = file_processing_start.elapsed();
        
        debug!("Processed file into {} IPFS blocks", blocks.len());
        
        // Extract and hash the target content
        let content_hash = self.extract_content_hash(&blocks, &content_selection)?;
        
        // Prepare input for the ZK circuit
        let proof_input = ProofInput {
            blocks: blocks.clone(),
            content_selection: content_selection.clone(),
            expected_content_hash: content_hash,
        };
        
        // Generate the ZK proof
        let zk_generation_start = Instant::now();
        let (receipt, zk_cycles) = self.generate_zk_proof(proof_input).await?;
        let zk_generation_time = zk_generation_start.elapsed();
        
        // Extract proof output from receipt
        let proof_output: ProofOutput = receipt.journal.decode()
            .map_err(|e| ProofError::serialization_error(
                "Failed to decode proof output from receipt",
                Some(Box::new(e))
            ))?;
        
        // Create proof metadata
        let total_time = start_time.elapsed();
        let metadata = self.create_proof_metadata(
            proof_output.metadata,
            file_info,
            total_time,
            file_processing_time,
            zk_generation_time,
            zk_cycles,
            &receipt,
        )?;
        
        // Create the final proof
        let proof = Proof {
            id: Uuid::new_v4().to_string(),
            zk_proof: ZkProofData {
                receipt: bincode::serialize(&receipt)
                    .map_err(|e| ProofError::serialization_error(
                        "Failed to serialize receipt",
                        Some(Box::new(e))
                    ))?,
                public_inputs: bincode::serialize(&content_hash)
                    .map_err(|e| ProofError::serialization_error(
                        "Failed to serialize public inputs",
                        Some(Box::new(e))
                    ))?,
                format_version: "1.0".to_string(),
                compression: Some(self.config.compression.clone()),
            },
            metadata,
            content_selection,
            content_hash,
            root_hash: proof_output.root_hash,
            created_at: Utc::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        };
        
        // Update statistics
        self.update_generation_stats(&proof, total_time);
        
        info!(
            "Proof generation completed in {}ms (proof_id: {})",
            total_time.as_millis(),
            &proof.id[..8]
        );
        
        Ok(proof)
    }

    /// Verifies a zero-knowledge proof against the claimed content
    #[instrument(skip(self, proof, claimed_content))]
    pub async fn verify_proof(
        &mut self,
        proof: &Proof,
        claimed_content: &[u8],
    ) -> Result<bool> {
        let start_time = Instant::now();
        
        info!("Starting proof verification for proof: {}", &proof.id[..8]);
        
        // Deserialize the receipt
        let receipt: Receipt = bincode::deserialize(&proof.zk_proof.receipt)
            .map_err(|e| ProofError::serialization_error(
                "Failed to deserialize receipt",
                Some(Box::new(e))
            ))?;
        
        // Verify the receipt against the expected image ID
        let verification_result = receipt.verify(IPFS_CONTENT_VERIFIER_ID)
            .map_err(|e| ProofError::verification_error(
                format!("Receipt verification failed: {}", e)
            ))?;
        
        // Extract proof output from the receipt
        let proof_output: ProofOutput = receipt.journal.decode()
            .map_err(|e| ProofError::serialization_error(
                "Failed to decode proof output",
                Some(Box::new(e))
            ))?;
        
        // Verify that the claimed content hash matches the proof
        let claimed_hash = Sha256::digest(claimed_content);
        let is_valid = claimed_hash.as_slice() == proof.content_hash.as_slice() &&
                      proof_output.content_hash == proof.content_hash &&
                      proof_output.root_hash == proof.root_hash;
        
        let verification_time = start_time.elapsed();
        
        // Update statistics
        self.update_verification_stats(is_valid, verification_time);
        
        if is_valid {
            info!(
                "Proof verification successful in {}ms",
                verification_time.as_millis()
            );
        } else {
            warn!(
                "Proof verification failed in {}ms",
                verification_time.as_millis()
            );
        }
        
        Ok(is_valid)
    }

    /// Returns current proof generation statistics
    pub fn get_statistics(&self) -> &ProofStatistics {
        &self.stats
    }

    /// Updates the configuration
    pub fn update_config(&mut self, config: ProofConfig) {
        self.config = config;
    }

    /// Validates input parameters
    fn validate_inputs(
        &self,
        file_path: &Path,
        content_selection: &ContentSelection,
    ) -> Result<()> {
        // Check if file exists and is readable
        if !file_path.exists() {
            return Err(ProofError::invalid_input_error(
                "file_path",
                format!("File does not exist: {}", file_path.display())
            ));
        }

        if !file_path.is_file() {
            return Err(ProofError::invalid_input_error(
                "file_path",
                format!("Path is not a file: {}", file_path.display())
            ));
        }

        // Validate content selection
        if !content_selection.is_valid() {
            return Err(ProofError::content_selection_error(
                "Invalid content selection parameters"
            ));
        }

        // Check file size limits
        let metadata = std::fs::metadata(file_path)?;
        let file_size = metadata.len();
        
        const MAX_FILE_SIZE: u64 = 50 * 1024 * 1024 * 1024; // 50GB
        if file_size > MAX_FILE_SIZE {
            return Err(ProofError::resource_limit_error(
                "file_size",
                format!("File size ({} bytes) exceeds maximum allowed size ({} bytes)", 
                       file_size, MAX_FILE_SIZE)
            ));
        }

        Ok(())
    }

    /// Extracts and hashes the content specified by the selection
    fn extract_content_hash(
        &self,
        blocks: &[IpfsBlock],
        content_selection: &ContentSelection,
    ) -> Result<[u8; 32]> {
        let content = self.extract_content(blocks, content_selection)?;
        let hash = Sha256::digest(&content);
        Ok(hash.into())
    }

    /// Extracts the actual content bytes from IPFS blocks
    fn extract_content(
        &self,
        blocks: &[IpfsBlock],
        content_selection: &ContentSelection,
    ) -> Result<Vec<u8>> {
        match content_selection {
            ContentSelection::ByteRange { start, end } => {
                self.extract_byte_range(blocks, *start, *end)
            }
            ContentSelection::Pattern { content } => {
                self.extract_pattern(blocks, content)
            }
            ContentSelection::Multiple(selections) => {
                let mut combined = Vec::new();
                for selection in selections {
                    let mut content = self.extract_content(blocks, selection)?;
                    combined.append(&mut content);
                }
                Ok(combined)
            }
        }
    }

    /// Extracts content from a byte range
    fn extract_byte_range(
        &self,
        blocks: &[IpfsBlock],
        start: usize,
        end: usize,
    ) -> Result<Vec<u8>> {
        let mut content = Vec::new();
        let mut current_offset = 0;

        for block in blocks {
            let block_start = current_offset;
            let block_end = current_offset + block.data.len();

            if block_start < end && block_end > start {
                let extract_start = if start > block_start { start - block_start } else { 0 };
                let extract_end = if end < block_end { end - block_start } else { block.data.len() };
                
                content.extend_from_slice(&block.data[extract_start..extract_end]);
            }

            current_offset = block_end;
            if current_offset >= end {
                break;
            }
        }

        if content.is_empty() {
            return Err(ProofError::content_selection_error(
                format!("No content found in byte range {}..{}", start, end)
            ));
        }

        Ok(content)
    }

    /// Extracts content matching a pattern
    fn extract_pattern(
        &self,
        blocks: &[IpfsBlock],
        pattern: &[u8],
    ) -> Result<Vec<u8>> {
        // Concatenate all block data for pattern searching
        let mut all_data = Vec::new();
        for block in blocks {
            all_data.extend_from_slice(&block.data);
        }

        // Find pattern
        if let Some(_pos) = self.find_pattern(&all_data, pattern) {
            Ok(pattern.to_vec())
        } else {
            Err(ProofError::content_selection_error(
                "Pattern not found in file content"
            ))
        }
    }

    /// Finds the first occurrence of a pattern in data
    fn find_pattern(&self, data: &[u8], pattern: &[u8]) -> Option<usize> {
        if pattern.is_empty() || pattern.len() > data.len() {
            return None;
        }

        for i in 0..=(data.len() - pattern.len()) {
            if &data[i..i + pattern.len()] == pattern {
                return Some(i);
            }
        }

        None
    }

    /// Generates the ZK proof using Risc0
    async fn generate_zk_proof(
        &self,
        input: ProofInput,
    ) -> Result<(Receipt, u64)> {
        let env = ExecutorEnv::builder()
            .write(&input)
            .map_err(|e| ProofError::zk_proof_error(
                "input_preparation",
                "Failed to prepare input for ZK circuit",
                Some(e)
            ))?
            .build()
            .map_err(|e| ProofError::zk_proof_error(
                "environment_setup",
                "Failed to build executor environment",
                Some(e)
            ))?;

        let prover = default_prover();
        let opts = ProverOpts::default();

        // Apply timeout if configured
        let prove_future = async {
            prover.prove_with_opts(env, IPFS_CONTENT_VERIFIER_ELF, &opts)
                .map_err(|e| ProofError::zk_proof_error(
                    "proof_generation",
                    "Failed to generate ZK proof",
                    Some(e)
                ))
        };

        let receipt = if let Some(timeout_secs) = self.config.timeout_seconds {
            timeout(Duration::from_secs(timeout_secs), prove_future)
                .await
                .map_err(|_| ProofError::timeout_error(
                    "zk_proof_generation",
                    timeout_secs * 1000
                ))?
        } else {
            prove_future.await
        }?;

        // Extract cycle count from receipt
        let cycles = receipt.get_metadata()
            .map(|m| m.cycles)
            .unwrap_or(0);

        Ok((receipt, cycles))
    }

    /// Creates comprehensive proof metadata
    fn create_proof_metadata(
        &self,
        guest_metadata: crate::guest_types::ProofMetadata,
        file_info: FileInfo,
        total_time: Duration,
        file_processing_time: Duration,
        zk_generation_time: Duration,
        zk_cycles: u64,
        receipt: &Receipt,
    ) -> Result<ProofMetadata> {
        let receipt_bytes = bincode::serialize(receipt)
            .map_err(|e| ProofError::serialization_error(
                "Failed to serialize receipt for size calculation",
                Some(Box::new(e))
            ))?;

        let performance = PerformanceMetrics {
            generation_time_ms: total_time.as_millis() as u64,
            file_processing_time_ms: file_processing_time.as_millis() as u64,
            zk_generation_time_ms: zk_generation_time.as_millis() as u64,
            peak_memory_bytes: self.get_peak_memory_usage(),
            zk_cycles,
            proof_size_bytes: receipt_bytes.len() as u64,
            compression_ratio: self.calculate_compression_ratio(&receipt_bytes, &file_info),
        };

        let security = SecurityParameters {
            security_level: self.config.security_level,
            hash_function: "SHA-256".to_string(),
            proof_system: "Risc0".to_string(),
            risc0_version: self.get_risc0_version(),
            formal_verification: false,
        };

        let environment = GenerationEnvironment {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            hardware_acceleration: self.detect_hardware_acceleration(),
            prover_type: self.config.prover_type.clone(),
            library_version: env!("CARGO_PKG_VERSION").to_string(),
            git_commit: option_env!("GIT_COMMIT").map(|s| s.to_string()),
        };

        Ok(ProofMetadata {
            guest_metadata,
            file_info,
            performance,
            security,
            environment,
            custom: self.config.custom_metadata.clone(),
        })
    }

    /// Detects available hardware acceleration
    fn detect_hardware_acceleration(&self) -> Option<HardwareAcceleration> {
        if self.config.use_hardware_acceleration {
            // Check for CUDA support
            if self.has_cuda_support() {
                return Some(HardwareAcceleration::Cuda);
            }
            
            // Check for Metal support (macOS)
            if self.has_metal_support() {
                return Some(HardwareAcceleration::Metal);
            }
            
            // Check for other hardware acceleration
            if self.has_hardware_acceleration() {
                return Some(HardwareAcceleration::Other("CPU-optimized".to_string()));
            }
        }
        
        Some(HardwareAcceleration::None)
    }

    /// Gets peak memory usage during proof generation
    fn get_peak_memory_usage(&self) -> u64 {
        // Use system memory information
        #[cfg(target_os = "linux")]
        {
            if let Ok(contents) = std::fs::read_to_string("/proc/self/status") {
                for line in contents.lines() {
                    if line.starts_with("VmPeak:") {
                        if let Some(kb_str) = line.split_whitespace().nth(1) {
                            if let Ok(kb) = kb_str.parse::<u64>() {
                                return kb * 1024; // Convert KB to bytes
                            }
                        }
                    }
                }
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            // Use mach system calls for macOS
            use std::process::Command;
            if let Ok(output) = Command::new("ps")
                .args(&["-o", "rss=", "-p"])
                .arg(std::process::id().to_string())
                .output()
            {
                if let Ok(rss_str) = String::from_utf8(output.stdout) {
                    if let Ok(rss_kb) = rss_str.trim().parse::<u64>() {
                        return rss_kb * 1024; // Convert KB to bytes
                    }
                }
            }
        }
        
        // Fallback: estimate based on proof size and overhead
        let base_memory = 256 * 1024 * 1024; // 256MB base
        let proof_overhead = self.stats.total_data_processed_bytes / 10; // 10% overhead estimate
        base_memory + proof_overhead
    }

    /// Calculates compression ratio if compression is used
    fn calculate_compression_ratio(&self, proof_bytes: &[u8], file_info: &FileInfo) -> Option<f64> {
        if self.config.compression.enabled {
            let original_size = file_info.size as f64;
            let compressed_size = proof_bytes.len() as f64;
            
            if original_size > 0.0 {
                Some(compressed_size / original_size)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Gets the actual Risc0 version
    fn get_risc0_version(&self) -> String {
        // Try to get version from Cargo.toml or environment
        if let Some(version) = option_env!("RISC0_VERSION") {
            return version.to_string();
        }
        
        // Parse from Cargo.lock if available
        if let Ok(cargo_lock) = std::fs::read_to_string("Cargo.lock") {
            for line in cargo_lock.lines() {
                if line.contains("risc0-zkvm") {
                    if let Some(version_line) = cargo_lock.lines()
                        .skip_while(|l| !l.contains("risc0-zkvm"))
                        .nth(1)
                    {
                        if let Some(version) = version_line.split('"').nth(1) {
                            return version.to_string();
                        }
                    }
                }
            }
        }
        
        // Fallback to known version
        "1.2.0".to_string()
    }

    /// Checks for CUDA support
    fn has_cuda_support(&self) -> bool {
        #[cfg(feature = "cuda")]
        {
            // Check if CUDA runtime is available
            std::process::Command::new("nvidia-smi")
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false)
        }
        #[cfg(not(feature = "cuda"))]
        false
    }

    /// Checks for Metal support (macOS)
    fn has_metal_support(&self) -> bool {
        #[cfg(all(target_os = "macos", feature = "metal"))]
        {
            // Check if Metal is available on macOS
            true // Metal is available on all modern macOS systems
        }
        #[cfg(not(all(target_os = "macos", feature = "metal")))]
        false
    }

    /// Checks for general hardware acceleration
    fn has_hardware_acceleration(&self) -> bool {
        // Check CPU features
        #[cfg(target_arch = "x86_64")]
        {
            is_x86_feature_detected!("avx2") || is_x86_feature_detected!("avx")
        }
        #[cfg(target_arch = "aarch64")]
        {
            // ARM64 typically has NEON support
            true
        }
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        false
    }

    /// Updates generation statistics
    fn update_generation_stats(&mut self, proof: &Proof, duration: Duration) {
        self.stats.total_proofs_generated += 1;
        self.stats.total_data_processed_bytes += proof.metadata.file_info.size;
        
        // Update average generation time
        let new_time = duration.as_millis() as f64;
        let count = self.stats.total_proofs_generated as f64;
        self.stats.avg_generation_time_ms = 
            (self.stats.avg_generation_time_ms * (count - 1.0) + new_time) / count;
    }

    /// Updates verification statistics
    fn update_verification_stats(&mut self, is_valid: bool, duration: Duration) {
        self.stats.total_proofs_verified += 1;
        
        // Update average verification time
        let new_time = duration.as_millis() as f64;
        let count = self.stats.total_proofs_verified as f64;
        self.stats.avg_verification_time_ms = 
            (self.stats.avg_verification_time_ms * (count - 1.0) + new_time) / count;
        
        // Update success rate
        let successful_verifications = if is_valid {
            (self.stats.verification_success_rate * (count - 1.0)) + 1.0
        } else {
            self.stats.verification_success_rate * (count - 1.0)
        };
        self.stats.verification_success_rate = successful_verifications / count;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[tokio::test]
    async fn test_proof_generator_creation() {
        let generator = ProofGenerator::new().await;
        assert!(generator.is_ok());
    }

    #[tokio::test]
    async fn test_input_validation() {
        let generator = ProofGenerator::new().await.unwrap();
        
        // Test non-existent file
        let result = generator.validate_inputs(
            Path::new("/non/existent/file"),
            &ContentSelection::Pattern { content: b"test".to_vec() }
        );
        assert!(result.is_err());
        
        // Test invalid content selection
        let temp_file = NamedTempFile::new().unwrap();
        let result = generator.validate_inputs(
            temp_file.path(),
            &ContentSelection::ByteRange { start: 10, end: 5 }
        );
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_pattern_finding() {
        let generator = ProofGenerator::new().await.unwrap();
        let data = b"Hello, world! This is a test.";
        
        assert_eq!(generator.find_pattern(data, b"world"), Some(7));
        assert_eq!(generator.find_pattern(data, b"test"), Some(25));
        assert_eq!(generator.find_pattern(data, b"missing"), None);
        assert_eq!(generator.find_pattern(data, b""), None);
    }
}

