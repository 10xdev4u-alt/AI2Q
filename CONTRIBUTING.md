# Contributing to AIQL

We welcome contributions from the community! To ensure high quality and consistency, please follow these guidelines.

## Conventional Commits

We strictly follow [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/).

- `feat`: A new feature
- `fix`: A bug fix
- `docs`: Documentation changes
- `style`: Changes that do not affect the meaning of the code (white-space, formatting, etc)
- `refactor`: A code change that neither fixes a bug nor adds a feature
- `perf`: A code change that improves performance
- `test`: Adding missing tests or correcting existing tests
- `chore`: Changes to the build process or auxiliary tools and libraries

Example: `feat(core): implement postgres schema crawler`

## Development Workflow

1. **Fork the repository**.
2. **Create a branch**: `git checkout -b my-feature`.
3. **Write code and tests**.
4. **Run linting**: `cargo clippy` and `npm run lint`.
5. **Commit your changes**: Follow conventional commit guidelines.
6. **Push to your fork**: `git push origin my-feature`.
7. **Create a Pull Request**.

## Architecture Overview

- `crates/aiql-core`: The main Rust engine.
- `crates/aiql-ffi`: C-compatible bridge.
- `apps/api-go`: Go backend using Grit framework.
- `apps/web`: Next.js frontend with Brutalist design.
- `bindings/`: Language-specific wrappers (Python, Go).

## License

By contributing, you agree that your contributions will be licensed under both the MIT and Apache 2.0 licenses.
