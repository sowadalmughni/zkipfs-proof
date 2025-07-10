# zkIPFS-Proof API Reference

## Overview

The zkIPFS-Proof API provides comprehensive functionality for generating and verifying zero-knowledge proofs of content existence within IPFS-stored files. This reference covers all public APIs across the Rust library, CLI tool, smart contracts, and web interface.

## Rust Library API

### Core Types

#### `ProofGenerator`

The main interface for generating zero-knowledge proofs.

```rust
use zkipfs_proof_core::{ProofGenerator, ContentSelection};

impl ProofGenerator {
    /// Create a new proof generator instance
    pub async fn new() -> Result<Self, ProofError>;
    
    /// Generate a proof for content within a file
    pub async fn generate_proof(
        &self,
        file_path: &Path,
        content_selection: ContentSelection,
    ) -> Result<Proof, ProofError>;
    
    /// Verify a generated proof
    pub async fn verify_proof(
        &self,
        proof: &Proof,
        expected_content: &[u8],
    ) -> Result<bool, ProofError>;
}
```

#### `ContentSelection`

Specifies what content to prove within a file.

```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ContentSelection {
    /// Prove content exists within a specific byte range
    ByteRange { start: usize, end: usize },
    
    /// Prove specific content pattern exists
    Pattern { content: Vec<u8> },
    
    /// Prove multiple content selections
    Multiple(Vec<ContentSelection>),
}
```

#### `Proof`

Contains the generated zero-knowledge proof and metadata.

```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Proof {
    /// The actual ZK proof data
    pub proof_data: Vec<u8>,
    
    /// Root hash of the IPFS DAG structure
    pub root_hash: [u8; 32],
    
    /// Hash of the proven content
    pub content_hash: [u8; 32],
    
    /// Merkle proof demonstrating content inclusion
    pub inclusion_proof: Vec<[u8; 32]>,
    
    /// Metadata about the proof
    pub metadata: ProofMetadata,
}
```

### Advanced Features

#### `AdvancedVerificationEngine`

Provides sophisticated verification strategies for enterprise use cases.

```rust
use zkipfs_proof_core::{AdvancedVerificationEngine, VerificationStrategy};

impl AdvancedVerificationEngine {
    /// Create a new advanced verification engine
    pub fn new() -> Self;
    
    /// Verify a proof using advanced strategies
    pub async fn verify_proof_advanced(
        &mut self,
        proof_data: &[u8],
        public_inputs: &[u8],
        strategy: &VerificationStrategy,
    ) -> Result<IndividualVerificationResult, ZkIPFSError>;
    
    /// Process a batch of verifications
    pub async fn verify_batch(
        &mut self,
        request: BatchVerificationRequest,
    ) -> Result<BatchVerificationResult, ZkIPFSError>;
}
```

#### `EcosystemManager`

Manages integration with various ZK proof systems and IPFS networks.

```rust
use zkipfs_proof_core::{EcosystemManager, ZkSystem, ProofInput};

impl EcosystemManager {
    /// Create a new ecosystem manager
    pub fn new() -> Self;
    
    /// Generate proof using specified ZK system
    pub fn generate_proof_with_system(
        &self,
        system: &str,
        input: &ProofInput,
    ) -> Result<Vec<u8>, ZkIPFSError>;
    
    /// Get optimal ZK system for requirements
    pub fn recommend_zk_system(
        &self,
        requirements: &ZkRequirements
    ) -> Option<String>;
}
```

### Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum ProofError {
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Invalid content selection: {0}")]
    InvalidContentSelection(String),
    
    #[error("Proof generation failed: {0}")]
    ProofGenerationFailed(String),
    
    #[error("Proof verification failed: {0}")]
    ProofVerificationFailed(String),
    
    #[error("IPFS error: {0}")]
    IpfsError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
}
```

## CLI Tool API

### Basic Commands

#### Generate Proof

```bash
zkipfs-proof generate [OPTIONS] <FILE_PATH>

OPTIONS:
    -p, --pattern <PATTERN>        Content pattern to prove
    -r, --range <START:END>        Byte range to prove
    -o, --output <OUTPUT_FILE>     Output file for the proof
    -s, --security <LEVEL>         Security level (128, 192, 256)
    -f, --format <FORMAT>          Output format (json, binary)
    --ipfs-upload                  Upload file to IPFS before proving
    --compression                  Enable proof compression
```

#### Verify Proof

```bash
zkipfs-proof verify [OPTIONS] <PROOF_FILE>

OPTIONS:
    -c, --content <CONTENT>        Expected content to verify
    -f, --content-file <FILE>      File containing expected content
    --format <FORMAT>              Proof format (json, binary)
    --verbose                      Show detailed verification info
```

#### Batch Operations

```bash
zkipfs-proof batch [OPTIONS] <BATCH_CONFIG>

OPTIONS:
    -j, --jobs <NUM>               Number of parallel jobs
    -t, --timeout <SECONDS>        Timeout per proof
    --strategy <STRATEGY>          Verification strategy
    --output-dir <DIR>             Output directory for results
```

### Configuration

```bash
zkipfs-proof config [SUBCOMMAND]

SUBCOMMANDS:
    init                           Initialize configuration
    set <KEY> <VALUE>             Set configuration value
    get <KEY>                     Get configuration value
    list                          List all configuration
    reset                         Reset to defaults
```

### IPFS Integration

```bash
zkipfs-proof ipfs [SUBCOMMAND]

SUBCOMMANDS:
    upload <FILE>                 Upload file to IPFS
    download <CID>                Download file from IPFS
    pin <CID>                     Pin content to local node
    unpin <CID>                   Unpin content from local node
    status                        Show IPFS node status
```

## Smart Contract API

### ZkIPFSVerifier Contract

#### Core Functions

```solidity
contract ZkIPFSVerifier {
    /// Verify a single proof
    function verifyProof(
        bytes calldata proof,
        bytes32 contentHash,
        address submitter
    ) external view returns (bool);
    
    /// Verify multiple proofs in batch
    function batchVerifyProofs(
        bytes[] calldata proofs,
        bytes32[] calldata contentHashes,
        address[] calldata submitters
    ) external view returns (bool[] memory);
    
    /// Get verification cost estimate
    function getVerificationCost(
        uint256 proofSize
    ) external view returns (uint256);
}
```

#### Events

```solidity
event ProofVerified(
    bytes32 indexed contentHash,
    address indexed submitter,
    bool verified,
    uint256 timestamp
);

event BatchVerificationCompleted(
    uint256 totalProofs,
    uint256 successfulVerifications,
    uint256 gasUsed
);
```

#### Access Control

```solidity
/// Update verifier implementation (admin only)
function updateVerifier(
    address newVerifier
) external onlyRole(ADMIN_ROLE);

/// Pause/unpause verification (emergency)
function pause() external onlyRole(PAUSER_ROLE);
function unpause() external onlyRole(PAUSER_ROLE);
```

## Web Interface API

### REST API Endpoints

#### Proof Generation

```http
POST /api/v1/proofs/generate
Content-Type: application/json

{
    "file_data": "base64_encoded_file_content",
    "content_selection": {
        "type": "pattern",
        "content": "secret_content"
    },
    "options": {
        "security_level": 128,
        "compression": true
    }
}
```

Response:
```json
{
    "proof_id": "uuid",
    "proof_data": "base64_encoded_proof",
    "content_hash": "hex_string",
    "metadata": {
        "generation_time_ms": 1250,
        "proof_size_bytes": 3072,
        "security_level": 128
    }
}
```

#### Proof Verification

```http
POST /api/v1/proofs/verify
Content-Type: application/json

{
    "proof_data": "base64_encoded_proof",
    "expected_content": "secret_content",
    "verification_strategy": "single"
}
```

Response:
```json
{
    "verified": true,
    "confidence_score": 1.0,
    "verification_time_ms": 45,
    "verifier_nodes": ["single_verifier"]
}
```

#### Batch Processing

```http
POST /api/v1/proofs/batch
Content-Type: application/json

{
    "proofs": [
        {
            "proof_id": "uuid1",
            "proof_data": "base64_proof1",
            "expected_content": "content1"
        }
    ],
    "strategy": {
        "type": "multi_verifier",
        "count": 3,
        "threshold": 2
    }
}
```

### WebSocket API

#### Real-time Proof Generation

```javascript
const ws = new WebSocket('ws://localhost:8080/ws/v1/proofs'); // Use your deployed endpoint

// Send proof generation request
ws.send(JSON.stringify({
    type: 'generate_proof',
    data: {
        file_data: 'base64_content',
        content_selection: {
            type: 'pattern',
            content: 'search_pattern'
        }
    }
}));

// Receive progress updates
ws.onmessage = (event) => {
    const message = JSON.parse(event.data);
    switch (message.type) {
        case 'progress':
            console.log(`Progress: ${message.percentage}%`);
            break;
        case 'completed':
            console.log('Proof generated:', message.proof);
            break;
        case 'error':
            console.error('Error:', message.error);
            break;
    }
};
```

## SDK Integration

### JavaScript/TypeScript SDK

```typescript
import { ZkIPFSProof } from '@zkipfs-proof/sdk';

const client = new ZkIPFSProof({
    apiKey: 'your_api_key',
    endpoint: 'http://localhost:8080' // Use your deployed endpoint
});

// Generate proof
const proof = await client.generateProof({
    fileData: fileBuffer,
    contentSelection: {
        type: 'pattern',
        content: 'secret_data'
    },
    options: {
        securityLevel: 128
    }
});

// Verify proof
const isValid = await client.verifyProof({
    proofData: proof.proofData,
    expectedContent: 'secret_data'
});
```

### Python SDK

```python
from zkipfs_proof import ZkIPFSProofClient

client = ZkIPFSProofClient(
    api_key='your_api_key',
    endpoint='http://localhost:8080'  # Use your deployed endpoint
)

# Generate proof
proof = client.generate_proof(
    file_data=file_bytes,
    content_selection={
        'type': 'pattern',
        'content': b'secret_data'
    },
    options={
        'security_level': 128
    }
)

# Verify proof
is_valid = client.verify_proof(
    proof_data=proof['proof_data'],
    expected_content=b'secret_data'
)
```

## Rate Limits and Quotas

### API Rate Limits

| Endpoint | Rate Limit | Burst Limit |
|----------|------------|-------------|
| `/api/v1/proofs/generate` | 10/minute | 20 |
| `/api/v1/proofs/verify` | 100/minute | 200 |
| `/api/v1/proofs/batch` | 5/minute | 10 |

### Resource Quotas

| Resource | Free Tier | Pro Tier | Enterprise |
|----------|-----------|----------|------------|
| Max file size | 10MB | 100MB | 1GB |
| Proofs per month | 1,000 | 10,000 | Unlimited |
| Batch size | 10 | 100 | 1,000 |
| Storage duration | 7 days | 30 days | 1 year |

## Error Codes

### HTTP Status Codes

| Code | Description |
|------|-------------|
| 200 | Success |
| 400 | Bad Request - Invalid input |
| 401 | Unauthorized - Invalid API key |
| 403 | Forbidden - Insufficient permissions |
| 413 | Payload Too Large - File size exceeded |
| 429 | Too Many Requests - Rate limit exceeded |
| 500 | Internal Server Error |
| 503 | Service Unavailable - Maintenance mode |

### Custom Error Codes

| Code | Description |
|------|-------------|
| `PROOF_GENERATION_FAILED` | ZK proof generation failed |
| `PROOF_VERIFICATION_FAILED` | Proof verification failed |
| `INVALID_CONTENT_SELECTION` | Invalid content selection format |
| `IPFS_UPLOAD_FAILED` | Failed to upload to IPFS |
| `UNSUPPORTED_FILE_TYPE` | File type not supported |
| `SECURITY_LEVEL_INVALID` | Invalid security level specified |

## Authentication

### API Key Authentication

```http
Authorization: Bearer your_api_key_here
```

### JWT Authentication

```http
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

### Smart Contract Authentication

```solidity
// Role-based access control
bytes32 public constant VERIFIER_ROLE = keccak256("VERIFIER_ROLE");
bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");

modifier onlyVerifier() {
    require(hasRole(VERIFIER_ROLE, msg.sender), "Not authorized");
    _;
}
```

## Performance Considerations

### Optimization Guidelines

1. **File Size**: Larger files take longer to process. Consider chunking files > 100MB.
2. **Security Level**: Higher security levels (256-bit) take ~3x longer than 128-bit.
3. **Content Selection**: Pattern matching is faster than byte range proofs for small patterns.
4. **Batch Processing**: Use batch APIs for multiple proofs to reduce overhead.
5. **Caching**: Verification results are cached for 1 hour by default.

### Best Practices

1. **Async Processing**: Use WebSocket API for large files to get progress updates.
2. **Error Handling**: Implement exponential backoff for rate-limited requests.
3. **Resource Management**: Clean up temporary files and close connections properly.
4. **Monitoring**: Monitor API usage to stay within quotas.

## Support and Resources

- **GitHub Repository**: [https://github.com/sowadalmughni/zkipfs-proof](https://github.com/sowadalmughni/zkipfs-proof)
- **Issue Tracker**: [https://github.com/sowadalmughni/zkipfs-proof/issues](https://github.com/sowadalmughni/zkipfs-proof/issues)
- **Email Support**: sowadalmughni@gmail.com

## Changelog

See [CHANGELOG.md](./CHANGELOG.md) for version history and breaking changes.

