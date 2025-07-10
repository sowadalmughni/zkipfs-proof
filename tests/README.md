# zkIPFS-Proof Testing Suite

This directory contains comprehensive tests for the zkIPFS-Proof system, covering all components from the core Rust library to the web interface.

## Test Structure

```
tests/
├── unit/                 # Unit tests for individual components
│   ├── core/            # Core library tests
│   ├── cli/             # CLI tool tests
│   └── contracts/       # Smart contract tests
├── integration/         # Integration tests
│   ├── proof_generation/
│   ├── verification/
│   └── ipfs_integration/
├── e2e/                 # End-to-end tests
│   ├── web_interface/
│   └── cli_workflows/
├── performance/         # Performance benchmarks
├── security/           # Security tests
└── fixtures/           # Test data and fixtures
```

## Running Tests

### Rust Tests
```bash
# Run all Rust tests
cargo test --workspace

# Run specific test suite
cargo test -p zkipfs-proof-core
cargo test -p zkipfs-proof-cli

# Run with coverage
cargo test --workspace -- --nocapture
```

### Smart Contract Tests
```bash
cd contracts
forge test
```

### Web Interface Tests
```bash
cd frontend/web
npm test
```

### End-to-End Tests
```bash
# Run full E2E test suite
./scripts/run_e2e_tests.sh
```

## Test Categories

### 1. Unit Tests
- Core proof generation logic
- IPFS client functionality
- Cryptographic operations
- CLI command parsing
- Smart contract functions

### 2. Integration Tests
- Proof generation and verification workflow
- IPFS content storage and retrieval
- CLI tool integration with core library
- Smart contract deployment and interaction

### 3. End-to-End Tests
- Complete user workflows through web interface
- CLI tool complete workflows
- Cross-component integration

### 4. Performance Tests
- Proof generation benchmarks
- Memory usage analysis
- Large file handling
- Concurrent operations

### 5. Security Tests
- Input validation
- Cryptographic security
- Access control
- Vulnerability scanning

## Test Data

The `fixtures/` directory contains:
- Sample files for testing (various sizes and formats)
- Pre-generated proofs for verification tests
- Mock IPFS responses
- Test configuration files

## Continuous Integration

Tests are automatically run on:
- Every pull request
- Main branch commits
- Nightly builds for performance regression detection

## Coverage Requirements

- Core library: >95% code coverage
- CLI tool: >90% code coverage
- Smart contracts: >95% code coverage
- Web interface: >85% code coverage

## Writing New Tests

When adding new functionality:
1. Write unit tests for individual functions
2. Add integration tests for component interactions
3. Update E2E tests if user-facing changes
4. Add performance tests for critical paths
5. Include security tests for new attack vectors

## Test Environment Setup

See `scripts/setup_test_env.sh` for automated test environment configuration.

