# hodei-scan DSL Support for VS Code

This extension provides intelligent language support for the hodei-scan DSL, including:

- **Syntax highlighting** for `.hodei` files
- **Language Server Protocol (LSP)** integration for:
  - Autocompletion of fact types and functions
  - Hover documentation
  - Real-time error validation
- **Commands** for testing rules and dumping IR
- **Keybindings** for common operations

## Features

### Intelligent Editing
- **Autocompletion**: Trigger with `.` to see available fact types
- **Hover Documentation**: Hover over keywords, fact types, and functions to see documentation
- **Error Detection**: Real-time validation as you type

### Commands
- `hodei-scan: Test Rule` (Ctrl+Shift+T): Run tests for the current rule file
- `hodei-scan: Dump IR` (Ctrl+Shift+D): Dump IR in various formats
- `hodei-scan: Show Documentation` (Ctrl+Shift+H): Open embedded documentation

## Installation

### From Marketplace
Coming soon to VS Code Marketplace

### Manual Installation
```bash
cd extensions/vscode-hodei-dsl
npm install
npm run compile
vsce package
code --install-extension hodei-scan-dsl-support-0.1.0.vsix
```

## Building from Source

```bash
# Install dependencies
npm install

# Compile TypeScript
npm run compile

# Bundle with webpack
npm run watch

# Package extension
npm run package

# Publish to marketplace (requires publisher account)
npm run publish
```

## Requirements

- VS Code 1.74.0 or higher
- `hodei-dsl-lsp` binary in PATH (or configure in settings)

## Configuration

```json
{
  "hodei.lsp.serverPath": "hodei-dsl-lsp",
  "hodei.lsp.trace.server": "off",
  "hodei.test.ruleCommand": "cargo test",
  "hodei.dump.irCommand": "cargo run --bin ir-dump"
}
```

## Development

### Project Structure
```
├── src/
│   └── extension.ts      # Main extension entry point
├── syntaxes/
│   └── hodei-dsl.tmLanguage.json  # TextMate grammar
├── language-configuration.json   # Language settings
├── package.json           # Extension manifest
├── tsconfig.json         # TypeScript config
└── webpack.config.js     # Bundler config
```

### Running Tests
```bash
npm test
```

### Linting
```bash
npx eslint src --ext ts
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## License

MIT
