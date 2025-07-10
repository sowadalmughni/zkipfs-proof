//! CLI command implementations
//!
//! This module contains all the command implementations for the zkIPFS-Proof CLI.
//! Each command is implemented as a separate module with its own argument parsing
//! and execution logic.

pub mod generate;
pub mod verify;
pub mod info;
pub mod init;
pub mod version;
pub mod benchmark;
pub mod config;
pub mod ipfs;

use zkipfs_proof_core::error::Result;

/// Common trait for all CLI commands
pub trait Command {
    /// Execute the command with the given configuration and output format
    async fn execute(&self, config: &crate::config::Config, output_format: &str) -> Result<()>;
}

/// Output formatting utilities
pub mod output {
    use serde::Serialize;
    use zkipfs_proof_core::error::{ProofError, Result};
    use std::io::{self, Write};

    /// Format and print output based on the specified format
    pub fn print_output<T: Serialize>(
        data: &T,
        format: &str,
        pretty: bool,
    ) -> Result<()> {
        match format.to_lowercase().as_str() {
            "json" => {
                let output = if pretty {
                    serde_json::to_string_pretty(data)
                } else {
                    serde_json::to_string(data)
                }.map_err(|e| ProofError::serialization_error(
                    "Failed to serialize output to JSON",
                    Some(Box::new(e))
                ))?;
                println!("{}", output);
            }
            "yaml" => {
                let output = serde_yaml::to_string(data)
                    .map_err(|e| ProofError::serialization_error(
                        "Failed to serialize output to YAML",
                        Some(Box::new(e))
                    ))?;
                print!("{}", output);
            }
            "table" => {
                // For table format, we'll implement custom formatting per command
                // This is a fallback to JSON for complex data
                let output = serde_json::to_string_pretty(data)
                    .map_err(|e| ProofError::serialization_error(
                        "Failed to serialize output",
                        Some(Box::new(e))
                    ))?;
                println!("{}", output);
            }
            _ => {
                return Err(ProofError::invalid_input_error(
                    "output_format",
                    format!("Unsupported output format: {}", format)
                ));
            }
        }
        
        io::stdout().flush().map_err(|e| ProofError::internal_error(
            "Failed to flush stdout",
            Some(Box::new(e))
        ))?;
        
        Ok(())
    }

    /// Print a simple message
    pub fn print_message(message: &str) {
        println!("{}", message);
    }

    /// Print an error message
    pub fn print_error(message: &str) {
        eprintln!("Error: {}", message);
    }

    /// Print a warning message
    pub fn print_warning(message: &str) {
        eprintln!("Warning: {}", message);
    }

    /// Print a success message
    pub fn print_success(message: &str) {
        println!("✅ {}", message);
    }
}

