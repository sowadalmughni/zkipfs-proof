# zkIPFS-Proof GitHub Action

Integrate zero-knowledge proof generation and verification into your CI/CD pipeline. Prove file authenticity without revealing sensitive content.

## Features

- **Generate Proofs**: Create cryptographic proofs for any file content
- **Verify Proofs**: Validate existing proofs in your pipeline
- **IPFS Integration**: Upload files to IPFS and generate proofs
- **Multiple Modes**: Generate, verify, or validate file integrity
- **Flexible Configuration**: Support for various security levels and file patterns
- **Artifact Management**: Automatic upload of proof files as GitHub artifacts
- **PR Comments**: Automatic commenting on pull requests with proof results

## Quick Start

```yaml
name: Content Verification
on: [push, pull_request]

jobs:
  verify:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: sowadmim/zkipfs-proof@v1
        with:
          mode: 'generate'
          files: 'docs/**/*.md'
```

## Inputs

| Input | Description | Required | Default |
|-------|-------------|----------|---------|
| `mode` | Action mode: `generate`, `verify`, or `validate` | Yes | `generate` |
| `files` | Files to process (glob patterns supported) | No | `**/*` |
| `content` | Specific content to prove (for generate mode) | No | - |
| `proof` | Proof file path (for verify mode) | No | - |
| `output-dir` | Output directory for generated proofs | No | `./proofs` |
| `security-level` | Security level: `64`, `128`, or `256` | No | `128` |
| `fail-on-error` | Fail the action if operations fail | No | `true` |
| `upload-to-ipfs` | Upload files to IPFS before generating proofs | No | `false` |
| `ipfs-api-url` | IPFS API URL | No | `http://127.0.0.1:5001` |
| `comment-pr` | Comment proof results on pull requests | No | `true` |
| `artifact-name` | Name for the proof artifacts | No | `zkipfs-proofs` |

## Outputs

| Output | Description |
|--------|-------------|
| `success` | Whether the action completed successfully |
| `proof-count` | Number of proofs generated or verified |
| `proof-files` | JSON array of generated proof file paths |
| `ipfs-cids` | JSON array of IPFS CIDs (if uploaded) |
| `summary` | Summary of the action results |

## Usage Examples

### Basic Proof Generation

Generate proofs for all Markdown files:

```yaml
- uses: sowadmim/zkipfs-proof@v1
  with:
    mode: 'generate'
    files: '**/*.md'
    security-level: '128'
```

### Verify Existing Proofs

Verify all proof files in a directory:

```yaml
- uses: sowadmim/zkipfs-proof@v1
  with:
    mode: 'verify'
    output-dir: './existing-proofs'
```

### Content-Specific Proof

Generate a proof for specific content within files:

```yaml
- uses: sowadmim/zkipfs-proof@v1
  with:
    mode: 'generate'
    files: 'contracts/**/*.sol'
    content: 'function transfer'
    security-level: '256'
```

### IPFS Integration

Upload files to IPFS and generate proofs:

```yaml
- uses: sowadmim/zkipfs-proof@v1
  with:
    mode: 'generate'
    files: 'README.md'
    upload-to-ipfs: true
    ipfs-api-url: 'http://localhost:5001'
```

### File Integrity Validation

Validate that files haven't changed since proof generation:

```yaml
- uses: sowadmim/zkipfs-proof@v1
  with:
    mode: 'validate'
    files: 'src/**/*.rs'
    output-dir: './proofs'
```

### Multiple File Patterns

Process multiple file types:

```yaml
- uses: sowadmim/zkipfs-proof@v1
  with:
    mode: 'generate'
    files: |
      docs/**/*.md
      src/**/*.rs
      contracts/**/*.sol
```

## Advanced Workflows

### Security Audit Pipeline

```yaml
name: Security Audit
on:
  schedule:
    - cron: '0 2 * * *'  # Daily at 2 AM

jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Generate audit proofs
        uses: sowadmim/zkipfs-proof@v1
        with:
          mode: 'generate'
          files: |
            contracts/**/*.sol
            src/**/*.rs
          security-level: '256'
          artifact-name: 'security-audit-proofs'
      
      - name: Validate previous proofs
        uses: sowadmim/zkipfs-proof@v1
        with:
          mode: 'validate'
          files: '**/*'
          fail-on-error: false
```

### Documentation Integrity

```yaml
name: Documentation Integrity
on:
  push:
    paths: ['docs/**']

jobs:
  verify-docs:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Generate documentation proofs
        uses: sowadmim/zkipfs-proof@v1
        with:
          mode: 'generate'
          files: 'docs/**/*.md'
          upload-to-ipfs: true
          comment-pr: true
      
      - name: Store proofs
        run: |
          git config --local user.email "action@github.com"
          git config --local user.name "GitHub Action"
          git add proofs/
          git commit -m "Update documentation proofs" || exit 0
          git push
```

### Release Verification

```yaml
name: Release Verification
on:
  release:
    types: [published]

jobs:
  verify-release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Generate release proofs
        uses: sowadmim/zkipfs-proof@v1
        with:
          mode: 'generate'
          files: |
            README.md
            CHANGELOG.md
            src/**/*.rs
          security-level: '256'
          upload-to-ipfs: true
          artifact-name: 'release-${{ github.event.release.tag_name }}-proofs'
```

## Error Handling

The action provides detailed error messages and can be configured to continue on errors:

```yaml
- uses: sowadmim/zkipfs-proof@v1
  with:
    mode: 'generate'
    files: '**/*.md'
    fail-on-error: false  # Continue even if some proofs fail
```

## Security Considerations

- **Security Levels**: Higher security levels (256-bit) provide stronger guarantees but take longer to generate
- **Sensitive Content**: The action never exposes file content in logs or outputs
- **IPFS Privacy**: When using IPFS, consider whether files should be publicly accessible
- **Proof Storage**: Store proof files securely and consider their retention policies

## Troubleshooting

### Common Issues

**IPFS Connection Failed**
```yaml
# Ensure IPFS is running or use a remote node
- name: Setup IPFS
  run: |
    ipfs init
    ipfs daemon &
    sleep 5
```

**Large File Processing**
```yaml
# Increase security level for better performance with large files
- uses: sowadmim/zkipfs-proof@v1
  with:
    security-level: '64'  # Faster for large files
```

**Permission Errors**
```yaml
# Ensure proper permissions for output directory
- name: Setup permissions
  run: mkdir -p ./proofs && chmod 755 ./proofs
```

## Contributing

This action is part of the [zkIPFS-Proof](https://github.com/sowadmim/zkipfs-proof) project. Contributions are welcome!

## License

MIT License - see the [LICENSE](../../../LICENSE) file for details.

