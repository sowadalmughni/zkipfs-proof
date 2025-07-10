# Getting Started with zkIPFS-Proof

Welcome! If you're here, you probably need to prove something exists in a file without showing the whole thing. Maybe you're a journalist with sensitive documents, a researcher with proprietary data, or just someone who values privacy. Whatever brought you here, you're in the right place.

## What You'll Learn

By the end of this guide, you'll know how to:
- Install and set up zkIPFS-Proof
- Generate your first zero-knowledge proof
- Verify proofs from others
- Integrate zkIPFS-Proof into your workflow
- Understand when and why to use different features

## Before We Start

### What You Need
- A computer running Linux, macOS, or Windows
- About 15 minutes of your time
- A file you want to create a proof for (any file works!)

### What You Don't Need
- Deep knowledge of cryptography or zero-knowledge proofs
- Blockchain experience (though it helps)
- Programming skills (the CLI is designed for everyone)

## Installation

### Option 1: Pre-built Binaries (Recommended)

The easiest way to get started is downloading a pre-built binary:

```bash
# Linux
curl -L https://github.com/sowadmim/zkipfs-proof/releases/latest/download/zkipfs-proof-linux.tar.gz | tar xz
sudo mv zkipfs-proof /usr/local/bin/

# macOS
curl -L https://github.com/sowadmim/zkipfs-proof/releases/latest/download/zkipfs-proof-macos.tar.gz | tar xz
sudo mv zkipfs-proof /usr/local/bin/

# Windows (PowerShell)
Invoke-WebRequest -Uri "https://github.com/sowadmim/zkipfs-proof/releases/latest/download/zkipfs-proof-windows.zip" -OutFile "zkipfs-proof.zip"
Expand-Archive zkipfs-proof.zip
```

### Option 2: Install from Source

If you're a developer or want the latest features:

```bash
# Make sure you have Rust installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install zkIPFS-Proof
cargo install zkipfs-proof
```

### Verify Installation

```bash
zkipfs-proof --version
```

You should see something like `zkipfs-proof 0.1.0`. If you do, you're ready to go!

## Your First Proof

Let's start with something simple. Create a text file with some content:

```bash
echo "The secret ingredient is love" > recipe.txt
```

Now, let's prove that the word "love" exists in this file without revealing anything else:

```bash
zkipfs-proof generate --file recipe.txt --content "love" --output love-proof.json
```

You'll see output like this:

```
🔍 Analyzing file: recipe.txt
📊 File size: 29 bytes
🔐 Generating zero-knowledge proof...
⚡ Proof generated in 2.3 seconds
💾 Proof saved to: love-proof.json
🌐 IPFS CID: QmYourFileHashHere
```

Congratulations! You've just created your first zero-knowledge proof. The `love-proof.json` file contains cryptographic evidence that "love" exists in your recipe file, but reveals nothing else about the content.

## Verifying the Proof

Now let's verify the proof. This is what someone else would do to check your claim:

```bash
zkipfs-proof verify --proof love-proof.json
```

Output:
```
✅ Proof verified successfully
📋 Proof Details:
   - Content proven: "love"
   - File CID: QmYourFileHashHere
   - Security level: 128-bit
   - Generated: 2024-01-15 14:30:22 UTC
   - Verification time: 0.8 seconds
```

The verification confirms that "love" definitely exists in the file with CID `QmYourFileHashHere`, but the verifier learns nothing else about the file's contents.

## Understanding What Just Happened

When you generated the proof, several things happened behind the scenes:

1. **File Analysis**: zkIPFS-Proof read your file and identified where "love" appears
2. **IPFS Upload**: Your file was uploaded to IPFS and assigned a unique content identifier (CID)
3. **Proof Generation**: A zero-knowledge circuit analyzed the file and created a cryptographic proof
4. **Proof Packaging**: The proof was packaged into a JSON file with metadata

The beautiful thing is that the proof is completely self-contained. Anyone can verify it without needing access to your original file.

## More Advanced Examples

### Proving Multiple Words

You can prove multiple pieces of content exist:

```bash
zkipfs-proof generate --file document.pdf --content "confidential,2024,approved" --output multi-proof.json
```

### Proving Byte Ranges

Sometimes you want to prove specific sections of a file:

```bash
zkipfs-proof generate --file data.csv --range "100-200" --output range-proof.json
```

### Different Security Levels

For highly sensitive content, use higher security:

```bash
zkipfs-proof generate --file sensitive.txt --content "classified" --security-level 256 --output secure-proof.json
```

Higher security levels take longer to generate but provide stronger guarantees.

## Working with IPFS

zkIPFS-Proof integrates seamlessly with IPFS for decentralized storage. Here are some useful commands:

### Upload a File to IPFS

```bash
zkipfs-proof ipfs upload --file document.pdf
```

### Download from IPFS

```bash
zkipfs-proof ipfs download --cid QmYourCIDHere --output downloaded-file.pdf
```

### Check IPFS Status

```bash
zkipfs-proof ipfs status
```

## Configuration

zkIPFS-Proof can be configured to match your workflow. Initialize a config file:

```bash
zkipfs-proof init
```

This creates a configuration file at `~/.zkipfs-proof/config.toml`. You can customize:

- Default security levels
- IPFS endpoints
- Output formats
- Performance settings

## Common Use Cases

### Journalist Scenario

Sarah is an investigative journalist who received a leaked document containing evidence of corporate wrongdoing. She needs to prove specific information exists in the document without revealing her source or other sensitive details.

```bash
# Generate proof for specific evidence
zkipfs-proof generate --file leaked-emails.pdf --content "Project Blackwater budget: $2.3M" --output evidence-proof.json

# Share the proof with editors and fact-checkers
zkipfs-proof verify --proof evidence-proof.json
```

### Researcher Scenario

Dr. Kim has a proprietary dataset and needs to prove certain findings without sharing the raw data.

```bash
# Prove statistical findings exist in the dataset
zkipfs-proof generate --file research-data.csv --content "correlation coefficient: 0.847" --security-level 256 --output research-proof.json
```

### Auditor Scenario

Alex is auditing a company's financial records and needs to prove compliance without exposing confidential information.

```bash
# Prove specific transactions exist
zkipfs-proof generate --file transactions.xlsx --content "Compliance check: PASSED" --output audit-proof.json
```

## Best Practices

### Security
- Use higher security levels (256-bit) for sensitive content
- Store proof files securely - they contain metadata about your files
- Regularly update zkIPFS-Proof to get security patches

### Performance
- For large files (>1GB), consider using byte ranges instead of content search
- Use local IPFS nodes for better performance
- Generate proofs on machines with sufficient RAM

### Privacy
- Be careful about what content you choose to prove - it will be visible in the proof
- Consider the implications of IPFS storage (files are publicly accessible via CID)
- Use private IPFS networks for sensitive files

## Troubleshooting

### "Command not found"
Make sure zkIPFS-Proof is in your PATH. Try the full path to the binary or reinstall.

### "IPFS connection failed"
Start a local IPFS node:
```bash
ipfs daemon
```

Or configure a remote IPFS endpoint in your config file.

### "Proof generation failed"
- Check that your file exists and is readable
- Ensure you have enough disk space and memory
- Try a lower security level for testing

### "Content not found"
- Verify the content exists in your file (case-sensitive)
- Try using byte ranges instead of content search
- Check for encoding issues with non-ASCII content

## Next Steps

Now that you understand the basics, you might want to:

- Read the [API Documentation](api-reference.md) for programmatic usage
- Check out [Integration Examples](examples/) for real-world scenarios
- Learn about [Smart Contract Integration](smart-contracts.md) for on-chain verification
- Explore [Advanced Features](advanced-usage.md) like batch processing and custom circuits

## Getting Help

Stuck on something? Here's how to get help:

- Check the [FAQ](faq.md) for common questions
- Browse [GitHub Issues](https://github.com/sowadalmughni/zkipfs-proof/issues) for known problems
- Start a discussion in GitHub Discussions for community help
- Email us at sowadalmughni@gmail.com for direct assistance

Remember, the best way to learn is by doing. Try generating proofs for different types of files and content. The more you experiment, the better you'll understand when and how to use zkIPFS-Proof effectively.

Welcome to the world of zero-knowledge proofs!

