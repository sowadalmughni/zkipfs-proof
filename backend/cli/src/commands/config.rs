//! Config command implementation

use crate::{config::{Config, get_config_value, set_config_value, save_config}, commands::output};
use zkipfs_proof_core::error::Result;

/// Manage configuration settings
pub async fn execute(action: crate::ConfigAction, config: &Config) -> Result<()> {
    match action {
        crate::ConfigAction::Show => {
            println!("📋 Current Configuration:");
            println!("Security:");
            println!("  default_security_level: {}", config.default_security_level);
            println!("  default_prover: {}", config.default_prover);
            println!("  default_compression: {}", config.default_compression);
            println!("  use_hardware_acceleration: {}", config.use_hardware_acceleration);
            
            println!("API:");
            if let Some(endpoint) = &config.api.ipfs_endpoint {
                println!("  ipfs_endpoint: {}", endpoint);
            }
            if let Some(endpoint) = &config.api.bonsai_endpoint {
                println!("  bonsai_endpoint: {}", endpoint);
            }
            println!("  request_timeout_seconds: {}", config.api.request_timeout_seconds);
            println!("  max_retries: {}", config.api.max_retries);
            
            println!("Performance:");
            if let Some(threads) = config.performance.worker_threads {
                println!("  worker_threads: {}", threads);
            }
            println!("  chunk_size_bytes: {}", config.performance.chunk_size_bytes);
            println!("  max_in_memory_size_mb: {}", config.performance.max_in_memory_size_mb);
        }
        
        crate::ConfigAction::Get { key } => {
            if let Some(value) = get_config_value(config, &key) {
                println!("{}", value);
            } else {
                eprintln!("Unknown configuration key: {}", key);
                std::process::exit(1);
            }
        }
        
        crate::ConfigAction::Set { key, value } => {
            let mut new_config = config.clone();
            set_config_value(&mut new_config, &key, &value)?;
            save_config(&new_config, None).await?;
            println!("✅ Configuration updated: {} = {}", key, value);
        }
        
        crate::ConfigAction::Reset { yes } => {
            if !yes {
                print!("Are you sure you want to reset configuration to defaults? (y/N): ");
                use std::io::{self, Write};
                io::stdout().flush().unwrap();
                
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                
                if !input.trim().to_lowercase().starts_with('y') {
                    println!("Configuration reset cancelled.");
                    return Ok(());
                }
            }
            
            let default_config = Config::default();
            save_config(&default_config, None).await?;
            println!("✅ Configuration reset to defaults");
        }
    }
    
    Ok(())
}

