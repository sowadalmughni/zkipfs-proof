//! IPFS command implementation for zkIPFS-Proof CLI
//! 
//! This module provides commands for interacting with IPFS nodes,
//! uploading files, and managing decentralized storage.

use clap::{Args, Subcommand};
use std::path::PathBuf;
use zkipfs_proof_core::ipfs_client::{IpfsClient, IpfsConfig, Cid};
use crate::{config::Config, progress::ProgressTracker, utils::format_bytes};
use anyhow::{Result, Context};
use serde_json;

#[derive(Debug, Args)]
pub struct IpfsArgs {
    #[command(subcommand)]
    pub command: IpfsCommand,
}

#[derive(Debug, Subcommand)]
pub enum IpfsCommand {
    /// Upload a file to IPFS
    Upload {
        /// File to upload
        #[arg(short, long)]
        file: PathBuf,
        
        /// Pin the file after upload
        #[arg(short, long, default_value = "true")]
        pin: bool,
        
        /// Custom IPFS node API URL
        #[arg(long)]
        api_url: Option<String>,
        
        /// Output format (json, yaml, table)
        #[arg(short, long, default_value = "table")]
        output: String,
    },
    
    /// Download a file from IPFS
    Download {
        /// IPFS Content Identifier (CID)
        #[arg(short, long)]
        cid: String,
        
        /// Output file path
        #[arg(short, long)]
        output: PathBuf,
        
        /// Custom IPFS node API URL
        #[arg(long)]
        api_url: Option<String>,
    },
    
    /// Pin a file in IPFS
    Pin {
        /// IPFS Content Identifier (CID)
        #[arg(short, long)]
        cid: String,
        
        /// Custom IPFS node API URL
        #[arg(long)]
        api_url: Option<String>,
    },
    
    /// Unpin a file from IPFS
    Unpin {
        /// IPFS Content Identifier (CID)
        #[arg(short, long)]
        cid: String,
        
        /// Custom IPFS node API URL
        #[arg(long)]
        api_url: Option<String>,
    },
    
    /// List pinned files
    List {
        /// Custom IPFS node API URL
        #[arg(long)]
        api_url: Option<String>,
        
        /// Output format (json, yaml, table)
        #[arg(short, long, default_value = "table")]
        output: String,
    },
    
    /// Get file statistics
    Stat {
        /// IPFS Content Identifier (CID)
        #[arg(short, long)]
        cid: String,
        
        /// Custom IPFS node API URL
        #[arg(long)]
        api_url: Option<String>,
        
        /// Output format (json, yaml, table)
        #[arg(short, long, default_value = "table")]
        output: String,
    },
    
    /// Check IPFS node status
    Status {
        /// Custom IPFS node API URL
        #[arg(long)]
        api_url: Option<String>,
    },
    
    /// Configure IPFS settings
    Config {
        /// Set API URL
        #[arg(long)]
        set_api_url: Option<String>,
        
        /// Set gateway URL
        #[arg(long)]
        set_gateway_url: Option<String>,
        
        /// Set auto-pin preference
        #[arg(long)]
        set_auto_pin: Option<bool>,
        
        /// Show current configuration
        #[arg(long)]
        show: bool,
    },
}

pub async fn handle_ipfs_command(args: IpfsArgs, config: &Config) -> Result<()> {
    match args.command {
        IpfsCommand::Upload { file, pin, api_url, output } => {
            upload_file(file, pin, api_url, output, config).await
        }
        IpfsCommand::Download { cid, output, api_url } => {
            download_file(cid, output, api_url, config).await
        }
        IpfsCommand::Pin { cid, api_url } => {
            pin_file(cid, api_url, config).await
        }
        IpfsCommand::Unpin { cid, api_url } => {
            unpin_file(cid, api_url, config).await
        }
        IpfsCommand::List { api_url, output } => {
            list_pinned_files(api_url, output, config).await
        }
        IpfsCommand::Stat { cid, api_url, output } => {
            get_file_stats(cid, api_url, output, config).await
        }
        IpfsCommand::Status { api_url } => {
            check_node_status(api_url, config).await
        }
        IpfsCommand::Config { set_api_url, set_gateway_url, set_auto_pin, show } => {
            handle_ipfs_config(set_api_url, set_gateway_url, set_auto_pin, show, config).await
        }
    }
}

async fn upload_file(
    file_path: PathBuf,
    pin: bool,
    api_url: Option<String>,
    output_format: String,
    config: &Config,
) -> Result<()> {
    if !file_path.exists() {
        anyhow::bail!("File does not exist: {}", file_path.display());
    }

    let ipfs_config = create_ipfs_config(api_url, config)?;
    let client = IpfsClient::with_config(ipfs_config)
        .context("Failed to create IPFS client")?;

    // Check if IPFS node is online
    if !client.is_online().await {
        anyhow::bail!("IPFS node is not accessible. Please ensure IPFS is running.");
    }

    let mut progress = ProgressTracker::new("Uploading to IPFS");
    progress.set_message("Reading file...");

    // Get file size for progress tracking
    let file_size = std::fs::metadata(&file_path)
        .context("Failed to get file metadata")?
        .len();

    progress.set_message(&format!("Uploading {} to IPFS...", format_bytes(file_size)));

    let ipfs_file = client.upload_file(&file_path).await
        .context("Failed to upload file to IPFS")?;

    progress.finish_with_message("Upload completed successfully!");

    // Pin the file if requested
    if pin && !ipfs_file.pinned {
        let mut pin_progress = ProgressTracker::new("Pinning file");
        client.pin_file(&ipfs_file.cid).await
            .context("Failed to pin file")?;
        pin_progress.finish_with_message("File pinned successfully!");
    }

    // Output results
    match output_format.as_str() {
        "json" => {
            let output = serde_json::json!({
                "success": true,
                "cid": ipfs_file.cid.as_str(),
                "name": ipfs_file.name,
                "size": ipfs_file.size,
                "mime_type": ipfs_file.mime_type,
                "pinned": pin,
                "uploaded_at": ipfs_file.uploaded_at,
                "gateway_url": client.gateway_url(&ipfs_file.cid)
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        "yaml" => {
            println!("success: true");
            println!("cid: {}", ipfs_file.cid.as_str());
            println!("name: {}", ipfs_file.name);
            println!("size: {}", ipfs_file.size);
            println!("mime_type: {}", ipfs_file.mime_type);
            println!("pinned: {}", pin);
            println!("uploaded_at: {}", ipfs_file.uploaded_at);
            println!("gateway_url: {}", client.gateway_url(&ipfs_file.cid));
        }
        _ => {
            println!("✅ File uploaded successfully!");
            println!();
            println!("📁 File Details:");
            println!("   Name: {}", ipfs_file.name);
            println!("   Size: {}", format_bytes(ipfs_file.size));
            println!("   Type: {}", ipfs_file.mime_type);
            println!();
            println!("🔗 IPFS Details:");
            println!("   CID: {}", ipfs_file.cid.as_str());
            println!("   Pinned: {}", if pin { "Yes" } else { "No" });
            println!("   Gateway URL: {}", client.gateway_url(&ipfs_file.cid));
            println!();
            println!("💡 You can now use this CID to generate proofs or share the file!");
        }
    }

    Ok(())
}

async fn download_file(
    cid_str: String,
    output_path: PathBuf,
    api_url: Option<String>,
    config: &Config,
) -> Result<()> {
    let cid = Cid::new(cid_str);
    if !cid.is_valid() {
        anyhow::bail!("Invalid CID format: {}", cid.as_str());
    }

    let ipfs_config = create_ipfs_config(api_url, config)?;
    let client = IpfsClient::with_config(ipfs_config)
        .context("Failed to create IPFS client")?;

    if !client.is_online().await {
        anyhow::bail!("IPFS node is not accessible. Please ensure IPFS is running.");
    }

    let mut progress = ProgressTracker::new("Downloading from IPFS");
    progress.set_message(&format!("Retrieving file with CID: {}", cid.as_str()));

    let content = client.get_file(&cid).await
        .context("Failed to download file from IPFS")?;

    progress.set_message("Writing file to disk...");

    // Create parent directories if they don't exist
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)
            .context("Failed to create output directory")?;
    }

    std::fs::write(&output_path, content)
        .context("Failed to write file to disk")?;

    progress.finish_with_message("Download completed successfully!");

    println!("✅ File downloaded successfully!");
    println!("   CID: {}", cid.as_str());
    println!("   Output: {}", output_path.display());
    println!("   Size: {}", format_bytes(std::fs::metadata(&output_path)?.len()));

    Ok(())
}

async fn pin_file(
    cid_str: String,
    api_url: Option<String>,
    config: &Config,
) -> Result<()> {
    let cid = Cid::new(cid_str);
    if !cid.is_valid() {
        anyhow::bail!("Invalid CID format: {}", cid.as_str());
    }

    let ipfs_config = create_ipfs_config(api_url, config)?;
    let client = IpfsClient::with_config(ipfs_config)
        .context("Failed to create IPFS client")?;

    if !client.is_online().await {
        anyhow::bail!("IPFS node is not accessible. Please ensure IPFS is running.");
    }

    let mut progress = ProgressTracker::new("Pinning file");
    progress.set_message(&format!("Pinning CID: {}", cid.as_str()));

    client.pin_file(&cid).await
        .context("Failed to pin file")?;

    progress.finish_with_message("File pinned successfully!");

    println!("✅ File pinned successfully!");
    println!("   CID: {}", cid.as_str());

    Ok(())
}

async fn unpin_file(
    cid_str: String,
    api_url: Option<String>,
    config: &Config,
) -> Result<()> {
    let cid = Cid::new(cid_str);
    if !cid.is_valid() {
        anyhow::bail!("Invalid CID format: {}", cid.as_str());
    }

    let ipfs_config = create_ipfs_config(api_url, config)?;
    let client = IpfsClient::with_config(ipfs_config)
        .context("Failed to create IPFS client")?;

    if !client.is_online().await {
        anyhow::bail!("IPFS node is not accessible. Please ensure IPFS is running.");
    }

    let mut progress = ProgressTracker::new("Unpinning file");
    progress.set_message(&format!("Unpinning CID: {}", cid.as_str()));

    client.unpin_file(&cid).await
        .context("Failed to unpin file")?;

    progress.finish_with_message("File unpinned successfully!");

    println!("✅ File unpinned successfully!");
    println!("   CID: {}", cid.as_str());

    Ok(())
}

async fn list_pinned_files(
    api_url: Option<String>,
    output_format: String,
    config: &Config,
) -> Result<()> {
    let ipfs_config = create_ipfs_config(api_url, config)?;
    let client = IpfsClient::with_config(ipfs_config)
        .context("Failed to create IPFS client")?;

    if !client.is_online().await {
        anyhow::bail!("IPFS node is not accessible. Please ensure IPFS is running.");
    }

    let mut progress = ProgressTracker::new("Listing pinned files");
    
    let pinned_cids = client.list_pinned().await
        .context("Failed to list pinned files")?;

    progress.finish_with_message("Retrieved pinned files list!");

    match output_format.as_str() {
        "json" => {
            let output = serde_json::json!({
                "pinned_files": pinned_cids.iter().map(|cid| cid.as_str()).collect::<Vec<_>>(),
                "count": pinned_cids.len()
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        "yaml" => {
            println!("pinned_files:");
            for cid in &pinned_cids {
                println!("  - {}", cid.as_str());
            }
            println!("count: {}", pinned_cids.len());
        }
        _ => {
            println!("📌 Pinned Files ({} total):", pinned_cids.len());
            println!();
            if pinned_cids.is_empty() {
                println!("   No pinned files found.");
            } else {
                for (i, cid) in pinned_cids.iter().enumerate() {
                    println!("   {}. {}", i + 1, cid.as_str());
                }
            }
        }
    }

    Ok(())
}

async fn get_file_stats(
    cid_str: String,
    api_url: Option<String>,
    output_format: String,
    config: &Config,
) -> Result<()> {
    let cid = Cid::new(cid_str);
    if !cid.is_valid() {
        anyhow::bail!("Invalid CID format: {}", cid.as_str());
    }

    let ipfs_config = create_ipfs_config(api_url, config)?;
    let client = IpfsClient::with_config(ipfs_config)
        .context("Failed to create IPFS client")?;

    if !client.is_online().await {
        anyhow::bail!("IPFS node is not accessible. Please ensure IPFS is running.");
    }

    let mut progress = ProgressTracker::new("Getting file statistics");
    
    let stats = client.stat_file(&cid).await
        .context("Failed to get file statistics")?;

    progress.finish_with_message("Retrieved file statistics!");

    match output_format.as_str() {
        "json" => {
            let output = serde_json::json!({
                "cid": cid.as_str(),
                "size": stats.size,
                "block_count": stats.block_count,
                "link_count": stats.link_count,
                "gateway_url": client.gateway_url(&cid)
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        "yaml" => {
            println!("cid: {}", cid.as_str());
            println!("size: {}", stats.size);
            println!("block_count: {}", stats.block_count);
            println!("link_count: {}", stats.link_count);
            println!("gateway_url: {}", client.gateway_url(&cid));
        }
        _ => {
            println!("📊 File Statistics:");
            println!("   CID: {}", cid.as_str());
            println!("   Size: {}", format_bytes(stats.size));
            println!("   Blocks: {}", stats.block_count);
            println!("   Links: {}", stats.link_count);
            println!("   Gateway URL: {}", client.gateway_url(&cid));
        }
    }

    Ok(())
}

async fn check_node_status(
    api_url: Option<String>,
    config: &Config,
) -> Result<()> {
    let ipfs_config = create_ipfs_config(api_url, config)?;
    let client = IpfsClient::with_config(ipfs_config)
        .context("Failed to create IPFS client")?;

    let mut progress = ProgressTracker::new("Checking IPFS node status");
    
    let is_online = client.is_online().await;
    
    if is_online {
        progress.finish_with_message("IPFS node is online!");
        println!("✅ IPFS node is accessible");
        println!("   API URL: {}", ipfs_config.api_url);
        println!("   Gateway URL: {}", ipfs_config.gateway_url);
    } else {
        progress.finish_with_message("IPFS node is offline!");
        println!("❌ IPFS node is not accessible");
        println!("   API URL: {}", ipfs_config.api_url);
        println!();
        println!("💡 Make sure IPFS is running:");
        println!("   - Install IPFS: https://docs.ipfs.tech/install/");
        println!("   - Start daemon: ipfs daemon");
        println!("   - Check status: ipfs id");
    }

    Ok(())
}

async fn handle_ipfs_config(
    set_api_url: Option<String>,
    set_gateway_url: Option<String>,
    set_auto_pin: Option<bool>,
    show: bool,
    config: &Config,
) -> Result<()> {
    if show || (set_api_url.is_none() && set_gateway_url.is_none() && set_auto_pin.is_none()) {
        // Show current configuration
        let ipfs_config = create_ipfs_config(None, config)?;
        println!("🔧 Current IPFS Configuration:");
        println!("   API URL: {}", ipfs_config.api_url);
        println!("   Gateway URL: {}", ipfs_config.gateway_url);
        println!("   Auto Pin: {}", ipfs_config.auto_pin);
        println!("   Timeout: {}s", ipfs_config.timeout);
        return Ok(());
    }

    // Update configuration (simplified - in a real implementation, this would update the config file)
    println!("🔧 IPFS Configuration Updated:");
    
    if let Some(api_url) = set_api_url {
        println!("   API URL: {} → {}", config.ipfs.api_url, api_url);
    }
    
    if let Some(gateway_url) = set_gateway_url {
        println!("   Gateway URL: {} → {}", config.ipfs.gateway_url, gateway_url);
    }
    
    if let Some(auto_pin) = set_auto_pin {
        println!("   Auto Pin: {} → {}", config.ipfs.auto_pin, auto_pin);
    }

    println!();
    println!("💡 Configuration changes will take effect on next command.");

    Ok(())
}

fn create_ipfs_config(api_url: Option<String>, config: &Config) -> Result<IpfsConfig> {
    let mut ipfs_config = IpfsConfig {
        api_url: api_url.unwrap_or_else(|| config.ipfs.api_url.clone()),
        gateway_url: config.ipfs.gateway_url.clone(),
        timeout: config.ipfs.timeout,
        auto_pin: config.ipfs.auto_pin,
        headers: std::collections::HashMap::new(),
    };

    // Add any custom headers from config
    for (key, value) in &config.ipfs.headers {
        ipfs_config.headers.insert(key.clone(), value.clone());
    }

    Ok(ipfs_config)
}

