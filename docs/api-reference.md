# API Reference

This document provides a complete reference for the zkIPFS-Proof API, including the CLI commands, Rust library, and smart contract interfaces.

## CLI Commands

### `zkipfs-proof generate`

Generate a zero-knowledge proof for file content.

**Usage:**
```bash
zkipfs-proof generate [OPTIONS] --file <FILE>
```

**Options:**
- `--file <FILE>` - Path to the file to generate proof for
- `--content <CONTENT>` - Specific content to prove exists in the file
- `--range <RANGE>` - Byte range to prove (format: "start-end")
- `--output <OUTPUT>` - Output file path for the proof (default: proof.json)
- `--security-level <LEVEL>` - Security level: 64, 128, or 256 (default: 128)
- `--compression <TYPE>` - Compression type: none, gzip, or zstd (default: gzip)
- `--metadata <JSON>` - Custom metadata to include in the proof
- `--no-ipfs` - Skip IPFS upload
- `--ipfs-pin` - Pin file to IPFS after upload

**Examples:**
```bash
# Basic proof generation
zkipfs-proof generate --file document.pdf --content "confidential"

# Prove byte range with high security
zkipfs-proof generate --file data.csv --range "1000-2000" --security-level 256

# Include custom metadata
zkipfs-proof generate --file report.txt --content "approved" --metadata '{"author":"john","version":"1.0"}'
```

### `zkipfs-proof verify`

Verify a zero-knowledge proof.

**Usage:**
```bash
zkipfs-proof verify [OPTIONS] --proof <PROOF>
```

**Options:**
- `--proof <PROOF>` - Path to the proof file to verify
- `--file <FILE>` - Optional: verify against specific file
- `--output <FORMAT>` - Output format: table, json, or yaml (default: table)
- `--verbose` - Show detailed verification information

**Examples:**
```bash
# Basic verification
zkipfs-proof verify --proof proof.json

# Verify with detailed output
zkipfs-proof verify --proof proof.json --verbose --output json
```

### `zkipfs-proof info`

Display information about proofs, files, or system status.

**Usage:**
```bash
zkipfs-proof info [SUBCOMMAND]
```

**Subcommands:**
- `proof <PROOF>` - Show detailed information about a proof file
- `file <FILE>` - Show file information and generate CID
- `system` - Show system information and capabilities

**Examples:**
```bash
# Show proof details
zkipfs-proof info proof proof.json

# Get file information
zkipfs-proof info file document.pdf

# Check system status
zkipfs-proof info system
```

### `zkipfs-proof ipfs`

Interact with IPFS for file storage and retrieval.

**Usage:**
```bash
zkipfs-proof ipfs <SUBCOMMAND>
```

**Subcommands:**
- `upload <FILE>` - Upload file to IPFS
- `download <CID>` - Download file from IPFS
- `pin <CID>` - Pin file to IPFS
- `unpin <CID>` - Unpin file from IPFS
- `list` - List pinned files
- `stat <CID>` - Get file statistics
- `status` - Check IPFS node status

**Examples:**
```bash
# Upload and pin file
zkipfs-proof ipfs upload document.pdf --pin

# Download file
zkipfs-proof ipfs download QmYourCIDHere --output downloaded.pdf

# Check what's pinned
zkipfs-proof ipfs list
```

## Rust Library API

### Core Types

#### `ProofRequest`

Represents a request to generate a proof.

```rust
pub struct ProofRequest {
    pub file_path: PathBuf,
    pub content_selection: ContentSelection,
    pub security_level: SecurityLevel,
    pub compression: CompressionType,
    pub custom_metadata: HashMap<String, serde_json::Value>,
}
```

#### `ContentSelection`

Defines what content to prove.

```rust
pub enum ContentSelection {
    ByteRange { start: usize, end: usize },
    Pattern { pattern: String },
    Multiple { selections: Vec<ContentSelection> },
}
```

#### `ProofData`

Contains the generated proof and metadata.

```rust
pub struct ProofData {
    pub proof: Vec<u8>,
    pub public_inputs: PublicInputs,
    pub metadata: ProofMetadata,
    pub verification_key: VerificationKey,
}
```

### Core Functions

#### `generate_proof`

Generate a zero-knowledge proof for file content.

```rust
pub async fn generate_proof(
    request: ProofRequest,
    progress_callback: Option<Box<dyn Fn(f64) + Send + Sync>>,
) -> Result<ProofData, ProofError>
```

**Parameters:**
- `request` - The proof generation request
- `progress_callback` - Optional callback for progress updates

**Returns:**
- `Ok(ProofData)` - The generated proof data
- `Err(ProofError)` - Error if proof generation fails

**Example:**
```rust
use zkipfs_proof_core::{generate_proof, ProofRequest, ContentSelection, SecurityLevel};

let request = ProofRequest {
    file_path: "document.pdf".into(),
    content_selection: ContentSelection::Pattern {
        pattern: "confidential".to_string(),
    },
    security_level: SecurityLevel::High,
    compression: CompressionType::Gzip,
    custom_metadata: HashMap::new(),
};

let proof = generate_proof(request, None).await?;
```

#### `verify_proof`

Verify a zero-knowledge proof.

```rust
pub async fn verify_proof(
    proof_data: &ProofData,
    verification_options: VerificationOptions,
) -> Result<VerificationResult, ProofError>
```

**Parameters:**
- `proof_data` - The proof data to verify
- `verification_options` - Verification configuration

**Returns:**
- `Ok(VerificationResult)` - Verification result with details
- `Err(ProofError)` - Error if verification fails

**Example:**
```rust
use zkipfs_proof_core::{verify_proof, VerificationOptions};

let options = VerificationOptions::default();
let result = verify_proof(&proof_data, options).await?;

if result.is_valid {
    println!("Proof is valid!");
} else {
    println!("Proof verification failed: {}", result.error_message);
}
```

### IPFS Integration

#### `IpfsClient`

Client for interacting with IPFS.

```rust
pub struct IpfsClient {
    api_url: String,
    timeout: Duration,
    headers: HashMap<String, String>,
}

impl IpfsClient {
    pub fn new(config: IpfsConfig) -> Self;
    pub async fn upload_file(&self, file_path: &Path) -> Result<String, IpfsError>;
    pub async fn download_file(&self, cid: &str, output_path: &Path) -> Result<(), IpfsError>;
    pub async fn pin_file(&self, cid: &str) -> Result<(), IpfsError>;
    pub async fn unpin_file(&self, cid: &str) -> Result<(), IpfsError>;
    pub async fn list_pins(&self) -> Result<Vec<PinInfo>, IpfsError>;
    pub async fn get_file_stat(&self, cid: &str) -> Result<FileStat, IpfsError>;
}
```

**Example:**
```rust
use zkipfs_proof_core::{IpfsClient, IpfsConfig};

let config = IpfsConfig {
    api_url: "http://127.0.0.1:5001".to_string(),
    timeout: Duration::from_secs(300),
    headers: HashMap::new(),
};

let client = IpfsClient::new(config);
let cid = client.upload_file(Path::new("document.pdf")).await?;
println!("File uploaded with CID: {}", cid);
```

## Smart Contract API

### ZkIPFSVerifier Contract

The main smart contract for on-chain proof verification.

#### Functions

##### `verifyProof`

Verify a single proof on-chain.

```solidity
function verifyProof(
    bytes calldata proof,
    bytes32[] calldata publicInputs,
    uint256 securityLevel
) external returns (bool)
```

**Parameters:**
- `proof` - The zero-knowledge proof bytes
- `publicInputs` - Array of public inputs for the proof
- `securityLevel` - Required security level (64, 128, or 256)

**Returns:**
- `bool` - True if proof is valid, false otherwise

##### `batchVerifyProofs`

Verify multiple proofs in a single transaction.

```solidity
function batchVerifyProofs(
    bytes[] calldata proofs,
    bytes32[][] calldata publicInputs,
    uint256[] calldata securityLevels
) external returns (bool[] memory)
```

**Parameters:**
- `proofs` - Array of proof bytes
- `publicInputs` - Array of public input arrays
- `securityLevels` - Array of required security levels

**Returns:**
- `bool[]` - Array of verification results

##### `getProofInfo`

Get information about a verified proof.

```solidity
function getProofInfo(bytes32 proofHash) 
    external 
    view 
    returns (ProofInfo memory)
```

**Parameters:**
- `proofHash` - Hash of the proof to query

**Returns:**
- `ProofInfo` - Struct containing proof metadata

#### Events

##### `ProofVerified`

Emitted when a proof is successfully verified.

```solidity
event ProofVerified(
    bytes32 indexed proofHash,
    address indexed verifier,
    uint256 securityLevel,
    uint256 timestamp
);
```

##### `BatchProofVerified`

Emitted when multiple proofs are verified.

```solidity
event BatchProofVerified(
    bytes32[] proofHashes,
    address indexed verifier,
    uint256 totalProofs,
    uint256 timestamp
);
```

### Usage Example

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./ZkIPFSVerifier.sol";

contract MyContract {
    ZkIPFSVerifier public verifier;
    
    constructor(address _verifier) {
        verifier = ZkIPFSVerifier(_verifier);
    }
    
    function checkProof(
        bytes calldata proof,
        bytes32[] calldata publicInputs
    ) external {
        bool isValid = verifier.verifyProof(proof, publicInputs, 128);
        require(isValid, "Invalid proof");
        
        // Your logic here
    }
}
```

## Error Handling

### Error Types

#### `ProofError`

Main error type for proof operations.

```rust
pub enum ProofError {
    FileNotFound(String),
    InvalidContent(String),
    ProofGenerationFailed(String),
    VerificationFailed(String),
    IpfsError(IpfsError),
    SerializationError(String),
    InvalidInput(String),
    InternalError(String),
}
```

#### `IpfsError`

Error type for IPFS operations.

```rust
pub enum IpfsError {
    ConnectionFailed(String),
    UploadFailed(String),
    DownloadFailed(String),
    InvalidCid(String),
    TimeoutError,
    AuthenticationError,
}
```

### Error Handling Best Practices

```rust
use zkipfs_proof_core::{generate_proof, ProofError};

match generate_proof(request, None).await {
    Ok(proof) => {
        println!("Proof generated successfully");
        // Handle success
    }
    Err(ProofError::FileNotFound(path)) => {
        eprintln!("File not found: {}", path);
        // Handle file not found
    }
    Err(ProofError::InvalidContent(msg)) => {
        eprintln!("Invalid content: {}", msg);
        // Handle invalid content
    }
    Err(e) => {
        eprintln!("Unexpected error: {}", e);
        // Handle other errors
    }
}
```

## Configuration

### CLI Configuration

The CLI uses a TOML configuration file located at `~/.zkipfs-proof/config.toml`.

```toml
[default]
security_level = 128
prover = "local"
compression = "gzip"
output_dir = "./proofs"
include_metrics = false

[api]
bonsai_endpoint = "https://api.bonsai.xyz"
ipfs_endpoint = "http://127.0.0.1:5001"
request_timeout_seconds = 30
max_retries = 3

[ipfs]
api_url = "http://127.0.0.1:5001"
gateway_url = "http://127.0.0.1:8080"
timeout = 300
auto_pin = true

[logging]
level = "info"
log_to_file = false
max_log_size_mb = 10
log_file_count = 5

[performance]
worker_threads = 4
chunk_size_bytes = 1048576
max_in_memory_size_mb = 1024
enable_profiling = false
```

### Environment Variables

You can override configuration values using environment variables:

- `ZKIPFS_PROOF_SECURITY_LEVEL` - Default security level
- `ZKIPFS_PROOF_IPFS_API_URL` - IPFS API URL
- `ZKIPFS_PROOF_LOG_LEVEL` - Logging level
- `ZKIPFS_PROOF_OUTPUT_DIR` - Default output directory

## Rate Limits and Performance

### CLI Performance

- **Small files (<1MB)**: Proof generation typically takes 1-5 seconds
- **Medium files (1-100MB)**: Proof generation takes 5-30 seconds
- **Large files (100MB-1GB)**: Proof generation takes 30-300 seconds
- **Very large files (>1GB)**: Consider using byte ranges for better performance

### Memory Usage

- **Minimum RAM**: 2GB
- **Recommended RAM**: 8GB for files up to 1GB
- **Large file processing**: RAM usage scales with file size

### IPFS Performance

- **Local node**: Best performance for uploads/downloads
- **Remote node**: Network latency affects performance
- **File pinning**: Ensures persistence but uses storage

## Security Considerations

### Proof Security

- **64-bit security**: Fast but suitable only for non-critical applications
- **128-bit security**: Good balance of speed and security for most use cases
- **256-bit security**: Maximum security for highly sensitive content

### IPFS Security

- **Public networks**: Files are publicly accessible via CID
- **Private networks**: Use for sensitive content
- **Content addressing**: CIDs are deterministic based on content

### Best Practices

1. **Choose appropriate security levels** based on content sensitivity
2. **Validate all inputs** before proof generation
3. **Store proofs securely** as they contain metadata
4. **Use HTTPS** for all API communications
5. **Regularly update** the software for security patches

## Migration Guide

### Upgrading from v0.1.x to v0.2.x

The API has some breaking changes in v0.2.x:

```rust
// Old API (v0.1.x)
let proof = generate_proof(file_path, content).await?;

// New API (v0.2.x)
let request = ProofRequest {
    file_path: file_path.into(),
    content_selection: ContentSelection::Pattern { pattern: content },
    security_level: SecurityLevel::Medium,
    compression: CompressionType::Gzip,
    custom_metadata: HashMap::new(),
};
let proof = generate_proof(request, None).await?;
```

### CLI Changes

- `--pattern` flag renamed to `--content`
- New `--range` flag for byte range proofs
- Configuration file format updated

## Support

For additional help:

- **Documentation**: [https://docs.zkipfs-proof.com](https://docs.zkipfs-proof.com)
- **GitHub Issues**: [https://github.com/sowadmim/zkipfs-proof/issues](https://github.com/sowadmim/zkipfs-proof/issues)
- **Discord**: [https://discord.gg/zkipfs-proof](https://discord.gg/zkipfs-proof)
- **Email**: sowadalmughni@gmail.com

