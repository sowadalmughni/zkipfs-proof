//! Core data types and structures for zkIPFS-Proof
//!
//! This module defines the main data structures used throughout the zkIPFS-Proof system,
//! including proof representations, content selections, and IPFS block structures.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use chrono::{DateTime, Utc};

// Re-export guest types for convenience
pub use crate::guest_types::{
    ProofInput, ProofOutput, IpfsBlock, BlockLink, 
    ContentSelection, ProofMetadata as GuestProofMetadata
};

/// A complete zero-knowledge proof for IPFS content verification
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Proof {
    /// Unique identifier for this proof
    pub id: String,
    /// The actual ZK proof data
    pub zk_proof: ZkProofData,
    /// Metadata about the proof generation
    pub metadata: ProofMetadata,
    /// Content selection that was proven
    pub content_selection: ContentSelection,
    /// Hash of the proven content
    pub content_hash: [u8; 32],
    /// Root hash of the IPFS structure
    pub root_hash: [u8; 32],
    /// Timestamp when the proof was generated
    pub created_at: DateTime<Utc>,
    /// Version of the proof format
    pub version: String,
}

/// Zero-knowledge proof data containing the cryptographic proof
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ZkProofData {
    /// The receipt from Risc0 containing the proof
    pub receipt: Vec<u8>,
    /// Public inputs to the proof
    pub public_inputs: Vec<u8>,
    /// Proof format version
    pub format_version: String,
    /// Compression algorithm used (if any)
    pub compression: Option<CompressionType>,
}

/// Supported compression types for proof data
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum CompressionType {
    /// No compression
    None,
    /// Gzip compression
    Gzip,
    /// Zstd compression
    Zstd,
}

/// Extended metadata about proof generation and verification
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProofMetadata {
    /// Basic metadata from the guest program
    pub guest_metadata: GuestProofMetadata,
    /// File information
    pub file_info: FileInfo,
    /// Performance metrics
    pub performance: PerformanceMetrics,
    /// Security parameters
    pub security: SecurityParameters,
    /// Generation environment
    pub environment: GenerationEnvironment,
    /// Additional custom metadata
    pub custom: HashMap<String, serde_json::Value>,
}

/// Information about the file that was proven
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileInfo {
    /// Original filename (if available)
    pub filename: Option<String>,
    /// File size in bytes
    pub size: u64,
    /// MIME type (if detected)
    pub mime_type: Option<String>,
    /// File hash (SHA-256)
    pub file_hash: [u8; 32],
    /// IPFS CID of the file
    pub ipfs_cid: String,
    /// Number of IPFS blocks
    pub block_count: u32,
    /// Average block size
    pub avg_block_size: u64,
}

/// Performance metrics for proof generation
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PerformanceMetrics {
    /// Total time taken for proof generation (milliseconds)
    pub generation_time_ms: u64,
    /// Time taken for file processing (milliseconds)
    pub file_processing_time_ms: u64,
    /// Time taken for ZK proof generation (milliseconds)
    pub zk_generation_time_ms: u64,
    /// Peak memory usage during generation (bytes)
    pub peak_memory_bytes: u64,
    /// Number of CPU cycles used in the ZK-VM
    pub zk_cycles: u64,
    /// Proof size in bytes
    pub proof_size_bytes: u64,
    /// Compression ratio (if compression was used)
    pub compression_ratio: Option<f64>,
}

/// Security parameters used for proof generation
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SecurityParameters {
    /// Security level in bits (e.g., 128, 256)
    pub security_level: u32,
    /// Hash function used for content hashing
    pub hash_function: String,
    /// ZK proof system used
    pub proof_system: String,
    /// Risc0 version used
    pub risc0_version: String,
    /// Whether formal verification was used
    pub formal_verification: bool,
}

/// Information about the environment where the proof was generated
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GenerationEnvironment {
    /// Operating system
    pub os: String,
    /// CPU architecture
    pub arch: String,
    /// Whether hardware acceleration was used
    pub hardware_acceleration: Option<HardwareAcceleration>,
    /// Prover type (local or cloud)
    pub prover_type: ProverType,
    /// Library version
    pub library_version: String,
    /// Git commit hash (if available)
    pub git_commit: Option<String>,
}

/// Hardware acceleration types
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum HardwareAcceleration {
    /// CUDA GPU acceleration
    Cuda,
    /// Metal GPU acceleration (Apple)
    Metal,
    /// CPU-only (no acceleration)
    None,
}

/// Prover types
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ProverType {
    /// Local proving using available hardware
    Local,
    /// Cloud proving using Bonsai
    Bonsai,
    /// Custom prover implementation
    Custom(String),
}

/// Verification result containing detailed information
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VerificationResult {
    /// Whether the proof is valid
    pub is_valid: bool,
    /// Verification timestamp
    pub verified_at: DateTime<Utc>,
    /// Time taken for verification (milliseconds)
    pub verification_time_ms: u64,
    /// Verifier information
    pub verifier_info: VerifierInfo,
    /// Any warnings or notes
    pub warnings: Vec<String>,
    /// Detailed verification steps (for debugging)
    pub verification_steps: Vec<VerificationStep>,
}

/// Information about the verifier
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VerifierInfo {
    /// Verifier version
    pub version: String,
    /// Verification method used
    pub method: VerificationMethod,
    /// Environment where verification was performed
    pub environment: String,
}

/// Verification methods
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum VerificationMethod {
    /// Local verification
    Local,
    /// On-chain verification
    OnChain { network: String, contract_address: String },
    /// Remote verification service
    Remote { service_url: String },
}

/// Individual verification step for debugging
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VerificationStep {
    /// Step name
    pub name: String,
    /// Whether this step passed
    pub passed: bool,
    /// Time taken for this step (milliseconds)
    pub duration_ms: u64,
    /// Additional details
    pub details: Option<String>,
}

/// Configuration for proof generation
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProofConfig {
    /// Security level to use (128, 192, or 256 bits)
    pub security_level: u32,
    /// Whether to use hardware acceleration if available
    pub use_hardware_acceleration: bool,
    /// Prover type preference
    pub prover_type: ProverType,
    /// Maximum memory usage (bytes)
    pub max_memory_bytes: Option<u64>,
    /// Timeout for proof generation (seconds)
    pub timeout_seconds: Option<u64>,
    /// Whether to compress the proof
    pub compression: CompressionType,
    /// Custom metadata to include
    pub custom_metadata: HashMap<String, serde_json::Value>,
    /// Whether to include performance metrics
    pub include_performance_metrics: bool,
    /// Whether to include detailed verification steps
    pub include_verification_steps: bool,
}

impl Default for ProofConfig {
    fn default() -> Self {
        Self {
            security_level: 128,
            use_hardware_acceleration: true,
            prover_type: ProverType::Local,
            max_memory_bytes: None,
            timeout_seconds: Some(600), // 10 minutes default
            compression: CompressionType::Gzip,
            custom_metadata: HashMap::new(),
            include_performance_metrics: true,
            include_verification_steps: false,
        }
    }
}

/// Statistics about proof generation and verification
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProofStatistics {
    /// Total number of proofs generated
    pub total_proofs_generated: u64,
    /// Total number of proofs verified
    pub total_proofs_verified: u64,
    /// Average proof generation time (milliseconds)
    pub avg_generation_time_ms: f64,
    /// Average verification time (milliseconds)
    pub avg_verification_time_ms: f64,
    /// Success rate for proof generation
    pub generation_success_rate: f64,
    /// Success rate for proof verification
    pub verification_success_rate: f64,
    /// Total data processed (bytes)
    pub total_data_processed_bytes: u64,
    /// Most common file types processed
    pub common_file_types: HashMap<String, u64>,
}

impl fmt::Display for Proof {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Proof {} (created: {}, content: {} bytes)",
            &self.id[..8],
            self.created_at.format("%Y-%m-%d %H:%M:%S UTC"),
            self.metadata.guest_metadata.content_size
        )
    }
}

impl fmt::Display for VerificationResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.is_valid { "VALID" } else { "INVALID" };
        write!(
            f,
            "Verification: {} ({}ms)",
            status,
            self.verification_time_ms
        )
    }
}

impl ContentSelection {
    /// Returns the estimated size of content that will be proven
    pub fn estimated_size(&self) -> Option<usize> {
        match self {
            ContentSelection::ByteRange { start, end } => Some(end - start),
            ContentSelection::Pattern { content } => Some(content.len()),
            ContentSelection::Regex { .. } => None,
            ContentSelection::Multiple(selections) => {
                selections.iter()
                    .map(|s| s.estimated_size())
                    .sum::<Option<usize>>()
            }
        }
    }

    /// Returns true if this selection is valid
    pub fn is_valid(&self) -> bool {
        match self {
            ContentSelection::ByteRange { start, end } => start < end,
            ContentSelection::Pattern { content } => !content.is_empty(),
            ContentSelection::Regex { pattern } => !pattern.is_empty(),
            ContentSelection::Multiple(selections) => {
                !selections.is_empty() && selections.iter().all(|s| s.is_valid())
            }
        }
    }

    /// Returns a human-readable description of the selection
    pub fn description(&self) -> String {
        match self {
            ContentSelection::ByteRange { start, end } => {
                format!("Bytes {}-{} ({} bytes)", start, end, end - start)
            }
            ContentSelection::Pattern { content } => {
                format!("Pattern: {} ({} bytes)", 
                    String::from_utf8_lossy(&content[..content.len().min(50)]),
                    content.len()
                )
            }
            ContentSelection::Regex { pattern } => {
                format!("Regex: {}", pattern)
            }
            ContentSelection::Multiple(selections) => {
                format!("Multiple selections ({})", selections.len())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_selection_validation() {
        let valid_range = ContentSelection::ByteRange { start: 0, end: 10 };
        assert!(valid_range.is_valid());
        assert_eq!(valid_range.estimated_size(), Some(10));

        let invalid_range = ContentSelection::ByteRange { start: 10, end: 5 };
        assert!(!invalid_range.is_valid());

        let pattern = ContentSelection::Pattern { content: b"test".to_vec() };
        assert!(pattern.is_valid());
        assert_eq!(pattern.estimated_size(), Some(4));

        let empty_pattern = ContentSelection::Pattern { content: vec![] };
        assert!(!empty_pattern.is_valid());

        let regex = ContentSelection::Regex { pattern: "^test".to_string() };
        assert!(regex.is_valid());
        assert_eq!(regex.estimated_size(), None);
    }

    #[test]
    fn test_proof_config_default() {
        let config = ProofConfig::default();
        assert_eq!(config.security_level, 128);
        assert!(config.use_hardware_acceleration);
        assert!(matches!(config.prover_type, ProverType::Local));
        assert_eq!(config.timeout_seconds, Some(600));
    }

    #[test]
    fn test_content_selection_description() {
        let range = ContentSelection::ByteRange { start: 100, end: 200 };
        assert_eq!(range.description(), "Bytes 100-200 (100 bytes)");

        let pattern = ContentSelection::Pattern { content: b"hello world".to_vec() };
        assert!(pattern.description().contains("hello world"));
        assert!(pattern.description().contains("11 bytes"));
    }
}

