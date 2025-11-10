# EPIC-15: Release & Deployment Pipeline

**Estado**: 📝 Draft  
**Versión**: 1.0  
**Épica padre**: Hodei Scan v3.2  
**Dependencias**: EPIC-11 (CLI), EPIC-13 (Testing)  
**Owner**: Release Team  
**Prioridad**: High

---

## 1. Resumen Ejecutivo

Pipeline automatizado de release: versionado semántico, changelog automático, distribución de binarios, publicación de crates, Docker images.

### Objetivo
- Releases automáticas con tags semánticos.
- Binarios multi-plataforma (Linux, macOS, Windows).
- Publicación a crates.io, Docker Hub, GitHub Releases.

---

## 2. Release Workflow

### 2.1. GitHub Actions Release Pipeline

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags:
      - 'v*.*.*'

env:
  CARGO_TERM_COLOR: always

jobs:
  create-release:
    runs-on: ubuntu-latest
    outputs:
      upload_url: ${{ steps.create_release.outputs.upload_url }}
    steps:
      - uses: actions/checkout@v3
      
      - name: Extract version
        id: version
        run: echo "VERSION=${GITHUB_REF#refs/tags/v}" >> $GITHUB_OUTPUT
      
      - name: Generate changelog
        id: changelog
        run: |
          # Extract changelog section for this version
          sed -n "/## \[${VERSION}\]/,/## \[/p" CHANGELOG.md | head -n -1 > release_notes.md
        env:
          VERSION: ${{ steps.version.outputs.VERSION }}
      
      - name: Create GitHub Release
        id: create_release
        uses: actions/create-release@v1
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          tag_name: ${{ github.ref }}
          release_name: Release ${{ steps.version.outputs.VERSION }}
          body_path: release_notes.md
          draft: false
          prerelease: false
  
  build-binaries:
    needs: create-release
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            artifact_name: hodei-linux-amd64
          - os: ubuntu-latest
            target: x86_64-unknown-linux-musl
            artifact_name: hodei-linux-musl
          - os: macos-latest
            target: x86_64-apple-darwin
            artifact_name: hodei-macos-amd64
          - os: macos-latest
            target: aarch64-apple-darwin
            artifact_name: hodei-macos-arm64
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            artifact_name: hodei-windows-amd64.exe
    
    runs-on: ${{ matrix.os }}
    
    steps:
      - uses: actions/checkout@v3
      
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      
      - name: Build
        run: cargo build --release --target ${{ matrix.target }}
      
      - name: Strip binary (Linux/macOS)
        if: matrix.os != 'windows-latest'
        run: strip target/${{ matrix.target }}/release/hodei
      
      - name: Rename binary
        run: |
          if [ "${{ matrix.os }}" == "windows-latest" ]; then
            mv target/${{ matrix.target }}/release/hodei.exe ${{ matrix.artifact_name }}
          else
            mv target/${{ matrix.target }}/release/hodei ${{ matrix.artifact_name }}
          fi
        shell: bash
      
      - name: Upload binary
        uses: actions/upload-release-asset@v1
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          upload_url: ${{ needs.create-release.outputs.upload_url }}
          asset_path: ./${{ matrix.artifact_name }}
          asset_name: ${{ matrix.artifact_name }}
          asset_content_type: application/octet-stream
  
  publish-crates:
    needs: build-binaries
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      
      - name: Publish to crates.io
        run: |
          cargo publish -p hodei-ir --token ${{ secrets.CARGO_TOKEN }}
          sleep 10
          cargo publish -p hodei-dsl --token ${{ secrets.CARGO_TOKEN }}
          sleep 10
          cargo publish -p hodei-extractors --token ${{ secrets.CARGO_TOKEN }}
          sleep 10
          cargo publish -p hodei-engine --token ${{ secrets.CARGO_TOKEN }}
          sleep 10
          cargo publish -p hodei-cli --token ${{ secrets.CARGO_TOKEN }}
  
  build-docker:
    needs: build-binaries
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v2
      
      - name: Login to Docker Hub
        uses: docker/login-action@v2
        with:
          username: ${{ secrets.DOCKERHUB_USERNAME }}
          password: ${{ secrets.DOCKERHUB_TOKEN }}
      
      - name: Extract version
        id: version
        run: echo "VERSION=${GITHUB_REF#refs/tags/v}" >> $GITHUB_OUTPUT
      
      - name: Build and push
        uses: docker/build-push-action@v4
        with:
          context: .
          push: true
          tags: |
            hodeiteam/hodei-scan:latest
            hodeiteam/hodei-scan:${{ steps.version.outputs.VERSION }}
          platforms: linux/amd64,linux/arm64
```

### 2.2. Dockerfile

```dockerfile
# Dockerfile
FROM rust:1.75 as builder

WORKDIR /app

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
COPY hodei-ir/Cargo.toml hodei-ir/
COPY hodei-dsl/Cargo.toml hodei-dsl/
COPY hodei-extractors/Cargo.toml hodei-extractors/
COPY hodei-engine/Cargo.toml hodei-engine/
COPY hodei-cli/Cargo.toml hodei-cli/

RUN mkdir -p hodei-ir/src hodei-dsl/src hodei-extractors/src hodei-engine/src hodei-cli/src \
    && echo "fn main() {}" > hodei-ir/src/lib.rs \
    && echo "fn main() {}" > hodei-dsl/src/lib.rs \
    && echo "fn main() {}" > hodei-extractors/src/lib.rs \
    && echo "fn main() {}" > hodei-engine/src/lib.rs \
    && echo "fn main() {}" > hodei-cli/src/main.rs \
    && cargo build --release

# Build actual application
COPY . .
RUN touch hodei-ir/src/lib.rs hodei-dsl/src/lib.rs hodei-extractors/src/lib.rs hodei-engine/src/lib.rs hodei-cli/src/main.rs \
    && cargo build --release --bin hodei

# Runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/hodei /usr/local/bin/hodei

ENTRYPOINT ["hodei"]
CMD ["--help"]
```

### 2.3. Install Script

```bash
# install.sh
#!/bin/bash
set -e

VERSION="${HODEI_VERSION:-latest}"
INSTALL_DIR="${HODEI_INSTALL_DIR:-$HOME/.hodei/bin}"

detect_platform() {
    local os=$(uname -s | tr '[:upper:]' '[:lower:]')
    local arch=$(uname -m)
    
    case "$os" in
        linux)
            case "$arch" in
                x86_64) echo "linux-amd64" ;;
                aarch64|arm64) echo "linux-arm64" ;;
                *) echo "unsupported"; return 1 ;;
            esac
            ;;
        darwin)
            case "$arch" in
                x86_64) echo "macos-amd64" ;;
                arm64) echo "macos-arm64" ;;
                *) echo "unsupported"; return 1 ;;
            esac
            ;;
        mingw*|msys*)
            echo "windows-amd64.exe"
            ;;
        *)
            echo "unsupported"
            return 1
            ;;
    esac
}

main() {
    local platform=$(detect_platform)
    
    if [ "$platform" = "unsupported" ]; then
        echo "❌ Unsupported platform: $(uname -s) $(uname -m)"
        exit 1
    fi
    
    echo "🔍 Detected platform: $platform"
    echo "📦 Installing Hodei Scan $VERSION..."
    
    local download_url="https://github.com/hodei-team/hodei-scan/releases/download/v${VERSION}/hodei-${platform}"
    
    mkdir -p "$INSTALL_DIR"
    
    if command -v curl &> /dev/null; then
        curl -sSL "$download_url" -o "$INSTALL_DIR/hodei"
    elif command -v wget &> /dev/null; then
        wget -q "$download_url" -O "$INSTALL_DIR/hodei"
    else
        echo "❌ curl or wget required"
        exit 1
    fi
    
    chmod +x "$INSTALL_DIR/hodei"
    
    echo "✅ Hodei Scan installed to $INSTALL_DIR/hodei"
    echo ""
    echo "Add to PATH:"
    echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
    echo ""
    echo "Verify installation:"
    echo "  hodei --version"
}

main
```

---

## 3. Changelog Automation

### 3.1. Conventional Commits + git-cliff

```toml
# cliff.toml
[changelog]
header = """
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

"""
body = """
{% for group, commits in commits | group_by(attribute="group") %}
    ### {{ group | upper_first }}
    {% for commit in commits %}
        - {{ commit.message | upper_first }}
    {% endfor %}
{% endfor %}
"""

[git]
conventional_commits = true
filter_commits = true
commit_parsers = [
    { message = "^feat", group = "Features" },
    { message = "^fix", group = "Bug Fixes" },
    { message = "^perf", group = "Performance" },
    { message = "^refactor", group = "Refactoring" },
    { message = "^docs", group = "Documentation" },
    { message = "^test", group = "Testing" },
    { message = "^chore", skip = true },
]
```

```bash
# Generate changelog
git cliff --tag v1.0.0 > CHANGELOG.md
```

---

## 4. Versioning Strategy

### 4.1. Semantic Versioning

```
MAJOR.MINOR.PATCH

MAJOR: Breaking changes (IR schema, DSL syntax, CLI args)
MINOR: New features (backwards-compatible)
PATCH: Bug fixes
```

### 4.2. Release Cadence

- **Patch**: Weekly (bug fixes)
- **Minor**: Monthly (new features)
- **Major**: Quarterly (breaking changes)

---

## 5. Plan de Implementación

**Fase 1: CI/CD Pipeline** (Semana 1)
- [ ] GitHub Actions workflow.
- [ ] Multi-platform builds.

**Fase 2: Distribution** (Semana 1-2)
- [ ] Install script.
- [ ] Docker image.
- [ ] crates.io publication.

**Fase 3: Automation** (Semana 2)
- [ ] Changelog automation.
- [ ] Version bumping.

---

## 6. Criterios de Aceptación

- [ ] Releases automáticas con tags.
- [ ] Binarios para Linux/macOS/Windows.
- [ ] Docker image published.
- [ ] Install script funcional.
- [ ] Changelog automático.

---

**Última Actualización**: 2025-01-XX
