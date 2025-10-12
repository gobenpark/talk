# Contributing to Talk

Thank you for your interest in contributing to Talk! This document provides guidelines and instructions for contributing.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Making Changes](#making-changes)
- [Testing](#testing)
- [Submitting Changes](#submitting-changes)
- [Code Style](#code-style)
- [Commit Guidelines](#commit-guidelines)
- [Documentation](#documentation)

## Code of Conduct

This project adheres to a code of conduct that all contributors are expected to follow. Please be respectful and constructive in all interactions.

### Expected Behavior

- Use welcoming and inclusive language
- Be respectful of differing viewpoints and experiences
- Gracefully accept constructive criticism
- Focus on what is best for the community
- Show empathy towards other community members

## Getting Started

### Prerequisites

- Rust 1.90 or later
- Git
- A GitHub account

### Finding Issues to Work On

- Look for issues labeled `good first issue` for beginner-friendly tasks
- Check issues labeled `help wanted` for tasks that need contributors
- Browse open issues and comment if you'd like to work on something

## Development Setup

### 1. Fork and Clone

```bash
# Fork the repository on GitHub, then clone your fork
git clone https://github.com/YOUR_USERNAME/talk.git
cd talk

# Add upstream remote
git remote add upstream https://github.com/gobenpark/talk.git
```

### 2. Install Dependencies

```bash
# Build the project
cargo build

# Run tests to verify setup
cargo test
```

### 3. Optional: Install Development Tools

```bash
# Code formatting
rustup component add rustfmt

# Linting
rustup component add clippy

# Documentation generation
cargo install cargo-docs
```

## Making Changes

### 1. Create a Branch

```bash
# Update your main branch
git checkout main
git pull upstream main

# Create a feature branch
git checkout -b feature/my-feature
# or
git checkout -b fix/my-bugfix
```

### 2. Make Your Changes

- Write clear, concise code
- Follow the existing code style
- Add tests for new functionality
- Update documentation as needed
- Keep commits focused and atomic

### 3. Run Quality Checks

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Run tests
cargo test

# Test documentation
cargo test --doc

# Build documentation
cargo doc --no-deps
```

## Testing

### Test Organization

- **Unit tests**: In `#[cfg(test)]` modules within source files
- **Integration tests**: In `tests/` directory
- **Doc tests**: In documentation comments

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        // Arrange
        let input = "test";

        // Act
        let result = my_function(input);

        // Assert
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_async_function() {
        let result = async_function().await;
        assert!(result.is_ok());
    }
}
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run tests for specific module
cargo test guideline

# Run integration tests only
cargo test --test '*'
```

## Submitting Changes

### 1. Before Submitting

- [ ] All tests pass (`cargo test`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Documentation is updated
- [ ] Commit messages follow guidelines

### 2. Push Your Changes

```bash
git push origin feature/my-feature
```

### 3. Create Pull Request

1. Go to your fork on GitHub
2. Click "New Pull Request"
3. Select your branch
4. Fill out the PR template:

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Added unit tests
- [ ] Added integration tests
- [ ] Tested manually

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] No new warnings
```

### 4. Review Process

- Maintainers will review your PR
- Address feedback by pushing new commits
- Once approved, your PR will be merged

## Code Style

### Rust Style Guide

Follow the [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/):

```rust
// Good: Clear, idiomatic Rust
pub fn calculate_score(items: &[Item]) -> f32 {
    items.iter()
        .map(|item| item.value)
        .sum::<f32>() / items.len() as f32
}

// Avoid: Unclear, non-idiomatic
pub fn calc(i: &[Item]) -> f32 {
    let mut s = 0.0;
    for x in i {
        s += x.value;
    }
    s / i.len() as f32
}
```

### Naming Conventions

- **Types**: `PascalCase` (e.g., `Agent`, `GuidelineMatch`)
- **Functions/Variables**: `snake_case` (e.g., `create_session`, `tool_id`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `DEFAULT_TIMEOUT`)
- **Lifetimes**: `'lowercase` (e.g., `'a`, `'static`)

### Documentation

```rust
/// Brief one-line description.
///
/// Detailed explanation with examples.
///
/// # Examples
///
/// ```
/// use talk::Agent;
/// let agent = Agent::builder().build()?;
/// ```
///
/// # Errors
///
/// Returns `Err` if...
///
/// # Panics
///
/// Panics if...
pub fn my_function() -> Result<()> {
    // Implementation
}
```

## Commit Guidelines

### Commit Message Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

- **feat**: New feature
- **fix**: Bug fix
- **docs**: Documentation changes
- **style**: Code style changes (formatting, etc.)
- **refactor**: Code refactoring
- **test**: Adding or updating tests
- **chore**: Maintenance tasks
- **perf**: Performance improvements

### Examples

```bash
# Feature
git commit -m "feat(agent): add journey state tracking"

# Bug fix
git commit -m "fix(tool): handle timeout correctly in retry logic"

# Documentation
git commit -m "docs: add examples for guideline matching"

# Breaking change
git commit -m "feat(api)!: change Agent::builder API

BREAKING CHANGE: Agent::builder() now requires name parameter"
```

## Documentation

### Writing Documentation

1. **Module-level docs** (`//!`): Overview and examples
2. **Item-level docs** (`///`): API documentation with examples
3. **Inline comments** (`//`): Explain complex logic

### Documentation Checklist

- [ ] All public items have doc comments
- [ ] Examples compile and run (`cargo test --doc`)
- [ ] Links to related items work (`[`Agent`]`)
- [ ] Code examples use `no_run` when needed

## Project Structure

```
talk/
├── src/
│   ├── lib.rs           # Crate root with public API
│   ├── agent.rs         # Core agent implementation
│   ├── guideline.rs     # Pattern matching
│   ├── tool.rs          # Tool integration
│   ├── journey.rs       # Multi-step flows
│   ├── provider/        # LLM providers
│   ├── storage/         # Session storage
│   └── ...
├── tests/               # Integration tests
├── examples/            # Usage examples
├── specs/               # Feature specifications
└── README.md
```

## Feature Development Workflow

### 1. Specification Phase

Before implementing major features:

1. Create or update specification in `specs/`
2. Define data models and API contracts
3. Write tests first (TDD approach)
4. Discuss design in issues/discussions

### 2. Implementation Phase

1. Implement core functionality
2. Add unit tests
3. Add integration tests
4. Update documentation
5. Add examples

### 3. Review Phase

1. Self-review code
2. Run all quality checks
3. Update CHANGELOG (if applicable)
4. Submit PR

## Need Help?

- **Documentation**: https://docs.rs/talk
- **Issues**: https://github.com/gobenpark/talk/issues
- **Discussions**: https://github.com/gobenpark/talk/discussions

## Recognition

All contributors will be:
- Listed in project contributors
- Credited in release notes
- Mentioned in CHANGELOG

Thank you for contributing to Talk! 🦀🎉
