# zkIPFS-Proof Examples

This directory contains real-world examples of how to use zkIPFS-Proof in different scenarios. Each example includes complete code, explanations, and expected outputs.

## Directory Structure

```
examples/
├── journalism/          # Investigative journalism use cases
├── research/            # Academic and scientific research
├── auditing/            # Financial and compliance auditing
├── legal/               # Legal document verification
├── integration/         # API and system integration examples
└── advanced/            # Advanced features and custom implementations
```

## Quick Examples

### Basic Proof Generation

```bash
# Create a simple text file
echo "This document contains sensitive information about Project Alpha." > document.txt

# Generate proof that "Project Alpha" exists without revealing other content
zkipfs-proof generate --file document.txt --content "Project Alpha" --output alpha-proof.json

# Verify the proof
zkipfs-proof verify --proof alpha-proof.json
```

### Journalist Whistleblower Scenario

A journalist receives leaked documents and needs to prove specific information exists without exposing sources.

```bash
# Generate proof for specific evidence
zkipfs-proof generate \
  --file leaked-emails.pdf \
  --content "Budget allocation: $2.3M for Project Blackwater" \
  --security-level 256 \
  --output evidence-proof.json

# The proof can now be shared with editors, fact-checkers, and the public
# without revealing the source or other sensitive information
```

### Research Data Validation

A researcher needs to prove statistical findings exist in a proprietary dataset.

```bash
# Prove specific statistical results
zkipfs-proof generate \
  --file research-data.csv \
  --content "correlation coefficient: 0.847, p-value: 0.003" \
  --metadata '{"study":"climate-impact","version":"2.1"}' \
  --output research-proof.json

# Upload to IPFS for permanent storage
zkipfs-proof ipfs upload research-data.csv --pin
```

### Financial Audit Compliance

An auditor needs to prove compliance without exposing confidential financial data.

```bash
# Generate proof for compliance check
zkipfs-proof generate \
  --file financial-records.xlsx \
  --content "SOX compliance: PASSED" \
  --range "1000-2000" \
  --security-level 256 \
  --output compliance-proof.json
```

## Integration Examples

### GitHub Actions Workflow

```yaml
name: Document Verification
on: [push, pull_request]

jobs:
  verify-docs:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: sowadmim/zkipfs-proof@v1
        with:
          mode: 'generate'
          files: 'docs/**/*.md'
          security-level: '128'
```

### Python Integration

```python
import subprocess
import json

def generate_proof(file_path, content):
    """Generate a zkIPFS proof using the CLI."""
    result = subprocess.run([
        'zkipfs-proof', 'generate',
        '--file', file_path,
        '--content', content,
        '--output', 'proof.json',
        '--format', 'json'
    ], capture_output=True, text=True)
    
    if result.returncode == 0:
        with open('proof.json', 'r') as f:
            return json.load(f)
    else:
        raise Exception(f"Proof generation failed: {result.stderr}")

# Usage
proof = generate_proof('document.pdf', 'confidential information')
print(f"Proof generated with CID: {proof['file_cid']}")
```

### Node.js Integration

```javascript
const { exec } = require('child_process');
const fs = require('fs');

async function generateProof(filePath, content) {
    return new Promise((resolve, reject) => {
        exec(`zkipfs-proof generate --file "${filePath}" --content "${content}" --output proof.json`, 
            (error, stdout, stderr) => {
                if (error) {
                    reject(new Error(`Proof generation failed: ${stderr}`));
                } else {
                    const proof = JSON.parse(fs.readFileSync('proof.json', 'utf8'));
                    resolve(proof);
                }
            });
    });
}

// Usage
generateProof('document.pdf', 'sensitive data')
    .then(proof => console.log('Proof generated:', proof.file_cid))
    .catch(err => console.error('Error:', err.message));
```

## Advanced Examples

### Batch Processing

Process multiple files and generate proofs for each:

```bash
#!/bin/bash

# Directory containing files to process
FILES_DIR="./documents"
PROOFS_DIR="./proofs"

mkdir -p "$PROOFS_DIR"

# Process each file
for file in "$FILES_DIR"/*; do
    if [ -f "$file" ]; then
        filename=$(basename "$file")
        echo "Processing: $filename"
        
        zkipfs-proof generate \
            --file "$file" \
            --content "confidential" \
            --output "$PROOFS_DIR/${filename}.proof.json" \
            --security-level 128
        
        echo "Proof generated: $PROOFS_DIR/${filename}.proof.json"
    fi
done

echo "Batch processing complete!"
```

### Smart Contract Integration

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./ZkIPFSVerifier.sol";

contract DocumentRegistry {
    ZkIPFSVerifier public verifier;
    
    struct Document {
        bytes32 proofHash;
        string ipfsCid;
        address submitter;
        uint256 timestamp;
        bool verified;
    }
    
    mapping(bytes32 => Document) public documents;
    
    event DocumentRegistered(bytes32 indexed proofHash, string ipfsCid, address submitter);
    
    constructor(address _verifier) {
        verifier = ZkIPFSVerifier(_verifier);
    }
    
    function registerDocument(
        bytes calldata proof,
        bytes32[] calldata publicInputs,
        string calldata ipfsCid
    ) external {
        // Verify the proof
        bool isValid = verifier.verifyProof(proof, publicInputs, 128);
        require(isValid, "Invalid proof");
        
        bytes32 proofHash = keccak256(proof);
        
        documents[proofHash] = Document({
            proofHash: proofHash,
            ipfsCid: ipfsCid,
            submitter: msg.sender,
            timestamp: block.timestamp,
            verified: true
        });
        
        emit DocumentRegistered(proofHash, ipfsCid, msg.sender);
    }
}
```

## Performance Benchmarks

### File Size vs Generation Time

| File Size | Security Level | Generation Time | Memory Usage |
|-----------|----------------|-----------------|--------------|
| 1 KB      | 128-bit        | 0.5s           | 50 MB        |
| 1 MB      | 128-bit        | 2.1s           | 120 MB       |
| 10 MB     | 128-bit        | 8.7s           | 350 MB       |
| 100 MB    | 128-bit        | 45s            | 1.2 GB       |
| 1 GB      | 128-bit        | 380s           | 4.5 GB       |

### Security Level Comparison

| Security Level | Generation Time | Proof Size | Verification Time |
|----------------|-----------------|------------|-------------------|
| 64-bit         | 1x (baseline)   | 2.1 KB     | 0.3s             |
| 128-bit        | 2.3x            | 4.2 KB     | 0.8s             |
| 256-bit        | 5.1x            | 8.4 KB     | 1.9s             |

## Troubleshooting Common Issues

### Large File Processing

For files larger than 1GB, consider these optimizations:

```bash
# Use byte ranges instead of content search
zkipfs-proof generate --file large-file.dat --range "1000000-2000000" --output proof.json

# Use lower security level for faster processing
zkipfs-proof generate --file large-file.dat --content "pattern" --security-level 64

# Increase memory limits in config
zkipfs-proof config set performance.max_in_memory_size_mb 8192
```

### IPFS Connection Issues

```bash
# Check IPFS status
zkipfs-proof ipfs status

# Start local IPFS node
ipfs daemon

# Use remote IPFS node
zkipfs-proof config set ipfs.api_url "https://ipfs.infura.io:5001"
```

### Performance Optimization

```bash
# Enable hardware acceleration
zkipfs-proof config set default.use_hardware_acceleration true

# Adjust worker threads
zkipfs-proof config set performance.worker_threads 8

# Use compression for large files
zkipfs-proof generate --file large.pdf --content "text" --compression zstd
```

## Contributing Examples

We welcome contributions of new examples! Please follow these guidelines:

1. **Real-world scenarios**: Examples should solve actual problems
2. **Complete code**: Include all necessary files and dependencies
3. **Clear documentation**: Explain the use case and expected outcomes
4. **Test thoroughly**: Ensure examples work on different systems
5. **Follow conventions**: Use consistent naming and structure

### Example Template

```
examples/your-use-case/
├── README.md           # Detailed explanation
├── setup.sh           # Setup script
├── example.sh          # Main example script
├── sample-data/        # Sample input files
└── expected-output/    # Expected results
```

## Getting Help

If you're having trouble with any of these examples:

1. Check the [troubleshooting guide](../docs/troubleshooting.md)
2. Review the [API documentation](../docs/api-reference.md)
3. Ask questions in our [Discord community](https://discord.gg/zkipfs-proof)
4. Open an issue on [GitHub](https://github.com/sowadmim/zkipfs-proof/issues)

## License

All examples are released under the MIT License, same as the main project.

