//! # zkIPFS-Proof Core Library
//!
//! This library provides the core functionality for generating and verifying zero-knowledge proofs
//! of IPFS content. It enables users to cryptographically prove that specific content exists within
//! larger files without revealing the complete file contents.
//!
//! ## Features
//!
//! - **Zero-Knowledge Proofs**: Generate proofs using Risc0's ZK-VM technology
//! - **IPFS Integration**: Native support for IPFS content addressing and DAG structures
//! - **Flexible Content Selection**: Prove byte ranges, patterns, or multiple content selections
//! - **High Performance**: Optimized for large files with streaming processing
//! - **Security**: Cryptographic guarantees with 128-bit security level
//!
//! ## Quick Start
//!
//! ```rust
//! use zkipfs_proof_core::{ProofGenerator, ContentSelection};
//! use std::path::Path;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a proof generator
//!     let generator = ProofGenerator::new().await?;
//!     
//!     // Generate a proof for specific content
//!     let proof = generator.generate_proof(
//!         Path::new("example.txt"),
//!         ContentSelection::Pattern {
//!             content: b"secret information".to_vec()
//!         }
//!     ).await?;
//!     
//!     // Verify the proof
//!     let is_valid = generator.verify_proof(&proof, b"secret information").await?;
//!     assert!(is_valid);
//!     
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod types;
pub mod proof;
pub mod ipfs;
pub mod ipfs_client;
pub mod verifier;
pub mod error_patterns;
pub mod monitoring;
pub mod profiling;
pub mod i18n;
pub mod performance;
pub mod cache;
pub mod proof_types;
pub mod ecosystem_integration;
pub mod advanced_verification;

// Re-export main types for convenience
pub use error::{ProofError, Result};
pub use proof::{ProofGenerator, ProofConfig};
pub use types::{
    ContentSelection, IpfsBlock, Proof, ProofMetadata, 
    ProofInput, ProofOutput, BlockLink
};
pub use verifier::ProofVerifier;
pub use performance::{PerformanceMonitor, OptimizationSettings, PerformanceStats};
pub use cache::{CacheManager, CacheConfig, CacheKey};

// Re-export guest program types for host-guest communication
pub use crate::guest_types::*;

/// Guest program types that are shared between host and guest
mod guest_types {
    use serde::{Deserialize, Serialize};

    /// Input data structure for the ZK circuit
    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct ProofInput {
        /// IPFS blocks that form the complete file structure
        pub blocks: Vec<IpfsBlock>,
        /// Specification of which content to prove exists
        pub content_selection: ContentSelection,
        /// Expected content hash for verification
        pub expected_content_hash: [u8; 32],
    }

    /// Represents an IPFS block with its data and metadata
    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct IpfsBlock {
        /// Raw block data
        pub data: Vec<u8>,
        /// Block's content identifier (CID)
        pub cid: Vec<u8>,
        /// Links to other blocks (for DAG structure)
        pub links: Vec<BlockLink>,
    }

    /// Link to another IPFS block
    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct BlockLink {
        /// Name/path of the linked content
        pub name: String,
        /// CID of the linked block
        pub cid: Vec<u8>,
        /// Size of the linked content
        pub size: u64,
    }

    /// Specification of content to prove within the file
    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub enum ContentSelection {
        /// Prove content exists within a specific byte range
        ByteRange { start: usize, end: usize },
        /// Prove specific content pattern exists
        Pattern { content: Vec<u8> },
        /// Prove content matching a regular expression exists
        Regex { pattern: String },
        /// Prove multiple content selections
        Multiple(Vec<ContentSelection>),
    }

    /// Output data structure from the ZK circuit
    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct ProofOutput {
        /// Root hash of the IPFS DAG structure
        pub root_hash: [u8; 32],
        /// Hash of the proven content
        pub content_hash: [u8; 32],
        /// Merkle proof demonstrating content inclusion
        pub inclusion_proof: Vec<[u8; 32]>,
        /// Metadata about the proof
        pub metadata: ProofMetadata,
    }

    /// Metadata about the generated proof
    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct ProofMetadata {
        /// Total number of blocks processed
        pub block_count: u32,
        /// Total size of content proven
        pub content_size: u64,
        /// Timestamp of proof generation (block number)
        pub timestamp: u64,
    }
}

// Include the compiled guest program
pub use zkipfs_proof_methods::IPFS_CONTENT_VERIFIER_ELF;
pub use zkipfs_proof_methods::IPFS_CONTENT_VERIFIER_ID;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[tokio::test]
    async fn test_basic_proof_generation() {
        // Create a temporary test file
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "This is a test file with secret content").unwrap();
        
        let generator = ProofGenerator::new().await.unwrap();
        
        let proof = generator.generate_proof(
            temp_file.path(),
            ContentSelection::Pattern {
                content: b"secret content".to_vec()
            }
        ).await.unwrap();
        
        // Verify the proof
        let is_valid = generator.verify_proof(&proof, b"secret content").await.unwrap();
        assert!(is_valid);
    }

    #[tokio::test]
    async fn test_byte_range_proof() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "0123456789abcdef").unwrap();
        
        let generator = ProofGenerator::new().await.unwrap();
        
        let proof = generator.generate_proof(
            temp_file.path(),
            ContentSelection::ByteRange { start: 5, end: 10 }
        ).await.unwrap();
        
        // The content at bytes 5-10 should be "56789"
        let is_valid = generator.verify_proof(&proof, b"56789").await.unwrap();
        assert!(is_valid);
    }

    #[tokio::test]
    async fn test_invalid_proof_rejection() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "This is a test file").unwrap();
        
        let generator = ProofGenerator::new().await.unwrap();
        
        let proof = generator.generate_proof(
            temp_file.path(),
            ContentSelection::Pattern {
                content: b"test file".to_vec()
            }
        ).await.unwrap();
        
        // Try to verify with wrong content - should fail
        let is_valid = generator.verify_proof(&proof, b"wrong content").await.unwrap();
        assert!(!is_valid);
    }
}

