# Contributing to RustForge

Thank you for your interest in contributing to RustForge! This document provides guidelines and information for contributors.

## Project Status

RustForge is currently in **Phase 2: LLM + Agent Layer**. We've completed the core foundation and LLM integration with Ollama and OpenAI providers. Phase 3 (Tool & Security Layer) is next.

## How to Contribute

### Reporting Issues

- Use GitHub Issues to report bugs or suggest features
- Search existing issues before creating a new one
- Include clear reproduction steps for bugs
- Provide system information (OS, Rust version)

### Submitting Pull Requests

1. **Fork the repository**
   ```bash
   git clone https://github.com/apus3404-oss/RustForge.git
   cd RustForge
   ```

2. **Create a feature branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **Make your changes**
   - Write clear, documented code
   - Follow Rust conventions and idioms
   - Add tests for new functionality
   - Update documentation as needed

4. **Run tests**
   ```bash
   cargo test
   cargo clippy -- -D warnings
   cargo fmt --check
   ```

5. **Commit your changes**
   ```bash
   git add .
   git commit -m "feat: add your feature description"
   ```

6. **Push and create PR**
   ```bash
   git push origin feature/your-feature-name
   ```

## Development Setup

### Prerequisites

- Rust 1.70 or later
- Git
- A code editor (VS Code with rust-analyzer recommended)

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run with logging
RUSTFORGE_LOG_LEVEL=debug cargo run -- run workflows/example.yaml
```

### Project Structure

```
rustforge/
├── src/
│   ├── cli/          # CLI commands and handlers
│   ├── config/       # Configuration management
│   ├── engine/       # Workflow execution engine
│   ├── storage/      # State persistence
│   ├── error.rs      # Error types
│   └── main.rs       # Entry point
├── tests/
│   ├── integration/  # Integration tests
│   └── fixtures/     # Test data
└── docs/             # Documentation
```

## Code Style

### Rust Conventions

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Write idiomatic Rust code

### Documentation

- Add doc comments (`///`) for public APIs
- Include examples in doc comments when helpful
- Update README.md and docs/ when adding features

### Testing

- Write unit tests in the same file as the code (`#[cfg(test)]` modules)
- Write integration tests in `tests/` directory
- Aim for high test coverage on critical paths
- Test both success and error cases

### Commit Messages

Follow conventional commits format:

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**Types:**
- `feat` - New feature
- `fix` - Bug fix
- `docs` - Documentation changes
- `test` - Test additions or changes
- `refactor` - Code refactoring
- `perf` - Performance improvements
- `chore` - Maintenance tasks

**Examples:**
```
feat(engine): add parallel execution mode
fix(cli): handle missing workflow file gracefully
docs(readme): update installation instructions
test(parser): add tests for invalid YAML handling
```

## Testing Guidelines

### Unit Tests

Place unit tests in the same file:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        // Test implementation
    }
}
```

### Integration Tests

Place integration tests in `tests/`:

```rust
// tests/integration/my_feature.rs
use rustforge::*;

#[tokio::test]
async fn test_end_to_end_workflow() {
    // Test implementation
}
```

### Running Tests

```bash
# All tests
cargo test

# Specific test
cargo test test_name

# With output
cargo test -- --nocapture

# Integration tests only
cargo test --test integration
```

## Areas for Contribution

### Phase 2 (Current)

- Bug fixes in LLM integration
- Additional LLM provider support (Anthropic, local models)
- Memory store improvements
- Agent system enhancements
- Documentation improvements
- Additional test coverage

### Phase 3 (Next)

- Tool system implementation
- Function calling support
- Permission and security layer
- Sandboxed execution
- Tool registry and discovery

### Phase 4+ (Future)

- API execution layer
- Parallel execution mode
- Web UI dashboard
- Plugin system
- Advanced retry logic

## Code Review Process

1. All PRs require review before merging
2. CI must pass (tests, clippy, formatting)
3. Maintain or improve test coverage
4. Update documentation for user-facing changes
5. Address review feedback promptly

## Questions?

- Open a GitHub Discussion for questions
- Check existing issues and documentation
- Reach out to maintainers

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to RustForge!
