import * as vscode from 'vscode';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind
} from 'vscode-languageclient/node';

/**
 * Activates the extension
 */
export function activate(context: vscode.ExtensionContext) {
  console.log('hodei-scan DSL extension is now active');

  // Get configuration
  const config = vscode.workspace.getConfiguration('hodei');
  const serverPath = config.get<string>('lsp.serverPath', 'hodei-dsl-lsp');

  // Set up the language server
  const serverOptions: ServerOptions = {
    command: serverPath,
    transport: TransportKind.stdio,
    args: []
  };

  // Set up the client
  const clientOptions: LanguageClientOptions = {
    // Register the server for hodei-dsl documents
    documentSelector: [
      { scheme: 'file', language: 'hodei-dsl' },
      { scheme: 'untitled', language: 'hodei-dsl' }
    ],
    // Notify the server when documents are opened or changed
    synchronize: {
      fileEvents: vscode.workspace.createFileSystemWatcher('**/*.hodei')
    },
    // Trace server communications
    traceOutputChannel: vscode.window.createOutputChannel('hodei-dsl Language Server')
  };

  // Create the language client
  const client = new LanguageClient(
    'hodei-dsl',
    'hodei-scan DSL Language Server',
    serverOptions,
    clientOptions
  );

  // Start the language server
  const serverStart = client.start();

  // Push the client to the extension's subscriptions
  context.subscriptions.push(client);

  // Register commands
  context.subscriptions.push(
    vscode.commands.registerCommand('hodei.testRule', async () => {
      await testRuleCommand();
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand('hodei.dumpIR', async () => {
      await dumpIRCommand();
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand('hodei.showRuleDocumentation', async () => {
      await showDocumentationCommand();
    })
  );

  // Wait for server to start
  serverStart.then(() => {
    console.log('hodei-dsl language server started');
  }).catch((error) => {
    console.error('Failed to start hodei-dsl language server:', error);
    vscode.window.showErrorMessage(
      `Failed to start hodei-dsl language server. Make sure 'hodei-dsl-lsp' is in your PATH.`
    );
  });
}

/**
 * Deactivates the extension
 */
export function deactivate(): Thenable<void> | undefined {
  console.log('hodei-scan DSL extension is now deactivated');
  return undefined;
}

/**
 * Test Rule command implementation
 */
async function testRuleCommand() {
  const editor = vscode.window.activeTextEditor;
  if (!editor) {
    vscode.window.showErrorMessage('No active editor found');
    return;
  }

  const document = editor.document;
  if (document.languageId !== 'hodei-dsl') {
    vscode.window.showErrorMessage('Active file is not a hodei-dsl file');
    return;
  }

  const outputChannel = vscode.window.createOutputChannel('hodei-test Results');
  outputChannel.clear();
  outputChannel.show();
  outputChannel.appendLine('Running hodei-scan rule tests...\n');

  try {
    // Get the document URI
    const docUri = document.uri.fsPath;
    
    // Ask user for test file path
    const testFile = await vscode.window.showInputBox({
      prompt: 'Enter path to test file',
      value: docUri.replace(/\.hodei$/, '.hodei.test')
    });

    if (!testFile) {
      return;
    }

    // Run tests
    outputChannel.appendLine(`Test file: ${testFile}\n`);
    outputChannel.appendLine('Running tests...');
    
    // In a real implementation, you would execute the test command here
    // For now, we just show a message
    vscode.window.showInformationMessage('Test command executed (implementation pending)');
    
  } catch (error) {
    outputChannel.appendLine(`Error: ${error}`);
    vscode.window.showErrorMessage(`Test execution failed: ${error}`);
  }
}

/**
 * Dump IR command implementation
 */
async function dumpIRCommand() {
  const editor = vscode.window.activeTextEditor;
  if (!editor) {
    vscode.window.showErrorMessage('No active editor found');
    return;
  }

  const document = editor.document;
  if (document.languageId !== 'hodei-dsl') {
    vscode.window.showErrorMessage('Active file is not a hodei-dsl file');
    return;
  }

  const outputChannel = vscode.window.createOutputChannel('hodei-dump IR');
  outputChannel.clear();
  outputChannel.show();
  outputChannel.appendLine('Dumping IR...\n');

  try {
    // Ask user for input IR file
    const irFile = await vscode.window.showInputBox({
      prompt: 'Enter path to IR file (.capnp, .json, or .yaml)'
    });

    if (!irFile) {
      return;
    }

    // Ask for output format
    const format = await vscode.window.showQuickPick(
      ['visual', 'json', 'yaml'],
      { placeHolder: 'Select output format' }
    );

    if (!format) {
      return;
    }

    outputChannel.appendLine(`IR file: ${irFile}`);
    outputChannel.appendLine(`Format: ${format}\n`);
    outputChannel.appendLine('IR content:');
    
    // In a real implementation, you would execute the dump command here
    vscode.window.showInformationMessage('IR dump command executed (implementation pending)');
    
  } catch (error) {
    outputChannel.appendLine(`Error: ${error}`);
    vscode.window.showErrorMessage(`IR dump failed: ${error}`);
  }
}

/**
 * Show Documentation command implementation
 */
async function showDocumentationCommand() {
  const documentation = `
# hodei-scan DSL Documentation

## Fact Types

### Vulnerability
Represents a security vulnerability in the code.

**Fields:**
- \`severity\` (Severity): Critical, Major, or Minor
- \`message\` (String): Human-readable description

### CodeSmell
Represents a code quality issue.

**Fields:**
- \`type\` (String): Type of code smell
- \`severity\` (Severity): Impact level

## Functions

### matches(field, pattern)
Checks if a string matches a regular expression pattern.

**Parameters:**
- \`field\` (String): The string field to check
- \`pattern\` (String): Regular expression pattern

### contains(field, substring)
Checks if a string contains a substring.

**Parameters:**
- \`field\` (String): The string field to check
- \`substring\` (String): The substring to search for

## Example Rule

\`\`\`hodei
rule password_strength {
  when {
    func validatePassword(pwd: string): boolean {
      return pwd.length >= 8;
    }
  }
  then {
    emit CodeSmell {
      type: "weak_password",
      severity: "Major",
      message: "Password validation too weak"
    };
  }
}
\`\`\`

## Commands

- \`Ctrl+Shift+T\`: Test Rule
- \`Ctrl+Shift+D\`: Dump IR
`;

  const panel = vscode.window.createWebviewPanel(
    'hodeiDocumentation',
    'hodei-scan DSL Documentation',
    vscode.ViewColumn.One,
    { enableScripts: true }
  );

  panel.webview.html = `
    <!DOCTYPE html>
    <html>
    <head>
      <meta charset="UTF-8">
      <meta name="viewport" content="width=device-width, initial-scale=1.0">
      <title>hodei-scan DSL Documentation</title>
      <style>
        body {
          font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
          padding: 20px;
          max-width: 800px;
          margin: 0 auto;
          background: var(--vscode-editor-background);
          color: var(--vscode-editor-foreground);
        }
        pre {
          background: var(--vscode-textBlockQuote-background);
          padding: 16px;
          border-radius: 4px;
          overflow-x: auto;
        }
        code {
          font-family: 'Courier New', Courier, monospace;
        }
        h1, h2, h3 {
          color: var(--vscode-gitDecoration-addedResourceForeground);
        }
        table {
          border-collapse: collapse;
          width: 100%;
          margin: 16px 0;
        }
        th, td {
          border: 1px solid var(--vscode-panel-border);
          padding: 8px;
          text-align: left;
        }
        th {
          background: var(--vscode-list-hoverBackground);
        }
      </style>
    </head>
    <body>
      <pre>${documentation.replace(/</g, '&lt;')}</pre>
    </body>
    </html>
  `;
}
