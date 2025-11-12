# VS Code Extension Guide

The hodei-scan DSL Support extension provides complete IDE integration for the hodei-scan DSL.

## Features

### Language Support
- **Syntax Highlighting** - `.hodei` file support
- **Language Server Protocol** - Autocompletion, hover docs, validation
- **Error Diagnostics** - Real-time error detection

### Commands
- `hodei-scan: Test Rule` (Ctrl+Shift+T)
- `hodei-scan: Dump IR` (Ctrl+Shift+D)
- `hodei-scan: Show Documentation` (Ctrl+Shift+H)

### Keybindings
| Action | Windows/Linux | macOS |
|--------|---------------|-------|
| Test Rule | Ctrl+Shift+T | Cmd+Shift+T |
| Dump IR | Ctrl+Shift+D | Cmd+Shift+D |
| Show Documentation | Ctrl+Shift+H | Cmd+Shift+H |

## Installation

### From Source
```bash
cd extensions/vscode-hodei-dsl
npm install
npm run compile
npm run package
code --install-extension dist/hodei-scan-dsl-support-0.1.0.vsix
```

### Development Mode
```bash
cd extensions/vscode-hodei-dsl
npm install
npm run watch
```

Then press F5 in VS Code to launch the extension in debug mode.

## Configuration

### Extension Settings
```json
{
  "hodei.lsp.serverPath": "hodei-dsl-lsp",
  "hodei.lsp.trace.server": "off",
  "hodei.test.ruleCommand": "cargo test",
  "hodei.dump.irCommand": "cargo run --bin ir-dump"
}
```

### Setting Up LSP Server Path

**Option 1: Use cargo install**
```bash
cargo install --path crates/hodei-dsl-lsp
# The extension will find it in PATH
```

**Option 2: Use absolute path**
```json
{
  "hodei.lsp.serverPath": "/path/to/hodei-dsl-lsp"
}
```

**Option 3: Use cargo run**
```json
{
  "hodei.lsp.serverPath": "cargo run --package hodei-dsl-lsp --"
}
```

## Architecture

### Project Structure
```
extensions/vscode-hodei-dsl/
├── src/
│   └── extension.ts          # Main extension logic
├── syntaxes/
│   └── hodei-dsl.tmLanguage.json  # TextMate grammar
├── language-configuration.json    # Language settings
├── package.json             # Extension manifest
├── tsconfig.json           # TypeScript config
└── webpack.config.js       # Bundler configuration
```

### Key Components

#### extension.ts
Main extension entry point that:
- Registers the LSP client
- Sets up command handlers
- Manages extension lifecycle

#### Language Client Setup
```typescript
const serverOptions: ServerOptions = {
  command: serverPath,
  transport: TransportKind.stdio,
};

const clientOptions: LanguageClientOptions = {
  documentSelector: [{ scheme: 'file', language: 'hodei-dsl' }],
  synchronize: {
    fileEvents: vscode.workspace.createFileSystemWatcher('**/*.hodei')
  }
};
```

### TextMate Grammar
Defines syntax highlighting in `syntaxes/hodei-dsl.tmLanguage.json`:

- Rule declarations (keyword.control.rule)
- Fact types (support.type)
- Functions (support.function)
- Strings and comments

### Language Configuration
Configures editing behavior in `language-configuration.json`:
- Comment syntax
- Bracket pairs
- Auto-closing pairs
- Indentation rules

## Development

### Setup
```bash
# Clone the repo
git clone https://github.com/hodei-scan/hodei-scan.git
cd hodei-scan/extensions/vscode-hodei-dsl

# Install dependencies
npm install

# Compile TypeScript
npm run compile

# Watch for changes
npm run watch
```

### Testing
```bash
# Run extension tests
npm test

# Run integration tests
npm run test-integration
```

### Building
```bash
# Bundle the extension
npm run compile

# Package for distribution
npm run package

# Publish to marketplace (requires publisher)
npm run publish
```

### Debugging
1. Set breakpoints in `src/extension.ts`
2. Press F5 to launch extension in debug mode
3. Use "Developer Tools" for extension console logs

## Customization

### Adding New Commands
1. Register command in `package.json`:
```json
"commands": [
  {
    "command": "hodei.myCommand",
    "title": "My Custom Command"
  }
]
```

2. Implement handler in `extension.ts`:
```typescript
context.subscriptions.push(
  vscode.commands.registerCommand('hodei.myCommand', async () => {
    // Command implementation
  })
);
```

3. Add to menus:
```json
"menus": {
  "editor/context": [
    {
      "command": "hodei.myCommand",
      "when": "resourceExtname == .hodei"
    }
  ]
}
```

### Custom Syntax Highlighting
Edit `syntaxes/hodei-dsl.tmLanguage.json`:

```json
{
  "patterns": [
    {
      "include": "#my_feature"
    }
  ],
  "repository": {
    "my_feature": {
      "match": "\\bmy_keyword\\b",
      "name": "keyword.myfeature.hodei-dsl"
    }
  }
}
```

### Custom LSP Features
1. Update LSP server in `crates/hodei-dsl-lsp/`
2. Implement new method in `HodeiDslServer`
3. Add client support in `extension.ts`:
```typescript
client.onReady().then(() => {
  client.sendRequest(CustomRequest.myRequest, params);
});
```

## Integration with Other Tools

### With hodei-test
```typescript
async function testRuleCommand() {
  const terminal = vscode.window.createTerminal('hodei-test');
  terminal.sendText(`hodei-test test-file --rule ${filePath}`);
  terminal.show();
}
```

### With ir-dump
```typescript
async function dumpIRCommand() {
  const terminal = vscode.window.createTerminal('ir-dump');
  terminal.sendText(`ir-dump --input ${irPath} --format visual`);
  terminal.show();
}
```

## Troubleshooting

### LSP Server Not Starting
**Problem**: Extension shows "Failed to start language server"

**Solutions**:
1. Check server path in settings
2. Ensure `hodei-dsl-lsp` binary is installed
3. Check PATH environment variable
4. Enable trace logging:
```json
{
  "hodei.lsp.trace.server": "verbose"
}
```

### Commands Not Working
**Problem**: Commands are greyed out or don't execute

**Solutions**:
1. Check file is a `.hodei` file
2. Ensure extension is activated (check status bar)
3. Check output channel for errors
4. Restart VS Code

### Syntax Highlighting Not Working
**Problem**: Code appears as plain text

**Solutions**:
1. Verify file has `.hodei` extension
2. Check language status in bottom-right of VS Code
3. Manually set language: Ctrl+Shift+P → "Change Language Mode"
4. Reload window: Ctrl+Shift+P → "Developer: Reload Window"

### Performance Issues
**Problem**: Extension is slow or freezes

**Solutions**:
1. Disable trace logging in production
2. Check for large files ( LSP has size limits)
3. Enable server logs: `"hodei.lsp.trace.server": "messages"`
4. Restart LSP: Ctrl+Shift+P → "hodei-scan: Restart LSP Server"

## Publishing

### Prerequisites
1. VS Code publisher account
2. `vsce` CLI installed: `npm install -g @vscode/vsce`

### Steps
1. Update version in `package.json`
2. Build the extension: `npm run compile`
3. Package: `npm run package`
4. Publish: `vsce publish`
5. Verify in marketplace

### Marketplace Assets
Required files:
- `README.md` - Extension documentation
- `package.json` - Extension manifest
- `LICENSE` - License file
- Extension icon (128x128px)

## Best Practices

### 1. Always Test Changes
```bash
npm run compile
# Test in debug mode (F5)
npm test
```

### 2. Follow VS Code API Patterns
- Use async/await for async operations
- Handle errors gracefully
- Show user-friendly messages
- Use OutputChannel for logging

### 3. Respect User Settings
```typescript
const config = vscode.workspace.getConfiguration('hodei');
const serverPath = config.get<string>('lsp.serverPath', 'hodei-dsl-lsp');
```

### 4. Clean Up Resources
```typescript
context.subscriptions.push(terminal); // Auto-dispose on close
```

### 5. Provide Good UX
- Show progress for long operations
- Use status bar for feedback
- Provide helpful error messages
- Support cancellation

## Future Enhancements

Planned features:
- Code lenses for rule information
- Code actions for quick fixes
- Symbol outline view
- Go to definition
- Find references
- Rename symbol

## Contributing

See the main project contributing guidelines. For extension-specific contributions:

1. Follow TypeScript best practices
2. Add tests for new features
3. Update documentation
4. Test with multiple VS Code versions
5. Ensure accessibility compliance

## Resources

- [VS Code Extension API](https://code.visualstudio.com/api)
- [Language Server Protocol](https://microsoft.github.io/language-server-protocol/)
- [TextMate Grammars](https://macromates.com/manual/en/language_grammars)
- [VS Code Extension Samples](https://github.com/microsoft/vscode-extension-samples)
