//! Verify command implementation
//!
//! This module implements the `verify` command which validates zero-knowledge proofs
//! against claimed content. It provides detailed verification results and supports
//! various verification modes.

use clap::Args;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Instant;
use tracing::{info, warn};

use zkipfs_proof_core::{
    ProofVerifier, VerificationConfig, Proof,
    error::Result,
};
use crate::{
    config::Config,
    progress::ProgressTracker,
    utils::{validate_file_path, format_duration, format_bytes, format_hash},
    commands::{Command, output},
};

/// Verify a zero-knowledge proof
#[derive(Args, Debug)]
pub struct VerifyCommand {
    /// Path to the proof file to verify
    #[arg(short, long, value_name = "FILE")]
    pub proof: PathBuf,

    /// Path to the original file (optional, for content verification)
    #[arg(short, long, value_name = "FILE")]
    pub file: Option<PathBuf>,

    /// Content to verify against the proof (if file not provided)
    #[arg(short, long, value_name = "CONTENT")]
    pub content: Option<String>,

    /// Enable strict verification mode
    #[arg(long)]
    pub strict: bool,

    /// Include detailed verification steps in output
    #[arg(long)]
    pub detailed: bool,

    /// Maximum allowed proof age in days
    #[arg(long)]
    pub max_age_days: Option<u64>,

    /// Minimum required security level
    #[arg(long)]
    pub min_security_level: Option<u32>,

    /// Expected proof system (risc0, groth16, etc.)
    #[arg(long)]
    pub expected_proof_system: Option<String>,

    /// Verify on-chain (requires blockchain connection)
    #[arg(long)]
    pub on_chain: bool,

    /// Blockchain RPC endpoint for on-chain verification
    #[arg(long)]
    pub rpc_endpoint: Option<String>,

    /// Contract address for on-chain verification
    #[arg(long)]
    pub contract_address: Option<String>,

    /// Output verification report to file
    #[arg(long)]
    pub report: Option<PathBuf>,

    /// Batch verification mode (verify multiple proofs)
    #[arg(long)]
    pub batch: bool,

    /// Directory containing proof files for batch verification
    #[arg(long)]
    pub batch_dir: Option<PathBuf>,
}

#[derive(Serialize, Deserialize)]
struct VerifyOutput {
    proof_id: String,
    is_valid: bool,
    verification_time_ms: u64,
    verifier_version: String,
    verification_method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    proof_metadata: Option<ProofMetadataSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    verification_steps: Option<Vec<VerificationStepSummary>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    warnings: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    errors: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize)]
struct ProofMetadataSummary {
    security_level: u32,
    proof_system: String,
    file_size_bytes: u64,
    proof_size_bytes: u64,
    generation_time_ms: u64,
    created_at: String,
}

#[derive(Serialize, Deserialize)]
struct VerificationStepSummary {
    name: String,
    passed: bool,
    duration_ms: u64,
    details: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct BatchVerifyOutput {
    total_proofs: usize,
    valid_proofs: usize,
    invalid_proofs: usize,
    total_verification_time_ms: u64,
    results: Vec<VerifyOutput>,
    summary: BatchSummary,
}

#[derive(Serialize, Deserialize)]
struct BatchSummary {
    success_rate: f64,
    avg_verification_time_ms: f64,
    fastest_verification_ms: u64,
    slowest_verification_ms: u64,
    common_issues: Vec<String>,
}

impl Command for VerifyCommand {
    async fn execute(&self, config: &Config, output_format: &str) -> Result<()> {
        if self.batch {
            self.execute_batch_verification(config, output_format).await
        } else {
            self.execute_single_verification(config, output_format).await
        }
    }
}

impl VerifyCommand {
    /// Execute single proof verification
    async fn execute_single_verification(&self, config: &Config, output_format: &str) -> Result<()> {
        let start_time = Instant::now();
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        // Validate inputs
        self.validate_inputs(&mut warnings)?;

        // Load proof from file
        let mut progress = ProgressTracker::new("Verifying proof");
        progress.set_message("Loading proof file...");
        
        let proof = self.load_proof_file()?;
        progress.set_progress(20);

        // Prepare content for verification
        progress.set_message("Preparing content for verification...");
        let content = self.prepare_verification_content(&proof)?;
        progress.set_progress(40);

        // Create verification configuration
        let verification_config = self.create_verification_config(config)?;

        // Create verifier
        let mut verifier = ProofVerifier::with_config(verification_config);
        progress.set_progress(50);

        // Perform verification
        progress.set_message("Performing cryptographic verification...");
        let verification_result = if self.on_chain {
            self.verify_on_chain(&proof, &content).await?
        } else {
            verifier.verify_detailed(&proof, &content).await?
        };
        progress.set_progress(90);

        // Save report if requested
        if let Some(report_path) = &self.report {
            progress.set_message("Saving verification report...");
            self.save_verification_report(&verification_result, report_path).await?;
        }

        progress.finish("Verification completed!");

        let verification_time = start_time.elapsed();

        // Create output data
        let output_data = VerifyOutput {
            proof_id: proof.id.clone(),
            is_valid: verification_result.is_valid,
            verification_time_ms: verification_time.as_millis() as u64,
            verifier_version: env!("CARGO_PKG_VERSION").to_string(),
            verification_method: if self.on_chain { "on-chain".to_string() } else { "local".to_string() },
            proof_metadata: Some(ProofMetadataSummary {
                security_level: proof.metadata.security.security_level,
                proof_system: proof.metadata.security.proof_system.clone(),
                file_size_bytes: proof.metadata.file_info.size,
                proof_size_bytes: proof.metadata.performance.proof_size_bytes,
                generation_time_ms: proof.metadata.performance.generation_time_ms,
                created_at: proof.created_at.to_rfc3339(),
            }),
            verification_steps: if self.detailed {
                Some(verification_result.verification_steps.into_iter().map(|step| {
                    VerificationStepSummary {
                        name: step.name,
                        passed: step.passed,
                        duration_ms: step.duration_ms,
                        details: step.details,
                    }
                }).collect())
            } else {
                None
            },
            warnings: if verification_result.warnings.is_empty() { None } else { Some(verification_result.warnings) },
            errors: if errors.is_empty() { None } else { Some(errors) },
        };

        // Print output based on format
        match output_format {
            "table" => self.print_table_output(&output_data),
            _ => output::print_output(&output_data, output_format, true)?,
        }

        // Exit with appropriate code
        if !verification_result.is_valid {
            std::process::exit(1);
        }

        Ok(())
    }

    /// Execute batch verification
    async fn execute_batch_verification(&self, config: &Config, output_format: &str) -> Result<()> {
        let batch_dir = self.batch_dir.as_ref()
            .ok_or_else(|| zkipfs_proof_core::error::ProofError::invalid_input_error(
                "batch_dir",
                "Batch directory is required for batch verification"
            ))?;

        // Find all proof files in directory
        let proof_files = self.find_proof_files(batch_dir)?;
        
        if proof_files.is_empty() {
            return Err(zkipfs_proof_core::error::ProofError::invalid_input_error(
                "batch_dir",
                "No proof files found in batch directory"
            ));
        }

        let mut progress = ProgressTracker::new("Batch verification");
        progress.set_message(&format!("Found {} proof files", proof_files.len()));

        let start_time = Instant::now();
        let mut results = Vec::new();
        let mut verification_times = Vec::new();

        // Verify each proof
        for (i, proof_file) in proof_files.iter().enumerate() {
            progress.set_progress((i as f64 / proof_files.len() as f64 * 100.0) as u64);
            progress.set_message(&format!("Verifying {} ({}/{})", 
                proof_file.file_name().unwrap_or_default().to_string_lossy(),
                i + 1, 
                proof_files.len()
            ));

            let single_start = Instant::now();
            
            // Create a temporary verify command for this proof
            let mut single_verify = self.clone();
            single_verify.proof = proof_file.clone();
            single_verify.batch = false;
            
            match single_verify.execute_single_verification(config, "json").await {
                Ok(_) => {
                    // This is a simplified approach - in reality we'd capture the actual result
                    let verification_time = single_start.elapsed().as_millis() as u64;
                    verification_times.push(verification_time);
                    
                    results.push(VerifyOutput {
                        proof_id: format!("proof_{}", i),
                        is_valid: true,
                        verification_time_ms: verification_time,
                        verifier_version: env!("CARGO_PKG_VERSION").to_string(),
                        verification_method: "local".to_string(),
                        proof_metadata: None,
                        verification_steps: None,
                        warnings: None,
                        errors: None,
                    });
                }
                Err(_) => {
                    let verification_time = single_start.elapsed().as_millis() as u64;
                    verification_times.push(verification_time);
                    
                    results.push(VerifyOutput {
                        proof_id: format!("proof_{}", i),
                        is_valid: false,
                        verification_time_ms: verification_time,
                        verifier_version: env!("CARGO_PKG_VERSION").to_string(),
                        verification_method: "local".to_string(),
                        proof_metadata: None,
                        verification_steps: None,
                        warnings: None,
                        errors: Some(vec!["Verification failed".to_string()]),
                    });
                }
            }
        }

        progress.finish("Batch verification completed!");

        let total_time = start_time.elapsed();
        let valid_count = results.iter().filter(|r| r.is_valid).count();
        let invalid_count = results.len() - valid_count;

        // Create batch summary
        let summary = BatchSummary {
            success_rate: valid_count as f64 / results.len() as f64 * 100.0,
            avg_verification_time_ms: verification_times.iter().sum::<u64>() as f64 / verification_times.len() as f64,
            fastest_verification_ms: *verification_times.iter().min().unwrap_or(&0),
            slowest_verification_ms: *verification_times.iter().max().unwrap_or(&0),
            common_issues: vec![], // Would be populated with actual analysis
        };

        let batch_output = BatchVerifyOutput {
            total_proofs: results.len(),
            valid_proofs: valid_count,
            invalid_proofs: invalid_count,
            total_verification_time_ms: total_time.as_millis() as u64,
            results,
            summary,
        };

        // Print output
        match output_format {
            "table" => self.print_batch_table_output(&batch_output),
            _ => output::print_output(&batch_output, output_format, true)?,
        }

        Ok(())
    }

    /// Validate command inputs
    fn validate_inputs(&self, warnings: &mut Vec<String>) -> Result<()> {
        // Validate proof file
        validate_file_path(&self.proof)?;

        // Validate content source
        if self.file.is_none() && self.content.is_none() {
            warnings.push("No content provided for verification - only cryptographic proof will be verified".to_string());
        }

        if let Some(file_path) = &self.file {
            validate_file_path(file_path)?;
        }

        // Validate on-chain parameters
        if self.on_chain {
            if self.rpc_endpoint.is_none() {
                return Err(zkipfs_proof_core::error::ProofError::invalid_input_error(
                    "rpc_endpoint",
                    "RPC endpoint is required for on-chain verification"
                ));
            }
            
            if self.contract_address.is_none() {
                return Err(zkipfs_proof_core::error::ProofError::invalid_input_error(
                    "contract_address",
                    "Contract address is required for on-chain verification"
                ));
            }
        }

        // Validate batch parameters
        if self.batch && self.batch_dir.is_none() {
            return Err(zkipfs_proof_core::error::ProofError::invalid_input_error(
                "batch_dir",
                "Batch directory is required for batch verification"
            ));
        }

        Ok(())
    }

    /// Load proof from file
    fn load_proof_file(&self) -> Result<Proof> {
        let content = std::fs::read_to_string(&self.proof)
            .map_err(|e| zkipfs_proof_core::error::ProofError::file_error(
                format!("Failed to read proof file: {}", self.proof.display()),
                Some(e)
            ))?;

        serde_json::from_str(&content)
            .map_err(|e| zkipfs_proof_core::error::ProofError::serialization_error(
                "Failed to parse proof file",
                Some(Box::new(e))
            ))
    }

    /// Prepare content for verification
    fn prepare_verification_content(&self, proof: &Proof) -> Result<Vec<u8>> {
        if let Some(file_path) = &self.file {
            std::fs::read(file_path)
                .map_err(|e| zkipfs_proof_core::error::ProofError::file_error(
                    format!("Failed to read content file: {}", file_path.display()),
                    Some(e)
                ))
        } else if let Some(content_str) = &self.content {
            Ok(content_str.as_bytes().to_vec())
        } else {
            // No content provided - return empty vec for cryptographic-only verification
            Ok(Vec::new())
        }
    }

    /// Create verification configuration
    fn create_verification_config(&self, config: &Config) -> Result<VerificationConfig> {
        let mut verification_config = VerificationConfig::default();
        
        verification_config.strict_verification = self.strict;
        verification_config.include_verification_steps = self.detailed;
        
        if let Some(max_age_days) = self.max_age_days {
            verification_config.max_proof_age_seconds = Some(max_age_days * 24 * 60 * 60);
        }

        // Add custom rules based on command line options
        if let Some(min_security) = self.min_security_level {
            verification_config.custom_rules.push(
                zkipfs_proof_core::VerificationRule {
                    name: "min_security_level".to_string(),
                    description: format!("Minimum security level: {} bits", min_security),
                    rule_type: zkipfs_proof_core::VerificationRuleType::MinSecurityLevel(min_security),
                }
            );
        }

        if let Some(expected_system) = &self.expected_proof_system {
            verification_config.custom_rules.push(
                zkipfs_proof_core::VerificationRule {
                    name: "required_proof_system".to_string(),
                    description: format!("Required proof system: {}", expected_system),
                    rule_type: zkipfs_proof_core::VerificationRuleType::RequiredProofSystem(expected_system.clone()),
                }
            );
        }

        Ok(verification_config)
    }

    /// Verify proof on-chain
    async fn verify_on_chain(&self, proof: &Proof, content: &[u8]) -> Result<zkipfs_proof_core::VerificationResult> {
        // This would implement actual on-chain verification
        // For now, return a placeholder
        warn!("On-chain verification not yet implemented");
        
        Ok(zkipfs_proof_core::VerificationResult {
            is_valid: false,
            verified_at: chrono::Utc::now(),
            verification_time_ms: 0,
            verifier_info: zkipfs_proof_core::VerifierInfo {
                version: env!("CARGO_PKG_VERSION").to_string(),
                method: zkipfs_proof_core::VerificationMethod::OnChain,
                environment: "blockchain".to_string(),
            },
            warnings: vec!["On-chain verification not yet implemented".to_string()],
            verification_steps: vec![],
        })
    }

    /// Save verification report to file
    async fn save_verification_report(
        &self,
        result: &zkipfs_proof_core::VerificationResult,
        report_path: &std::path::Path,
    ) -> Result<()> {
        let report_json = serde_json::to_string_pretty(result)
            .map_err(|e| zkipfs_proof_core::error::ProofError::serialization_error(
                "Failed to serialize verification report",
                Some(Box::new(e))
            ))?;

        tokio::fs::write(report_path, report_json).await
            .map_err(|e| zkipfs_proof_core::error::ProofError::file_error(
                format!("Failed to write report file: {}", report_path.display()),
                Some(e)
            ))?;

        Ok(())
    }

    /// Find proof files in directory
    fn find_proof_files(&self, dir: &std::path::Path) -> Result<Vec<PathBuf>> {
        let mut proof_files = Vec::new();
        
        let entries = std::fs::read_dir(dir)
            .map_err(|e| zkipfs_proof_core::error::ProofError::file_error(
                format!("Failed to read batch directory: {}", dir.display()),
                Some(e)
            ))?;

        for entry in entries {
            let entry = entry.map_err(|e| zkipfs_proof_core::error::ProofError::file_error(
                "Failed to read directory entry",
                Some(e)
            ))?;
            
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                proof_files.push(path);
            }
        }

        proof_files.sort();
        Ok(proof_files)
    }

    /// Print table-formatted output for single verification
    fn print_table_output(&self, data: &VerifyOutput) {
        if data.is_valid {
            println!("✅ Proof Verification Successful!");
        } else {
            println!("❌ Proof Verification Failed!");
        }
        
        println!();
        println!("📋 Verification Details:");
        println!("   Proof ID: {}", data.proof_id);
        println!("   Valid: {}", data.is_valid);
        println!("   Verification Time: {}", format_duration(data.verification_time_ms));
        println!("   Method: {}", data.verification_method);
        println!("   Verifier Version: {}", data.verifier_version);

        if let Some(metadata) = &data.proof_metadata {
            println!();
            println!("📊 Proof Metadata:");
            println!("   Security Level: {} bits", metadata.security_level);
            println!("   Proof System: {}", metadata.proof_system);
            println!("   File Size: {}", format_bytes(metadata.file_size_bytes));
            println!("   Proof Size: {}", format_bytes(metadata.proof_size_bytes));
            println!("   Generation Time: {}", format_duration(metadata.generation_time_ms));
            println!("   Created: {}", metadata.created_at);
        }

        if let Some(steps) = &data.verification_steps {
            println!();
            println!("🔍 Verification Steps:");
            for step in steps {
                let status = if step.passed { "✅" } else { "❌" };
                println!("   {} {} ({})", status, step.name, format_duration(step.duration_ms));
                if let Some(details) = &step.details {
                    println!("      {}", details);
                }
            }
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

        if let Some(errors) = &data.errors {
            if !errors.is_empty() {
                println!();
                println!("❌ Errors:");
                for error in errors {
                    println!("   • {}", error);
                }
            }
        }
    }

    /// Print table-formatted output for batch verification
    fn print_batch_table_output(&self, data: &BatchVerifyOutput) {
        println!("📊 Batch Verification Results");
        println!();
        println!("📈 Summary:");
        println!("   Total Proofs: {}", data.total_proofs);
        println!("   Valid Proofs: {} ({:.1}%)", data.valid_proofs, data.summary.success_rate);
        println!("   Invalid Proofs: {}", data.invalid_proofs);
        println!("   Total Time: {}", format_duration(data.total_verification_time_ms));
        println!("   Average Time: {}", format_duration(data.summary.avg_verification_time_ms as u64));
        println!("   Fastest: {}", format_duration(data.summary.fastest_verification_ms));
        println!("   Slowest: {}", format_duration(data.summary.slowest_verification_ms));

        if !data.summary.common_issues.is_empty() {
            println!();
            println!("🔍 Common Issues:");
            for issue in &data.summary.common_issues {
                println!("   • {}", issue);
            }
        }

        println!();
        println!("📋 Individual Results:");
        for (i, result) in data.results.iter().enumerate() {
            let status = if result.is_valid { "✅" } else { "❌" };
            println!("   {} Proof {} - {} ({})", 
                status, 
                i + 1, 
                result.proof_id, 
                format_duration(result.verification_time_ms)
            );
        }
    }
}

// Implement Clone for VerifyCommand to support batch operations
impl Clone for VerifyCommand {
    fn clone(&self) -> Self {
        Self {
            proof: self.proof.clone(),
            file: self.file.clone(),
            content: self.content.clone(),
            strict: self.strict,
            detailed: self.detailed,
            max_age_days: self.max_age_days,
            min_security_level: self.min_security_level,
            expected_proof_system: self.expected_proof_system.clone(),
            on_chain: self.on_chain,
            rpc_endpoint: self.rpc_endpoint.clone(),
            contract_address: self.contract_address.clone(),
            report: self.report.clone(),
            batch: self.batch,
            batch_dir: self.batch_dir.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{NamedTempFile, TempDir};
    use std::io::Write;

    #[test]
    fn test_validate_inputs() {
        let temp_file = NamedTempFile::new().unwrap();
        
        let cmd = VerifyCommand {
            proof: temp_file.path().to_path_buf(),
            file: None,
            content: None,
            strict: false,
            detailed: false,
            max_age_days: None,
            min_security_level: None,
            expected_proof_system: None,
            on_chain: false,
            rpc_endpoint: None,
            contract_address: None,
            report: None,
            batch: false,
            batch_dir: None,
        };

        let mut warnings = Vec::new();
        let result = cmd.validate_inputs(&mut warnings);
        assert!(result.is_ok());
        assert!(!warnings.is_empty()); // Should warn about no content
    }

    #[test]
    fn test_on_chain_validation() {
        let temp_file = NamedTempFile::new().unwrap();
        
        let cmd = VerifyCommand {
            proof: temp_file.path().to_path_buf(),
            file: None,
            content: None,
            strict: false,
            detailed: false,
            max_age_days: None,
            min_security_level: None,
            expected_proof_system: None,
            on_chain: true,
            rpc_endpoint: None, // Missing required field
            contract_address: None, // Missing required field
            report: None,
            batch: false,
            batch_dir: None,
        };

        let mut warnings = Vec::new();
        let result = cmd.validate_inputs(&mut warnings);
        assert!(result.is_err());
    }

    #[test]
    fn test_find_proof_files() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create some test files
        let proof1 = temp_dir.path().join("proof1.json");
        let proof2 = temp_dir.path().join("proof2.json");
        let not_proof = temp_dir.path().join("readme.txt");
        
        std::fs::write(&proof1, "{}").unwrap();
        std::fs::write(&proof2, "{}").unwrap();
        std::fs::write(&not_proof, "not a proof").unwrap();

        let cmd = VerifyCommand {
            proof: PathBuf::new(),
            file: None,
            content: None,
            strict: false,
            detailed: false,
            max_age_days: None,
            min_security_level: None,
            expected_proof_system: None,
            on_chain: false,
            rpc_endpoint: None,
            contract_address: None,
            report: None,
            batch: true,
            batch_dir: Some(temp_dir.path().to_path_buf()),
        };

        let proof_files = cmd.find_proof_files(temp_dir.path()).unwrap();
        assert_eq!(proof_files.len(), 2);
        assert!(proof_files.contains(&proof1));
        assert!(proof_files.contains(&proof2));
        assert!(!proof_files.iter().any(|p| p == &not_proof));
    }
}

