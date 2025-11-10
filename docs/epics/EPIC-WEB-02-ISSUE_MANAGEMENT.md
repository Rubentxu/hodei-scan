# Épica Web 2: Issue Management & Code Viewer
## Sistema de Gestión de Issues con Visualización de Código

**Versión:** 1.0
**Fecha:** 10 de noviembre de 2025
**Estado:** 🚧 Planning
**Época:** Fase 1 (Meses 1-6)
**Prioridad:** 🔴 Critical

---

## 📋 Resumen Ejecutivo

Desarrollar la interfaz de gestión de issues para hodei-scan, incluyendo listado, filtrado, búsqueda y visualizador de código. Los usuarios podrán navegar por los issues encontrados, ver el código fuente con highlighting, y tomar acciones (fix, ignore, false positive).

**Objetivos:**
- ✅ Issue list con paginación y filtrado avanzado
- ✅ Code viewer con syntax highlighting
- ✅ Inline issue display en código
- ✅ Bulk actions (mark as fixed, false positive, etc.)
- ✅ Issue detail modal/page
- ✅ Search functionality
- ✅ Export issues (CSV, JSON)
- ✅ Integration con Git providers

**Tecnologías:**
- **Code Editor**: Monaco Editor o Prism
- **Syntax Highlighting**: highlight.js o prism-react-renderer
- **Search**: Fuse.js para fuzzy search
- **Table**: TanStack Table (React Table v8)
- **Virtualization**: react-window para performance
- **Diff**: diff2html para code diffs

---

## 👥 Historias de Usuario

### US-WEB-04: Como developer, quiero ver todos los issues de mi proyecto

**Prioridad:** 🔴 Critical
**Story Points:** 8
**Criterios de Aceptación:**

```gherkin
Feature: Issue List
  Como developer revisando issues
  Quiero ver lista completa de issues
  Para prioritize fixes

  Scenario: Ver issues list
    Given proyecto con 150 issues
    When navega a Issues page
    Then debería ver:
      And tabla con 25 issues por página
      And columnas: File, Line, Rule, Severity, Message, Status
      And paginación en bottom
      And "Showing 1-25 of 150 issues"
      And sort by cualquier columna

  Scenario: Filter issues by severity
    Given proyecto con issues de multiple severities
    When selecciona filter "Critical" only
    Then debería mostrar solo Critical issues
      And count de issues filtrados
      And clear filter button

  Scenario: Filter by rule type
    Given proyecto con SQL injection, XSS, code smells
    When selecciona filter "Security" category
    Then debería mostrar solo security issues
```

**Tareas de Desarrollo:**

1. **TASK-WEB-02-01: Implementar Issues Table con TanStack Table**
   - Criterio: Tests en verde
   - Estimación: 3 días
   - Dependencias: Épica Web 1
   - Deliverable: IssuesTable component

   ```typescript
   // Implementación mínima requerida:
   describe('IssuesTable', () => {
     it('should display issues with correct columns', async () => {
       const issues = generateMockIssues(25);
       render(<IssuesTable issues={issues} />);
       
       expect(screen.getByText('File')).toBeInTheDocument();
       expect(screen.getByText('Line')).toBeInTheDocument();
       expect(screen.getByText('Rule')).toBeInTheDocument();
       expect(screen.getByText('Severity')).toBeInTheDocument();
     });
   });
   ```

2. **TASK-WEB-02-02: Implementar Filtering System**
   - Criterio: Tests en verde
   - Estimación: 2 días
   - Dependencias: TASK-WEB-02-01
   - Deliverable: FilterBar component

3. **TASK-WEB-02-03: Implementar Pagination**
   - Criterio: Tests en verde
   - Estimación: 1 día
   - Dependencias: TASK-WEB-02-01
   - Deliverable: Pagination component

4. **TASK-WEB-02-04: Implementar Search con Fuse.js**
   - Criterio: Tests en verde
   - Estimación: 2 días
   - Dependencias: TASK-WEB-02-01
   - Deliverable: SearchBox component

**Tests de Validación:**

```typescript
// TEST-WEB-02-01: Issues table rendering
describe('IssuesTable', () => {
  it('should render 25 issues per page', async () => {
    const issues = generateMockIssues(50);
    render(<IssuesTable issues={issues} />);
    
    const rows = screen.getAllByTestId('issue-row');
    expect(rows).toHaveLength(25);
  });

  it('should highlight critical issues in red', () => {
    const issues = [{ severity: 'critical' }];
    render(<IssuesTable issues={issues} />);
    
    const row = screen.getByTestId('issue-row');
    expect(row).toHaveClass('border-l-4 border-l-red-500');
  });

  it('should sort by severity when header clicked', async () => {
    render(<IssuesTable issues={mockIssues} />);
    
    fireEvent.click(screen.getByText('Severity'));
    
    await waitFor(() => {
      const rows = screen.getAllByTestId('issue-row');
      expect(rows[0]).toHaveTextContent('critical');
    });
  });
});

// TEST-WEB-02-02: Filtering
describe('FilterBar', () => {
  it('should filter by severity', async () => {
    render(<FilterBar />);
    
    fireEvent.change(screen.getByPlaceholderText('Filter by severity'), {
      target: { value: 'critical' },
    });
    
    await waitFor(() => {
      expect(screen.getByTestId('issues-list')).toHaveTextContent('5 issues found');
    });
  });

  it('should clear all filters', async () => {
    render(<FilterBar initialFilters={{ severity: 'critical' }} />);
    
    fireEvent.click(screen.getByText('Clear All'));
    
    expect(screen.getByPlaceholderText('Filter by severity')).toHaveValue('');
  });
});
```

---

### US-WEB-05: Como developer, quiero ver el código con issues highlighted

**Prioridad:** 🔴 Critical
**Story Points:** 8
**Criterios de Aceptación:**

```gherkin
Feature: Code Viewer
  Como developer revisando issue
  Quiero ver código con issue highlighted
  Para understand context del problema

  Scenario: View code file con issues
    Given issue en src/main.rs línea 42
    When hace click en issue row
    Then debería abrir code viewer modal
      And mostrar file con syntax highlighting
      And highlight línea 42 con background rojo
      And mostrar issue details en sidebar
      And show code context (líneas 40-44)
      And poder scroll through entire file

  Scenario: Navigate entre issues en same file
    Given file con multiple issues
    When está en code viewer
    Then debería poder:
      And navigate to next issue con "Next Issue" button
      And navigate to previous issue con "Prev Issue" button
      And show progress "Issue 3 of 5 in this file"

  Scenario: View code sin issues
    Given file sin issues
    When navega a file
    Then debería:
      And show file content con syntax highlighting
      And show "No issues found in this file"
      And NOT show issue sidebar
```

**Tareas de Desarrollo:**

1. **TASK-WEB-02-05: Implementar Code Viewer con Monaco**
   - Criterio: Tests en verde
   - Estimación: 3 días
   - Dependencias: TASK-WEB-02-01
   - Deliverable: CodeViewer component

2. **TASK-WEB-02-06: Implementar Syntax Highlighting**
   - Criterio: Tests en verde
   - Estimación: 2 días
   - Dependencias: TASK-WEB-02-05
   - Deliverable: Language-specific highlighting

3. **TASK-WEB-02-07: Implementar Inline Issue Display**
   - Criterio: Tests en verde
   - Estimación: 2 días
   - Dependencias: TASK-WEB-02-06
   - Deliverable: Issue markers en code

4. **TASK-WEB-02-08: Implementar Issue Details Panel**
   - Criterio: Tests en verde
   - Estimación: 1 día
   - Dependencias: TASK-WEB-02-07
   - Deliverable: IssueDetailsPanel component

**Tests de Validación:**

```typescript
// TEST-WEB-02-03: Code viewer
describe('CodeViewer', () => {
  it('should display file with syntax highlighting', async () => {
    const file = { path: 'src/main.rs', content: 'fn main() {}' };
    render(<CodeViewer file={file} />);
    
    expect(screen.getByTestId('code-viewer')).toBeInTheDocument();
    expect(screen.getByText('fn main()')).toHaveClass('hljs-keyword');
  });

  it('should highlight issue line', () => {
    const file = { path: 'src/main.rs', content: 'fn main() {}' };
    const issue = { line: 1, severity: 'critical' };
    
    render(<CodeViewer file={file} issue={issue} />);
    
    const line = screen.getByTestId('code-line-1');
    expect(line).toHaveClass('bg-red-100');
  });

  it('should navigate to next issue', async () => {
    const file = { path: 'src/main.rs', content: 'fn main() {}\nfn test() {}' };
    const issues = [{ line: 1 }, { line: 2 }];
    
    render(<CodeViewer file={file} issue={issues[0]} allIssues={issues} />);
    
    fireEvent.click(screen.getByText('Next Issue'));
    
    await waitFor(() => {
      const currentLine = screen.getByTestId('highlighted-line');
      expect(currentLine).toHaveAttribute('data-line', '2');
    });
  });
});
```

---

### US-WEB-06: Como developer, quiero tomar actions en issues

**Prioridad:** 🔴 Critical
**Story Points:** 5
**Criterios de Aceptación:**

```gherkin
Feature: Issue Actions
  Como developer fixing issues
  Quiero take actions en issues
  Para manage issue lifecycle

  Scenario: Mark issue as fixed
    Given issue en estado "open"
    When hace click "Mark as Fixed"
    Then debería:
      And update issue status a "fixed"
      And remove from issues list (o strike through)
      And show success toast
      And ask for commit SHA para tracking

  Scenario: Mark issue as false positive
    Given issue correctamente flagged
    When hace click "Mark as False Positive"
    Then debería:
      And update status a "false-positive"
      And add reason/comment
      And remove from critical count
      And NOT show en default views

  Scenario: Create ticket para issue
    Given issue necesita feature work
    When hace click "Create Ticket"
    Then debería:
      And open Jira/GitHub issue modal
      And pre-populate title y description
      And allow user to edit
      And create ticket
      And link issue to ticket ID

  Scenario: Bulk actions
    Given selected multiple issues
    When hace click "Bulk Actions"
    Then debería show dropdown con:
      And "Mark as Fixed"
      And "Mark as False Positive"
      And "Assign to Me"
      And "Add to Project Board"
```

**Tareas de Desarrollo:**

1. **TASK-WEB-02-09: Implementar Issue Actions Menu**
   - Criterio: Tests en verde
   - Estimación: 2 días
   - Dependencias: TASK-WEB-02-08
   - Deliverable: IssueActionsMenu component

2. **TASK-WEB-02-10: Implementar Bulk Selection**
   - Criterio: Tests en verde
   - Estimación: 2 días
   - Dependencias: TASK-WEB-02-09
   - Deliverable: Bulk selection logic

3. **TASK-WEB-02-11: Implementar Issue Status Updates**
   - Criterio: Tests en verde
   - Estimación: 1 día
   - Dependencias: TASK-WEB-02-10
   - Deliverable: Status update API integration

**Tests de Validación:**

```typescript
// TEST-WEB-02-04: Issue actions
describe('IssueActions', () => {
  it('should mark issue as fixed', async () => {
    const issue = { id: '1', status: 'open' };
    render(<IssueActions issue={issue} />);
    
    fireEvent.click(screen.getByText('Mark as Fixed'));
    fireEvent.click(screen.getByText('Confirm'));
    
    await waitFor(() => {
      expect(screen.getByText('Fixed')).toBeInTheDocument();
    });
  });

  it('should allow bulk selection', async () => {
    const issues = generateMockIssues(10);
    render(<IssuesTable issues={issues} selectionEnabled />);
    
    // Select first 3 items
    fireEvent.click(screen.getAllByTestId('checkbox')[0]);
    fireEvent.click(screen.getAllByTestId('checkbox')[1]);
    fireEvent.click(screen.getAllByTestId('checkbox')[2]);
    
    expect(screen.getByText('3 selected')).toBeInTheDocument();
    expect(screen.getByText('Bulk Actions')).toBeEnabled();
  });

  it('should create GitHub issue', async () => {
    const issue = { id: '1', message: 'SQL Injection detected' };
    render(<IssueActions issue={issue} />);
    
    fireEvent.click(screen.getByText('Create Ticket'));
    
    await waitFor(() => {
      expect(screen.getByText('Create GitHub Issue')).toBeInTheDocument();
    });
  });
});
```

---

### US-WEB-07: Como security officer, quiero filter security vulnerabilities separately

**Prioridad:** 🟡 High
**Story Points:** 5
**Criterios de Aceptación:**

```gherkin
Feature: Security-Focused View
  Como security officer
  Quiero view focused en security issues
  Para prioritize security fixes

  Scenario: Security tab
    Given proyecto con 100 code smells y 15 vulnerabilities
    When navega a "Security" tab
    Then debería mostrar solo security vulnerabilities
      And hide code smells y other issues
      And show CVSS score para cada vulnerability
      And show OWASP category
      And show "Fixed" vs "Open" count

  Scenario: Filter by OWASP Top 10
    Given project con multiple OWASP categories
    When selecciona "A03 - Injection"
    Then debería mostrar solo injection vulnerabilities
      And show count: "8 injection issues found"
```

**Tareas de Desarrollo:**

1. **TASK-WEB-02-12: Implementar Security Tab**
   - Criterio: Tests en verde
   - Estimación: 2 días
   - Dependencias: TASK-WEB-02-01
   - Deliverable: Security tab component

2. **TASK-WEB-02-13: Implementar OWASP Filter**
   - Criterio: Tests en verde
   - Estimación: 1 día
   - Dependencias: TASK-WEB-02-12
   - Deliverable: OWASP category filter

**Tests de Validación:**

```typescript
// TEST-WEB-02-05: Security view
describe('SecurityTab', () => {
  it('should filter security vulnerabilities', () => {
    const allIssues = [...mockCodeSmells, ...mockVulnerabilities];
    render(<SecurityTab issues={allIssues} />);
    
    const securityIssues = screen.getAllByTestId('security-issue');
    expect(securityIssues).toHaveLength(mockVulnerabilities.length);
  });

  it('should show CVSS score', () => {
    const vulnerability = { cvss: 9.8, severity: 'critical' };
    render(<SecurityVulnerability issue={vulnerability} />);
    
    expect(screen.getByText('9.8')).toBeInTheDocument();
    expect(screen.getByText('Critical')).toHaveClass('text-red-600');
  });
});
```

---

## 🏗️ Arquitectura Code Viewer

### Componentes Principales

```typescript
// components/issues/IssuesTable.tsx
import { createColumnHelper } from '@tanstack/react-table';

interface Issue {
  id: string;
  file: string;
  line: number;
  rule: string;
  severity: 'critical' | 'major' | 'minor' | 'info';
  message: string;
  status: 'open' | 'fixed' | 'false-positive' | 'wont-fix';
}

export const IssuesTable: React.FC<{ issues: Issue[] }> = ({ issues }) => {
  const columnHelper = createColumnHelper<Issue>();

  const columns = [
    columnHelper.accessor('file', {
      header: 'File',
      cell: (info) => (
        <a href={`#file-${info.getValue()}`} className="text-blue-600 hover:underline">
          {info.getValue()}
        </a>
      ),
    }),
    columnHelper.accessor('line', {
      header: 'Line',
      cell: (info) => (
        <code className="bg-gray-100 px-2 py-1 rounded">
          {info.getValue()}
        </code>
      ),
    }),
    columnHelper.accessor('severity', {
      header: 'Severity',
      cell: (info) => {
        const severity = info.getValue();
        return (
          <span className={`px-2 py-1 rounded text-xs font-semibold ${
            severity === 'critical' ? 'bg-red-100 text-red-800' :
            severity === 'major' ? 'bg-orange-100 text-orange-800' :
            severity === 'minor' ? 'bg-yellow-100 text-yellow-800' :
            'bg-blue-100 text-blue-800'
          }`}>
            {severity}
          </span>
        );
      },
    }),
    columnHelper.accessor('rule', {
      header: 'Rule',
    }),
    columnHelper.accessor('message', {
      header: 'Message',
      cell: (info) => (
        <p className="max-w-md truncate" title={info.getValue()}>
          {info.getValue()}
        </p>
      ),
    }),
  ];

  return (
    <Table
      data={issues}
      columns={columns}
      enableSelection
      enableSorting
      enableFiltering
    />
  );
};
```

### Code Viewer Implementation

```typescript
// components/code/CodeViewer.tsx
import Editor from '@monaco-editor/react';
import { DiffEditor } from '@monaco-editor/react';

export const CodeViewer: React.FC<{
  file: FileInfo;
  issue?: Issue;
  issues?: Issue[];
  mode: 'view' | 'diff';
}> = ({ file, issue, issues = [], mode }) => {
  const [decorations, setDecorations] = useState<string[]>([]);

  useEffect(() => {
    if (issue) {
      // Highlight issue line
      const newDecorations = [
        {
          range: new monaco.Range(issue.line, 1, issue.line, 1),
          options: {
            isWholeLine: true,
            className: 'issue-highlight',
            glyphMarginClassName: 'issue-glyph',
          },
        },
      ];
      
      setDecorations(
        editorRef.current?.deltaDecorations(decorations, newDecorations) || []
      );
    }
  }, [issue]);

  return (
    <div className="h-full flex">
      <div className="flex-1">
        {mode === 'view' ? (
          <Editor
            height="100%"
            defaultLanguage={getLanguageFromFile(file.path)}
            value={file.content}
            options={{
              readOnly: true,
              minimap: { enabled: false },
              scrollBeyondLastLine: false,
              fontSize: 14,
              lineNumbers: 'on',
              renderLineHighlight: 'all',
            }}
          />
        ) : (
          <DiffEditor
            height="100%"
            original={file.originalContent}
            modified={file.content}
            options={{
              readOnly: true,
              minimap: { enabled: false },
            }}
          />
        )}
      </div>
      
      {issue && (
        <div className="w-80 border-l p-4 overflow-y-auto">
          <IssueDetailsPanel issue={issue} />
        </div>
      )}
    </div>
  );
};
```

### Search Implementation

```typescript
// hooks/useIssueSearch.ts
import Fuse from 'fuse.js';
import { useMemo } from 'react';

export const useIssueSearch = (issues: Issue[], searchTerm: string) => {
  const fuse = useMemo(() => {
    return new Fuse(issues, {
      keys: [
        { name: 'file', weight: 0.3 },
        { name: 'message', weight: 0.4 },
        { name: 'rule', weight: 0.2 },
        { name: 'severity', weight: 0.1 },
      ],
      threshold: 0.3,
      includeScore: true,
    });
  }, [issues]);

  const results = useMemo(() => {
    if (!searchTerm) return issues;
    return fuse.search(searchTerm).map(result => result.item);
  }, [fuse, issues, searchTerm]);

  return results;
};
```

---

## 📊 Estimación y Plan de Entrega

### Cronograma Épica Web 2 (6 semanas)

| Semana | Tareas | Story Points | Entregable |
|--------|--------|--------------|------------|
| 1-2 | TASK-WEB-02-01 a 02-04 | 21 | Issues table + filtering |
| 3-4 | TASK-WEB-02-05 a 02-08 | 21 | Code viewer + highlighting |
| 5 | TASK-WEB-02-09 a 02-11 | 13 | Issue actions + bulk ops |
| 6 | TASK-WEB-02-12 a 02-13 | 8 | Security-focused view |

**Total Story Points:** 63
**Sprints Necesarios:** 3 sprints
**Duración:** 6 semanas

---

## 🧪 Testing Frontend Issues

### Component Testing

```typescript
// tests/components/IssuesTable.test.tsx
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { IssuesTable } from '../IssuesTable';
import { generateMockIssues } from '../test-utils';

describe('IssuesTable', () => {
  const mockIssues = generateMockIssues(100);

  it('should render table headers', () => {
    render(<IssuesTable issues={mockIssues} />);
    
    expect(screen.getByText('File')).toBeInTheDocument();
    expect(screen.getByText('Line')).toBeInTheDocument();
    expect(screen.getByText('Rule')).toBeInTheDocument();
    expect(screen.getByText('Severity')).toBeInTheDocument();
  });

  it('should filter issues by severity', async () => {
    render(<IssuesTable issues={mockIssues} />);
    
    const severityFilter = screen.getByPlaceholderText('Filter by severity');
    fireEvent.change(severityFilter, { target: { value: 'critical' } });
    
    await waitFor(() => {
      const criticalIssues = screen.getAllByTestId('issue-row')
        .filter(row => row.className.includes('border-l-red'));
      expect(criticalIssues.length).toBeGreaterThan(0);
    });
  });

  it('should navigate to code viewer on row click', async () => {
    render(<IssuesTable issues={mockIssues} />);
    
    fireEvent.click(screen.getAllByTestId('issue-row')[0]);
    
    await waitFor(() => {
      expect(screen.getByTestId('code-viewer')).toBeInTheDocument();
    });
  });
});
```

---

## 🔄 Criterios de Done

Para que esta épica se considere **COMPLETADA**:

- [ ] ✅ Issues table con 25 issues por página
- [ ] ✅ Sorting por todas las columnas
- [ ] ✅ Filtering por severity, rule, file, status
- [ ] ✅ Search functionality (fuzzy search)
- [ ] ✅ Code viewer con syntax highlighting
- [ ] ✅ Monaco Editor integration
- [ ] ✅ Inline issue highlighting
- [ ] ✅ Issue details panel
- [ ] ✅ Actions: Fixed, False Positive, Won't Fix
- [ ] ✅ Bulk actions para multiple issues
- [ ] ✅ Security tab con OWASP filters
- [ ] ✅ Export issues (CSV)
- [ ] ✅ Performance: <500ms para 1000 issues
- [ ] ✅ 100% tests en verde
- [ ] ✅ Accessibility: keyboard navigation

---

## 🚀 Siguiente Épica

Una vez completada esta épica, proceder con:
**[Épica Web 3: Real-time Updates & WebSockets](./EPIC-WEB-03-REALTIME_UPDATES.md)**

---

## 📞 Contacto

**Frontend Lead:** [A definir]
**Epic Owner:** [A definir]
**Slack Channel:** #hodei-scan-frontend
**Monaco Editor Docs:** https://microsoft.github.io/monaco-editor/

---

*Última actualización: 10 de noviembre de 2025*
