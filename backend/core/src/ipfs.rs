//! IPFS processing functionality for zkIPFS-Proof
//!
//! This module handles the conversion of files into IPFS block structures
//! and provides utilities for working with IPFS content addressing.

use crate::{
    error::{ProofError, Result, ResultExt},
    types::{IpfsBlock, BlockLink, FileInfo},
};
use sha2::{Digest, Sha256};
use std::path::Path;
use tokio::fs;
use tracing::{debug, instrument};
use cid::{Cid, Version};
use multihash::{Code, MultihashDigest};

/// IPFS processor for converting files to IPFS block structures
pub struct IpfsProcessor {
    /// Maximum block size for IPFS blocks (default: 256KB)
    max_block_size: usize,
    /// Chunk size for reading large files
    chunk_size: usize,
}

impl IpfsProcessor {
    /// Creates a new IPFS processor with default settings
    pub async fn new() -> Result<Self> {
        Ok(Self {
            max_block_size: 256 * 1024, // 256KB
            chunk_size: 64 * 1024,      // 64KB
        })
    }

    /// Processes a file into IPFS blocks and returns file information
    #[instrument(skip(self), fields(file = %file_path.display()))]
    pub async fn process_file(
        &self,
        file_path: &Path,
    ) -> Result<(Vec<IpfsBlock>, FileInfo)> {
        debug!("Processing file into IPFS blocks: {}", file_path.display());

        // Read file metadata
        let metadata = fs::metadata(file_path).await
            .context("Failed to read file metadata")?;
        
        let file_size = metadata.len();
        
        // Read file content
        let content = fs::read(file_path).await
            .context("Failed to read file content")?;
        
        // Calculate file hash
        let file_hash = Sha256::digest(&content);
        
        // Split content into blocks
        let blocks = self.create_blocks(&content)?;
        
        // Calculate IPFS CID for the entire file
        let file_cid = self.calculate_file_cid(&content)?;
        
        // Detect MIME type
        let mime_type = self.detect_mime_type(file_path, &content);
        
        // Calculate average block size
        let avg_block_size = if !blocks.is_empty() {
            blocks.iter().map(|b| b.data.len() as u64).sum::<u64>() / blocks.len() as u64
        } else {
            0
        };
        
        let file_info = FileInfo {
            filename: file_path.file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string()),
            size: file_size,
            mime_type,
            file_hash: file_hash.into(),
            ipfs_cid: file_cid.to_string(),
            block_count: blocks.len() as u32,
            avg_block_size,
        };
        
        debug!(
            "File processed into {} blocks (avg size: {} bytes)",
            blocks.len(),
            avg_block_size
        );
        
        Ok((blocks, file_info))
    }

    /// Creates IPFS blocks from file content
    fn create_blocks(&self, content: &[u8]) -> Result<Vec<IpfsBlock>> {
        let mut blocks = Vec::new();
        let mut offset = 0;
        
        while offset < content.len() {
            let end = std::cmp::min(offset + self.max_block_size, content.len());
            let block_data = &content[offset..end];
            
            // Create block CID
            let cid = self.calculate_block_cid(block_data)?;
            
            // Create block with no links for now (simple chunking)
            let block = IpfsBlock {
                data: block_data.to_vec(),
                cid: cid.to_bytes(),
                links: Vec::new(),
            };
            
            blocks.push(block);
            offset = end;
        }
        
        // If we have multiple blocks, create a root block that links to all chunks
        if blocks.len() > 1 {
            let root_block = self.create_root_block(&blocks)?;
            blocks.insert(0, root_block);
        }
        
        Ok(blocks)
    }

    /// Creates a root block that links to all content blocks
    fn create_root_block(&self, content_blocks: &[IpfsBlock]) -> Result<IpfsBlock> {
        let mut links = Vec::new();
        let mut root_data = Vec::new();
        
        for (i, block) in content_blocks.iter().enumerate() {
            let link = BlockLink {
                name: format!("chunk_{}", i),
                cid: block.cid.clone(),
                size: block.data.len() as u64,
            };
            links.push(link);
            
            // Add link information to root block data
            root_data.extend_from_slice(&block.cid);
            root_data.extend_from_slice(&(block.data.len() as u64).to_le_bytes());
        }
        
        let root_cid = self.calculate_block_cid(&root_data)?;
        
        Ok(IpfsBlock {
            data: root_data,
            cid: root_cid.to_bytes(),
            links,
        })
    }

    /// Calculates the CID for a block of data
    fn calculate_block_cid(&self, data: &[u8]) -> Result<Cid> {
        let hash = Code::Sha2_256.digest(data);
        let cid = Cid::new_v1(0x55, hash); // 0x55 is the codec for raw data
        Ok(cid)
    }

    /// Calculates the CID for the entire file
    fn calculate_file_cid(&self, content: &[u8]) -> Result<Cid> {
        self.calculate_block_cid(content)
    }

    /// Detects MIME type of the file
    fn detect_mime_type(&self, file_path: &Path, content: &[u8]) -> Option<String> {
        // Simple MIME type detection based on file extension and content
        if let Some(extension) = file_path.extension().and_then(|e| e.to_str()) {
            match extension.to_lowercase().as_str() {
                "txt" => return Some("text/plain".to_string()),
                "json" => return Some("application/json".to_string()),
                "csv" => return Some("text/csv".to_string()),
                "pdf" => return Some("application/pdf".to_string()),
                "png" => return Some("image/png".to_string()),
                "jpg" | "jpeg" => return Some("image/jpeg".to_string()),
                "mp4" => return Some("video/mp4".to_string()),
                "zip" => return Some("application/zip".to_string()),
                _ => {}
            }
        }
        
        // Content-based detection for common formats
        if content.starts_with(b"%PDF") {
            Some("application/pdf".to_string())
        } else if content.starts_with(b"\x89PNG\r\n\x1a\n") {
            Some("image/png".to_string())
        } else if content.starts_with(b"\xFF\xD8\xFF") {
            Some("image/jpeg".to_string())
        } else if content.starts_with(b"PK") {
            Some("application/zip".to_string())
        } else if content.starts_with(b"{") || content.starts_with(b"[") {
            Some("application/json".to_string())
        } else if content.is_ascii() {
            Some("text/plain".to_string())
        } else {
            Some("application/octet-stream".to_string())
        }
    }

    /// Reconstructs file content from IPFS blocks
    pub fn reconstruct_content(&self, blocks: &[IpfsBlock]) -> Result<Vec<u8>> {
        if blocks.is_empty() {
            return Ok(Vec::new());
        }
        
        // If there's only one block, return its data
        if blocks.len() == 1 {
            return Ok(blocks[0].data.clone());
        }
        
        // If there are multiple blocks, the first one should be the root block
        let root_block = &blocks[0];
        if root_block.links.is_empty() {
            // No links, just concatenate all blocks
            let mut content = Vec::new();
            for block in blocks {
                content.extend_from_slice(&block.data);
            }
            return Ok(content);
        }
        
        // Reconstruct content using the link structure
        let mut content = Vec::new();
        for link in &root_block.links {
            // Find the corresponding block
            if let Some(block) = blocks.iter().find(|b| b.cid == link.cid) {
                content.extend_from_slice(&block.data);
            } else {
                return Err(ProofError::ipfs_error(
                    "content_reconstruction",
                    format!("Missing block for link: {}", link.name),
                    None,
                ));
            }
        }
        
        Ok(content)
    }

    /// Validates the integrity of IPFS blocks
    pub fn validate_blocks(&self, blocks: &[IpfsBlock]) -> Result<()> {
        for (i, block) in blocks.iter().enumerate() {
            // Verify block CID matches its content
            let expected_cid = self.calculate_block_cid(&block.data)?;
            let actual_cid = Cid::try_from(&block.cid[..])
                .map_err(|e| ProofError::ipfs_error(
                    "cid_parsing",
                    format!("Invalid CID in block {}: {}", i, e),
                    Some(Box::new(e)),
                ))?;
            
            if expected_cid != actual_cid {
                return Err(ProofError::ipfs_error(
                    "block_validation",
                    format!("CID mismatch in block {}: expected {}, got {}", 
                           i, expected_cid, actual_cid),
                    None,
                ));
            }
            
            // Validate links if present
            for (j, link) in block.links.iter().enumerate() {
                if link.name.is_empty() {
                    return Err(ProofError::ipfs_error(
                        "link_validation",
                        format!("Empty link name in block {} link {}", i, j),
                        None,
                    ));
                }
                
                if link.cid.is_empty() {
                    return Err(ProofError::ipfs_error(
                        "link_validation",
                        format!("Empty link CID in block {} link {}", i, j),
                        None,
                    ));
                }
            }
        }
        
        Ok(())
    }

    /// Gets the total size of all blocks
    pub fn get_total_size(&self, blocks: &[IpfsBlock]) -> u64 {
        blocks.iter().map(|b| b.data.len() as u64).sum()
    }

    /// Gets statistics about the block structure
    pub fn get_block_stats(&self, blocks: &[IpfsBlock]) -> BlockStatistics {
        if blocks.is_empty() {
            return BlockStatistics::default();
        }
        
        let sizes: Vec<usize> = blocks.iter().map(|b| b.data.len()).collect();
        let total_size: usize = sizes.iter().sum();
        let min_size = *sizes.iter().min().unwrap();
        let max_size = *sizes.iter().max().unwrap();
        let avg_size = total_size / sizes.len();
        
        let total_links: usize = blocks.iter().map(|b| b.links.len()).sum();
        
        BlockStatistics {
            block_count: blocks.len(),
            total_size: total_size as u64,
            min_block_size: min_size,
            max_block_size: max_size,
            avg_block_size: avg_size,
            total_links,
        }
    }
}

/// Statistics about IPFS block structure
#[derive(Debug, Clone)]
pub struct BlockStatistics {
    pub block_count: usize,
    pub total_size: u64,
    pub min_block_size: usize,
    pub max_block_size: usize,
    pub avg_block_size: usize,
    pub total_links: usize,
}

impl Default for BlockStatistics {
    fn default() -> Self {
        Self {
            block_count: 0,
            total_size: 0,
            min_block_size: 0,
            max_block_size: 0,
            avg_block_size: 0,
            total_links: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn test_ipfs_processor_creation() {
        let processor = IpfsProcessor::new().await;
        assert!(processor.is_ok());
    }

    #[tokio::test]
    async fn test_small_file_processing() {
        let processor = IpfsProcessor::new().await.unwrap();
        
        // Create a small test file
        let mut temp_file = NamedTempFile::new().unwrap();
        let test_content = b"Hello, IPFS world!";
        temp_file.write_all(test_content).unwrap();
        
        let (blocks, file_info) = processor.process_file(temp_file.path()).await.unwrap();
        
        // Small file should result in a single block
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].data, test_content);
        assert_eq!(file_info.size, test_content.len() as u64);
        assert_eq!(file_info.block_count, 1);
    }

    #[tokio::test]
    async fn test_large_file_processing() {
        let mut processor = IpfsProcessor::new().await.unwrap();
        processor.max_block_size = 10; // Small block size for testing
        
        // Create a larger test file
        let mut temp_file = NamedTempFile::new().unwrap();
        let test_content = b"This is a longer test content that should be split into multiple blocks";
        temp_file.write_all(test_content).unwrap();
        
        let (blocks, file_info) = processor.process_file(temp_file.path()).await.unwrap();
        
        // Should result in multiple blocks plus a root block
        assert!(blocks.len() > 1);
        assert_eq!(file_info.size, test_content.len() as u64);
        
        // Verify we can reconstruct the original content
        let reconstructed = processor.reconstruct_content(&blocks).unwrap();
        assert_eq!(reconstructed, test_content);
    }

    #[tokio::test]
    async fn test_mime_type_detection() {
        let processor = IpfsProcessor::new().await.unwrap();
        
        // Test extension-based detection
        let txt_path = Path::new("test.txt");
        let mime = processor.detect_mime_type(txt_path, b"hello");
        assert_eq!(mime, Some("text/plain".to_string()));
        
        // Test content-based detection
        let unknown_path = Path::new("unknown");
        let pdf_mime = processor.detect_mime_type(unknown_path, b"%PDF-1.4");
        assert_eq!(pdf_mime, Some("application/pdf".to_string()));
    }

    #[tokio::test]
    async fn test_block_validation() {
        let processor = IpfsProcessor::new().await.unwrap();
        
        // Create valid blocks
        let content = b"test content";
        let blocks = processor.create_blocks(content).unwrap();
        
        // Validation should pass
        assert!(processor.validate_blocks(&blocks).is_ok());
        
        // Create invalid block with wrong CID
        let mut invalid_blocks = blocks.clone();
        invalid_blocks[0].cid = vec![0, 1, 2, 3]; // Invalid CID
        
        // Validation should fail
        assert!(processor.validate_blocks(&invalid_blocks).is_err());
    }

    #[tokio::test]
    async fn test_block_statistics() {
        let processor = IpfsProcessor::new().await.unwrap();
        
        let content = b"test content for statistics";
        let blocks = processor.create_blocks(content).unwrap();
        
        let stats = processor.get_block_stats(&blocks);
        assert_eq!(stats.block_count, blocks.len());
        assert_eq!(stats.total_size, processor.get_total_size(&blocks));
        assert!(stats.avg_block_size > 0);
    }
}

