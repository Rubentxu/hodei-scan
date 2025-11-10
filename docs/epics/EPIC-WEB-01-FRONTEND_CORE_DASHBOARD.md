# EPIC-WEB-01: Frontend Core & Dashboard
## React 18 + TypeScript Dashboard with Real-Time Analysis Results

**Epic ID:** EPIC-WEB-01
**Version:** 2.0
**Created:** 2025-11-10
**Story Points:** 45
**Priority:** P0 (Critical)
**Status:** 🚧 In Progress

---

## 📋 Epic Overview

### Objective
Build the foundational React 18 frontend application with real-time dashboard, code viewer, and issue management. Implement a high-performance, accessible, and user-friendly interface for hodei-scan analysis results with <500KB bundle size target.

### Key Deliverables
1. **React 18 + TypeScript Core** - Modern React with hooks, suspense, and concurrent features
2. **Dashboard UI** - Real-time metrics, trends, and quality scores
3. **Code Viewer** - Monaco Editor with syntax highlighting and inline findings
4. **Issue Management** - Virtualized table with filtering, sorting, and bulk actions
5. **State Management** - Zustand for client state, TanStack Query for server state
6. **Real-Time Updates** - WebSocket integration for live analysis updates
7. **UI Components** - shadcn/ui component library with Tailwind CSS
8. **Performance** - Code splitting, lazy loading, virtualized rendering

### Success Criteria
- [ ] Bundle size: <500KB (gzipped)
- [ ] First Contentful Paint: <1.5s
- [ ] Time to Interactive: <3s
- [ ] Lighthouse Score: >90 (Performance, Accessibility, Best Practices, SEO)
- [ ] Real-time updates: <100ms latency
- [ ] Issue table: Render 10,000+ rows smoothly
- [ ] Code viewer: Support 7 languages with syntax highlighting
- [ ] Responsive: Mobile, tablet, desktop support
- [ ] Accessibility: WCAG 2.1 AA compliant

---

## 🎯 User Stories & BDD Specifications

### US-01: As a Developer, I want to view analysis results on a real-time dashboard

**Priority:** P0
**Story Points:** 13
**Component:** Dashboard UI

#### BDD Specification (Gherkin)

```gherkin
Feature: Real-Time Analysis Dashboard

  Scenario: View quality score overview
    Given I have completed analysis results
    When I view the dashboard
    Then I should see:
      | metric              | display format        | update frequency |
      | quality score       | 0-100 gauge          | real-time       |
      | security score      | 0-100 gauge          | real-time       |
      | coverage percentage | progress bar + %     | real-time       |
      | technical debt      | hours + $ estimate   | real-time       |
      | issues count        | severity breakdown   | real-time       |

  Scenario: View trends over time
    Given I have historical analysis data
    When I view the trends section
    Then I should see:
      | chart type | metric              | time range options      |
      | line       | quality score       | 7d, 30d, 90d, 1y       |
      | bar        | issues by severity  | 7d, 30d, 90d, 1y       |
      | area       | coverage trend      | 7d, 30d, 90d, 1y       |
      | line       | security score      | 7d, 30d, 90d, 1y       |

  Scenario: View project health
    Given I have multiple projects
    When I view the project health grid
    Then each project should display:
      | field          | format                  | color coding      |
      | name           | project name            | -                 |
      | status         | badge (healthy/warning) | green/yellow/red |
      | last scan      | timestamp               | -                 |
      | issues         | count by severity       | color-coded       |
      | coverage       | percentage              | progress bar      |

  Scenario: Real-time updates via WebSocket
    Given I have an active analysis running
    When the analysis produces new findings
    Then the dashboard should:
      | action           | behavior                          | latency target |
      | update metrics   | refresh scores and counters       | <100ms         |
      | new findings     | show toast notification           | <500ms         |
      | progress         | update progress bar               | real-time      |
      | trending        | animate chart updates             | <1s            |

  Scenario: View quality gate status
    Given I have quality gates configured
    When I view the dashboard
    Then I should see:
      | gate name        | status         | value       | threshold  |
      | coverage > 80%   | passed/failed  | 85%         | 80%        |
      | no critical      | passed/failed  | 0 issues    | 0          |
      | security score > 90 | passed/failed| 92         | 90         |
      | debt < 10 hours  | passed/failed  | 8 hours     | 10 hours   |

  Scenario: Quick actions panel
    Given I am on the dashboard
    When I view the quick actions
    Then I should have access to:
      | action             | icon  | result                          |
      | New Scan           | play  | trigger new analysis            |
      | View All Issues    | list  | navigate to issues page         |
      | Export Report      | download| download PDF report           |
      | Settings           | gear  | open settings modal             |
```

#### Implementation Tasks

**Task 1.1: Setup React 18 + TypeScript Project**
- Initialize Vite project with React 18 template
- Configure TypeScript strict mode
- Setup Tailwind CSS with shadcn/ui
- Configure path aliases and module resolution
- Setup ESLint + Prettier + Husky

**Task 1.2: Implement Dashboard Layout**
- Create responsive grid layout
- Build sidebar navigation
- Implement header with user menu
- Setup dark/light theme switching
- Add loading states and skeletons

**Task 1.3: Build Metrics Cards**
- Create reusable MetricCard component
- Implement animated gauge charts
- Add trend indicators (up/down arrows)
- Setup real-time data updates
- Add accessibility labels

**Task 1.4: Implement Charts**
- Setup Recharts library
- Create LineChart for trends
- Create BarChart for issue breakdown
- Create AreaChart for coverage
- Add interactive tooltips

**Task 1.5: WebSocket Integration**
- Setup WebSocket client
- Handle connection states
- Implement message parsing
- Update state on new data
- Add reconnection logic

#### Test Suite (Unit Tests - 100% Coverage)

```typescript
// src/components/Dashboard/__tests__/Dashboard.test.tsx

import { render, screen, waitFor } from '@testing-library/react';
import { Dashboard } from '../Dashboard';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';

describe('Dashboard', () => {
  it('should display quality score gauge', async () => {
    const queryClient = new QueryClient({
      defaultOptions: { queries: { staleTime: 0 } },
    });

    render(
      <QueryClientProvider client={queryClient}>
        <Dashboard />
      </QueryClientProvider>
    );

    await waitFor(() => {
      expect(screen.getByText('Quality Score')).toBeInTheDocument();
    });

    const gauge = screen.getByRole('meter');
    expect(gauge).toHaveAttribute('aria-valuenow', '85');
  });

  it('should display security score', async () => {
    const queryClient = new QueryClient();
    
    render(
      <QueryClientProvider client={queryClient}>
        <Dashboard />
      </QueryClientProvider>
    );

    await waitFor(() => {
      expect(screen.getByText('Security Score')).toBeInTheDocument();
    });

    const score = screen.getByText('92');
    expect(score).toBeInTheDocument();
  });

  it('should show project health grid', async () => {
    const queryClient = new QueryClient();
    
    render(
      <QueryClientProvider client={queryClient}>
        <Dashboard />
      </QueryClientProvider>
    );

    await waitFor(() => {
      const projectCards = screen.getAllByTestId('project-card');
      expect(projectCards.length).toBeGreaterThan(0);
    });
  });
});
```

---

### US-02: As a Security Engineer, I want to view code with security findings inline

**Priority:** P0
**Story Points:** 13
**Component:** Code Viewer

#### BDD Scenarios

```gherkin
Feature: Code Viewer with Inline Findings

  Scenario: View JavaScript code with syntax highlighting
    Given I have a JavaScript file
    When I open it in the code viewer
    Then the viewer should:
      | feature           | requirement                          |
      | syntax highlight  | keywords, strings, functions colored |
      | line numbers      | displayed on left                    |
      | theme             | dark/light mode support              |
      | font size         | adjustable (12px - 20px)             |
      | wrapping          | soft wrap toggle                     |

  Scenario: Display security findings inline
    Given I have a file with security issues
    When the code viewer loads
    Then it should show:
      | finding type  | display method        | color  | action             |
      | critical      | red underline         | red    | click for details  |
      | high          | orange underline      | orange | click for details  |
      | medium        | yellow underline      | yellow | click for details  |
      | low           | blue underline        | blue   | click for details  |

  Scenario: Show finding details on click
    Given I have a finding underlined in code
    When I click on the underlined code
    Then it should display:
      | field        | content                            |
      | rule ID      | e.g., SEC-001-SQL-INJECTION        |
      | severity     | e.g., Critical                      |
      | message      | descriptive error message           |
      | remediation  | fix suggestion                      |
      | CWE          | CWE ID and name                     |
      | line         | line number                         |

  Scenario: Support multiple languages
    Given I have different file types
    When opened in the viewer
    Then it should highlight:
      | language    | file extensions           |
      | JavaScript  | .js, .jsx, .mjs, .cjs     |
      | TypeScript  | .ts, .tsx, .mts, .cts     |
      | Python      | .py, .pyi                 |
      | Go          | .go                       |
      | Rust        | .rs                       |
      | Java        | .java                     |
      | C#          | .cs                       |

  Scenario: Navigate between findings
    Given I have a file with multiple findings
    When viewing the file
    Then I should be able to:
      | action        | behavior                          |
      | next finding  | jump to next underlined code      |
      | prev finding  | jump to previous underlined code  |
      | finding list  | show all findings in sidebar      |
      | filter        | filter by severity                |

  Scenario: Display taint flow visualization
    Given I have a taint analysis result
    When viewing the affected code
    Then it should show:
      | element       | display                  | color    |
      | taint source  | green indicator          | green    |
      | taint path    | connecting line          | yellow   |
      | taint sink    | red indicator            | red      |
      | sanitization  | blue indicator           | blue     |

  Scenario: Show code context
    Given I have a finding on line N
    When viewing the code
    Then it should display:
      | lines shown  | behavior                          |
      | above        | 3 lines before finding            |
      | below        | 3 lines after finding             |
      | expand       | click to show more context        |
      | collapse     | click to hide extra lines         |
```

#### Implementation Tasks

**Task 2.1: Setup Monaco Editor**
- Install @monaco-editor/react
- Configure for 7 supported languages
- Setup themes (dark, light, high contrast)
- Configure editor options
- Add custom theme if needed

**Task 2.2: Implement Inline Findings**
- Create FindingMarker component
- Calculate finding positions
- Render underlines/warnings
- Add hover tooltips
- Implement click handlers

**Task 2.3: Build Finding Details Panel**
- Create collapsible panel component
- Display finding metadata
- Add remediation suggestions
- Include CWE links
- Add copy to clipboard

**Task 2.4: Add Navigation Features**
- Implement finding navigation
- Create findings sidebar
- Add filter controls
- Setup keyboard shortcuts
- Add search functionality

**Task 2.5: Taint Flow Visualization**
- Draw connections between lines
- Color-code taint stages
- Add interactive hover
- Implement zoom/pan
- Add legend

#### Test Suite

```typescript
// src/components/CodeViewer/__tests__/CodeViewer.test.tsx

import { render, screen, fireEvent } from '@testing-library/react';
import { CodeViewer } from '../CodeViewer';

describe('CodeViewer', () => {
  it('should display JavaScript with syntax highlighting', () => {
    const code = 'function test() { return 42; }';
    
    render(
      <CodeViewer
        code={code}
        language="javascript"
        findings={[]}
      />
    );

    const editor = screen.getByRole('textbox');
    expect(editor).toBeInTheDocument();
  });

  it('should show inline finding for critical issue', () => {
    const code = 'eval(userInput);';
    const finding = {
      id: '1',
      rule: 'SEC-001',
      severity: 'critical',
      message: 'Use of eval() is dangerous',
      line: 1,
    };

    render(
      <CodeViewer
        code={code}
        language="javascript"
        findings={[finding]}
      />
    );

    const criticalFinding = screen.getByTestId('finding-critical');
    expect(criticalFinding).toBeInTheDocument();
  });

  it('should show finding details on click', async () => {
    const code = 'eval(userInput);';
    const finding = {
      id: '1',
      rule: 'SEC-001',
      severity: 'critical',
      message: 'Use of eval() is dangerous',
      line: 1,
    };

    render(
      <CodeViewer
        code={code}
        language="javascript"
        findings={[finding]}
      />
    );

    fireEvent.click(screen.getByTestId('finding-critical'));

    await screen.findByText('SEC-001');
    expect(screen.getByText('Use of eval() is dangerous')).toBeInTheDocument();
  });
});
```

---

### US-03: As a Team Lead, I want to manage issues in a sortable, filterable table

**Priority:** P0
**Story Points:** 8
**Component:** Issue Management

#### BDD Scenarios

```gherkin
Feature: Issue Management Table

  Scenario: Display all issues in virtualized table
    Given I have 10,000+ issues
    When I view the issues table
    Then it should:
      | feature         | behavior                          |
      | virtual scroll  | render only visible rows          |
      | smooth scrolling| maintain 60fps during scroll     |
      | lazy loading    | load data as needed              |
      | row height      | consistent for all items         |

  Scenario: Filter issues by severity
    Given I have issues of different severities
    When I filter by severity
    Then it should show:
      | filter value | results shown                  |
      | Critical     | only critical issues           |
      | High         | only high severity issues      |
      | All          | all issues                     |

  Scenario: Filter issues by type
    Given I have issues of different types
    When I filter by type
    Then it should support:
      | type filter      | shows                               |
      | Security         | all security findings              |
      | Quality          | all code quality issues            |
      | Coverage         | all coverage issues                |
      | Duplicate        | all duplicate code issues         |

  Scenario: Sort by multiple columns
    Given I have a table with issues
    When I click column headers
    Then it should sort by:
      | column       | sort behavior                    |
      | severity     | Critical → High → Medium → Low  |
      | file         | alphabetical                     |
      | line         | numerical                        |
      | rule         | alphabetical                     |
      | status       | Open → WontFix → Fixed           |

  Scenario: Bulk actions
    Given I have selected multiple issues
    When I perform bulk action
    Then I can:
      | action          | result                            |
      | mark as fixed   | update status to Fixed            |
      | mark wontfix    | update status to WontFix          |
      | assign to me    | set assignee to current user      |
      | export          | export selected to CSV/JSON       |

  Scenario: Search functionality
    Given I have a large issue list
    When I search
    Then it should match:
      | search field | matches                              |
      | file name    | exact match or partial               |
      | rule ID      | exact match                          |
      | message      | fuzzy match                          |
      | CWE          | exact match                          |
```

#### Implementation Tasks

**Task 3.1: Setup TanStack Table**
- Install @tanstack/react-table
- Configure columns
- Setup sorting
- Configure filtering
- Add selection

**Task 3.2: Implement Virtual Scrolling**
- Install react-window
- Setup row virtualization
- Implement row height calculation
- Add overscan for smooth scrolling
- Handle dynamic row heights

**Task 3.3: Build Filter UI**
- Create filter components
- Add severity filter dropdown
- Add type filter dropdown
- Add status filter
- Add date range filter

**Task 3.4: Add Search**
- Setup Fuse.js for fuzzy search
- Implement search input
- Add search highlighting
- Setup search index
- Add debounced search

**Task 3.5: Bulk Actions**
- Create selection state
- Implement select all
- Add bulk action toolbar
- Implement action handlers
- Add confirmation modals

#### Test Suite

```typescript
// src/components/IssueTable/__tests__/IssueTable.test.tsx

import { render, screen, fireEvent } from '@testing-library/react';
import { IssueTable } from '../IssueTable';

describe('IssueTable', () => {
  it('should render 10,000 issues with virtualization', () => {
    const issues = Array.from({ length: 10000 }, (_, i) => ({
      id: i.toString(),
      rule: `RULE-${i}`,
      severity: 'medium',
      file: `file${i}.js`,
      line: i,
    }));

    render(<IssueTable issues={issues} />);

    const table = screen.getByRole('table');
    expect(table).toBeInTheDocument();
    // Only visible rows should be rendered (performance check)
    const rows = screen.getAllByRole('row');
    expect(rows.length).toBeLessThan(100); // Only viewport rendered
  });

  it('should filter by severity', () => {
    const issues = [
      { id: '1', severity: 'critical' },
      { id: '2', severity: 'high' },
      { id: '3', severity: 'medium' },
    ];

    render(<IssueTable issues={issues} />);

    fireEvent.change(screen.getByLabelText('Severity'), {
      target: { value: 'critical' },
    });

    const rows = screen.getAllByRole('row');
    expect(rows).toHaveLength(2); // header + 1 data row
  });
});
```

---

### US-04: As a Developer, I want responsive design that works on mobile and desktop

**Priority:** P0
**Story Points:** 6
**Component:** Responsive Layout

#### BDD Scenarios

```gherkin
Feature: Responsive Design

  Scenario: Mobile view (320px - 768px)
    Given I am on a mobile device
    When I view the dashboard
    Then it should:
      | element        | behavior on mobile               |
      | sidebar        | hidden, accessible via drawer    |
      | navigation     | bottom tab bar                   |
      | charts         | stack vertically                 |
      | metrics cards  | full width, stacked              |
      | table          | horizontal scroll, simplified    |
      | code viewer    | full width, font auto-size       |

  Scenario: Tablet view (768px - 1024px)
    Given I am on a tablet
    When I view the dashboard
    Then it should:
      | element        | behavior on tablet               |
      | sidebar        | collapsible                      |
      | navigation     | side + bottom options            |
      | charts         | 2-column grid                    |
      | metrics cards  | 2-column grid                    |
      | table          | show all columns                 |
      | code viewer    | optimized width                  |

  Scenario: Desktop view (1024px+)
    Given I am on a desktop
    When I view the dashboard
    Then it should:
      | element        | behavior on desktop              |
      | sidebar        | always visible                   |
      | navigation     | side navigation                  |
      | charts         | 4-column grid                    |
      | metrics cards  | 4-column grid                    |
      | table          | all features visible             |
      | code viewer    | 3-pane layout                    |

  Scenario: Touch gestures
    Given I am on a touch device
    When I interact with the UI
    Then it should support:
      | gesture       | behavior                          |
      | swipe         | navigate between views            |
      | pinch-to-zoom | zoom in code viewer               |
      | tap           | select items                      |
      | long press    | show context menu                 |

  Scenario: Keyboard navigation
    Given I am using keyboard
    When I navigate the interface
    Then it should:
      | key           | behavior                          |
      | Tab           | focus next element                |
      | Shift+Tab     | focus previous element            |
      | Enter         | activate focused element          |
      | Space         | toggle checkbox/dropdown          |
      | Arrow keys    | navigate within components        |
```

#### Implementation Tasks

**Task 4.1: Setup Tailwind Responsive**
- Configure Tailwind breakpoints
- Setup responsive utilities
- Test on multiple screen sizes
- Document responsive patterns
- Setup visual regression tests

**Task 4.2: Mobile Navigation**
- Create drawer component
- Implement bottom tab bar
- Setup gesture handling
- Add haptic feedback
- Test on iOS/Android

**Task 4.3: Responsive Tables**
- Configure responsive table
- Add horizontal scroll
- Implement column hiding
- Setup row details modal
- Test with large datasets

**Task 4.4: Mobile Code Viewer**
- Setup mobile Monaco config
- Add pinch-to-zoom
- Implement font size controls
- Add landscape mode
- Test on various devices

#### Test Suite

```typescript
// src/components/__tests__/Responsive.test.tsx

import { render } from '@testing-library/react';
import { Dashboard } from '../Dashboard';

describe('Responsive Design', () => {
  it('should show sidebar on desktop', () => {
    Object.defineProperty(window, 'innerWidth', {
      writable: true,
      configurable: true,
      value: 1920,
    });

    render(<Dashboard />);

    const sidebar = screen.getByTestId('sidebar');
    expect(sidebar).toBeVisible();
  });

  it('should hide sidebar on mobile', () => {
    Object.defineProperty(window, 'innerWidth', {
      writable: true,
      configurable: true,
      value: 375,
    });

    render(<Dashboard />);

    const sidebar = screen.getByTestId('sidebar');
    expect(sidebar).not.toBeVisible();
  });
});
```

---

## 🏗️ Technical Implementation

### Tech Stack

```json
{
  "frontend": {
    "framework": "React 18.3.1",
    "language": "TypeScript 5.0+",
    "bundler": "Vite 5.0+",
    "styling": "Tailwind CSS 3.4+",
    "components": "shadcn/ui + Radix UI",
    "state": {
      "client": "Zustand 4.0+",
      "server": "TanStack Query 5.0+"
    },
    "code_editor": "Monaco Editor",
    "charts": "Recharts 2.0+",
    "virtual_scrolling": "react-window 1.0+",
    "search": "Fuse.js 6.0+",
    "forms": "React Hook Form 7.0+",
    "notifications": "Sonner 1.0+",
    "icons": "Lucide React",
    "web_socket": "native WebSocket API"
  }
}
```

### Project Structure

```
src/
├── components/
│   ├── ui/                    # shadcn/ui components
│   ├── dashboard/             # Dashboard components
│   │   ├── Dashboard.tsx
│   │   ├── MetricsCard.tsx
│   │   ├── TrendsChart.tsx
│   │   └── ProjectHealth.tsx
│   ├── code-viewer/           # Code viewer components
│   │   ├── CodeViewer.tsx
│   │   ├── FindingMarker.tsx
│   │   └── TaintFlow.tsx
│   ├── issue-table/           # Issue table components
│   │   ├── IssueTable.tsx
│   │   ├── IssueRow.tsx
│   │   └── Filters.tsx
│   └── layout/                # Layout components
│       ├── Header.tsx
│       ├── Sidebar.tsx
│       └── Navigation.tsx
├── stores/                    # Zustand stores
│   ├── dashboardStore.ts
│   ├── issueStore.ts
│   └── themeStore.ts
├── hooks/                     # Custom hooks
│   ├── useWebSocket.ts
│   ├── useMetrics.ts
│   └── useIssues.ts
├── services/                  # API services
│   ├── api.ts
│   ├── websocket.ts
│   └── metrics.ts
├── types/                     # TypeScript types
│   ├── finding.ts
│   ├── issue.ts
│   └── metrics.ts
├── utils/                     # Utilities
│   ├── formatters.ts
│   ├── validators.ts
│   └── constants.ts
├── App.tsx
├── main.tsx
└── index.css
```

### State Management Architecture

```typescript
// stores/dashboardStore.ts

interface DashboardState {
  metrics: Metrics;
  projects: Project[];
  filters: DashboardFilters;
  theme: 'light' | 'dark';
  setMetrics: (metrics: Metrics) => void;
  updateProject: (id: string, updates: Partial<Project>) => void;
  setTheme: (theme: 'light' | 'dark') => void;
}

export const useDashboardStore = create<DashboardState>((set) => ({
  metrics: initialMetrics,
  projects: [],
  filters: {},
  theme: 'light',
  setMetrics: (metrics) => set({ metrics }),
  updateProject: (id, updates) =>
    set((state) => ({
      projects: state.projects.map((p) =>
        p.id === id ? { ...p, ...updates } : p
      ),
    })),
  setTheme: (theme) => set({ theme }),
}));
```

### WebSocket Integration

```typescript
// hooks/useWebSocket.ts

export const useWebSocket = (url: string) => {
  const [socket, setSocket] = useState<WebSocket | null>(null);
  const [messages, setMessages] = useState<any[]>([]);
  const [connected, setConnected] = useState(false);

  useEffect(() => {
    const ws = new WebSocket(url);

    ws.onopen = () => setConnected(true);
    ws.onclose = () => setConnected(false);
    ws.onmessage = (event) => {
      const data = JSON.parse(event.data);
      setMessages((prev) => [...prev, data]);
    };

    setSocket(ws);

    return () => {
      ws.close();
    };
  }, [url]);

  const send = (data: any) => {
    if (socket && connected) {
      socket.send(JSON.stringify(data));
    }
  };

  return { socket, connected, messages, send };
};
```

---

## 📊 Performance Benchmarks

### Target Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Bundle Size** | <500KB gzipped | Build output |
| **First Contentful Paint** | <1.5s | Lighthouse |
| **Largest Contentful Paint** | <2.5s | Lighthouse |
| **Time to Interactive** | <3s | Lighthouse |
| **Cumulative Layout Shift** | <0.1 | Lighthouse |
| **First Input Delay** | <100ms | Lighthouse |
| **Runtime Performance** | 60fps | Chrome DevTools |
| **Memory Usage** | <100MB | Performance tab |
| **Table Render** | 10,000+ rows | Custom benchmark |
| **Code Viewer Load** | <500ms | Custom benchmark |

### Performance Optimization

```typescript
// Code splitting
const Dashboard = lazy(() => import('./pages/Dashboard'));
const CodeViewer = lazy(() => import('./components/CodeViewer'));
const IssueTable = lazy(() => import('./components/IssueTable'));

// Memoization
const MemoizedMetricsCard = memo(MetricsCard);
const MemoizedTrendChart = memo(TrendChart);

// Virtual scrolling
const VirtualizedIssueTable = FixedSizeList({
  height: 600,
  itemCount: issues.length,
  itemSize: 50,
  itemData: issues,
});

// Image optimization
<img
  src={lazyImage}
  loading="lazy"
  alt="chart"
/>
```

### Bundle Analysis

```bash
# Analyze bundle size
npm run build
npx vite-bundle-analyzer dist

# Check dependencies
npm run depcheck
npx vite-bundle-analyzer --analyze
```

---

## 🧪 Test Strategy

### Test Types

| Type | Coverage Target | Tools |
|------|----------------|-------|
| **Unit** | 90% | Jest, React Testing Library |
| **Component** | 100% | RTL, MSW |
| **Integration** | 80% | Cypress, Playwright |
| **E2E** | Critical paths | Playwright |
| **Visual** | Major pages | Chromatic, Percy |

### Test Commands

```bash
# Run all tests
npm test

# Run with coverage
npm test -- --coverage

# Run E2E tests
npm run test:e2e

# Visual regression
npm run test:visual

# Performance tests
npm run test:performance
```

---

## ✅ Definition of Done

### Code Quality
- [ ] TypeScript strict mode enabled
- [ ] ESLint + Prettier configured
- [ ] 90% test coverage
- [ ] All components accessible (WCAG 2.1 AA)
- [ ] No console errors/warnings
- [ ] No security vulnerabilities

### Performance
- [ ] Bundle size <500KB
- [ ] FCP <1.5s
- [ ] TTI <3s
- [ ] Lighthouse score >90
- [ ] 60fps runtime performance
- [ ] 10,000+ table rows smooth

### User Experience
- [ ] Real-time updates <100ms
- [ ] Responsive design (320px - 2560px)
- [ ] Dark/light theme
- [ ] Keyboard navigation
- [ ] Touch gestures
- [ ] Offline support (Service Worker)

### Features
- [ ] Dashboard with real-time metrics
- [ ] Code viewer with inline findings
- [ ] Issue table with 10,000+ rows
- [ ] WebSocket integration
- [ ] Search and filtering
- [ ] Export functionality

---

## 📝 Commit Validation Requirements

```bash
feat(epic-web-01): implement React 18 frontend core and dashboard

- Setup React 18 + TypeScript + Vite + Tailwind + shadcn/ui
- Implement real-time dashboard with metrics and trends
- Build Monaco Editor code viewer with inline security findings
- Create virtualized issue table supporting 10,000+ rows
- Add state management (Zustand + TanStack Query)
- Implement WebSocket for real-time updates
- Setup responsive design (mobile, tablet, desktop)
- Add dark/light theme support
- Implement search, filtering, and sorting
- Add accessibility (WCAG 2.1 AA)
- Bundle size optimization: <500KB gzipped
- Performance: FCP <1.5s, TTI <3s, 60fps runtime
- Test coverage: 90% unit, 100% components
- Lighthouse score: >90 (all categories)

Validation:
- All user stories implemented and tested
- Real-time dashboard with metrics working
- Code viewer with inline findings working
- Issue table with virtualization working
- Responsive design tested on all devices
- Performance benchmarks passing
- Accessibility compliance verified
- Bundle size within target

Closes: EPIC-WEB-01
```

---

**Epic Owner:** Frontend Engineering Team
**Reviewers:** UX Team, Performance Team, Architecture Team
**Status:** 🚧 In Progress
**Next Steps:** Begin Phase 1 - React 18 Setup and Dashboard

---

**Copyright © 2025 hodei-scan. All rights reserved.**
