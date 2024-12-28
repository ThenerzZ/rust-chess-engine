# Contributing to the Chess Engine

## Table of Contents
1. [Introduction](#introduction)
2. [Code of Conduct](#code-of-conduct)
3. [Getting Started](#getting-started)
4. [Development Environment](#development-environment)
5. [Code Style](#code-style)
6. [Making Contributions](#making-contributions)
7. [Testing Guidelines](#testing-guidelines)
8. [Documentation](#documentation)
9. [Performance Considerations](#performance-considerations)
10. [Review Process](#review-process)

## Introduction

Thank you for considering contributing to our chess engine! This document provides guidelines and best practices for contributing to the project.

## Code of Conduct

### Our Pledge
We are committed to providing a friendly, safe, and welcoming environment for all contributors, regardless of experience level, gender identity and expression, sexual orientation, disability, personal appearance, body size, race, ethnicity, age, religion, nationality, or other similar characteristics.

### Expected Behavior
- Use welcoming and inclusive language
- Be respectful of differing viewpoints and experiences
- Gracefully accept constructive criticism
- Focus on what is best for the community
- Show empathy towards other community members

## Getting Started

### Prerequisites
- Rust 1.70 or higher
- Cargo package manager
- Git
- A GitHub account
- (Optional) An IDE with Rust support (VS Code with rust-analyzer recommended)

### Initial Setup
1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/your-username/chess-engine.git
   cd chess-engine
   ```
3. Add the upstream repository:
   ```bash
   git remote add upstream https://github.com/ThenerzZ/chess-engine.git
   ```
4. Create a new branch for your work:
   ```bash
   git checkout -b feature/your-feature-name
   ```

## Development Environment

### Recommended Tools
- VS Code with extensions:
  - rust-analyzer
  - CodeLLDB
  - Better TOML
  - Error Lens
- Clippy for linting
- Rustfmt for formatting
- Cargo audit for security checks

### Environment Setup
1. Install Rust toolchain:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
2. Install development tools:
   ```bash
   rustup component add clippy rustfmt
   cargo install cargo-audit
   ```

## Code Style

### General Guidelines
- Follow Rust idioms and best practices
- Use meaningful variable and function names
- Keep functions focused and small
- Document complex algorithms
- Use type inference when obvious
- Prefer pattern matching over if-let chains

### Formatting
- Use `rustfmt` with the project's configuration
- Maximum line length: 100 characters
- Use 4 spaces for indentation
- Run before committing:
  ```bash
  cargo fmt --all
  ```

### Naming Conventions
- Use snake_case for functions and variables
- Use PascalCase for types and traits
- Use SCREAMING_SNAKE_CASE for constants
- Prefix unsafe functions with `unsafe_`
- Use descriptive names for generics (e.g., `T` for general types, `E` for errors)

## Making Contributions

### Branch Strategy
- `main` - stable release branch
- `develop` - main development branch
- `feature/*` - new features
- `bugfix/*` - bug fixes
- `refactor/*` - code refactoring
- `docs/*` - documentation updates

### Commit Guidelines
- Use conventional commits format:
  ```
  <type>(<scope>): <description>

  [optional body]

  [optional footer]
  ```
- Types: feat, fix, docs, style, refactor, test, chore
- Keep commits atomic and focused
- Reference issues in commit messages

### Pull Request Process
1. Update documentation
2. Add/update tests
3. Run the test suite
4. Update the changelog
5. Submit PR against `develop`
6. Address review comments
7. Ensure CI passes

## Testing Guidelines

### Test Structure
- Unit tests in the same file as the code
- Integration tests in `tests/` directory
- Benchmarks in `benches/` directory
- Documentation tests in doc comments

### Test Categories
1. Unit Tests
   - Test individual functions
   - Mock dependencies
   - Fast execution

2. Integration Tests
   - Test component interaction
   - Real dependencies
   - Complete workflows

3. Performance Tests
   - Benchmark critical paths
   - Compare against baselines
   - Test with large datasets

### Running Tests
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run benchmarks
cargo bench

# Run with release optimizations
cargo test --release
```

## Documentation

### Code Documentation
- Document all public items
- Include examples in doc comments
- Explain complex algorithms
- Document performance implications
- Include links to relevant resources

### Project Documentation
- Keep README.md updated
- Document architectural decisions
- Maintain CHANGELOG.md
- Update API documentation
- Include diagrams where helpful

### Documentation Style
- Use clear, concise language
- Include code examples
- Explain why, not just what
- Keep formatting consistent
- Update docs with code changes

## Performance Considerations

### Optimization Guidelines
1. Profile before optimizing
2. Document performance requirements
3. Use benchmarks to verify improvements
4. Consider memory usage
5. Test with realistic workloads

### Common Optimizations
- Use appropriate data structures
- Minimize allocations
- Leverage parallelism
- Cache frequently used values
- Use efficient algorithms

### Performance Testing
- Benchmark critical operations
- Test with large datasets
- Compare against baselines
- Document performance characteristics

## Review Process

### Before Submitting
- [ ] Run all tests
- [ ] Update documentation
- [ ] Format code
- [ ] Run lints
- [ ] Update changelog
- [ ] Self-review changes

### Review Criteria
- Code quality and style
- Test coverage
- Documentation
- Performance impact
- Security implications
- Maintainability

### Review Timeline
1. Initial review within 2 business days
2. Address feedback within 1 week
3. Final review within 2 business days
4. Merge within 1 business day

### After Merge
- Delete feature branch
- Update related issues
- Monitor CI/CD pipeline
- Verify deployment
- Update project board 