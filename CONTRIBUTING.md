# Contributing to zkIPFS-Proof

First off, thanks for considering contributing to zkIPFS-Proof! It's people like you that make this project possible.

## Code of Conduct

This project and everyone participating in it is governed by our Code of Conduct. By participating, you are expected to uphold this code. Please report unacceptable behavior to sowadalmughni@gmail.com.

## How Can I Contribute?

### Reporting Bugs

Before creating bug reports, please check the existing issues as you might find out that you don't need to create one. When you are creating a bug report, please include as many details as possible using our bug report template.

### Suggesting Enhancements

Enhancement suggestions are tracked as GitHub issues. When creating an enhancement suggestion, please use our feature request template and provide as much detail as possible.

### Your First Code Contribution

Unsure where to begin contributing? You can start by looking through these `beginner` and `help-wanted` issues:

- **Beginner issues** - issues which should only require a few lines of code, and a test or two
- **Help wanted issues** - issues which should be a bit more involved than `beginner` issues

### Pull Requests

The process described here has several goals:

- Maintain zkIPFS-Proof's quality
- Fix problems that are important to users
- Engage the community in working toward the best possible zkIPFS-Proof
- Enable a sustainable system for maintainers to review contributions

Please follow these steps to have your contribution considered by the maintainers:

1. Follow all instructions in the template
2. Follow the styleguides
3. After you submit your pull request, verify that all status checks are passing

## Development Setup

### Prerequisites

- **Rust** (latest stable version)
- **Node.js** (v18 or later)
- **Git**
- **IPFS** (optional, for testing IPFS features)

### Setting Up Your Development Environment

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/YOUR-USERNAME/zkipfs-proof.git
   cd zkipfs-proof
   ```

3. **Add the original repository** as upstream:
   ```bash
   git remote add upstream https://github.com/sowadalmughni/zkipfs-proof.git
   ```

4. **Install Rust dependencies**:
   ```bash
   cargo build
   ```

5. **Install Node.js dependencies** (for web app):
   ```bash
   cd frontend/web
   npm install
   cd ../..
   ```

6. **Run tests** to make sure everything works:
   ```bash
   cargo test
   ```

### Project Structure

```
zkipfs-proof/
├── backend/
│   ├── core/              # Core Rust library
│   ├── cli/               # Command-line interface
│   └── contracts/         # Smart contracts
├── frontend/
│   └── web/               # React web application
├── docs/                  # Documentation
├── examples/              # Usage examples
└── .github/               # GitHub workflows and templates
```

## Styleguides

### Git Commit Messages

We use [Conventional Commits](https://www.conventionalcommits.org/) for our commit messages:

- `feat:` for new features
- `fix:` for bug fixes
- `docs:` for documentation changes
- `style:` for formatting changes
- `refactor:` for code refactoring
- `test:` for adding tests
- `chore:` for maintenance tasks

Examples:
```
feat(cli): add batch proof generation command
fix(core): resolve memory leak in proof verification
docs(api): update API reference for new endpoints
```

### Rust Style Guide

- Follow the official [Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/)
- Use `cargo fmt` to format your code
- Use `cargo clippy` to catch common mistakes
- Write comprehensive tests for new functionality
- Document public APIs with doc comments

### JavaScript/TypeScript Style Guide

- Use Prettier for code formatting
- Follow ESLint rules configured in the project
- Use TypeScript for type safety
- Write unit tests for components and utilities

### Documentation Style Guide

- Use clear, concise language
- Include code examples where appropriate
- Keep line length under 80 characters for markdown files
- Use proper markdown formatting

## Testing

### Running Tests

```bash
# Run all Rust tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run web app tests
cd frontend/web
npm test
```

### Writing Tests

- Write unit tests for all new functions
- Include integration tests for major features
- Test error conditions and edge cases
- Use descriptive test names

Example:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proof_generation_with_valid_content() {
        // Test implementation
    }

    #[test]
    fn test_proof_generation_with_invalid_file() {
        // Test implementation
    }
}
```

## Documentation

### API Documentation

- Document all public functions with doc comments
- Include examples in doc comments
- Use `cargo doc` to generate documentation

### User Documentation

- Update relevant documentation when adding features
- Include examples and use cases
- Keep documentation up to date with code changes

## Release Process

### Versioning

We use [Semantic Versioning](https://semver.org/):

- **MAJOR** version for incompatible API changes
- **MINOR** version for backwards-compatible functionality additions
- **PATCH** version for backwards-compatible bug fixes

### Creating a Release

1. Update version numbers in `Cargo.toml` files
2. Update `CHANGELOG.md` with new changes
3. Create a pull request with these changes
4. After merge, create a git tag and GitHub release

## Community

### Getting Help

- **GitHub Discussions**: Use GitHub Discussions for questions and longer-form conversations
- **Issues**: Create issues for bugs and feature requests
- **Email**: Contact sowadalmughni@gmail.com for direct support

### Staying Updated

- Watch the repository for notifications
- Follow [@sowadalmughni](https://github.com/sowadalmughni) for updates
- Enable GitHub notifications for important announcements

## Recognition

Contributors are recognized in several ways:

- Listed in the `CONTRIBUTORS.md` file
- Mentioned in release notes for significant contributions
- Invited to join the core team for sustained contributions

## Questions?

Don't hesitate to ask questions! You can:

- Open an issue with the `question` label
- Start a discussion in GitHub Discussions
- Email sowadalmughni@gmail.com

## License

By contributing to zkIPFS-Proof, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to zkIPFS-Proof! 🚀

*This project is maintained by [Sowad Al-Mughni](https://github.com/sowadalmughni) and the open-source community.*

