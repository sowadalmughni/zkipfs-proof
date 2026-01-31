use risc0_zkvm::guest::env;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use regex::Regex;
use sxd_document::parser;
use sxd_xpath::{evaluate_xpath, Value};

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
    /// Prove content matching an XPath expression exists (for XML/HTML)
    XPath { selector: String },
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
    
    // Extract content based on selection (optimized to avoid full concatenation if possible)
    let extracted_content = extract_content(&input.blocks, &input.content_selection);
    
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

fn extract_content(blocks: &[IpfsBlock], selection: &ContentSelection) -> Vec<u8> {
    match selection {
        ContentSelection::ByteRange { start, end } => {
            let total_len: usize = blocks.iter().map(|b| b.data.len()).sum();
            if *start >= total_len || *end > total_len || start >= end {
                panic!("Invalid byte range: {}..{} (len: {})", start, end, total_len);
            }

            let mut result = Vec::with_capacity(end - start);
            let mut current_pos = 0;

            for block in blocks {
                let block_len = block.data.len();
                let block_end = current_pos + block_len;

                // Check if this block overlaps with the requested range
                if block_end > *start && current_pos < *end {
                     let slice_start = if current_pos < *start { start - current_pos } else { 0 };
                     let slice_end = if block_end > *end { end - current_pos } else { block_len };
                     
                     result.extend_from_slice(&block.data[slice_start..slice_end]);
                }
                
                current_pos += block_len;
                if current_pos >= *end {
                    break;
                }
            }
            result
        }
        ContentSelection::Multiple(selections) => {
            let mut result = Vec::new();
            for sel in selections {
                result.extend(extract_content(blocks, sel));
            }
            result
        }
        // For other types, we currently need the full data
        _ => {
            let data = concatenate_blocks(blocks);
            match selection {
                ContentSelection::Pattern { content } => {
                    if let Some(_pos) = find_pattern(&data, content) {
                        content.clone()
                    } else {
                        panic!("Pattern not found");
                    }
                }
                ContentSelection::Regex { pattern } => {
                     let data_str = std::str::from_utf8(&data).expect("File content is not valid UTF-8");
                     let re = Regex::new(pattern).expect("Invalid regex");
                     
                     if let Some(mat) = re.find(data_str) {
                         mat.as_str().as_bytes().to_vec()
                     } else {
                         panic!("Regex pattern not found");
                     }
                }
                ContentSelection::XPath { selector } => {
                    let data_str = std::str::from_utf8(&data).expect("File content is not valid UTF-8");
                    let package = parser::parse(data_str).expect("Failed to parse XML/HTML content");
                    let document = package.as_document();
                    
                    let value = evaluate_xpath(&document, selector).expect("Invalid XPath expression");
                    
                    match value {
                        Value::String(s) => s.into_bytes(),
                        Value::nodeset(nodes) => {
                            if nodes.size() > 0 {
                                 let mut result = String::new();
                                 for node in nodes.document_order() {
                                     result.push_str(&node.string_value());
                                 }
                                 result.into_bytes()
                            } else {
                                panic!("XPath selector found no matches");
                            }
                        },
                        Value::Number(n) => n.to_string().into_bytes(),
                        Value::Boolean(b) => b.to_string().into_bytes(),
                    }
                }
                _ => panic!("Unreachable: handled above"),
            }
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
