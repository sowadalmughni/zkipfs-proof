//! Version command implementation

use zkipfs_proof_core::error::Result;

/// Show version and build information
pub async fn execute(detailed: bool) -> Result<()> {
    if detailed {
        println!("zkIPFS-Proof {}", env!("CARGO_PKG_VERSION"));
        println!("Build Information:");
        println!("  Target: {}", env!("TARGET"));
        println!("  Profile: {}", if cfg!(debug_assertions) { "debug" } else { "release" });
        println!("  Rustc: {}", env!("RUSTC_VERSION"));
        
        #[cfg(feature = "git-version")]
        {
            println!("  Git Commit: {}", env!("GIT_HASH"));
            println!("  Git Branch: {}", env!("GIT_BRANCH"));
        }
        
        println!("Dependencies:");
        println!("  risc0-zkvm: {}", "1.2.6"); // Would be dynamic in real implementation
        println!("  tokio: {}", "1.46");
        println!("  clap: {}", "4.0");
    } else {
        println!("zkIPFS-Proof {}", env!("CARGO_PKG_VERSION"));
    }
    
    Ok(())
}

