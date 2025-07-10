//! Caching system for zkIPFS-Proof
//!
//! This module provides intelligent caching for proof generation,
//! IPFS content, and intermediate results to improve performance.

use crate::{error::Result, types::*};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::fs;
use tracing::{debug, info, warn};

/// Cache manager for zkIPFS-Proof operations
#[derive(Debug)]
pub struct CacheManager {
    /// Cache configuration
    config: CacheConfig,
    /// In-memory cache for frequently accessed data
    memory_cache: HashMap<String, CacheEntry>,
    /// Disk cache directory
    disk_cache_dir: PathBuf,
    /// Cache statistics
    stats: CacheStatistics,
}

/// Configuration for the cache system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Enable memory caching
    pub memory_cache_enabled: bool,
    /// Maximum memory cache size in bytes
    pub max_memory_cache_bytes: u64,
    /// Enable disk caching
    pub disk_cache_enabled: bool,
    /// Maximum disk cache size in bytes
    pub max_disk_cache_bytes: u64,
    /// Cache entry TTL (time to live)
    pub entry_ttl_seconds: u64,
    /// Enable cache compression
    pub compression_enabled: bool,
    /// Cache cleanup interval
    pub cleanup_interval_seconds: u64,
}

/// Cache entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Cached data
    pub data: Vec<u8>,
    /// Entry creation time
    pub created_at: SystemTime,
    /// Last access time
    pub last_accessed: SystemTime,
    /// Access count
    pub access_count: u64,
    /// Data size in bytes
    pub size_bytes: u64,
    /// Whether data is compressed
    pub compressed: bool,
}

/// Cache statistics for monitoring and optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStatistics {
    /// Total cache hits
    pub hits: u64,
    /// Total cache misses
    pub misses: u64,
    /// Total entries stored
    pub total_entries: u64,
    /// Total bytes cached
    pub total_bytes_cached: u64,
    /// Memory cache usage
    pub memory_cache_bytes: u64,
    /// Disk cache usage
    pub disk_cache_bytes: u64,
    /// Cache hit ratio
    pub hit_ratio: f64,
    /// Average entry size
    pub avg_entry_size_bytes: u64,
}

/// Types of cacheable data
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum CacheKey {
    /// IPFS file processing result
    IpfsFile(String), // File hash
    /// Proof generation result
    Proof(String), // Content hash
    /// Content selection result
    ContentSelection(String), // Selection hash
    /// Verification result
    Verification(String), // Proof ID
}

impl CacheManager {
    /// Creates a new cache manager with default configuration
    pub async fn new() -> Result<Self> {
        let config = CacheConfig::default();
        Self::with_config(config).await
    }

    /// Creates a new cache manager with custom configuration
    pub async fn with_config(config: CacheConfig) -> Result<Self> {
        let cache_dir = std::env::temp_dir().join("zkipfs-proof-cache");
        
        // Create cache directory if it doesn't exist
        if config.disk_cache_enabled {
            fs::create_dir_all(&cache_dir).await?;
        }

        let mut manager = Self {
            config,
            memory_cache: HashMap::new(),
            disk_cache_dir: cache_dir,
            stats: CacheStatistics::default(),
        };

        // Load existing disk cache statistics
        if manager.config.disk_cache_enabled {
            manager.load_disk_cache_stats().await?;
        }

        info!("Cache manager initialized with {} memory cache and {} disk cache",
              if manager.config.memory_cache_enabled { "enabled" } else { "disabled" },
              if manager.config.disk_cache_enabled { "enabled" } else { "disabled" });

        Ok(manager)
    }

    /// Stores data in the cache
    pub async fn store<T: Serialize>(&mut self, key: CacheKey, data: &T) -> Result<()> {
        let serialized = bincode::serialize(data)
            .map_err(|e| crate::error::ProofError::serialization_error(
                "Failed to serialize cache data",
                Some(Box::new(e))
            ))?;

        let compressed_data = if self.config.compression_enabled {
            self.compress_data(&serialized)?
        } else {
            serialized
        };

        let entry = CacheEntry {
            data: compressed_data.clone(),
            created_at: SystemTime::now(),
            last_accessed: SystemTime::now(),
            access_count: 0,
            size_bytes: compressed_data.len() as u64,
            compressed: self.config.compression_enabled,
        };

        let key_str = self.key_to_string(&key);

        // Store in memory cache if enabled and within size limits
        if self.config.memory_cache_enabled {
            if self.stats.memory_cache_bytes + entry.size_bytes <= self.config.max_memory_cache_bytes {
                self.memory_cache.insert(key_str.clone(), entry.clone());
                self.stats.memory_cache_bytes += entry.size_bytes;
                debug!("Stored {} bytes in memory cache for key: {}", entry.size_bytes, key_str);
            } else {
                // Evict least recently used entries
                self.evict_memory_cache_entries(entry.size_bytes).await?;
                self.memory_cache.insert(key_str.clone(), entry.clone());
                self.stats.memory_cache_bytes += entry.size_bytes;
            }
        }

        // Store in disk cache if enabled
        if self.config.disk_cache_enabled {
            self.store_disk_cache(&key_str, &entry).await?;
        }

        self.stats.total_entries += 1;
        self.stats.total_bytes_cached += entry.size_bytes;
        self.update_cache_stats();

        Ok(())
    }

    /// Retrieves data from the cache
    pub async fn retrieve<T: for<'de> Deserialize<'de>>(&mut self, key: &CacheKey) -> Result<Option<T>> {
        let key_str = self.key_to_string(key);

        // Check memory cache first
        if self.config.memory_cache_enabled {
            if let Some(entry) = self.memory_cache.get_mut(&key_str) {
                // Update access statistics
                entry.last_accessed = SystemTime::now();
                entry.access_count += 1;

                let data = if entry.compressed {
                    self.decompress_data(&entry.data)?
                } else {
                    entry.data.clone()
                };

                let result: T = bincode::deserialize(&data)
                    .map_err(|e| crate::error::ProofError::serialization_error(
                        "Failed to deserialize cache data",
                        Some(Box::new(e))
                    ))?;

                self.stats.hits += 1;
                self.update_cache_stats();
                debug!("Cache hit in memory for key: {}", key_str);
                return Ok(Some(result));
            }
        }

        // Check disk cache
        if self.config.disk_cache_enabled {
            if let Some(entry) = self.retrieve_disk_cache(&key_str).await? {
                // Check if entry is still valid (not expired)
                if !self.is_entry_expired(&entry) {
                    let data = if entry.compressed {
                        self.decompress_data(&entry.data)?
                    } else {
                        entry.data.clone()
                    };

                    let result: T = bincode::deserialize(&data)
                        .map_err(|e| crate::error::ProofError::serialization_error(
                            "Failed to deserialize cache data",
                            Some(Box::new(e))
                        ))?;

                    // Promote to memory cache if enabled
                    if self.config.memory_cache_enabled {
                        let mut updated_entry = entry;
                        updated_entry.last_accessed = SystemTime::now();
                        updated_entry.access_count += 1;
                        
                        if self.stats.memory_cache_bytes + updated_entry.size_bytes <= self.config.max_memory_cache_bytes {
                            self.memory_cache.insert(key_str.clone(), updated_entry);
                            self.stats.memory_cache_bytes += updated_entry.size_bytes;
                        }
                    }

                    self.stats.hits += 1;
                    self.update_cache_stats();
                    debug!("Cache hit in disk for key: {}", key_str);
                    return Ok(Some(result));
                } else {
                    // Remove expired entry
                    self.remove_disk_cache(&key_str).await?;
                }
            }
        }

        self.stats.misses += 1;
        self.update_cache_stats();
        debug!("Cache miss for key: {}", key_str);
        Ok(None)
    }

    /// Removes an entry from the cache
    pub async fn remove(&mut self, key: &CacheKey) -> Result<bool> {
        let key_str = self.key_to_string(key);
        let mut removed = false;

        // Remove from memory cache
        if let Some(entry) = self.memory_cache.remove(&key_str) {
            self.stats.memory_cache_bytes -= entry.size_bytes;
            self.stats.total_bytes_cached -= entry.size_bytes;
            removed = true;
        }

        // Remove from disk cache
        if self.config.disk_cache_enabled {
            if self.remove_disk_cache(&key_str).await? {
                removed = true;
            }
        }

        if removed {
            self.stats.total_entries = self.stats.total_entries.saturating_sub(1);
            self.update_cache_stats();
        }

        Ok(removed)
    }

    /// Clears all cache entries
    pub async fn clear(&mut self) -> Result<()> {
        // Clear memory cache
        self.memory_cache.clear();
        self.stats.memory_cache_bytes = 0;

        // Clear disk cache
        if self.config.disk_cache_enabled {
            if self.disk_cache_dir.exists() {
                fs::remove_dir_all(&self.disk_cache_dir).await?;
                fs::create_dir_all(&self.disk_cache_dir).await?;
            }
            self.stats.disk_cache_bytes = 0;
        }

        self.stats.total_entries = 0;
        self.stats.total_bytes_cached = 0;
        self.update_cache_stats();

        info!("Cache cleared");
        Ok(())
    }

    /// Performs cache cleanup (removes expired entries)
    pub async fn cleanup(&mut self) -> Result<u64> {
        let mut removed_count = 0;

        // Cleanup memory cache
        let expired_keys: Vec<String> = self.memory_cache.iter()
            .filter(|(_, entry)| self.is_entry_expired(entry))
            .map(|(key, _)| key.clone())
            .collect();

        for key in expired_keys {
            if let Some(entry) = self.memory_cache.remove(&key) {
                self.stats.memory_cache_bytes -= entry.size_bytes;
                self.stats.total_bytes_cached -= entry.size_bytes;
                removed_count += 1;
            }
        }

        // Cleanup disk cache
        if self.config.disk_cache_enabled {
            removed_count += self.cleanup_disk_cache().await?;
        }

        self.stats.total_entries = self.stats.total_entries.saturating_sub(removed_count);
        self.update_cache_stats();

        if removed_count > 0 {
            info!("Cache cleanup removed {} expired entries", removed_count);
        }

        Ok(removed_count)
    }

    /// Gets cache statistics
    pub fn get_statistics(&self) -> &CacheStatistics {
        &self.stats
    }

    /// Updates cache configuration
    pub fn update_config(&mut self, config: CacheConfig) {
        self.config = config;
        info!("Cache configuration updated");
    }

    /// Converts cache key to string representation
    fn key_to_string(&self, key: &CacheKey) -> String {
        match key {
            CacheKey::IpfsFile(hash) => format!("ipfs_file_{}", hash),
            CacheKey::Proof(hash) => format!("proof_{}", hash),
            CacheKey::ContentSelection(hash) => format!("content_selection_{}", hash),
            CacheKey::Verification(id) => format!("verification_{}", id),
        }
    }

    /// Checks if a cache entry is expired
    fn is_entry_expired(&self, entry: &CacheEntry) -> bool {
        if let Ok(elapsed) = entry.created_at.elapsed() {
            elapsed.as_secs() > self.config.entry_ttl_seconds
        } else {
            true // Consider invalid timestamps as expired
        }
    }

    /// Compresses data using a simple compression algorithm
    fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Simple compression using flate2 (gzip)
        use std::io::Write;
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(data)?;
        Ok(encoder.finish()?)
    }

    /// Decompresses data
    fn decompress_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        use std::io::Read;
        let mut decoder = flate2::read::GzDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        Ok(decompressed)
    }

    /// Evicts entries from memory cache to make space
    async fn evict_memory_cache_entries(&mut self, needed_bytes: u64) -> Result<()> {
        let mut entries_to_remove = Vec::new();
        let mut freed_bytes = 0u64;

        // Sort entries by last access time (LRU)
        let mut sorted_entries: Vec<_> = self.memory_cache.iter().collect();
        sorted_entries.sort_by_key(|(_, entry)| entry.last_accessed);

        for (key, entry) in sorted_entries {
            entries_to_remove.push(key.clone());
            freed_bytes += entry.size_bytes;
            
            if freed_bytes >= needed_bytes {
                break;
            }
        }

        // Remove selected entries
        for key in entries_to_remove {
            if let Some(entry) = self.memory_cache.remove(&key) {
                self.stats.memory_cache_bytes -= entry.size_bytes;
            }
        }

        debug!("Evicted {} bytes from memory cache", freed_bytes);
        Ok(())
    }

    /// Stores entry in disk cache
    async fn store_disk_cache(&mut self, key: &str, entry: &CacheEntry) -> Result<()> {
        let file_path = self.disk_cache_dir.join(format!("{}.cache", key));
        let serialized = bincode::serialize(entry)
            .map_err(|e| crate::error::ProofError::serialization_error(
                "Failed to serialize cache entry",
                Some(Box::new(e))
            ))?;

        fs::write(&file_path, serialized).await?;
        self.stats.disk_cache_bytes += entry.size_bytes;
        
        debug!("Stored {} bytes in disk cache for key: {}", entry.size_bytes, key);
        Ok(())
    }

    /// Retrieves entry from disk cache
    async fn retrieve_disk_cache(&self, key: &str) -> Result<Option<CacheEntry>> {
        let file_path = self.disk_cache_dir.join(format!("{}.cache", key));
        
        if !file_path.exists() {
            return Ok(None);
        }

        let data = fs::read(&file_path).await?;
        let entry: CacheEntry = bincode::deserialize(&data)
            .map_err(|e| crate::error::ProofError::serialization_error(
                "Failed to deserialize cache entry",
                Some(Box::new(e))
            ))?;

        Ok(Some(entry))
    }

    /// Removes entry from disk cache
    async fn remove_disk_cache(&mut self, key: &str) -> Result<bool> {
        let file_path = self.disk_cache_dir.join(format!("{}.cache", key));
        
        if file_path.exists() {
            if let Ok(entry) = self.retrieve_disk_cache(key).await? {
                self.stats.disk_cache_bytes -= entry.size_bytes;
            }
            fs::remove_file(&file_path).await?;
            return Ok(true);
        }

        Ok(false)
    }

    /// Cleans up expired entries from disk cache
    async fn cleanup_disk_cache(&mut self) -> Result<u64> {
        let mut removed_count = 0;

        if !self.disk_cache_dir.exists() {
            return Ok(0);
        }

        let mut entries = fs::read_dir(&self.disk_cache_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            if let Some(extension) = entry.path().extension() {
                if extension == "cache" {
                    if let Ok(Some(cache_entry)) = self.retrieve_disk_cache(
                        &entry.file_name().to_string_lossy().replace(".cache", "")
                    ).await {
                        if self.is_entry_expired(&cache_entry) {
                            fs::remove_file(entry.path()).await?;
                            self.stats.disk_cache_bytes -= cache_entry.size_bytes;
                            removed_count += 1;
                        }
                    }
                }
            }
        }

        Ok(removed_count)
    }

    /// Loads disk cache statistics
    async fn load_disk_cache_stats(&mut self) -> Result<()> {
        if !self.disk_cache_dir.exists() {
            return Ok(());
        }

        let mut total_size = 0u64;
        let mut entries = fs::read_dir(&self.disk_cache_dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            if let Some(extension) = entry.path().extension() {
                if extension == "cache" {
                    if let Ok(metadata) = entry.metadata().await {
                        total_size += metadata.len();
                    }
                }
            }
        }

        self.stats.disk_cache_bytes = total_size;
        Ok(())
    }

    /// Updates cache statistics
    fn update_cache_stats(&mut self) {
        let total_requests = self.stats.hits + self.stats.misses;
        self.stats.hit_ratio = if total_requests > 0 {
            self.stats.hits as f64 / total_requests as f64
        } else {
            0.0
        };

        self.stats.avg_entry_size_bytes = if self.stats.total_entries > 0 {
            self.stats.total_bytes_cached / self.stats.total_entries
        } else {
            0
        };
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            memory_cache_enabled: true,
            max_memory_cache_bytes: 512 * 1024 * 1024, // 512MB
            disk_cache_enabled: true,
            max_disk_cache_bytes: 2 * 1024 * 1024 * 1024, // 2GB
            entry_ttl_seconds: 24 * 60 * 60, // 24 hours
            compression_enabled: true,
            cleanup_interval_seconds: 60 * 60, // 1 hour
        }
    }
}

impl Default for CacheStatistics {
    fn default() -> Self {
        Self {
            hits: 0,
            misses: 0,
            total_entries: 0,
            total_bytes_cached: 0,
            memory_cache_bytes: 0,
            disk_cache_bytes: 0,
            hit_ratio: 0.0,
            avg_entry_size_bytes: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_cache_manager_creation() {
        let manager = CacheManager::new().await;
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_cache_store_and_retrieve() {
        let mut manager = CacheManager::new().await.unwrap();
        let key = CacheKey::IpfsFile("test_hash".to_string());
        let data = vec![1, 2, 3, 4, 5];

        // Store data
        assert!(manager.store(key.clone(), &data).await.is_ok());

        // Retrieve data
        let retrieved: Option<Vec<u8>> = manager.retrieve(&key).await.unwrap();
        assert_eq!(retrieved, Some(data));

        // Check statistics
        let stats = manager.get_statistics();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 0);
    }

    #[tokio::test]
    async fn test_cache_miss() {
        let mut manager = CacheManager::new().await.unwrap();
        let key = CacheKey::Proof("nonexistent".to_string());

        let retrieved: Option<Vec<u8>> = manager.retrieve(&key).await.unwrap();
        assert_eq!(retrieved, None);

        let stats = manager.get_statistics();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 1);
    }
}

