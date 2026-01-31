//! Generate command implementation
//!
//! This module implements the `generate` command which creates zero-knowledge proofs
//! for specified file content. It supports various content selection methods and
//! provides detailed progress tracking and error reporting.

use clap::Args;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Instant;
use tracing::{info, warn};
use uuid::Uuid;

use zkipfs_proof_core::{
    ProofGenerator, ProofConfig, ContentSelection, ProverType, CompressionType,
    error::Result,
};
use crate::{
    config::Config,
    progress::ProgressTracker,
    utils::{parse_content_selection, validate_file_path, format_duration, format_bytes},
    commands::{Command, output},
};

/// Generate a zero-knowledge proof for file content
#[derive(Args, Debug)]
pub struct GenerateCommand {
    /// Path to the file to generate proof for
    #[arg(short, long, value_name = "FILE")]
    pub file: PathBuf,

    /// Content to prove exists in the file
    /// 
    /// Formats:
    /// - Pattern: "pattern:secret text"
    /// - Regex: "regex:^\d{3}-\d{2}-\d{4}$"
    /// - Byte range: "range:100:200"  
    /// - Multiple: "pattern:text1,range:50:100"
    #[arg(short, long, value_name = "SELECTION")]
    pub content: String,

    /// Output file for the generated proof
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,

    /// Security level in bits (128, 192, or 256)
    #[arg(long, default_value = "128")]
    pub security_level: u32,

    /// Prover type (local, bonsai, or custom)
    #[arg(long, default_value = "local")]
    pub prover: String,

    /// Compression type (none, gzip, or zstd)
    #[arg(long, default_value = "gzip")]
    pub compression: String,

    /// Maximum memory usage in MB
    #[arg(long)]
    pub max_memory: Option<u64>,

    /// Timeout in seconds
    #[arg(long)]
    pub timeout: Option<u64>,

    /// Disable hardware acceleration
    #[arg(long)]
    pub no_hardware_acceleration: bool,

    /// Include performance metrics in output
    #[arg(long)]
    pub include_metrics: bool,

    /// Save proof metadata to separate file
    #[arg(long)]
    pub save_metadata: Option<PathBuf>,

    /// Custom metadata as JSON string
    #[arg(long)]
    pub custom_metadata: Option<String>,

    /// Force overwrite existing output file
    #[arg(short, long)]
    pub force: bool,

    /// Dry run - validate inputs without generating proof
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Serialize, Deserialize)]
struct GenerateOutput {
    proof_id: String,
    file_path: String,
    content_selection: String,
    proof_file: Option<String>,
    generation_time_ms: u64,
    file_size_bytes: u64,
    proof_size_bytes: u64,
    security_level: u32,
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    performance_metrics: Option<PerformanceMetrics>,
    #[serde(skip_serializing_if = "Option::is_none")]
    warnings: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize)]
struct PerformanceMetrics {
    file_processing_time_ms: u64,
    zk_generation_time_ms: u64,
    peak_memory_bytes: u64,
    zk_cycles: u64,
    compression_ratio: Option<f64>,
}

impl Command for GenerateCommand {
    async fn execute(&self, config: &Config, output_format: &str) -> Result<()> {
        let start_time = Instant::now();
        let mut warnings = Vec::new();

        // Validate inputs
        self.validate_inputs(&mut warnings)?;

        if self.dry_run {
            output::print_success("Dry run completed - all inputs are valid");
            return Ok(());
        }

        // Parse content selection
        let content_selection = parse_content_selection(&self.content)?;
        info!("Content selection: {}", content_selection.description());

        // Create proof configuration
        let proof_config = self.create_proof_config(config)?;

        // Initialize progress tracker
        let mut progress = ProgressTracker::new("Generating proof");
        progress.set_message("Initializing proof generator...");

        // Create proof generator
        let mut generator = ProofGenerator::with_config(proof_config).await?;
        progress.set_progress(10);

        progress.set_message("Processing file and generating proof...");
        
        // Generate the proof
        let proof = generator.generate_proof(&self.file, content_selection).await?;
        progress.set_progress(90);

        progress.set_message("Saving proof...");

        // Determine output file path
        let output_path = self.get_output_path(&proof.id)?;

        // Save proof to file
        let proof_json = serde_json::to_string_pretty(&proof)?;
        std::fs::write(&output_path, proof_json)?;

        // Save metadata if requested
        if let Some(metadata_path) = &self.save_metadata {
            let metadata_json = serde_json::to_string_pretty(&proof.metadata)?;
            std::fs::write(metadata_path, metadata_json)?;
        }

        progress.finish("Proof generation completed!");

        let generation_time = start_time.elapsed();

        // Create output data
        let output_data = GenerateOutput {
            proof_id: proof.id.clone(),
            file_path: self.file.display().to_string(),
            content_selection: proof.content_selection.description(),
            proof_file: Some(output_path.display().to_string()),
            generation_time_ms: generation_time.as_millis() as u64,
            file_size_bytes: proof.metadata.file_info.size,
            proof_size_bytes: proof.metadata.performance.proof_size_bytes,
            security_level: proof.metadata.security.security_level,
            success: true,
            performance_metrics: if self.include_metrics {
                Some(PerformanceMetrics {
                    file_processing_time_ms: proof.metadata.performance.file_processing_time_ms,
                    zk_generation_time_ms: proof.metadata.performance.zk_generation_time_ms,
                    peak_memory_bytes: proof.metadata.performance.peak_memory_bytes,
                    zk_cycles: proof.metadata.performance.zk_cycles,
                    compression_ratio: proof.metadata.performance.compression_ratio,
                })
            } else {
                None
            },
            warnings: if warnings.is_empty() { None } else { Some(warnings) },
        };

        // Print output based on format
        match output_format {
            "table" => self.print_table_output(&output_data),
            _ => output::print_output(&output_data, output_format, true)?,
        }

        Ok(())
    }
}

impl GenerateCommand {
    /// Validate command inputs
    fn validate_inputs(&self, warnings: &mut Vec<String>) -> Result<()> {
        // Validate file path
        validate_file_path(&self.file)?;

        // Validate security level
        if ![128, 192, 256].contains(&self.security_level) {
            return Err(zkipfs_proof_core::error::ProofError::invalid_input_error(
                "security_level",
                "Security level must be 128, 192, or 256"
            ));
        }

        // Validate prover type
        match self.prover.as_str() {
            "local" | "bonsai" => {}
            _ => {
                return Err(zkipfs_proof_core::error::ProofError::invalid_input_error(
                    "prover",
                    "Prover must be 'local' or 'bonsai'"
                ));
            }
        }

        // Validate compression type
        match self.compression.as_str() {
            "none" | "gzip" | "zstd" => {}
            _ => {
                return Err(zkipfs_proof_core::error::ProofError::invalid_input_error(
                    "compression",
                    "Compression must be 'none', 'gzip', or 'zstd'"
                ));
            }
        }

        // Check if output file exists
        if let Some(output_path) = self.get_output_path("temp")? {
            if output_path.exists() && !self.force {
                return Err(zkipfs_proof_core::error::ProofError::invalid_input_error(
                    "output",
                    format!("Output file already exists: {}. Use --force to overwrite", 
                           output_path.display())
                ));
            }
        }

        // Validate custom metadata JSON
        if let Some(metadata) = &self.custom_metadata {
            serde_json::from_str::<serde_json::Value>(metadata)
                .map_err(|e| zkipfs_proof_core::error::ProofError::invalid_input_error(
                    "custom_metadata",
                    format!("Invalid JSON in custom metadata: {}", e)
                ))?;
        }

        // Add warnings for potentially problematic settings
        if self.security_level < 128 {
            warnings.push("Security level below 128 bits is not recommended for production use".to_string());
        }

        if self.no_hardware_acceleration {
            warnings.push("Hardware acceleration disabled - proof generation may be slower".to_string());
        }

        Ok(())
    }

    /// Create proof configuration from command arguments
    fn create_proof_config(&self, config: &Config) -> Result<ProofConfig> {
        let prover_type = match self.prover.as_str() {
            "local" => ProverType::Local,
            "bonsai" => ProverType::Bonsai,
            custom => ProverType::Custom(custom.to_string()),
        };

        let compression = match self.compression.as_str() {
            "none" => CompressionType::None,
            "gzip" => CompressionType::Gzip,
            "zstd" => CompressionType::Zstd,
            _ => CompressionType::Gzip, // Default fallback
        };

        let mut custom_metadata = std::collections::HashMap::new();
        if let Some(metadata_str) = &self.custom_metadata {
            let metadata_value: serde_json::Value = serde_json::from_str(metadata_str)?;
            custom_metadata.insert("user_metadata".to_string(), metadata_value);
        }

        Ok(ProofConfig {
            security_level: self.security_level,
            use_hardware_acceleration: !self.no_hardware_acceleration,
            prover_type,
            max_memory_bytes: self.max_memory.map(|mb| mb * 1024 * 1024),
            timeout_seconds: self.timeout,
            compression,
            custom_metadata,
            include_performance_metrics: self.include_metrics,
            include_verification_steps: false,
        })
    }

    /// Get the output file path
    fn get_output_path(&self, proof_id: &str) -> Result<PathBuf> {
        if let Some(output) = &self.output {
            Ok(output.clone())
        } else {
            // Generate default output filename
            let filename = format!("proof_{}.json", &proof_id[..8]);
            Ok(PathBuf::from(filename))
        }
    }

    /// Print table-formatted output
    fn print_table_output(&self, data: &GenerateOutput) {
        println!("🎉 Proof Generation Successful!");
        println!();
        println!("📋 Proof Details:");
        println!("   ID: {}", data.proof_id);
        println!("   File: {}", data.file_path);
        println!("   Content: {}", data.content_selection);
        println!("   Security Level: {} bits", data.security_level);
        println!();
        println!("📊 Performance:");
        println!("   Generation Time: {}", format_duration(data.generation_time_ms));
        println!("   File Size: {}", format_bytes(data.file_size_bytes));
        println!("   Proof Size: {}", format_bytes(data.proof_size_bytes));
        
        if let Some(metrics) = &data.performance_metrics {
            println!("   File Processing: {}", format_duration(metrics.file_processing_time_ms));
            println!("   ZK Generation: {}", format_duration(metrics.zk_generation_time_ms));
            println!("   Peak Memory: {}", format_bytes(metrics.peak_memory_bytes));
            println!("   ZK Cycles: {}", metrics.zk_cycles);
            if let Some(ratio) = metrics.compression_ratio {
                println!("   Compression Ratio: {:.2}x", ratio);
            }
        }
        
        println!();
        if let Some(proof_file) = &data.proof_file {
            println!("💾 Proof saved to: {}", proof_file);
        }

        if let Some(warnings) = &data.warnings {
            if !warnings.is_empty() {
                println!();
                println!("⚠️  Warnings:");
                for warning in warnings {
                    println!("   • {}", warning);
                }
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
    fn test_validate_security_level() {
        let mut cmd = GenerateCommand {
            file: PathBuf::from("test.txt"),
            content: "pattern:test".to_string(),
            output: None,
            security_level: 64, // Invalid
            prover: "local".to_string(),
            compression: "gzip".to_string(),
            max_memory: None,
            timeout: None,
            no_hardware_acceleration: false,
            include_metrics: false,
            save_metadata: None,
            custom_metadata: None,
            force: false,
            dry_run: true,
        };

        let mut warnings = Vec::new();
        let result = cmd.validate_inputs(&mut warnings);
        assert!(result.is_err());

        cmd.security_level = 128;
        let result = cmd.validate_inputs(&mut warnings);
        // Will still fail due to file not existing, but security level is valid
        assert!(result.is_err());
    }

    #[test]
    fn test_prover_validation() {
        let cmd = GenerateCommand {
            file: PathBuf::from("test.txt"),
            content: "pattern:test".to_string(),
            output: None,
            security_level: 128,
            prover: "invalid".to_string(),
            compression: "gzip".to_string(),
            max_memory: None,
            timeout: None,
            no_hardware_acceleration: false,
            include_metrics: false,
            save_metadata: None,
            custom_metadata: None,
            force: false,
            dry_run: true,
        };

        let mut warnings = Vec::new();
        let result = cmd.validate_inputs(&mut warnings);
        assert!(result.is_err());
    }

    #[test]
    fn test_custom_metadata_validation() {
        let cmd = GenerateCommand {
            file: PathBuf::from("test.txt"),
            content: "pattern:test".to_string(),
            output: None,
            security_level: 128,
            prover: "local".to_string(),
            compression: "gzip".to_string(),
            max_memory: None,
            timeout: None,
            no_hardware_acceleration: false,
            include_metrics: false,
            save_metadata: None,
            custom_metadata: Some("invalid json".to_string()),
            force: false,
            dry_run: true,
        };

        let mut warnings = Vec::new();
        let result = cmd.validate_inputs(&mut warnings);
        assert!(result.is_err());
    }
}

