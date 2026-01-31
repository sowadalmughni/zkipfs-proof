//! Guest program for zkIPFS-Proof content verification
//! 
//! This program runs inside the Risc0 ZK-VM to verify that specific content
//! exists within an IPFS file without revealing the complete file contents.

#![no_main]

use risc0_zkvm::guest::env;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use regex::Regex;

risc0_zkvm::guest::entry!(main);

// ==========================================
// Shared Types (Must match Host Definition)
// ==========================================

/// Input data structure for the ZK circuit
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProofInput {
    /// IPFS blocks that form the complete file structure
    pub blocks: Vec<IpfsBlock>,
    /// Specification of which content to prove exists
    pub content_selection: ContentSelection,
    /// Expected content hash for verification
    pub expected_content_hash: [u8; 32],
}

/// Represents an IPFS block with its data and metadata
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IpfsBlock {
    /// Raw block data
    pub data: Vec<u8>,
    /// Block's content identifier (CID)
    pub cid: Vec<u8>,
    /// Links to other blocks (for DAG structure)
    pub links: Vec<BlockLink>,
}

/// Link to another IPFS block
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BlockLink {
    /// Name/path of the linked content
    pub name: String,
    /// CID of the linked block
    pub cid: Vec<u8>,
    /// Size of the linked content
    pub size: u64,
}

/// Specification of content to prove within the file
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ContentSelection {
    /// Prove content exists within a specific byte range
    ByteRange { start: usize, end: usize },
    /// Prove specific content pattern exists
    Pattern { content: Vec<u8> },
    /// Prove content matching a regular expression exists
    Regex { pattern: String },
    /// Prove multiple content selections
    Multiple(Vec<ContentSelection>),
}

/// Output data structure from the ZK circuit
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProofOutput {
    /// Root hash of the IPFS DAG structure
    pub root_hash: [u8; 32],
    /// Hash of the proven content
    pub content_hash: [u8; 32],
    /// Merkle proof demonstrating content inclusion
    pub inclusion_proof: Vec<[u8; 32]>,
    /// Metadata about the proof
    pub metadata: ProofMetadata,
}

/// Metadata about the generated proof
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProofMetadata {
    /// Total number of blocks processed
    pub block_count: u32,
    /// Total size of content proven
    pub content_size: u64,
    /// Timestamp of proof generation (block number)
    pub timestamp: u64,
}

// ==========================================
// Main Logic
// ==========================================

fn main() {
    // Read input from the host
    let input: ProofInput = env::read();
    
    // Process input to extract data
    let all_data = concatenate_blocks(&input.blocks);
    
    // Extract content based on selection
    let extracted_content = extract_content(&all_data, &input.content_selection);
    
    // Calculate content hash
    let content_hash: [u8; 32] = sha256_hash(&extracted_content);
    
    // Verify hash matches expected
    if content_hash != input.expected_content_hash {
        panic!("Content hash mismatch! Expected {:?}, got {:?}", input.expected_content_hash, content_hash);
    }
    
    // Determine root hash (simplified for now, using first block's CID or mock)
    // In a real IPFS verification, we'd verify the DAG/Merkle root.
    // Here we use a placeholder or derive it from blocks.
    let root_hash = [0u8; 32]; // TODO: Implement proper DAG root verification
    
    // Create output
    let output = ProofOutput {
        root_hash,
        content_hash,
        inclusion_proof: vec![], // TODO: Generate inclusion proof
        metadata: ProofMetadata {
            block_count: input.blocks.len() as u32,
            content_size: extracted_content.len() as u64,
            timestamp: 0, 
        },
    };
    
    // Commit the result
    env::commit(&output);
}

fn concatenate_blocks(blocks: &[IpfsBlock]) -> Vec<u8> {
    let mut data = Vec::new();
    for block in blocks {
        data.extend_from_slice(&block.data);
    }
    data
}

fn extract_content(data: &[u8], selection: &ContentSelection) -> Vec<u8> {
    match selection {
        ContentSelection::ByteRange { start, end } => {
            if *start >= data.len() || *end > data.len() || start >= end {
                panic!("Invalid byte range: {}..{} (len: {})", start, end, data.len());
            }
            data[*start..*end].to_vec()
        }
        ContentSelection::Pattern { content } => {
            if let Some(_pos) = find_pattern(data, content) {
                content.clone()
            } else {
                panic!("Pattern not found");
            }
        }
        ContentSelection::Regex { pattern } => {
             // Convert bytes to string for regex matching (assuming UTF-8)
             // Note: This panics if file content is not valid UTF-8
             let data_str = std::str::from_utf8(data).expect("File content is not valid UTF-8");
             let re = Regex::new(pattern).expect("Invalid regex");
             
             if let Some(mat) = re.find(data_str) {
                 mat.as_str().as_bytes().to_vec()
             } else {
                 panic!("Regex pattern not found");
             }
        }
        ContentSelection::Multiple(selections) => {
            let mut result = Vec::new();
            for sel in selections {
                result.extend(extract_content(data, sel));
            }
            result
        }
    }
}

fn find_pattern(data: &[u8], pattern: &[u8]) -> Option<usize> {
    if pattern.is_empty() || pattern.len() > data.len() {
        return None;
    }

    for i in 0..=(data.len() - pattern.len()) {
        if &data[i..i + pattern.len()] == pattern {
            return Some(i);
        }
    }
    None
}

fn sha256_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}
