//! Configuration management for zkIPFS-Proof CLI
//!
//! This module handles loading, saving, and managing configuration settings
//! for the CLI application. It supports both file-based and environment-based
//! configuration with sensible defaults.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, warn};

use zkipfs_proof_core::error::{ProofError, Result};

/// Main configuration structure for the CLI
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    /// Default security level for proof generation
    pub default_security_level: u32,
    
    /// Default prover type
    pub default_prover: String,
    
    /// Default compression type
    pub default_compression: String,
    
    /// Default output directory for proofs
    pub default_output_dir: Option<PathBuf>,
    
    /// Maximum memory usage in MB
    pub max_memory_mb: Option<u64>,
    
    /// Default timeout in seconds
    pub default_timeout_seconds: Option<u64>,
    
    /// Whether to use hardware acceleration by default
    pub use_hardware_acceleration: bool,
    
    /// Whether to include performance metrics by default
    pub include_metrics_by_default: bool,
    
    /// Custom metadata to include in all proofs
    pub default_custom_metadata: HashMap<String, serde_json::Value>,
    
    /// API endpoints and configuration
    pub api: ApiConfig,
    
    /// IPFS configuration
    pub ipfs: IpfsConfig,
    
    /// Logging configuration
    pub logging: LoggingConfig,
    
    /// Performance tuning settings
    pub performance: PerformanceConfig,
}

/// API configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ApiConfig {
    /// Bonsai API endpoint
    pub bonsai_endpoint: Option<String>,
    
    /// IPFS API endpoint
    pub ipfs_endpoint: Option<String>,
    
    /// Request timeout in seconds
    pub request_timeout_seconds: u64,
    
    /// Maximum retries for failed requests
    pub max_retries: u32,
}

/// IPFS configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IpfsConfig {
    /// IPFS API URL
    pub api_url: String,
    
    /// IPFS Gateway URL
    pub gateway_url: String,
    
    /// Request timeout in seconds
    pub timeout: u64,
    
    /// Whether to pin files by default
    pub auto_pin: bool,
    
    /// Custom headers for IPFS requests
    pub headers: HashMap<String, String>,
}

/// Logging configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LoggingConfig {
    /// Default log level
    pub level: String,
    
    /// Whether to log to file
    pub log_to_file: bool,
    
    /// Log file path
    pub log_file: Option<PathBuf>,
    
    /// Maximum log file size in MB
    pub max_log_size_mb: u64,
    
    /// Number of log files to keep
    pub log_file_count: u32,
}

/// Performance configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PerformanceConfig {
    /// Number of worker threads
    pub worker_threads: Option<usize>,
    
    /// Chunk size for file processing
    pub chunk_size_bytes: usize,
    
    /// Maximum file size to process in memory
    pub max_in_memory_size_mb: u64,
    
    /// Enable performance profiling
    pub enable_profiling: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_security_level: 128,
            default_prover: "local".to_string(),
            default_compression: "gzip".to_string(),
            default_output_dir: None,
            max_memory_mb: None,
            default_timeout_seconds: Some(600), // 10 minutes
            use_hardware_acceleration: true,
            include_metrics_by_default: false,
            default_custom_metadata: HashMap::new(),
            api: ApiConfig::default(),
            ipfs: IpfsConfig::default(),
            logging: LoggingConfig::default(),
            performance: PerformanceConfig::default(),
        }
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            bonsai_endpoint: None,
            ipfs_endpoint: Some("http://localhost:5001".to_string()),
            request_timeout_seconds: 30,
            max_retries: 3,
        }
    }
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

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            log_to_file: false,
            log_file: None,
            max_log_size_mb: 10,
            log_file_count: 5,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            worker_threads: None, // Use system default
            chunk_size_bytes: 64 * 1024, // 64KB
            max_in_memory_size_mb: 100,
            enable_profiling: false,
        }
    }
}

/// Load configuration from file or create default
pub async fn load_config(config_path: Option<&Path>) -> Result<Config> {
    let config_file = if let Some(path) = config_path {
        path.to_path_buf()
    } else {
        get_default_config_path()?
    };

    debug!("Loading configuration from: {}", config_file.display());

    if config_file.exists() {
        let content = fs::read_to_string(&config_file).await
            .map_err(|e| ProofError::file_error(
                format!("Failed to read config file: {}", config_file.display()),
                Some(e)
            ))?;

        let mut config: Config = toml::from_str(&content)
            .map_err(|e| ProofError::configuration_error(
                format!("Failed to parse config file: {}", e)
            ))?;

        // Override with environment variables
        override_with_env(&mut config)?;

        debug!("Configuration loaded successfully");
        Ok(config)
    } else {
        debug!("Config file not found, using defaults");
        let mut config = Config::default();
        override_with_env(&mut config)?;
        Ok(config)
    }
}

/// Save configuration to file
pub async fn save_config(config: &Config, config_path: Option<&Path>) -> Result<()> {
    let config_file = if let Some(path) = config_path {
        path.to_path_buf()
    } else {
        get_default_config_path()?
    };

    // Create config directory if it doesn't exist
    if let Some(parent) = config_file.parent() {
        fs::create_dir_all(parent).await
            .map_err(|e| ProofError::file_error(
                format!("Failed to create config directory: {}", parent.display()),
                Some(e)
            ))?;
    }

    let content = toml::to_string_pretty(config)
        .map_err(|e| ProofError::serialization_error(
            "Failed to serialize config",
            Some(Box::new(e))
        ))?;

    fs::write(&config_file, content).await
        .map_err(|e| ProofError::file_error(
            format!("Failed to write config file: {}", config_file.display()),
            Some(e)
        ))?;

    debug!("Configuration saved to: {}", config_file.display());
    Ok(())
}

/// Get the default configuration file path
fn get_default_config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| ProofError::configuration_error(
            "Could not determine config directory"
        ))?;

    Ok(config_dir.join("zkipfs-proof").join("config.toml"))
}

/// Override configuration with environment variables
fn override_with_env(config: &mut Config) -> Result<()> {
    // Security level
    if let Ok(level) = std::env::var("ZKIPFS_SECURITY_LEVEL") {
        config.default_security_level = level.parse()
            .map_err(|_| ProofError::configuration_error(
                "Invalid ZKIPFS_SECURITY_LEVEL environment variable"
            ))?;
    }

    // Prover type
    if let Ok(prover) = std::env::var("ZKIPFS_PROVER") {
        config.default_prover = prover;
    }

    // Compression
    if let Ok(compression) = std::env::var("ZKIPFS_COMPRESSION") {
        config.default_compression = compression;
    }

    // Output directory
    if let Ok(output_dir) = std::env::var("ZKIPFS_OUTPUT_DIR") {
        config.default_output_dir = Some(PathBuf::from(output_dir));
    }

    // Memory limit
    if let Ok(memory) = std::env::var("ZKIPFS_MAX_MEMORY_MB") {
        config.max_memory_mb = Some(memory.parse()
            .map_err(|_| ProofError::configuration_error(
                "Invalid ZKIPFS_MAX_MEMORY_MB environment variable"
            ))?);
    }

    // Timeout
    if let Ok(timeout) = std::env::var("ZKIPFS_TIMEOUT_SECONDS") {
        config.default_timeout_seconds = Some(timeout.parse()
            .map_err(|_| ProofError::configuration_error(
                "Invalid ZKIPFS_TIMEOUT_SECONDS environment variable"
            ))?);
    }

    // Hardware acceleration
    if let Ok(hw_accel) = std::env::var("ZKIPFS_USE_HARDWARE_ACCELERATION") {
        config.use_hardware_acceleration = hw_accel.to_lowercase() == "true";
    }

    // Bonsai endpoint
    if let Ok(endpoint) = std::env::var("BONSAI_API_URL") {
        config.api.bonsai_endpoint = Some(endpoint);
    }

    // IPFS endpoint
    if let Ok(endpoint) = std::env::var("IPFS_API_URL") {
        config.api.ipfs_endpoint = Some(endpoint);
    }

    // Log level
    if let Ok(level) = std::env::var("ZKIPFS_LOG_LEVEL") {
        config.logging.level = level;
    }

    Ok(())
}

/// Get a configuration value by key
pub fn get_config_value(config: &Config, key: &str) -> Option<String> {
    match key {
        "default_security_level" => Some(config.default_security_level.to_string()),
        "default_prover" => Some(config.default_prover.clone()),
        "default_compression" => Some(config.default_compression.clone()),
        "default_output_dir" => config.default_output_dir.as_ref().map(|p| p.display().to_string()),
        "max_memory_mb" => config.max_memory_mb.map(|m| m.to_string()),
        "default_timeout_seconds" => config.default_timeout_seconds.map(|t| t.to_string()),
        "use_hardware_acceleration" => Some(config.use_hardware_acceleration.to_string()),
        "include_metrics_by_default" => Some(config.include_metrics_by_default.to_string()),
        "bonsai_endpoint" => config.api.bonsai_endpoint.clone(),
        "ipfs_endpoint" => config.api.ipfs_endpoint.clone(),
        "request_timeout_seconds" => Some(config.api.request_timeout_seconds.to_string()),
        "max_retries" => Some(config.api.max_retries.to_string()),
        "log_level" => Some(config.logging.level.clone()),
        "log_to_file" => Some(config.logging.log_to_file.to_string()),
        "log_file" => config.logging.log_file.as_ref().map(|p| p.display().to_string()),
        "worker_threads" => config.performance.worker_threads.map(|t| t.to_string()),
        "chunk_size_bytes" => Some(config.performance.chunk_size_bytes.to_string()),
        "max_in_memory_size_mb" => Some(config.performance.max_in_memory_size_mb.to_string()),
        "enable_profiling" => Some(config.performance.enable_profiling.to_string()),
        _ => None,
    }
}

/// Set a configuration value by key
pub fn set_config_value(config: &mut Config, key: &str, value: &str) -> Result<()> {
    match key {
        "default_security_level" => {
            config.default_security_level = value.parse()
                .map_err(|_| ProofError::invalid_input_error(key, "Invalid security level"))?;
        }
        "default_prover" => {
            config.default_prover = value.to_string();
        }
        "default_compression" => {
            config.default_compression = value.to_string();
        }
        "default_output_dir" => {
            config.default_output_dir = if value.is_empty() { 
                None 
            } else { 
                Some(PathBuf::from(value)) 
            };
        }
        "max_memory_mb" => {
            config.max_memory_mb = if value.is_empty() {
                None
            } else {
                Some(value.parse()
                    .map_err(|_| ProofError::invalid_input_error(key, "Invalid memory limit"))?)
            };
        }
        "default_timeout_seconds" => {
            config.default_timeout_seconds = if value.is_empty() {
                None
            } else {
                Some(value.parse()
                    .map_err(|_| ProofError::invalid_input_error(key, "Invalid timeout"))?)
            };
        }
        "use_hardware_acceleration" => {
            config.use_hardware_acceleration = value.to_lowercase() == "true";
        }
        "include_metrics_by_default" => {
            config.include_metrics_by_default = value.to_lowercase() == "true";
        }
        "bonsai_endpoint" => {
            config.api.bonsai_endpoint = if value.is_empty() { None } else { Some(value.to_string()) };
        }
        "ipfs_endpoint" => {
            config.api.ipfs_endpoint = if value.is_empty() { None } else { Some(value.to_string()) };
        }
        "request_timeout_seconds" => {
            config.api.request_timeout_seconds = value.parse()
                .map_err(|_| ProofError::invalid_input_error(key, "Invalid timeout"))?;
        }
        "max_retries" => {
            config.api.max_retries = value.parse()
                .map_err(|_| ProofError::invalid_input_error(key, "Invalid retry count"))?;
        }
        "log_level" => {
            config.logging.level = value.to_string();
        }
        "log_to_file" => {
            config.logging.log_to_file = value.to_lowercase() == "true";
        }
        "log_file" => {
            config.logging.log_file = if value.is_empty() { None } else { Some(PathBuf::from(value)) };
        }
        "worker_threads" => {
            config.performance.worker_threads = if value.is_empty() {
                None
            } else {
                Some(value.parse()
                    .map_err(|_| ProofError::invalid_input_error(key, "Invalid thread count"))?)
            };
        }
        "chunk_size_bytes" => {
            config.performance.chunk_size_bytes = value.parse()
                .map_err(|_| ProofError::invalid_input_error(key, "Invalid chunk size"))?;
        }
        "max_in_memory_size_mb" => {
            config.performance.max_in_memory_size_mb = value.parse()
                .map_err(|_| ProofError::invalid_input_error(key, "Invalid memory size"))?;
        }
        "enable_profiling" => {
            config.performance.enable_profiling = value.to_lowercase() == "true";
        }
        _ => {
            return Err(ProofError::invalid_input_error(
                "key",
                format!("Unknown configuration key: {}", key)
            ));
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.default_security_level, 128);
        assert_eq!(config.default_prover, "local");
        assert_eq!(config.default_compression, "gzip");
        assert!(config.use_hardware_acceleration);
    }

    #[tokio::test]
    async fn test_config_save_load() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");
        
        let original_config = Config::default();
        save_config(&original_config, Some(&config_path)).await.unwrap();
        
        let loaded_config = load_config(Some(&config_path)).await.unwrap();
        assert_eq!(original_config.default_security_level, loaded_config.default_security_level);
        assert_eq!(original_config.default_prover, loaded_config.default_prover);
    }

    #[test]
    fn test_get_set_config_value() {
        let mut config = Config::default();
        
        // Test getting a value
        let value = get_config_value(&config, "default_security_level");
        assert_eq!(value, Some("128".to_string()));
        
        // Test setting a value
        set_config_value(&mut config, "default_security_level", "256").unwrap();
        assert_eq!(config.default_security_level, 256);
        
        // Test invalid key
        let result = set_config_value(&mut config, "invalid_key", "value");
        assert!(result.is_err());
    }

    #[test]
    fn test_env_override() {
        std::env::set_var("ZKIPFS_SECURITY_LEVEL", "256");
        std::env::set_var("ZKIPFS_PROVER", "bonsai");
        
        let mut config = Config::default();
        override_with_env(&mut config).unwrap();
        
        assert_eq!(config.default_security_level, 256);
        assert_eq!(config.default_prover, "bonsai");
        
        // Clean up
        std::env::remove_var("ZKIPFS_SECURITY_LEVEL");
        std::env::remove_var("ZKIPFS_PROVER");
    }
}

