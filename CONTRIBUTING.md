# Contributing to Reactive-TUI

Thank you for your interest in contributing to Reactive-TUI! This document provides guidelines and information for contributors.

## Code of Conduct

This project adheres to a code of conduct that we expect all contributors to follow. Please be respectful and constructive in all interactions.

## Getting Started

### Prerequisites

- Rust 1.70 or later
- A 24-bit color terminal (wezterm, kitty, alacritty, iTerm2)
- For image features: chafa or viu (optional but recommended)

### Setting Up the Development Environment

1. Fork and clone the repository:
   ```bash
   git clone https://github.com/your-username/reactive-tui.git
   cd reactive-tui
   ```

2. Build the project:
   ```bash
   cargo build
   ```

3. Run tests to ensure everything works:
   ```bash
   cargo test
   ```

4. Run examples to see the library in action:
   ```bash
   cargo run --example widget_catalog
   ```

## Development Workflow

### Code Quality Standards

We maintain high code quality standards:

- **Zero warnings**: `cargo clippy` must pass without warnings
- **All tests pass**: `cargo test` must pass completely
- **Clean formatting**: Use `cargo fmt` before committing
- **Documentation**: Public APIs must be documented

### Testing

- Write unit tests for new functionality
- Add integration tests for complex features
- Test examples to ensure they work correctly
- Consider property-based tests for mathematical operations

### Code Style

- Follow Rust naming conventions
- Use descriptive variable and function names
- Keep functions focused and reasonably sized
- Add comments for complex algorithms
- Use `#[allow(clippy::...)]` sparingly and with justification

## Contribution Types

### Bug Reports

When reporting bugs, please include:

- Rust version (`rustc --version`)
- Terminal type and version
- Minimal reproduction case
- Expected vs actual behavior
- Error messages or logs

### Feature Requests

For new features:

- Describe the use case and motivation
- Provide examples of the desired API
- Consider backward compatibility
- Discuss performance implications

### Pull Requests

1. **Create a feature branch** from `main`
2. **Make focused changes** - one feature/fix per PR
3. **Write tests** for new functionality
4. **Update documentation** as needed
5. **Ensure CI passes** - all tests and checks must pass
6. **Write clear commit messages**

### Commit Message Format

Use conventional commits format:

```
type(scope): description

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes
- `refactor`: Code refactoring
- `test`: Test additions/changes
- `chore`: Build/tooling changes

Examples:
- `feat(widgets): add image widget with multi-backend support`
- `fix(layout): correct aspect ratio calculation in image renderer`
- `docs(readme): update installation instructions`

## Architecture Guidelines

### Module Organization

- `src/core/`: Low-level terminal and rendering
- `src/widgets/`: UI components
- `src/layout/`: CSS-like layout system
- `src/animation/`: Animation and transitions
- `src/hooks/`: React-like hooks
- `src/reactive/`: Reactive state management

### Design Principles

1. **Performance**: Minimize terminal output through diff-based rendering
2. **Compatibility**: Support modern terminals with graceful fallbacks
3. **Developer Experience**: Provide intuitive APIs similar to web frameworks
4. **Type Safety**: Leverage Rust's type system for correctness
5. **Modularity**: Keep features loosely coupled and composable

### Adding New Features

When adding new features:

1. **Consider the API design** - should it be a widget, hook, or utility?
2. **Think about CSS integration** - can it use utility classes?
3. **Plan for terminal compatibility** - how does it degrade gracefully?
4. **Write comprehensive tests** - unit, integration, and examples
5. **Document thoroughly** - API docs, examples, and guides

## Release Process

Releases follow semantic versioning:

- **Patch** (0.0.x): Bug fixes, documentation
- **Minor** (0.x.0): New features, backward compatible
- **Major** (x.0.0): Breaking changes

## Getting Help

- **Documentation**: Check the [docs](docs/) directory
- **Examples**: Look at working examples in [examples/](examples/)
- **Issues**: Search existing issues or create a new one
- **Discussions**: Use GitHub Discussions for questions

## Recognition

Contributors are recognized in:

- Git commit history
- Release notes for significant contributions
- README acknowledgments for major features

Thank you for contributing to Reactive-TUI! 🚀
