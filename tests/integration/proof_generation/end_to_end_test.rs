//! End-to-end integration tests for proof generation and verification

use zkipfs_proof_core::{
    proof::{ProofGenerator, ProofVerifier},
    types::{FileInfo, ProofConfig},
    ipfs::IPFSClient,
};
use std::path::PathBuf;
use tempfile::{NamedTempFile, TempDir};
use std::io::Write;

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_environment() -> (TempDir, NamedTempFile) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let mut test_file = NamedTempFile::new_in(&temp_dir).expect("Failed to create temp file");
        
        let test_content = r#"
        Financial Report Q4 2024
        
        Revenue: $1,250,000
        Expenses: $890,000
        Net Profit: $360,000
        
        Confidential Transaction Details:
        Transaction ID: TX-2024-Q4-001
        Account: ACC-789-XYZ
        Amount: $125,000
        Date: 2024-12-15
        
        This document contains sensitive financial information.
        "#;
        
        test_file.write_all(test_content.as_bytes()).expect("Failed to write test content");
        (temp_dir, test_file)
    }

    #[tokio::test]
    async fn test_complete_proof_workflow() {
        let (_temp_dir, test_file) = setup_test_environment();
        
        // Step 1: Generate proof
        let config = ProofConfig {
            security_level: 128,
            chunk_size: 1024,
            max_file_size: 10 * 1024 * 1024,
            enable_compression: false,
        };
        
        let generator = ProofGenerator::new(config.clone()).expect("Failed to create generator");
        let file_info = FileInfo::from_path(test_file.path()).expect("Failed to create FileInfo");
        let pattern = "Transaction ID: TX-2024-Q4-001";
        
        let proof_result = generator.generate_proof(&file_info, pattern)
            .expect("Failed to generate proof");
        
        assert!(proof_result.content_found);
        assert!(proof_result.proof_data.len() > 0);
        assert!(proof_result.position.is_some());
        
        // Step 2: Verify proof
        let verifier = ProofVerifier::new(config).expect("Failed to create verifier");
        let verification_result = verifier.verify_proof(&proof_result.proof_data, pattern)
            .expect("Failed to verify proof");
        
        assert!(verification_result.is_valid);
        assert_eq!(verification_result.content_hash, proof_result.content_hash);
        
        // Step 3: Test with wrong pattern (should fail)
        let wrong_pattern = "Transaction ID: TX-2024-Q4-999";
        let wrong_verification = verifier.verify_proof(&proof_result.proof_data, wrong_pattern);
        
        assert!(wrong_verification.is_ok());
        assert!(!wrong_verification.unwrap().is_valid);
    }

    #[tokio::test]
    async fn test_ipfs_integration() {
        let (_temp_dir, test_file) = setup_test_environment();
        
        // Mock IPFS client for testing
        let ipfs_client = IPFSClient::new_mock().expect("Failed to create mock IPFS client");
        
        // Upload file to IPFS
        let file_content = std::fs::read(test_file.path()).expect("Failed to read test file");
        let ipfs_hash = ipfs_client.add_file(&file_content).await
            .expect("Failed to upload to IPFS");
        
        assert!(ipfs_hash.len() > 0);
        
        // Retrieve file from IPFS
        let retrieved_content = ipfs_client.get_file(&ipfs_hash).await
            .expect("Failed to retrieve from IPFS");
        
        assert_eq!(file_content, retrieved_content);
        
        // Generate proof for IPFS content
        let config = ProofConfig::default();
        let generator = ProofGenerator::new(config).expect("Failed to create generator");
        
        // Create temporary file with retrieved content
        let mut temp_retrieved = NamedTempFile::new().expect("Failed to create temp file");
        temp_retrieved.write_all(&retrieved_content).expect("Failed to write retrieved content");
        
        let file_info = FileInfo::from_path(temp_retrieved.path()).expect("Failed to create FileInfo");
        let pattern = "Confidential Transaction Details";
        
        let proof_result = generator.generate_proof(&file_info, pattern)
            .expect("Failed to generate proof for IPFS content");
        
        assert!(proof_result.content_found);
    }

    #[tokio::test]
    async fn test_large_file_handling() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let large_file_path = temp_dir.path().join("large_test_file.txt");
        
        // Create a large file (5MB)
        let chunk = "A".repeat(1024); // 1KB chunk
        let mut large_content = String::new();
        for i in 0..5120 { // 5MB total
            if i == 2560 { // Middle of file
                large_content.push_str("HIDDEN_SECRET_DATA_12345");
            }
            large_content.push_str(&chunk);
        }
        
        std::fs::write(&large_file_path, &large_content).expect("Failed to write large file");
        
        let config = ProofConfig {
            security_level: 128,
            chunk_size: 4096, // 4KB chunks
            max_file_size: 10 * 1024 * 1024, // 10MB limit
            enable_compression: true,
        };
        
        let generator = ProofGenerator::new(config).expect("Failed to create generator");
        let file_info = FileInfo::from_path(&large_file_path).expect("Failed to create FileInfo");
        let pattern = "HIDDEN_SECRET_DATA_12345";
        
        let start_time = std::time::Instant::now();
        let proof_result = generator.generate_proof(&file_info, pattern)
            .expect("Failed to generate proof for large file");
        let generation_time = start_time.elapsed();
        
        assert!(proof_result.content_found);
        assert!(proof_result.position.is_some());
        assert!(generation_time.as_secs() < 30); // Should complete within 30 seconds
        
        // Verify the proof
        let verifier = ProofVerifier::new(config).expect("Failed to create verifier");
        let verification_result = verifier.verify_proof(&proof_result.proof_data, pattern)
            .expect("Failed to verify large file proof");
        
        assert!(verification_result.is_valid);
    }

    #[tokio::test]
    async fn test_multiple_patterns_in_file() {
        let (_temp_dir, test_file) = setup_test_environment();
        let config = ProofConfig::default();
        let generator = ProofGenerator::new(config.clone()).expect("Failed to create generator");
        let file_info = FileInfo::from_path(test_file.path()).expect("Failed to create FileInfo");
        
        let patterns = vec![
            "Revenue: $1,250,000",
            "Transaction ID: TX-2024-Q4-001",
            "Account: ACC-789-XYZ",
            "Net Profit: $360,000",
        ];
        
        let mut proof_results = Vec::new();
        
        for pattern in &patterns {
            let proof_result = generator.generate_proof(&file_info, pattern)
                .expect(&format!("Failed to generate proof for pattern: {}", pattern));
            
            assert!(proof_result.content_found, "Pattern not found: {}", pattern);
            proof_results.push(proof_result);
        }
        
        // Verify all proofs
        let verifier = ProofVerifier::new(config).expect("Failed to create verifier");
        
        for (i, proof_result) in proof_results.iter().enumerate() {
            let verification_result = verifier.verify_proof(&proof_result.proof_data, &patterns[i])
                .expect(&format!("Failed to verify proof for pattern: {}", patterns[i]));
            
            assert!(verification_result.is_valid, "Verification failed for pattern: {}", patterns[i]);
        }
    }

    #[tokio::test]
    async fn test_proof_serialization() {
        let (_temp_dir, test_file) = setup_test_environment();
        
        let config = ProofConfig::default();
        let generator = ProofGenerator::new(config.clone()).expect("Failed to create generator");
        let file_info = FileInfo::from_path(test_file.path()).expect("Failed to create FileInfo");
        let pattern = "Financial Report Q4 2024";
        
        let proof_result = generator.generate_proof(&file_info, pattern)
            .expect("Failed to generate proof");
        
        // Serialize proof to JSON
        let serialized = serde_json::to_string(&proof_result)
            .expect("Failed to serialize proof");
        
        assert!(serialized.len() > 0);
        
        // Deserialize proof from JSON
        let deserialized: zkipfs_proof_core::types::ProofResult = 
            serde_json::from_str(&serialized)
                .expect("Failed to deserialize proof");
        
        assert_eq!(proof_result.content_hash, deserialized.content_hash);
        assert_eq!(proof_result.content_found, deserialized.content_found);
        assert_eq!(proof_result.position, deserialized.position);
        
        // Verify deserialized proof
        let verifier = ProofVerifier::new(config).expect("Failed to create verifier");
        let verification_result = verifier.verify_proof(&deserialized.proof_data, pattern)
            .expect("Failed to verify deserialized proof");
        
        assert!(verification_result.is_valid);
    }

    #[tokio::test]
    async fn test_error_handling() {
        let config = ProofConfig::default();
        let generator = ProofGenerator::new(config).expect("Failed to create generator");
        
        // Test with non-existent file
        let non_existent_path = PathBuf::from("/tmp/non_existent_file_12345.txt");
        let file_info_result = FileInfo::from_path(&non_existent_path);
        assert!(file_info_result.is_err());
        
        // Test with empty pattern
        let (_temp_dir, test_file) = setup_test_environment();
        let file_info = FileInfo::from_path(test_file.path()).expect("Failed to create FileInfo");
        let empty_pattern_result = generator.generate_proof(&file_info, "");
        assert!(empty_pattern_result.is_err());
        
        // Test with extremely long pattern
        let long_pattern = "A".repeat(10000);
        let long_pattern_result = generator.generate_proof(&file_info, &long_pattern);
        assert!(long_pattern_result.is_ok()); // Should handle gracefully
        assert!(!long_pattern_result.unwrap().content_found);
    }
}

