//! Guest program for zkIPFS-Proof content verification
//! 
//! This program runs inside the Risc0 ZK-VM to verify that specific content
//! exists within an IPFS file without revealing the complete file contents.

#![no_main]

use risc0_zkvm::guest::env;

risc0_zkvm::guest::entry!(main);

/// Input structure for the ZK proof
#[derive(serde::Deserialize)]
struct ProofInput {
    /// The content pattern to search for
    content_pattern: String,
    /// File chunks to search within
    file_chunks: Vec<Vec<u8>>,
    /// Expected content hash
    expected_hash: [u8; 32],
}

/// Output structure for the ZK proof
#[derive(serde::Serialize)]
struct ProofOutput {
    /// Whether the content was found
    content_found: bool,
    /// Hash of the found content (if any)
    content_hash: [u8; 32],
    /// Position where content was found (if any)
    position: Option<usize>,
}

fn main() {
    // Read input from the host
    let input: ProofInput = env::read();
    
    // Search for the content pattern in the file chunks
    let mut content_found = false;
    let mut position = None;
    let mut content_hash = [0u8; 32];
    
    for (chunk_idx, chunk) in input.file_chunks.iter().enumerate() {
        if let Ok(chunk_str) = std::str::from_utf8(chunk) {
            if let Some(pos) = chunk_str.find(&input.content_pattern) {
                content_found = true;
                position = Some(chunk_idx * 1024 + pos); // Assuming 1KB chunks
                
                // Calculate hash of the found content
                content_hash = sha256_hash(input.content_pattern.as_bytes());
                break;
            }
        }
    }
    
    // Verify the content hash matches expected
    if content_found && content_hash != input.expected_hash {
        content_found = false;
        position = None;
    }
    
    let output = ProofOutput {
        content_found,
        content_hash,
        position,
    };
    
    // Commit the result
    env::commit(&output);
}

/// Simple SHA-256 implementation for the guest program
fn sha256_hash(data: &[u8]) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

