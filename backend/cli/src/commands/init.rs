//! Init command implementation
//!
//! This module implements the `init` command which initializes the configuration
//! and sets up the zkIPFS-Proof environment.

use std::path::{Path, PathBuf};
use zkipfs_proof_core::error::{ProofError, Result};
use crate::{config::{Config, save_config}, commands::output};

/// Initialize configuration and setup
pub async fn execute(force: bool, config_dir: Option<&Path>) -> Result<()> {
    println!("🚀 Initializing zkIPFS-Proof...");
    
    // Determine config directory
    let config_path = if let Some(dir) = config_dir {
        dir.join("config.toml")
    } else {
        get_default_config_path()?
    };

    // Check if config already exists
    if config_path.exists() && !force {
        return Err(ProofError::configuration_error(
            format!("Configuration already exists at {}. Use --force to overwrite.", config_path.display())
        ));
    }

    // Create default configuration
    let config = Config::default();

    // Save configuration
    save_config(&config, Some(&config_path)).await?;

    println!("✅ Configuration initialized at: {}", config_path.display());
    println!();
    println!("📋 Default Settings:");
    println!("   Security Level: {} bits", config.default_security_level);
    println!("   Prover: {}", config.default_prover);
    println!("   Compression: {}", config.default_compression);
    println!("   Hardware Acceleration: {}", if config.use_hardware_acceleration { "Enabled" } else { "Disabled" });
    
    if let Some(ipfs_endpoint) = &config.api.ipfs_endpoint {
        println!("   IPFS Endpoint: {}", ipfs_endpoint);
    }

    println!();
    println!("🔧 Next Steps:");
    println!("   1. Review and customize your configuration:");
    println!("      zkipfs-proof config show");
    println!("   2. Test your setup:");
    println!("      zkipfs-proof info --system");
    println!("   3. Generate your first proof:");
    println!("      zkipfs-proof generate --file <your-file> --content \"pattern:your-secret\"");

    // Check system requirements
    check_system_requirements().await?;

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

/// Check system requirements and provide recommendations
async fn check_system_requirements() -> Result<()> {
    println!();
    println!("🔍 System Requirements Check:");

    // Check Rust installation
    let rust_available = std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);

    if rust_available {
        println!("   ✅ Rust compiler available");
    } else {
        println!("   ❌ Rust compiler not found");
        println!("      Install from: https://rustup.rs/");
    }

    // Check Cargo
    let cargo_available = std::process::Command::new("cargo")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);

    if cargo_available {
        println!("   ✅ Cargo package manager available");
    } else {
        println!("   ❌ Cargo not found (should come with Rust)");
    }

    // Check IPFS
    let ipfs_available = std::process::Command::new("ipfs")
        .arg("version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);

    if ipfs_available {
        println!("   ✅ IPFS available");
    } else {
        println!("   ⚠️  IPFS not found (optional but recommended)");
        println!("      Install from: https://ipfs.io/");
    }

    // Check hardware acceleration
    let cuda_available = std::env::var("CUDA_PATH").is_ok() || 
        std::process::Command::new("nvcc")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);

    if cuda_available {
        println!("   ✅ CUDA available (hardware acceleration)");
    } else {
        println!("   ⚠️  CUDA not found (optional, for GPU acceleration)");
    }

    let metal_available = cfg!(target_os = "macos");
    if metal_available {
        println!("   ✅ Metal available (hardware acceleration)");
    }

    // Check memory
    let available_memory = get_available_memory();
    if let Some(memory_gb) = available_memory.map(|m| m / (1024 * 1024 * 1024)) {
        if memory_gb >= 8 {
            println!("   ✅ Sufficient memory: {}GB", memory_gb);
        } else {
            println!("   ⚠️  Low memory: {}GB (8GB+ recommended)", memory_gb);
        }
    } else {
        println!("   ⚠️  Could not determine available memory");
    }

    // Check disk space
    if let Ok(current_dir) = std::env::current_dir() {
        if let Ok(metadata) = std::fs::metadata(&current_dir) {
            // This is a simplified check - in practice you'd use platform-specific APIs
            println!("   ℹ️  Check available disk space (proofs can be large)");
        }
    }

    println!();
    println!("💡 Recommendations:");
    
    if !rust_available || !cargo_available {
        println!("   • Install Rust and Cargo for building from source");
    }
    
    if !ipfs_available {
        println!("   • Install IPFS for distributed storage features");
    }
    
    if !cuda_available && !metal_available {
        println!("   • Consider GPU acceleration for faster proof generation");
    }

    println!("   • Ensure at least 8GB RAM for large file processing");
    println!("   • Allocate sufficient disk space for proof storage");

    Ok(())
}

/// Get available system memory in bytes (simplified implementation)
fn get_available_memory() -> Option<u64> {
    #[cfg(target_os = "linux")]
    {
        if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
            for line in meminfo.lines() {
                if line.starts_with("MemAvailable:") {
                    if let Some(kb_str) = line.split_whitespace().nth(1) {
                        if let Ok(kb) = kb_str.parse::<u64>() {
                            return Some(kb * 1024); // Convert KB to bytes
                        }
                    }
                }
            }
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        // On macOS, we could use sysctl, but this is a simplified implementation
        // For now, return None to indicate we couldn't determine memory
    }
    
    #[cfg(target_os = "windows")]
    {
        // On Windows, we could use GlobalMemoryStatusEx, but this is simplified
        // For now, return None to indicate we couldn't determine memory
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_init_new_config() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");
        
        let result = execute(false, Some(temp_dir.path())).await;
        assert!(result.is_ok());
        assert!(config_path.exists());
    }

    #[tokio::test]
    async fn test_init_existing_config_without_force() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        
        // Create existing config
        std::fs::write(&config_path, "test").unwrap();
        
        let result = execute(false, Some(temp_dir.path())).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_init_existing_config_with_force() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        
        // Create existing config
        std::fs::write(&config_path, "test").unwrap();
        
        let result = execute(true, Some(temp_dir.path())).await;
        assert!(result.is_ok());
        
        // Check that config was overwritten with proper content
        let content = std::fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("default_security_level"));
    }

    #[test]
    fn test_get_default_config_path() {
        let path = get_default_config_path();
        assert!(path.is_ok());
        
        let path = path.unwrap();
        assert!(path.to_string_lossy().contains("zkipfs-proof"));
        assert!(path.to_string_lossy().ends_with("config.toml"));
    }
}

