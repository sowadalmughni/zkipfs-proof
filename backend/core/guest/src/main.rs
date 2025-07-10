#![no_main]
#![no_std]

use risc0_zkvm::guest::env;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

risc0_zkvm::guest::entry!(main);

/// IPFS content identifier prefix for SHA-256 hashes
const SHA256_PREFIX: [u8; 2] = [0x12, 0x20]; // multihash prefix for SHA-256

/// Input data structure for the ZK circuit
#[derive(Serialize, Deserialize)]
pub struct ProofInput {
    /// IPFS blocks that form the complete file structure
    pub blocks: Vec<IpfsBlock>,
    /// Specification of which content to prove exists
    pub content_selection: ContentSelection,
    /// Expected content hash for verification
    pub expected_content_hash: [u8; 32],
}

/// Represents an IPFS block with its data and metadata
#[derive(Serialize, Deserialize)]
pub struct IpfsBlock {
    /// Raw block data
    pub data: Vec<u8>,
    /// Block's content identifier (CID)
    pub cid: Vec<u8>,
    /// Links to other blocks (for DAG structure)
    pub links: Vec<BlockLink>,
}

/// Link to another IPFS block
#[derive(Serialize, Deserialize)]
pub struct BlockLink {
    /// Name/path of the linked content
    pub name: String,
    /// CID of the linked block
    pub cid: Vec<u8>,
    /// Size of the linked content
    pub size: u64,
}

/// Specification of content to prove within the file
#[derive(Serialize, Deserialize)]
pub enum ContentSelection {
    /// Prove content exists within a specific byte range
    ByteRange { start: usize, end: usize },
    /// Prove specific content pattern exists
    Pattern { content: Vec<u8> },
    /// Prove multiple content selections
    Multiple(Vec<ContentSelection>),
}

/// Output data structure from the ZK circuit
#[derive(Serialize, Deserialize)]
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
#[derive(Serialize, Deserialize)]
pub struct ProofMetadata {
    /// Total number of blocks processed
    pub block_count: u32,
    /// Total size of content proven
    pub content_size: u64,
    /// Timestamp of proof generation (block number)
    pub timestamp: u64,
}

fn main() {
    // Read input data from the host
    let input: ProofInput = env::read();
    
    // Verify the IPFS block structure and compute root hash
    let root_hash = verify_ipfs_structure(&input.blocks);
    
    // Extract and verify the specified content
    let (content_hash, inclusion_proof) = extract_and_prove_content(
        &input.blocks,
        &input.content_selection,
    );
    
    // Verify that the content hash matches expectations
    assert_eq!(
        content_hash, 
        input.expected_content_hash,
        "Content hash mismatch - content may have been tampered with"
    );
    
    // Create proof metadata
    let metadata = ProofMetadata {
        block_count: input.blocks.len() as u32,
        content_size: calculate_content_size(&input.content_selection, &input.blocks),
        timestamp: env::cycle_count() as u64, // Use cycle count as timestamp
    };
    
    // Create the proof output
    let output = ProofOutput {
        root_hash,
        content_hash,
        inclusion_proof,
        metadata,
    };
    
    // Commit the proof output to the journal
    env::commit(&output);
}

/// Verifies the IPFS block structure and computes the root hash
fn verify_ipfs_structure(blocks: &[IpfsBlock]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    
    // Process blocks in topological order to build the DAG
    for block in blocks {
        // Verify block integrity
        let computed_cid = compute_block_cid(&block.data);
        assert_eq!(
            computed_cid, 
            block.cid,
            "Block CID mismatch - block may be corrupted"
        );
        
        // Add block hash to the overall structure hash
        hasher.update(&block.cid);
        
        // Verify links
        for link in &block.links {
            hasher.update(&link.cid);
            hasher.update(link.name.as_bytes());
            hasher.update(&link.size.to_le_bytes());
        }
    }
    
    hasher.finalize().into()
}

/// Extracts content according to the selection criteria and generates inclusion proof
fn extract_and_prove_content(
    blocks: &[IpfsBlock],
    selection: &ContentSelection,
) -> ([u8; 32], Vec<[u8; 32]>) {
    match selection {
        ContentSelection::ByteRange { start, end } => {
            extract_byte_range_content(blocks, *start, *end)
        }
        ContentSelection::Pattern { content } => {
            extract_pattern_content(blocks, content)
        }
        ContentSelection::Multiple(selections) => {
            extract_multiple_content(blocks, selections)
        }
    }
}

/// Extracts content from a specific byte range across IPFS blocks
fn extract_byte_range_content(
    blocks: &[IpfsBlock],
    start: usize,
    end: usize,
) -> ([u8; 32], Vec<[u8; 32]>) {
    let mut content = Vec::new();
    let mut inclusion_proof = Vec::new();
    let mut current_offset = 0;
    
    for block in blocks {
        let block_start = current_offset;
        let block_end = current_offset + block.data.len();
        
        // Check if this block contains part of our target range
        if block_start < end && block_end > start {
            let extract_start = if start > block_start { start - block_start } else { 0 };
            let extract_end = if end < block_end { end - block_start } else { block.data.len() };
            
            // Extract the relevant portion
            content.extend_from_slice(&block.data[extract_start..extract_end]);
            
            // Add block hash to inclusion proof
            let block_hash = Sha256::digest(&block.data);
            inclusion_proof.push(block_hash.into());
        }
        
        current_offset = block_end;
        
        // Early exit if we've collected all needed content
        if current_offset >= end {
            break;
        }
    }
    
    // Compute content hash
    let content_hash = Sha256::digest(&content);
    
    (content_hash.into(), inclusion_proof)
}

/// Extracts content matching a specific pattern
fn extract_pattern_content(
    blocks: &[IpfsBlock],
    pattern: &[u8],
) -> ([u8; 32], Vec<[u8; 32]>) {
    let mut found_content = Vec::new();
    let mut inclusion_proof = Vec::new();
    
    // Concatenate all block data for pattern searching
    let mut all_data = Vec::new();
    for block in blocks {
        all_data.extend_from_slice(&block.data);
    }
    
    // Search for pattern occurrences
    if let Some(pos) = find_pattern(&all_data, pattern) {
        found_content.extend_from_slice(&all_data[pos..pos + pattern.len()]);
        
        // Build inclusion proof for blocks containing the pattern
        let mut current_offset = 0;
        for block in blocks {
            let block_end = current_offset + block.data.len();
            
            if current_offset <= pos && block_end > pos {
                let block_hash = Sha256::digest(&block.data);
                inclusion_proof.push(block_hash.into());
            }
            
            current_offset = block_end;
        }
    }
    
    // Verify pattern was found
    assert!(!found_content.is_empty(), "Pattern not found in content");
    assert_eq!(found_content, pattern, "Extracted content doesn't match pattern");
    
    let content_hash = Sha256::digest(&found_content);
    (content_hash.into(), inclusion_proof)
}

/// Extracts content for multiple selections
fn extract_multiple_content(
    blocks: &[IpfsBlock],
    selections: &[ContentSelection],
) -> ([u8; 32], Vec<[u8; 32]>) {
    let mut combined_content = Vec::new();
    let mut combined_proof = Vec::new();
    
    for selection in selections {
        let (content_hash, mut proof) = extract_and_prove_content(blocks, selection);
        combined_content.extend_from_slice(&content_hash);
        combined_proof.append(&mut proof);
    }
    
    // Remove duplicate proof elements
    combined_proof.sort();
    combined_proof.dedup();
    
    let final_hash = Sha256::digest(&combined_content);
    (final_hash.into(), combined_proof)
}

/// Computes the CID for an IPFS block
fn compute_block_cid(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let hash = hasher.finalize();
    
    // Create CID with SHA-256 prefix
    let mut cid = Vec::with_capacity(34);
    cid.extend_from_slice(&SHA256_PREFIX);
    cid.extend_from_slice(&hash);
    
    cid
}

/// Finds the first occurrence of a pattern in data
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

/// Calculates the total size of content being proven
fn calculate_content_size(selection: &ContentSelection, blocks: &[IpfsBlock]) -> u64 {
    match selection {
        ContentSelection::ByteRange { start, end } => (end - start) as u64,
        ContentSelection::Pattern { content } => content.len() as u64,
        ContentSelection::Multiple(selections) => {
            selections.iter()
                .map(|s| calculate_content_size(s, blocks))
                .sum()
        }
    }
}

