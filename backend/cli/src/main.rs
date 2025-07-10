//! zkIPFS-Proof CLI Application
//!
//! Command-line interface for generating and verifying zero-knowledge proofs
//! of IPFS content. Provides an intuitive interface for developers, researchers,
//! and journalists to prove file authenticity without revealing sensitive data.

mod commands;
mod config;
mod progress;
mod utils;

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use commands::{generate::GenerateCommand, verify::VerifyCommand, info::InfoCommand};

/// zkIPFS-Proof: Zero-knowledge proofs for IPFS content verification
#[derive(Parser)]
#[command(
    name = "zkipfs-proof",
    version = env!("CARGO_PKG_VERSION"),
    about = "Generate and verify zero-knowledge proofs for IPFS content",
    long_about = "zkIPFS-Proof enables cryptographic verification of file content without revealing sensitive data. \
                  Built on Risc0's ZK-VM and integrated with IPFS, it allows you to prove that specific content \
                  exists within larger datasets while maintaining privacy."
)]
struct Cli {
    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Enable debug logging
    #[arg(short, long, global = true)]
    debug: bool,

    /// Configuration file path
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,

    /// Output format (json, yaml, table)
    #[arg(short, long, global = true, default_value = "table")]
    output: String,

    /// Disable colored output
    #[arg(long, global = true)]
    no_color: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a zero-knowledge proof for file content
    #[command(alias = "gen")]
    Generate(GenerateCommand),

    /// Verify a zero-knowledge proof
    #[command(alias = "ver")]
    Verify(VerifyCommand),

    /// Display information about proofs, files, or system status
    Info(InfoCommand),

    /// Initialize configuration and setup
    Init {
        /// Force overwrite existing configuration
        #[arg(short, long)]
        force: bool,
        
        /// Configuration directory
        #[arg(short, long)]
        config_dir: Option<PathBuf>,
    },

    /// Show version and build information
    Version {
        /// Show detailed version information
        #[arg(short, long)]
        detailed: bool,
    },

    /// Run performance benchmarks
    Benchmark {
        /// File to benchmark with
        #[arg(short, long)]
        file: Option<PathBuf>,
        
        /// Number of iterations
        #[arg(short, long, default_value = "10")]
        iterations: u32,
        
        /// Output benchmark results to file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Manage configuration settings
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Interact with IPFS for file storage and retrieval
    Ipfs(commands::ipfs::IpfsArgs),
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Show current configuration
    Show,
    
    /// Set a configuration value
    Set {
        /// Configuration key
        key: String,
        /// Configuration value
        value: String,
    },
    
    /// Get a configuration value
    Get {
        /// Configuration key
        key: String,
    },
    
    /// Reset configuration to defaults
    Reset {
        /// Confirm reset without prompt
        #[arg(short, long)]
        yes: bool,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    
    // Initialize logging
    init_logging(cli.verbose, cli.debug, cli.no_color);
    
    // Load configuration
    let config = match config::load_config(cli.config.as_deref()).await {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };
    
    info!("zkIPFS-Proof CLI v{} starting", env!("CARGO_PKG_VERSION"));
    
    // Execute command
    let result = match cli.command {
        Commands::Generate(cmd) => cmd.execute(&config, &cli.output).await,
        Commands::Verify(cmd) => cmd.execute(&config, &cli.output).await,
        Commands::Info(cmd) => cmd.execute(&config, &cli.output).await,
        Commands::Init { force, config_dir } => {
            commands::init::execute(force, config_dir.as_deref()).await
        }
        Commands::Version { detailed } => {
            commands::version::execute(detailed).await
        }
        Commands::Benchmark { file, iterations, output } => {
            commands::benchmark::execute(file.as_deref(), iterations, output.as_deref()).await
        }
        Commands::Config { action } => {
            commands::config::execute(action, &config).await
        }
        Commands::Ipfs(args) => {
            commands::ipfs::handle_ipfs_command(args, &config).await
        }
    };
    
    match result {
        Ok(_) => {
            info!("Command completed successfully");
        }
        Err(e) => {
            error!("Command failed: {}", e);
            
            // Print user-friendly error message
            eprintln!("Error: {}", e);
            
            // Print additional context for debugging if verbose
            if cli.verbose || cli.debug {
                eprintln!("\nDebug information:");
                eprintln!("{:?}", e);
            }
            
            std::process::exit(1);
        }
    }
}

/// Initialize logging based on verbosity level
fn init_logging(verbose: bool, debug: bool, no_color: bool) {
    let level = if debug {
        tracing::Level::DEBUG
    } else if verbose {
        tracing::Level::INFO
    } else {
        tracing::Level::WARN
    };
    
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_ansi(!no_color);
    
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(level.into())
                .from_env_lossy()
        )
        .with(fmt_layer)
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        Cli::command().debug_assert()
    }

    #[test]
    fn test_cli_parsing() {
        // Test basic command parsing
        let cli = Cli::try_parse_from(&["zkipfs-proof", "generate", "--help"]);
        assert!(cli.is_err()); // Should show help and exit
        
        // Test version command
        let cli = Cli::try_parse_from(&["zkipfs-proof", "version"]);
        assert!(cli.is_ok());
    }
}

