//! Unit tests for proof generation functionality

use zkipfs_proof_core::{
    proof::{ProofGenerator, ProofConfig},
    types::{FileInfo, ProofResult},
    error::ZkIPFSError,
};
use std::path::PathBuf;
use tempfile::NamedTempFile;
use std::io::Write;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_file(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().expect("Failed to create temp file");
        file.write_all(content.as_bytes()).expect("Failed to write to temp file");
        file
    }

    #[test]
    fn test_proof_generator_creation() {
        let config = ProofConfig::default();
        let generator = ProofGenerator::new(config);
        assert!(generator.is_ok());
    }

    #[test]
    fn test_file_info_extraction() {
        let test_content = "Hello, World! This is a test file for zkIPFS-Proof.";
        let file = create_test_file(test_content);
        
        let file_info = FileInfo::from_path(file.path()).expect("Failed to create FileInfo");
        
        assert_eq!(file_info.size, test_content.len() as u64);
        assert!(file_info.hash.len() > 0);
        assert_eq!(file_info.mime_type, "text/plain");
    }

    #[test]
    fn test_content_pattern_search() {
        let test_content = "This file contains sensitive data: SECRET_KEY=abc123";
        let file = create_test_file(test_content);
        
        let config = ProofConfig::default();
        let generator = ProofGenerator::new(config).expect("Failed to create generator");
        
        let file_info = FileInfo::from_path(file.path()).expect("Failed to create FileInfo");
        let pattern = "SECRET_KEY=abc123";
        
        let result = generator.search_content_pattern(&file_info, pattern);
        assert!(result.is_ok());
        assert!(result.unwrap().found);
    }

    #[test]
    fn test_content_pattern_not_found() {
        let test_content = "This file contains no secrets.";
        let file = create_test_file(test_content);
        
        let config = ProofConfig::default();
        let generator = ProofGenerator::new(config).expect("Failed to create generator");
        
        let file_info = FileInfo::from_path(file.path()).expect("Failed to create FileInfo");
        let pattern = "SECRET_KEY=abc123";
        
        let result = generator.search_content_pattern(&file_info, pattern);
        assert!(result.is_ok());
        assert!(!result.unwrap().found);
    }

    #[test]
    fn test_proof_generation_small_file() {
        let test_content = "Small test file for proof generation.";
        let file = create_test_file(test_content);
        
        let config = ProofConfig {
            security_level: 128,
            chunk_size: 1024,
            max_file_size: 10 * 1024 * 1024, // 10MB
            enable_compression: false,
        };
        
        let generator = ProofGenerator::new(config).expect("Failed to create generator");
        let file_info = FileInfo::from_path(file.path()).expect("Failed to create FileInfo");
        let pattern = "test file";
        
        let result = generator.generate_proof(&file_info, pattern);
        assert!(result.is_ok());
        
        let proof_result = result.unwrap();
        assert!(proof_result.proof_data.len() > 0);
        assert!(proof_result.content_found);
    }

    #[test]
    fn test_proof_generation_large_content() {
        // Create a larger test file
        let large_content = "A".repeat(5000) + "NEEDLE_IN_HAYSTACK" + &"B".repeat(5000);
        let file = create_test_file(&large_content);
        
        let config = ProofConfig::default();
        let generator = ProofGenerator::new(config).expect("Failed to create generator");
        let file_info = FileInfo::from_path(file.path()).expect("Failed to create FileInfo");
        let pattern = "NEEDLE_IN_HAYSTACK";
        
        let result = generator.generate_proof(&file_info, pattern);
        assert!(result.is_ok());
        
        let proof_result = result.unwrap();
        assert!(proof_result.content_found);
        assert!(proof_result.position.is_some());
        assert_eq!(proof_result.position.unwrap(), 5000);
    }

    #[test]
    fn test_invalid_file_path() {
        let config = ProofConfig::default();
        let generator = ProofGenerator::new(config).expect("Failed to create generator");
        
        let invalid_path = PathBuf::from("/nonexistent/file.txt");
        let result = FileInfo::from_path(&invalid_path);
        
        assert!(result.is_err());
        match result.unwrap_err() {
            ZkIPFSError::FileNotFound(_) => {},
            _ => panic!("Expected FileNotFound error"),
        }
    }

    #[test]
    fn test_empty_pattern() {
        let test_content = "Some content here.";
        let file = create_test_file(test_content);
        
        let config = ProofConfig::default();
        let generator = ProofGenerator::new(config).expect("Failed to create generator");
        let file_info = FileInfo::from_path(file.path()).expect("Failed to create FileInfo");
        
        let result = generator.generate_proof(&file_info, "");
        assert!(result.is_err());
        match result.unwrap_err() {
            ZkIPFSError::InvalidInput(_) => {},
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[test]
    fn test_proof_metadata() {
        let test_content = "Test content for metadata validation.";
        let file = create_test_file(test_content);
        
        let config = ProofConfig::default();
        let generator = ProofGenerator::new(config).expect("Failed to create generator");
        let file_info = FileInfo::from_path(file.path()).expect("Failed to create FileInfo");
        let pattern = "metadata";
        
        let result = generator.generate_proof(&file_info, pattern);
        assert!(result.is_ok());
        
        let proof_result = result.unwrap();
        assert!(proof_result.metadata.generation_time > 0);
        assert!(proof_result.metadata.proof_size > 0);
        assert_eq!(proof_result.metadata.security_level, 128);
    }

    #[test]
    fn test_concurrent_proof_generation() {
        use std::thread;
        use std::sync::Arc;
        
        let test_content = "Concurrent test content.";
        let file = create_test_file(test_content);
        let file_path = file.path().to_path_buf();
        
        let config = Arc::new(ProofConfig::default());
        let pattern = "concurrent";
        
        let handles: Vec<_> = (0..5).map(|i| {
            let config = Arc::clone(&config);
            let file_path = file_path.clone();
            let pattern = pattern.to_string();
            
            thread::spawn(move || {
                let generator = ProofGenerator::new((*config).clone()).expect("Failed to create generator");
                let file_info = FileInfo::from_path(&file_path).expect("Failed to create FileInfo");
                
                let result = generator.generate_proof(&file_info, &pattern);
                assert!(result.is_ok());
                
                let proof_result = result.unwrap();
                assert!(proof_result.content_found);
                
                i
            })
        }).collect();
        
        for handle in handles {
            handle.join().expect("Thread panicked");
        }
    }

    #[test]
    fn test_different_security_levels() {
        let test_content = "Security level test content.";
        let file = create_test_file(test_content);
        let pattern = "Security";
        
        for &security_level in &[128, 192, 256] {
            let config = ProofConfig {
                security_level,
                ..ProofConfig::default()
            };
            
            let generator = ProofGenerator::new(config).expect("Failed to create generator");
            let file_info = FileInfo::from_path(file.path()).expect("Failed to create FileInfo");
            
            let result = generator.generate_proof(&file_info, pattern);
            assert!(result.is_ok());
            
            let proof_result = result.unwrap();
            assert!(proof_result.content_found);
            assert_eq!(proof_result.metadata.security_level, security_level);
        }
    }
}

