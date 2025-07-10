//! IPFS client integration for zkIPFS-Proof
//! 
//! This module provides a comprehensive IPFS client implementation that handles
//! file uploads, content addressing, and decentralized storage management.

use crate::{Result, ZkIPFSError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;

/// IPFS Content Identifier (CID) representation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Cid {
    /// The CID string representation
    pub cid: String,
    /// The hash algorithm used
    pub hash_algorithm: String,
    /// The codec used for encoding
    pub codec: String,
    /// The multibase encoding
    pub multibase: String,
}

impl Cid {
    /// Create a new CID from a string
    pub fn new(cid: String) -> Self {
        // Parse CID components (simplified implementation)
        Self {
            cid: cid.clone(),
            hash_algorithm: "sha2-256".to_string(),
            codec: "dag-pb".to_string(),
            multibase: "base58btc".to_string(),
        }
    }

    /// Get the CID as a string
    pub fn as_str(&self) -> &str {
        &self.cid
    }

    /// Validate the CID format
    pub fn is_valid(&self) -> bool {
        // Basic CID validation (simplified)
        self.cid.len() >= 46 && (self.cid.starts_with("Qm") || self.cid.starts_with("bafy"))
    }
}

/// IPFS node configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpfsConfig {
    /// IPFS API endpoint
    pub api_url: String,
    /// Gateway URL for content retrieval
    pub gateway_url: String,
    /// Timeout for operations in seconds
    pub timeout: u64,
    /// Whether to pin content by default
    pub auto_pin: bool,
    /// Custom headers for API requests
    pub headers: HashMap<String, String>,
}

impl Default for IpfsConfig {
    fn default() -> Self {
        Self {
            api_url: "http://127.0.0.1:5001".to_string(),
            gateway_url: "http://127.0.0.1:8080".to_string(),
            timeout: 300, // 5 minutes
            auto_pin: true,
            headers: HashMap::new(),
        }
    }
}

/// IPFS file metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpfsFile {
    /// Content Identifier
    pub cid: Cid,
    /// Original file name
    pub name: String,
    /// File size in bytes
    pub size: u64,
    /// MIME type
    pub mime_type: String,
    /// Upload timestamp
    pub uploaded_at: chrono::DateTime<chrono::Utc>,
    /// Whether the file is pinned
    pub pinned: bool,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// IPFS client for interacting with IPFS nodes
#[derive(Debug, Clone)]
pub struct IpfsClient {
    config: IpfsConfig,
    client: reqwest::Client,
}

impl IpfsClient {
    /// Create a new IPFS client with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(IpfsConfig::default())
    }

    /// Create a new IPFS client with custom configuration
    pub fn with_config(config: IpfsConfig) -> Result<Self> {
        let mut headers = reqwest::header::HeaderMap::new();
        
        // Add custom headers
        for (key, value) in &config.headers {
            let header_name = reqwest::header::HeaderName::from_bytes(key.as_bytes())
                .map_err(|e| ZkIPFSError::IpfsError(format!("Invalid header name: {}", e)))?;
            let header_value = reqwest::header::HeaderValue::from_str(value)
                .map_err(|e| ZkIPFSError::IpfsError(format!("Invalid header value: {}", e)))?;
            headers.insert(header_name, header_value);
        }

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout))
            .default_headers(headers)
            .build()
            .map_err(|e| ZkIPFSError::IpfsError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { config, client })
    }

    /// Check if the IPFS node is accessible
    pub async fn is_online(&self) -> bool {
        match self.client.post(&format!("{}/api/v0/version", self.config.api_url)).send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }

    /// Upload a file to IPFS
    pub async fn upload_file<P: AsRef<Path>>(&self, file_path: P) -> Result<IpfsFile> {
        let file_path = file_path.as_ref();
        let file_name = file_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Read file content
        let content = fs::read(file_path).await
            .map_err(|e| ZkIPFSError::IoError(format!("Failed to read file: {}", e)))?;

        self.upload_bytes(&content, &file_name).await
    }

    /// Upload bytes to IPFS
    pub async fn upload_bytes(&self, content: &[u8], name: &str) -> Result<IpfsFile> {
        // Create multipart form data
        let form = reqwest::multipart::Form::new()
            .part("file", reqwest::multipart::Part::bytes(content.to_vec())
                .file_name(name.to_string()));

        // Upload to IPFS
        let response = self.client
            .post(&format!("{}/api/v0/add", self.config.api_url))
            .query(&[("pin", self.config.auto_pin.to_string())])
            .multipart(form)
            .send()
            .await
            .map_err(|e| ZkIPFSError::IpfsError(format!("Upload failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ZkIPFSError::IpfsError(format!(
                "Upload failed with status: {}", 
                response.status()
            )));
        }

        // Parse response
        let response_text = response.text().await
            .map_err(|e| ZkIPFSError::IpfsError(format!("Failed to read response: {}", e)))?;

        // Parse IPFS response (simplified JSON parsing)
        let cid_str = self.extract_cid_from_response(&response_text)?;
        let cid = Cid::new(cid_str);

        // Detect MIME type
        let mime_type = self.detect_mime_type(content, name);

        Ok(IpfsFile {
            cid,
            name: name.to_string(),
            size: content.len() as u64,
            mime_type,
            uploaded_at: chrono::Utc::now(),
            pinned: self.config.auto_pin,
            metadata: HashMap::new(),
        })
    }

    /// Retrieve file content from IPFS
    pub async fn get_file(&self, cid: &Cid) -> Result<Vec<u8>> {
        let response = self.client
            .post(&format!("{}/api/v0/cat", self.config.api_url))
            .query(&[("arg", cid.as_str())])
            .send()
            .await
            .map_err(|e| ZkIPFSError::IpfsError(format!("Failed to retrieve file: {}", e)))?;

        if !response.status().is_success() {
            return Err(ZkIPFSError::IpfsError(format!(
                "Failed to retrieve file with status: {}", 
                response.status()
            )));
        }

        let content = response.bytes().await
            .map_err(|e| ZkIPFSError::IpfsError(format!("Failed to read file content: {}", e)))?;

        Ok(content.to_vec())
    }

    /// Pin a file in IPFS
    pub async fn pin_file(&self, cid: &Cid) -> Result<()> {
        let response = self.client
            .post(&format!("{}/api/v0/pin/add", self.config.api_url))
            .query(&[("arg", cid.as_str())])
            .send()
            .await
            .map_err(|e| ZkIPFSError::IpfsError(format!("Failed to pin file: {}", e)))?;

        if !response.status().is_success() {
            return Err(ZkIPFSError::IpfsError(format!(
                "Failed to pin file with status: {}", 
                response.status()
            )));
        }

        Ok(())
    }

    /// Unpin a file from IPFS
    pub async fn unpin_file(&self, cid: &Cid) -> Result<()> {
        let response = self.client
            .post(&format!("{}/api/v0/pin/rm", self.config.api_url))
            .query(&[("arg", cid.as_str())])
            .send()
            .await
            .map_err(|e| ZkIPFSError::IpfsError(format!("Failed to unpin file: {}", e)))?;

        if !response.status().is_success() {
            return Err(ZkIPFSError::IpfsError(format!(
                "Failed to unpin file with status: {}", 
                response.status()
            )));
        }

        Ok(())
    }

    /// List pinned files
    pub async fn list_pinned(&self) -> Result<Vec<Cid>> {
        let response = self.client
            .post(&format!("{}/api/v0/pin/ls", self.config.api_url))
            .send()
            .await
            .map_err(|e| ZkIPFSError::IpfsError(format!("Failed to list pinned files: {}", e)))?;

        if !response.status().is_success() {
            return Err(ZkIPFSError::IpfsError(format!(
                "Failed to list pinned files with status: {}", 
                response.status()
            )));
        }

        let response_text = response.text().await
            .map_err(|e| ZkIPFSError::IpfsError(format!("Failed to read response: {}", e)))?;

        // Parse pinned CIDs (simplified implementation)
        let cids = self.extract_cids_from_pin_response(&response_text)?;
        Ok(cids)
    }

    /// Get file statistics
    pub async fn stat_file(&self, cid: &Cid) -> Result<IpfsFileStat> {
        let response = self.client
            .post(&format!("{}/api/v0/object/stat", self.config.api_url))
            .query(&[("arg", cid.as_str())])
            .send()
            .await
            .map_err(|e| ZkIPFSError::IpfsError(format!("Failed to get file stats: {}", e)))?;

        if !response.status().is_success() {
            return Err(ZkIPFSError::IpfsError(format!(
                "Failed to get file stats with status: {}", 
                response.status()
            )));
        }

        let response_text = response.text().await
            .map_err(|e| ZkIPFSError::IpfsError(format!("Failed to read response: {}", e)))?;

        // Parse file statistics (simplified implementation)
        self.parse_file_stat(&response_text)
    }

    /// Generate a gateway URL for a file
    pub fn gateway_url(&self, cid: &Cid) -> String {
        format!("{}/ipfs/{}", self.config.gateway_url, cid.as_str())
    }

    // Helper methods

    fn extract_cid_from_response(&self, response: &str) -> Result<String> {
        // Simplified JSON parsing - in a real implementation, use serde_json
        if let Some(start) = response.find("\"Hash\":\"") {
            let start = start + 8;
            if let Some(end) = response[start..].find("\"") {
                return Ok(response[start..start + end].to_string());
            }
        }
        Err(ZkIPFSError::IpfsError("Failed to extract CID from response".to_string()))
    }

    fn extract_cids_from_pin_response(&self, response: &str) -> Result<Vec<Cid>> {
        // Simplified implementation - parse pinned CIDs from response
        let mut cids = Vec::new();
        for line in response.lines() {
            if let Some(start) = line.find("\"") {
                let start = start + 1;
                if let Some(end) = line[start..].find("\"") {
                    let cid_str = line[start..start + end].to_string();
                    if !cid_str.is_empty() {
                        cids.push(Cid::new(cid_str));
                    }
                }
            }
        }
        Ok(cids)
    }

    fn detect_mime_type(&self, content: &[u8], name: &str) -> String {
        // Simple MIME type detection based on file extension and content
        if let Some(ext) = Path::new(name).extension().and_then(|e| e.to_str()) {
            match ext.to_lowercase().as_str() {
                "pdf" => "application/pdf".to_string(),
                "json" => "application/json".to_string(),
                "txt" => "text/plain".to_string(),
                "md" => "text/markdown".to_string(),
                "csv" => "text/csv".to_string(),
                "jpg" | "jpeg" => "image/jpeg".to_string(),
                "png" => "image/png".to_string(),
                "gif" => "image/gif".to_string(),
                "mp4" => "video/mp4".to_string(),
                "mp3" => "audio/mpeg".to_string(),
                _ => "application/octet-stream".to_string(),
            }
        } else {
            // Basic content-based detection
            if content.starts_with(b"%PDF") {
                "application/pdf".to_string()
            } else if content.starts_with(b"{") || content.starts_with(b"[") {
                "application/json".to_string()
            } else {
                "application/octet-stream".to_string()
            }
        }
    }

    fn parse_file_stat(&self, response: &str) -> Result<IpfsFileStat> {
        // Simplified JSON parsing for file statistics
        let size = if let Some(start) = response.find("\"CumulativeSize\":") {
            let start = start + 17;
            if let Some(end) = response[start..].find(",") {
                response[start..start + end].trim().parse::<u64>().unwrap_or(0)
            } else {
                0
            }
        } else {
            0
        };

        let block_count = if let Some(start) = response.find("\"NumLinks\":") {
            let start = start + 11;
            if let Some(end) = response[start..].find(",") {
                response[start..start + end].trim().parse::<u64>().unwrap_or(0)
            } else {
                0
            }
        } else {
            0
        };

        Ok(IpfsFileStat {
            size,
            block_count,
            link_count: block_count,
        })
    }
}

/// IPFS file statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpfsFileStat {
    /// Total file size in bytes
    pub size: u64,
    /// Number of blocks
    pub block_count: u64,
    /// Number of links
    pub link_count: u64,
}

/// IPFS content manager for handling file operations
#[derive(Debug)]
pub struct IpfsContentManager {
    client: IpfsClient,
    file_cache: HashMap<Cid, IpfsFile>,
}

impl IpfsContentManager {
    /// Create a new content manager
    pub fn new(client: IpfsClient) -> Self {
        Self {
            client,
            file_cache: HashMap::new(),
        }
    }

    /// Upload and track a file
    pub async fn upload_and_track<P: AsRef<Path>>(&mut self, file_path: P) -> Result<IpfsFile> {
        let ipfs_file = self.client.upload_file(file_path).await?;
        self.file_cache.insert(ipfs_file.cid.clone(), ipfs_file.clone());
        Ok(ipfs_file)
    }

    /// Get a tracked file
    pub fn get_tracked_file(&self, cid: &Cid) -> Option<&IpfsFile> {
        self.file_cache.get(cid)
    }

    /// List all tracked files
    pub fn list_tracked_files(&self) -> Vec<&IpfsFile> {
        self.file_cache.values().collect()
    }

    /// Remove a file from tracking
    pub fn untrack_file(&mut self, cid: &Cid) -> Option<IpfsFile> {
        self.file_cache.remove(cid)
    }

    /// Get the underlying IPFS client
    pub fn client(&self) -> &IpfsClient {
        &self.client
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cid_creation() {
        let cid = Cid::new("QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG".to_string());
        assert_eq!(cid.as_str(), "QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG");
        assert!(cid.is_valid());
    }

    #[test]
    fn test_cid_validation() {
        let valid_cid = Cid::new("QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG".to_string());
        assert!(valid_cid.is_valid());

        let invalid_cid = Cid::new("invalid".to_string());
        assert!(!invalid_cid.is_valid());
    }

    #[test]
    fn test_ipfs_config_default() {
        let config = IpfsConfig::default();
        assert_eq!(config.api_url, "http://127.0.0.1:5001");
        assert_eq!(config.gateway_url, "http://127.0.0.1:8080");
        assert_eq!(config.timeout, 300);
        assert!(config.auto_pin);
    }
}

