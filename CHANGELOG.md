# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [v3.2.0-epic-01] - 2025-01-10

### Added
- **Project Structure**: Complete monorepo setup with Cargo workspace
  - `hodei-ir`: Core IR types and data structures
  - `hodei-engine`: Rule evaluation engine
  - `hodei-dsl`: DSL parser for rules
  - `hodei-extractors`: Code analysis extractors
  - `hodei-cli`: Command-line interface

- **CI/CD Pipeline**: GitHub Actions workflow with:
  - Multi-platform testing (Ubuntu, Windows, macOS)
  - Multiple Rust versions (stable, beta, nightly)
  - Clippy linting
  - Rustfmt code formatting check
  - Security audit with cargo-audit
  - Code coverage with tarpaulin

- **Development Tooling**:
  - rustfmt.toml configuration
  - .clippy.toml linting rules
  - rust-toolchain.toml pinned toolchain
  - Pre-commit hooks setup (cargo-husky)

- **Architecture Documentation**:
  - ADR-001: Rust language selection decision
  - ADR-002: Hexagonal architecture decision
  - ADR index with methodology

- **Test Suite**: TDD-based tests for:
  - Workspace structure validation
  - CI/CD configuration validation
  - Development tooling configuration
  - ADR documentation presence

### Changed
- Project restructured from single crate to workspace monorepo
- Dependencies centralized in workspace.dependencies
- Source code moved to `crates/` directory structure

### Removed
- Legacy single-crate structure
- Duplicate documentation files

### Fixed
- All tests passing (11/11)
- Workspace compiles without warnings
- Code formatting matches rustfmt standards

### Security
- Security audit integrated into CI pipeline
- Dependency vulnerability scanning configured

---

## [v3.0.0] - Previous
### Note
Previous version before Epic-01 setup. Legacy structure.
