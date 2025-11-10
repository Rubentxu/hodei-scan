# EPIC-12: CI/CD Integration & GitHub Actions

**Estado**: 📝 Draft  
**Versión**: 1.0  
**Épica padre**: Hodei Scan v3.2  
**Dependencias**: EPIC-11 (CLI)  
**Owner**: DevOps Team  
**Prioridad**: High

---

## 1. Resumen Ejecutivo

Integración con CI/CD (GitHub Actions, GitLab CI, Jenkins). Proveer actions/orbs reutilizables para escaneo automático en PRs.

### Objetivo
- GitHub Action lista para usar.
- Ejemplos para GitLab CI, Jenkins.
- Comentarios automáticos en PRs con findings.

---

## 2. GitHub Action

### 2.1. Action Definition

```yaml
# .github/actions/hodei-scan/action.yml
name: 'Hodei Scan'
description: 'Unified security & quality scanner'
author: 'Hodei Team'

branding:
  icon: 'shield'
  color: 'blue'

inputs:
  project-path:
    description: 'Path to project root'
    required: false
    default: '.'
  
  rules-path:
    description: 'Path to rules directory'
    required: false
    default: '.hodei/rules'
  
  gates-config:
    description: 'Path to quality gates config'
    required: false
    default: '.hodei/quality-gates.yaml'
  
  fail-on-findings:
    description: 'Fail build if findings exceed threshold'
    required: false
    default: 'true'
  
  output-format:
    description: 'Output format (text, json, sarif)'
    required: false
    default: 'sarif'
  
  upload-sarif:
    description: 'Upload SARIF to GitHub Security tab'
    required: false
    default: 'true'

runs:
  using: 'composite'
  steps:
    - name: Setup Hodei CLI
      shell: bash
      run: |
        curl -sSL https://hodei.dev/install.sh | bash
        echo "$HOME/.hodei/bin" >> $GITHUB_PATH
    
    - name: Run Hodei Scan
      id: scan
      shell: bash
      run: |
        hodei check \
          --project "${{ inputs.project-path }}" \
          --rules "${{ inputs.rules-path }}" \
          --gates "${{ inputs.gates-config }}" \
          --output-format "${{ inputs.output-format }}" \
          > hodei-results.${{ inputs.output-format }}
        
        echo "results-file=hodei-results.${{ inputs.output-format }}" >> $GITHUB_OUTPUT
    
    - name: Upload SARIF to GitHub Security
      if: inputs.upload-sarif == 'true' && inputs.output-format == 'sarif'
      uses: github/codeql-action/upload-sarif@v2
      with:
        sarif_file: hodei-results.sarif
    
    - name: Comment PR with findings
      if: github.event_name == 'pull_request'
      uses: actions/github-script@v6
      with:
        script: |
          const fs = require('fs');
          const results = JSON.parse(fs.readFileSync('hodei-results.json', 'utf8'));
          
          const comment = `## 🛡️ Hodei Scan Results
          
          **Findings by Severity:**
          - 🔴 Critical: ${results.findings.filter(f => f.severity === 'Critical').length}
          - 🟠 High: ${results.findings.filter(f => f.severity === 'High').length}
          - 🟡 Medium: ${results.findings.filter(f => f.severity === 'Medium').length}
          - 🟢 Low: ${results.findings.filter(f => f.severity === 'Low').length}
          
          <details>
          <summary>View Details</summary>
          
          ${results.findings.slice(0, 10).map(f => `
          ### ${f.rule_name} (${f.severity})
          **Location:** \`${f.location?.file}:${f.location?.start.line}\`
          **Message:** ${f.message}
          `).join('\n')}
          
          </details>
          `;
          
          github.rest.issues.createComment({
            issue_number: context.issue.number,
            owner: context.repo.owner,
            repo: context.repo.repo,
            body: comment
          });
```

### 2.2. Example Workflow

```yaml
# .github/workflows/hodei-scan.yml
name: Security & Quality Scan

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  hodei-scan:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      security-events: write
      pull-requests: write
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Run Hodei Scan
        uses: hodei-team/hodei-scan-action@v1
        with:
          project-path: .
          rules-path: .hodei/rules
          gates-config: .hodei/quality-gates.yaml
          fail-on-findings: true
          upload-sarif: true
      
      - name: Upload artifacts
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: hodei-results
          path: |
            hodei-results.*
            hodei.ir
```

---

## 3. GitLab CI Integration

```yaml
# .gitlab-ci.yml
include:
  - remote: 'https://hodei.dev/gitlab-ci-template.yml'

hodei-scan:
  extends: .hodei-scan
  variables:
    HODEI_RULES_PATH: .hodei/rules
    HODEI_GATES_CONFIG: .hodei/quality-gates.yaml
  artifacts:
    reports:
      sast: hodei-results.json
    paths:
      - hodei-results.*
```

---

## 4. Jenkins Pipeline

```groovy
// Jenkinsfile
pipeline {
    agent any
    
    stages {
        stage('Hodei Scan') {
            steps {
                sh '''
                    curl -sSL https://hodei.dev/install.sh | bash
                    export PATH="$HOME/.hodei/bin:$PATH"
                    
                    hodei check \
                        --project . \
                        --rules .hodei/rules \
                        --gates .hodei/quality-gates.yaml \
                        --output-format json > hodei-results.json
                '''
                
                publishHTML([
                    reportDir: '.',
                    reportFiles: 'hodei-results.json',
                    reportName: 'Hodei Scan Results'
                ])
            }
        }
    }
    
    post {
        always {
            archiveArtifacts artifacts: 'hodei-results.*', allowEmptyArchive: true
        }
    }
}
```

---

## 5. Plan de Implementación

**Fase 1: GitHub Action** (Semana 1)
- [ ] Action definition.
- [ ] SARIF export.
- [ ] PR comments.

**Fase 2: Templates** (Semana 2)
- [ ] GitLab CI template.
- [ ] Jenkins pipeline example.

**Fase 3: Documentation** (Semana 2)
- [ ] Setup guides.
- [ ] Troubleshooting.

---

## 6. Criterios de Aceptación

- [ ] GitHub Action publicada en Marketplace.
- [ ] Templates para GitLab CI y Jenkins.
- [ ] Ejemplos end-to-end funcionales.
- [ ] Documentación completa.

---

**Última Actualización**: 2025-01-XX
