//! Info command implementation
//!
//! This module implements the `info` command which displays information about
//! proofs, files, and system status.

use clap::Args;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::info;

use zkipfs_proof_core::{Proof, error::Result};
use crate::{
    config::Config,
    utils::{validate_file_path, format_bytes, format_duration, format_hash, get_system_info, get_file_size},
    commands::{Command, output},
};

/// Display information about proofs, files, or system status
#[derive(Args, Debug)]
pub struct InfoCommand {
    /// Path to proof file to analyze
    #[arg(short, long, value_name = "FILE")]
    pub proof: Option<PathBuf>,

    /// Path to file to analyze
    #[arg(short, long, value_name = "FILE")]
    pub file: Option<PathBuf>,

    /// Show system information
    #[arg(long)]
    pub system: bool,

    /// Show configuration information
    #[arg(long)]
    pub config: bool,

    /// Show detailed information
    #[arg(long)]
    pub detailed: bool,

    /// Show performance metrics
    #[arg(long)]
    pub metrics: bool,

    /// Show security information
    #[arg(long)]
    pub security: bool,

    /// Verify proof integrity (quick check)
    #[arg(long)]
    pub verify_integrity: bool,
}

#[derive(Serialize, Deserialize)]
struct InfoOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    proof_info: Option<ProofInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    file_info: Option<FileInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_info: Option<SystemInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    config_info: Option<ConfigInfo>,
}

#[derive(Serialize, Deserialize)]
struct ProofInfo {
    id: String,
    version: String,
    created_at: String,
    content_selection: String,
    content_hash: String,
    root_hash: String,
    file_info: ProofFileInfo,
    security: SecurityInfo,
    performance: PerformanceInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    integrity_check: Option<IntegrityCheck>,
}

#[derive(Serialize, Deserialize)]
struct ProofFileInfo {
    filename: Option<String>,
    size_bytes: u64,
    mime_type: Option<String>,
    ipfs_cid: String,
    block_count: u64,
    avg_block_size: u64,
}

#[derive(Serialize, Deserialize)]
struct SecurityInfo {
    security_level: u32,
    hash_function: String,
    proof_system: String,
    risc0_version: String,
    formal_verification: bool,
}

#[derive(Serialize, Deserialize)]
struct PerformanceInfo {
    generation_time_ms: u64,
    file_processing_time_ms: u64,
    zk_generation_time_ms: u64,
    peak_memory_bytes: u64,
    zk_cycles: u64,
    proof_size_bytes: u64,
    compression_ratio: Option<f64>,
}

#[derive(Serialize, Deserialize)]
struct FileInfo {
    path: String,
    size_bytes: u64,
    size_human: String,
    exists: bool,
    readable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hash_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    estimated_ipfs_blocks: Option<u64>,
}

#[derive(Serialize, Deserialize)]
struct SystemInfo {
    os: String,
    arch: String,
    cpu_count: usize,
    available_memory_bytes: Option<u64>,
    available_memory_human: Option<String>,
    zkipfs_version: String,
    risc0_available: bool,
    hardware_acceleration: HardwareAcceleration,
    ipfs_available: bool,
}

#[derive(Serialize, Deserialize)]
struct HardwareAcceleration {
    cuda_available: bool,
    metal_available: bool,
    recommended: String,
}

#[derive(Serialize, Deserialize)]
struct ConfigInfo {
    config_file_path: Option<String>,
    default_security_level: u32,
    default_prover: String,
    default_compression: String,
    use_hardware_acceleration: bool,
    ipfs_endpoint: Option<String>,
    bonsai_endpoint: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct IntegrityCheck {
    valid_structure: bool,
    valid_json: bool,
    has_required_fields: bool,
    proof_size_reasonable: bool,
    timestamp_reasonable: bool,
    issues: Vec<String>,
}

impl Command for InfoCommand {
    async fn execute(&self, config: &Config, output_format: &str) -> Result<()> {
        info!("Gathering information...");

        let mut output_data = InfoOutput {
            proof_info: None,
            file_info: None,
            system_info: None,
            config_info: None,
        };

        // Gather proof information
        if let Some(proof_path) = &self.proof {
            output_data.proof_info = Some(self.gather_proof_info(proof_path).await?);
        }

        // Gather file information
        if let Some(file_path) = &self.file {
            output_data.file_info = Some(self.gather_file_info(file_path).await?);
        }

        // Gather system information
        if self.system {
            output_data.system_info = Some(self.gather_system_info().await?);
        }

        // Gather configuration information
        if self.config {
            output_data.config_info = Some(self.gather_config_info(config).await?);
        }

        // If no specific info requested, show system info by default
        if output_data.proof_info.is_none() 
            && output_data.file_info.is_none() 
            && output_data.system_info.is_none() 
            && output_data.config_info.is_none() {
            output_data.system_info = Some(self.gather_system_info().await?);
        }

        // Print output based on format
        match output_format {
            "table" => self.print_table_output(&output_data),
            _ => output::print_output(&output_data, output_format, true)?,
        }

        Ok(())
    }
}

impl InfoCommand {
    /// Gather information about a proof file
    async fn gather_proof_info(&self, proof_path: &PathBuf) -> Result<ProofInfo> {
        validate_file_path(proof_path)?;

        // Load proof
        let content = std::fs::read_to_string(proof_path)
            .map_err(|e| zkipfs_proof_core::error::ProofError::file_error(
                format!("Failed to read proof file: {}", proof_path.display()),
                Some(e)
            ))?;

        let proof: Proof = serde_json::from_str(&content)
            .map_err(|e| zkipfs_proof_core::error::ProofError::serialization_error(
                "Failed to parse proof file",
                Some(Box::new(e))
            ))?;

        // Perform integrity check if requested
        let integrity_check = if self.verify_integrity {
            Some(self.check_proof_integrity(&proof, &content))
        } else {
            None
        };

        Ok(ProofInfo {
            id: proof.id.clone(),
            version: proof.version.clone(),
            created_at: proof.created_at.to_rfc3339(),
            content_selection: proof.content_selection.description(),
            content_hash: format_hash(&proof.content_hash, Some(16)),
            root_hash: format_hash(&proof.root_hash, Some(16)),
            file_info: ProofFileInfo {
                filename: proof.metadata.file_info.filename.clone(),
                size_bytes: proof.metadata.file_info.size,
                mime_type: proof.metadata.file_info.mime_type.clone(),
                ipfs_cid: proof.metadata.file_info.ipfs_cid.clone(),
                block_count: proof.metadata.file_info.block_count,
                avg_block_size: proof.metadata.file_info.avg_block_size,
            },
            security: SecurityInfo {
                security_level: proof.metadata.security.security_level,
                hash_function: proof.metadata.security.hash_function.clone(),
                proof_system: proof.metadata.security.proof_system.clone(),
                risc0_version: proof.metadata.security.risc0_version.clone(),
                formal_verification: proof.metadata.security.formal_verification,
            },
            performance: PerformanceInfo {
                generation_time_ms: proof.metadata.performance.generation_time_ms,
                file_processing_time_ms: proof.metadata.performance.file_processing_time_ms,
                zk_generation_time_ms: proof.metadata.performance.zk_generation_time_ms,
                peak_memory_bytes: proof.metadata.performance.peak_memory_bytes,
                zk_cycles: proof.metadata.performance.zk_cycles,
                proof_size_bytes: proof.metadata.performance.proof_size_bytes,
                compression_ratio: proof.metadata.performance.compression_ratio,
            },
            integrity_check,
        })
    }

    /// Gather information about a file
    async fn gather_file_info(&self, file_path: &PathBuf) -> Result<FileInfo> {
        let exists = file_path.exists();
        let readable = if exists {
            std::fs::File::open(file_path).is_ok()
        } else {
            false
        };

        let size_bytes = if exists && readable {
            get_file_size(file_path)?
        } else {
            0
        };

        let hash_sha256 = if exists && readable && size_bytes < 100 * 1024 * 1024 { // Only hash files < 100MB
            let content = std::fs::read(file_path).ok();
            content.map(|data| {
                use sha2::{Digest, Sha256};
                let hash = Sha256::digest(&data);
                format_hash(hash.as_slice(), Some(16))
            })
        } else {
            None
        };

        let mime_type = if exists {
            // Simple MIME type detection based on extension
            file_path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| match ext.to_lowercase().as_str() {
                    "txt" => "text/plain",
                    "json" => "application/json",
                    "pdf" => "application/pdf",
                    "png" => "image/png",
                    "jpg" | "jpeg" => "image/jpeg",
                    "mp4" => "video/mp4",
                    "csv" => "text/csv",
                    _ => "application/octet-stream",
                }.to_string())
        } else {
            None
        };

        let estimated_ipfs_blocks = if size_bytes > 0 {
            // IPFS default block size is 256KB
            Some((size_bytes + 262143) / 262144) // Round up
        } else {
            None
        };

        Ok(FileInfo {
            path: file_path.display().to_string(),
            size_bytes,
            size_human: format_bytes(size_bytes),
            exists,
            readable,
            mime_type,
            hash_sha256,
            estimated_ipfs_blocks,
        })
    }

    /// Gather system information
    async fn gather_system_info(&self) -> Result<SystemInfo> {
        let sys_info = get_system_info();
        
        // Check for Risc0 availability
        let risc0_available = std::process::Command::new("cargo")
            .args(&["--version"])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);

        // Check hardware acceleration
        let cuda_available = std::env::var("CUDA_PATH").is_ok() || 
            std::process::Command::new("nvcc")
                .arg("--version")
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false);

        let metal_available = cfg!(target_os = "macos");

        let recommended_acceleration = if cuda_available {
            "CUDA"
        } else if metal_available {
            "Metal"
        } else {
            "CPU"
        }.to_string();

        // Check IPFS availability
        let ipfs_available = std::process::Command::new("ipfs")
            .arg("version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);

        Ok(SystemInfo {
            os: sys_info.os,
            arch: sys_info.arch,
            cpu_count: sys_info.cpu_count,
            available_memory_bytes: sys_info.available_memory,
            available_memory_human: sys_info.available_memory.map(format_bytes),
            zkipfs_version: env!("CARGO_PKG_VERSION").to_string(),
            risc0_available,
            hardware_acceleration: HardwareAcceleration {
                cuda_available,
                metal_available,
                recommended: recommended_acceleration,
            },
            ipfs_available,
        })
    }

    /// Gather configuration information
    async fn gather_config_info(&self, config: &Config) -> Result<ConfigInfo> {
        let config_file_path = dirs::config_dir()
            .map(|dir| dir.join("zkipfs-proof").join("config.toml"))
            .filter(|path| path.exists())
            .map(|path| path.display().to_string());

        Ok(ConfigInfo {
            config_file_path,
            default_security_level: config.default_security_level,
            default_prover: config.default_prover.clone(),
            default_compression: config.default_compression.clone(),
            use_hardware_acceleration: config.use_hardware_acceleration,
            ipfs_endpoint: config.api.ipfs_endpoint.clone(),
            bonsai_endpoint: config.api.bonsai_endpoint.clone(),
        })
    }

    /// Check proof integrity
    fn check_proof_integrity(&self, proof: &Proof, json_content: &str) -> IntegrityCheck {
        let mut issues = Vec::new();

        // Check JSON validity
        let valid_json = serde_json::from_str::<serde_json::Value>(json_content).is_ok();
        if !valid_json {
            issues.push("Invalid JSON format".to_string());
        }

        // Check required fields
        let has_required_fields = !proof.id.is_empty() 
            && !proof.version.is_empty()
            && !proof.zk_proof.receipt.is_empty()
            && proof.content_hash.len() == 32
            && proof.root_hash.len() == 32;
        
        if !has_required_fields {
            issues.push("Missing required fields".to_string());
        }

        // Check proof size reasonableness (should be between 1KB and 100MB)
        let proof_size_reasonable = proof.metadata.performance.proof_size_bytes >= 1024 
            && proof.metadata.performance.proof_size_bytes <= 100 * 1024 * 1024;
        
        if !proof_size_reasonable {
            issues.push("Proof size seems unreasonable".to_string());
        }

        // Check timestamp reasonableness (not in future, not too old)
        let now = chrono::Utc::now();
        let timestamp_reasonable = proof.created_at <= now 
            && (now - proof.created_at).num_days() <= 365; // Not older than 1 year
        
        if !timestamp_reasonable {
            issues.push("Timestamp seems unreasonable".to_string());
        }

        // Check security level
        if proof.metadata.security.security_level < 128 {
            issues.push("Security level below recommended minimum".to_string());
        }

        let valid_structure = has_required_fields && proof_size_reasonable && timestamp_reasonable;

        IntegrityCheck {
            valid_structure,
            valid_json,
            has_required_fields,
            proof_size_reasonable,
            timestamp_reasonable,
            issues,
        }
    }

    /// Print table-formatted output
    fn print_table_output(&self, data: &InfoOutput) {
        if let Some(proof_info) = &data.proof_info {
            println!("🔍 Proof Information");
            println!("═══════════════════");
            println!("ID: {}", proof_info.id);
            println!("Version: {}", proof_info.version);
            println!("Created: {}", proof_info.created_at);
            println!("Content Selection: {}", proof_info.content_selection);
            println!("Content Hash: {}", proof_info.content_hash);
            println!("Root Hash: {}", proof_info.root_hash);
            
            println!();
            println!("📁 File Information:");
            if let Some(filename) = &proof_info.file_info.filename {
                println!("   Filename: {}", filename);
            }
            println!("   Size: {}", format_bytes(proof_info.file_info.size_bytes));
            if let Some(mime_type) = &proof_info.file_info.mime_type {
                println!("   MIME Type: {}", mime_type);
            }
            println!("   IPFS CID: {}", proof_info.file_info.ipfs_cid);
            println!("   Block Count: {}", proof_info.file_info.block_count);
            println!("   Avg Block Size: {}", format_bytes(proof_info.file_info.avg_block_size));

            if self.security || self.detailed {
                println!();
                println!("🔒 Security Information:");
                println!("   Security Level: {} bits", proof_info.security.security_level);
                println!("   Hash Function: {}", proof_info.security.hash_function);
                println!("   Proof System: {}", proof_info.security.proof_system);
                println!("   Risc0 Version: {}", proof_info.security.risc0_version);
                println!("   Formal Verification: {}", proof_info.security.formal_verification);
            }

            if self.metrics || self.detailed {
                println!();
                println!("⚡ Performance Metrics:");
                println!("   Generation Time: {}", format_duration(proof_info.performance.generation_time_ms));
                println!("   File Processing: {}", format_duration(proof_info.performance.file_processing_time_ms));
                println!("   ZK Generation: {}", format_duration(proof_info.performance.zk_generation_time_ms));
                println!("   Peak Memory: {}", format_bytes(proof_info.performance.peak_memory_bytes));
                println!("   ZK Cycles: {}", proof_info.performance.zk_cycles);
                println!("   Proof Size: {}", format_bytes(proof_info.performance.proof_size_bytes));
                if let Some(ratio) = proof_info.performance.compression_ratio {
                    println!("   Compression Ratio: {:.2}x", ratio);
                }
            }

            if let Some(integrity) = &proof_info.integrity_check {
                println!();
                println!("🔍 Integrity Check:");
                println!("   Valid Structure: {}", if integrity.valid_structure { "✅" } else { "❌" });
                println!("   Valid JSON: {}", if integrity.valid_json { "✅" } else { "❌" });
                println!("   Required Fields: {}", if integrity.has_required_fields { "✅" } else { "❌" });
                println!("   Reasonable Size: {}", if integrity.proof_size_reasonable { "✅" } else { "❌" });
                println!("   Reasonable Timestamp: {}", if integrity.timestamp_reasonable { "✅" } else { "❌" });
                
                if !integrity.issues.is_empty() {
                    println!("   Issues:");
                    for issue in &integrity.issues {
                        println!("     • {}", issue);
                    }
                }
            }
        }

        if let Some(file_info) = &data.file_info {
            println!();
            println!("📄 File Information");
            println!("═══════════════════");
            println!("Path: {}", file_info.path);
            println!("Exists: {}", if file_info.exists { "✅" } else { "❌" });
            println!("Readable: {}", if file_info.readable { "✅" } else { "❌" });
            println!("Size: {}", file_info.size_human);
            
            if let Some(mime_type) = &file_info.mime_type {
                println!("MIME Type: {}", mime_type);
            }
            
            if let Some(hash) = &file_info.hash_sha256 {
                println!("SHA256: {}", hash);
            }
            
            if let Some(blocks) = file_info.estimated_ipfs_blocks {
                println!("Estimated IPFS Blocks: {}", blocks);
            }
        }

        if let Some(system_info) = &data.system_info {
            println!();
            println!("💻 System Information");
            println!("════════════════════");
            println!("OS: {} ({})", system_info.os, system_info.arch);
            println!("CPU Cores: {}", system_info.cpu_count);
            
            if let Some(memory) = &system_info.available_memory_human {
                println!("Available Memory: {}", memory);
            }
            
            println!("zkIPFS-Proof Version: {}", system_info.zkipfs_version);
            println!("Risc0 Available: {}", if system_info.risc0_available { "✅" } else { "❌" });
            println!("IPFS Available: {}", if system_info.ipfs_available { "✅" } else { "❌" });
            
            println!();
            println!("🚀 Hardware Acceleration:");
            println!("   CUDA: {}", if system_info.hardware_acceleration.cuda_available { "✅" } else { "❌" });
            println!("   Metal: {}", if system_info.hardware_acceleration.metal_available { "✅" } else { "❌" });
            println!("   Recommended: {}", system_info.hardware_acceleration.recommended);
        }

        if let Some(config_info) = &data.config_info {
            println!();
            println!("⚙️  Configuration");
            println!("═════════════════");
            
            if let Some(config_path) = &config_info.config_file_path {
                println!("Config File: {}", config_path);
            } else {
                println!("Config File: Using defaults (no config file found)");
            }
            
            println!("Default Security Level: {} bits", config_info.default_security_level);
            println!("Default Prover: {}", config_info.default_prover);
            println!("Default Compression: {}", config_info.default_compression);
            println!("Hardware Acceleration: {}", if config_info.use_hardware_acceleration { "Enabled" } else { "Disabled" });
            
            if let Some(ipfs_endpoint) = &config_info.ipfs_endpoint {
                println!("IPFS Endpoint: {}", ipfs_endpoint);
            }
            
            if let Some(bonsai_endpoint) = &config_info.bonsai_endpoint {
                println!("Bonsai Endpoint: {}", bonsai_endpoint);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_gather_file_info() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test content").unwrap();
        
        let cmd = InfoCommand {
            proof: None,
            file: Some(temp_file.path().to_path_buf()),
            system: false,
            config: false,
            detailed: false,
            metrics: false,
            security: false,
            verify_integrity: false,
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let file_info = rt.block_on(cmd.gather_file_info(&temp_file.path().to_path_buf())).unwrap();
        
        assert!(file_info.exists);
        assert!(file_info.readable);
        assert!(file_info.size_bytes > 0);
        assert_eq!(file_info.mime_type, Some("text/plain".to_string()));
    }

    #[test]
    fn test_gather_system_info() {
        let cmd = InfoCommand {
            proof: None,
            file: None,
            system: true,
            config: false,
            detailed: false,
            metrics: false,
            security: false,
            verify_integrity: false,
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let system_info = rt.block_on(cmd.gather_system_info()).unwrap();
        
        assert!(!system_info.os.is_empty());
        assert!(!system_info.arch.is_empty());
        assert!(system_info.cpu_count > 0);
        assert!(!system_info.zkipfs_version.is_empty());
    }

    #[test]
    fn test_check_proof_integrity() {
        let cmd = InfoCommand {
            proof: None,
            file: None,
            system: false,
            config: false,
            detailed: false,
            metrics: false,
            security: false,
            verify_integrity: true,
        };

        // Create a minimal valid proof structure for testing
        let proof = Proof {
            id: "test-proof-id".to_string(),
            version: "0.1.0".to_string(),
            created_at: chrono::Utc::now(),
            zk_proof: zkipfs_proof_core::ZkProofData {
                receipt: vec![1, 2, 3, 4],
                public_inputs: vec![],
                format_version: "1.0".to_string(),
                compression: None,
            },
            content_hash: [0; 32],
            root_hash: [1; 32],
            content_selection: zkipfs_proof_core::ContentSelection::Pattern { content: b"test".to_vec() },
            metadata: zkipfs_proof_core::ProofMetadata {
                guest_metadata: zkipfs_proof_core::guest_types::ProofMetadata {
                    block_count: 1,
                    content_size: 100,
                    timestamp: 12345,
                },
                file_info: zkipfs_proof_core::FileInfo {
                    filename: Some("test.txt".to_string()),
                    size: 100,
                    mime_type: Some("text/plain".to_string()),
                    file_hash: [0; 32],
                    ipfs_cid: "QmTest".to_string(),
                    block_count: 1,
                    avg_block_size: 100,
                },
                performance: zkipfs_proof_core::PerformanceMetrics {
                    generation_time_ms: 1000,
                    file_processing_time_ms: 100,
                    zk_generation_time_ms: 900,
                    peak_memory_bytes: 1024 * 1024,
                    zk_cycles: 10000,
                    proof_size_bytes: 2048, // Reasonable size
                    compression_ratio: None,
                },
                security: zkipfs_proof_core::SecurityParameters {
                    security_level: 128,
                    hash_function: "SHA-256".to_string(),
                    proof_system: "Risc0".to_string(),
                    risc0_version: "1.2".to_string(),
                    formal_verification: false,
                },
                environment: zkipfs_proof_core::GenerationEnvironment {
                    os: "linux".to_string(),
                    arch: "x86_64".to_string(),
                    hardware_acceleration: Some(zkipfs_proof_core::HardwareAcceleration::None),
                    prover_type: zkipfs_proof_core::ProverType::Local,
                    library_version: "0.1.0".to_string(),
                    git_commit: None,
                },
                custom: std::collections::HashMap::new(),
            },
        };

        let json_content = serde_json::to_string(&proof).unwrap();
        let integrity = cmd.check_proof_integrity(&proof, &json_content);
        
        assert!(integrity.valid_json);
        assert!(integrity.has_required_fields);
        assert!(integrity.proof_size_reasonable);
        assert!(integrity.timestamp_reasonable);
        assert!(integrity.valid_structure);
    }
}

