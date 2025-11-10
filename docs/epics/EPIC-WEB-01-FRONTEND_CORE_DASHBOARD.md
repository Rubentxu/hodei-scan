# Épica Web 1: Frontend Core & Dashboard
## Aplicación Web con Dashboard de Métricas en Tiempo Real

**Versión:** 1.0
**Fecha:** 10 de noviembre de 2025
**Estado:** 🚧 Planning
**Época:** Fase 1 (Meses 1-6)
**Prioridad:** 🔴 Critical

---

## 📋 Resumen Ejecutivo

Desarrollar la aplicación web core de hodei-scan con dashboard principal que muestre métricas de code quality en tiempo real. La aplicación será una SPA (Single Page Application) construida con React + TypeScript + Tailwind CSS, con WebSockets para updates en tiempo real y arquitectura component-first con shadcn/ui.

**Objetivos:**
- ✅ Dashboard principal con métricas de code quality
- ✅ React + TypeScript + Vite + Tailwind CSS stack
- ✅ shadcn/ui para componentes UI
- ✅ WebSocket client para real-time updates
- ✅ React Query para server state management
- ✅ Zustand para client state
- ✅ Code quality visualizations (Chart.js/Recharts)
- ✅ Responsive design (mobile-first)
- ✅ Performance: <2s initial load, <200ms interactions

**Tecnologías:**
- **Frontend**: React 18 + TypeScript
- **Build Tool**: Vite
- **Styling**: Tailwind CSS + shadcn/ui
- **State**: Zustand + TanStack Query
- **Charts**: Recharts/Chart.js
- **WebSocket**: native WebSocket + React Query
- **Testing**: Vitest + React Testing Library

---

## 👥 Historias de Usuario

### US-WEB-01: Como developer, quiero ver un dashboard con overview del proyecto

**Prioridad:** 🔴 Critical
**Story Points:** 8
**Criterios de Aceptación:**

```gherkin
Feature: Dashboard Overview
  Como developer accediendo a hodei-scan
  Quiero ver dashboard con métricas principales
  Para evaluar health del proyecto rápidamente

  Scenario: Dashboard carga con datos
    Given usuario autenticado con proyectos
    When navega a dashboard principal
    Then debería ver en <2 segundos:
      And overall quality score (0-100)
      And total issues count por severity
      And code coverage percentage
      And technical debt en horas
      And security vulnerabilities count
      And trend charts para últimos 7 días

  Scenario: Dashboard con proyecto sin datos
    Given usuario con proyecto nuevo sin análisis
    When navega a dashboard
    Then debería mostrar:
      And estado "Sin análisis aún"
      And botón "Ejecutar análisis"
      And instructions para primer uso

  Scenario: Dashboard responsive
    Given usuario en mobile (320px width)
    When navega a dashboard
    Then debería mostrar:
      And métricas en cards stack vertical
      And charts adaptados a width pequeño
      And navigation accesible (bottom tabs)
```

**Tareas de Desarrollo:**

1. **TASK-WEB-01-01: Setup React + TypeScript + Vite**
   - Criterio: Tests en verde
   - Estimación: 1 día
   - Dependencias: Ninguna
   - Deliverable: App structure funcionando

   ```typescript
   // Implementación mínima requerida:
   describe('App', () => {
     it('should render without crashing', () => {
       render(<App />);
       expect(screen.getByText('hodei-scan')).toBeInTheDocument();
     });
   });
   ```

2. **TASK-WEB-01-02: Configurar Tailwind + shadcn/ui**
   - Criterio: Tests en verde
   - Estimación: 1 día
   - Dependencias: TASK-WEB-01-01
   - Deliverable: Design system configurado

3. **TASK-WEB-01-03: Implementar Dashboard Layout**
   - Criterio: Tests en verde
   - Estimación: 2 días
   - Dependencias: TASK-WEB-01-02
   - Deliverable: Layout responsive

4. **TASK-WEB-01-04: Crear Metrics Cards Components**
   - Criterio: Tests en verde
   - Estimación: 2 días
   - Dependencias: TASK-WEB-01-03
   - Deliverable: QualityScoreCard, IssuesCard, CoverageCard, etc.

5. **TASK-WEB-01-05: Implementar Charts con Recharts**
   - Criterio: Tests en verde
   - Estimación: 2 días
   - Dependencias: TASK-WEB-01-04
   - Deliverable: QualityTrendChart, IssuesOverTimeChart

**Tests de Validación:**

```typescript
// TEST-WEB-01-01: Dashboard renders metrics
describe('Dashboard', () => {
  it('should display quality score', async () => {
    const { data: metrics } = mockUseMetrics();
    render(<Dashboard />);
    
    await waitFor(() => {
      expect(screen.getByText(`${metrics.qualityScore}/100`)).toBeInTheDocument();
    });
  });

  it('should display total issues count', async () => {
    const { data: metrics } = mockUseMetrics();
    render(<Dashboard />);
    
    await waitFor(() => {
      expect(screen.getByText(metrics.totalIssues.toString())).toBeInTheDocument();
    });
  });

  it('should display coverage percentage', async () => {
    const { data: metrics } = mockUseMetrics();
    render(<Dashboard />);
    
    await waitFor(() => {
      expect(screen.getByText(`${metrics.coverage}%`)).toBeInTheDocument();
    });
  });

  it('should show loading state while fetching', () => {
    render(<Dashboard />);
    expect(screen.getByTestId('loading-skeleton')).toBeInTheDocument();
  });
});

// TEST-WEB-01-02: Responsive design
describe('Dashboard Responsive', () => {
  it('should stack cards vertically on mobile', () => {
    render(<Dashboard />);
    const cards = screen.getAllByTestId('metric-card');
    
    // On mobile, cards should be in vertical stack
    cards.forEach(card => {
      expect(card).toHaveClass('w-full');
    });
  });
});
```

---

### US-WEB-02: Como engineering manager, quiero ver trends históricos

**Prioridad:** 🔴 Critical
**Story Points:** 5
**Criterios de Aceptación:**

```gherkin
Feature: Quality Trends
  Como engineering manager
  Quiero ver trends de quality over time
  Para track improvement y regressions

  Scenario: View trends last 7 days
    Given proyecto con análisis histórico
    When navega a "Trends" tab
    Then debería mostrar:
      And chart con quality score por día
      And chart con issues count por día
      And chart con coverage por día
      And poder filtrar por time range (7d, 30d, 90d)

  Scenario: View trends con regression
    Given proyecto con quality regression
    When ve trends chart
    Then debería:
      And mostrar trend line decreasing
      And highlight regression con different color
      And show tooltip con exact values
      And suggest actions para fix regression
```

**Tareas de Desarrollo:**

1. **TASK-WEB-01-06: Implementar Trends Charts**
   - Criterio: Tests en verde
   - Estimación: 2 días
   - Dependencias: TASK-WEB-01-05
   - Deliverable: TrendsChart component

2. **TASK-WEB-01-07: Implementar Time Range Filter**
   - Criterio: Tests en verde
   - Estimación: 1 día
   - Dependencias: TASK-WEB-01-06
   - Deliverable: TimeRangePicker component

3. **TASK-WEB-01-08: Highlight Regressions**
   - Criterio: Tests en verde
   - Estimación: 1 día
   - Dependencias: TASK-WEB-01-07
   - Deliverable: Regression detection logic

**Tests de Validación:**

```typescript
// TEST-WEB-01-03: Trends chart
describe('TrendsChart', () => {
  it('should render line chart with quality data', async () => {
    const trends = generateMockTrends();
    render(<TrendsChart data={trends} />);
    
    expect(screen.getByRole('img', { name: /quality trend/i })).toBeInTheDocument();
  });

  it('should filter by time range', async () => {
    const trends = generateMockTrends();
    render(<TrendsChart data={trends} />);
    
    fireEvent.click(screen.getByText('30d'));
    
    await waitFor(() => {
      const line = screen.getByTestId('trend-line');
      expect(line).toHaveAttribute('data-days', '30');
    });
  });
});
```

---

### US-WEB-03: Como developer, quiero navegación intuitiva

**Prioridad:** 🟡 High
**Story Points:** 5
**Criterios de Aceptación:**

```gherkin
Feature: Navigation
  Como developer navegando en la app
  Quiero navigation clara y intuitiva
  Para access features rápidamente

  Scenario: Navigate entre secciones
    Given usuario en dashboard
    When hace click en "Issues" en sidebar
    Then debería navegar a Issues page
      And debería highlight "Issues" en sidebar
      And debería update URL a /issues
      And debería show breadcrumb: Home > Issues

  Scenario: Quick actions desde navigation
    Given usuario en cualquier page
    When hace click en "Scan Project" en header
    Then debería mostrar modal o navigate a scan page
      And debería keep navigation state
```

**Tareas de Desarrollo:**

1. **TASK-WEB-01-09: Implementar Sidebar Navigation**
   - Criterio: Tests en verde
   - Estimación: 2 días
   - Dependencias: TASK-WEB-01-03
   - Deliverable: Sidebar component

2. **TASK-WEB-01-10: Implementar React Router**
   - Criterio: Tests en verde
   - Estimación: 1 día
   - Dependencias: TASK-WEB-01-09
   - Deliverable: Routing configuration

3. **TASK-WEB-01-11: Implementar Breadcrumbs**
   - Criterio: Tests en verde
   - Estimación: 1 día
   - Dependencias: TASK-WEB-01-10
   - Deliverable: Breadcrumb component

**Tests de Validación:**

```typescript
// TEST-WEB-01-04: Navigation
describe('Navigation', () => {
  it('should navigate to Issues page', async () => {
    render(
      <BrowserRouter>
        <App />
      </BrowserRouter>
    );
    
    fireEvent.click(screen.getByText('Issues'));
    
    await waitFor(() => {
      expect(window.location.pathname).toBe('/issues');
    });
  });

  it('should highlight active menu item', () => {
    render(<Sidebar />);
    
    fireEvent.click(screen.getByText('Dashboard'));
    
    const dashboardItem = screen.getByText('Dashboard').closest('a');
    expect(dashboardItem).toHaveClass('bg-primary');
  });
});
```

---

## 🏗️ Arquitectura Frontend

### Estructura de Directorios

```
src/
├── components/           # Reusable UI components
│   ├── ui/              # shadcn/ui components
│   │   ├── button.tsx
│   │   ├── card.tsx
│   │   └── ...
│   ├── charts/          # Chart components
│   │   ├── LineChart.tsx
│   │   ├── BarChart.tsx
│   │   └── ...
│   └── dashboard/       # Dashboard-specific components
│       ├── MetricsCard.tsx
│       ├── QualityScore.tsx
│       └── ...
├── pages/               # Page components
│   ├── Dashboard.tsx
│   ├── Issues.tsx
│   └── ...
├── hooks/               # Custom React hooks
│   ├── useWebSocket.ts
│   ├── useMetrics.ts
│   └── ...
├── store/               # Zustand stores
│   ├── authStore.ts
│   ├── uiStore.ts
│   └── ...
├── services/            # API services
│   ├── api.ts
│   ├── metrics.ts
│   └── ...
├── types/               # TypeScript types
│   ├── api.ts
│   ├── metrics.ts
│   └── ...
└── utils/               # Utilities
    ├── formatters.ts
    └── constants.ts
```

### Tech Stack Configuration

```typescript
// package.json
{
  "name": "hodei-scan-web",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "test": "vitest",
    "test:ui": "vitest --ui"
  },
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "react-router-dom": "^6.20.0",
    "@tanstack/react-query": "^5.0.0",
    "zustand": "^4.4.0",
    "recharts": "^2.10.0",
    "class-variance-authority": "^0.7.0",
    "clsx": "^2.0.0",
    "tailwind-merge": "^2.0.0"
  },
  "devDependencies": {
    "@types/react": "^18.2.0",
    "@types/react-dom": "^18.2.0",
    "@vitejs/plugin-react": "^4.2.0",
    "vite": "^5.0.0",
    "vitest": "^1.0.0",
    "@testing-library/react": "^13.4.0",
    "tailwindcss": "^3.4.0",
    "typescript": "^5.3.0"
  }
}
```

### State Management

```typescript
// store/dashboardStore.ts
import { create } from 'zustand';

interface DashboardState {
  selectedTimeRange: '7d' | '30d' | '90d';
  selectedProject: string | null;
  isLoading: boolean;
  setTimeRange: (range: '7d' | '30d' | '90d') => void;
  setProject: (project: string) => void;
}

export const useDashboardStore = create<DashboardState>((set) => ({
  selectedTimeRange: '7d',
  selectedProject: null,
  isLoading: false,
  setTimeRange: (range) => set({ selectedTimeRange: range }),
  setProject: (project) => set({ selectedProject: project }),
}));

// hooks/useMetrics.ts
export const useMetrics = (projectId: string, timeRange: string) => {
  return useQuery({
    queryKey: ['metrics', projectId, timeRange],
    queryFn: () => fetchMetrics(projectId, timeRange),
    refetchInterval: 30000, // 30s
  });
};
```

### Dashboard Components

```typescript
// components/dashboard/MetricsCard.tsx
interface MetricsCardProps {
  title: string;
  value: string | number;
  trend?: 'up' | 'down' | 'neutral';
  trendValue?: string;
  icon: React.ReactNode;
}

export const MetricsCard: React.FC<MetricsCardProps> = ({
  title,
  value,
  trend,
  trendValue,
  icon,
}) => {
  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle className="text-sm font-medium">{title}</CardTitle>
        {icon}
      </CardHeader>
      <CardContent>
        <div className="text-2xl font-bold">{value}</div>
        {trendValue && (
          <p className={`text-xs ${trend === 'up' ? 'text-green-600' : trend === 'down' ? 'text-red-600' : 'text-gray-600'}`}>
            {trendValue}
          </p>
        )}
      </CardContent>
    </Card>
  );
};
```

---

## 📊 Estimación y Plan de Entrega

### Cronograma Épica Web 1 (4 semanas)

| Semana | Tareas | Story Points | Entregable |
|--------|--------|--------------|------------|
| 1 | TASK-WEB-01-01 a 01-03 | 16 | Base structure + Layout |
| 2 | TASK-WEB-01-04 a 01-05 | 13 | Metrics cards + Charts |
| 3 | TASK-WEB-01-06 a 01-08 | 8 | Trends + Filters |
| 4 | TASK-WEB-01-09 a 01-11 | 8 | Navigation + Routing |

**Total Story Points:** 45
**Sprints Necesarios:** 2 sprints
**Duración:** 4 semanas

---

## 🧪 Estrategia de Testing Frontend

### Testing Pyramid

1. **Unit Tests (60%)**
   - Component tests (React Testing Library)
   - Hook tests
   - Utility function tests

2. **Integration Tests (30%)**
   - Page tests
   - API integration tests
   - State management tests

3. **E2E Tests (10%)**
   - User workflow tests (Playwright)
   - Cross-browser tests

### Testing Setup

```typescript
// vitest.config.ts
import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  test: {
    environment: 'jsdom',
    setupFiles: './src/test/setup.ts',
    globals: true,
  },
});

// src/test/setup.ts
import { expect, afterEach } from 'vitest';
import { cleanup } from '@testing-library/react';
import * as matchers from '@testing-library/jest-dom/matchers';

expect.extend(matchers);

afterEach(() => {
  cleanup();
});
```

---

## 📚 Documentación de Componentes

### Componentes UI (shadcn/ui)

Configurar shadcn/ui components:

```bash
npx shadcn-ui@latest init
npx shadcn-ui@latest add button card chart tooltip skeleton tabs
```

### Design System

```typescript
// tailwind.config.ts
export default {
  content: ['./index.html', './src/**/*.{js,ts,jsx,tsx}'],
  theme: {
    extend: {
      colors: {
        border: 'hsl(var(--border))',
        input: 'hsl(var(--input))',
        ring: 'hsl(var(--ring))',
        background: 'hsl(var(--background))',
        foreground: 'hsl(var(--foreground))',
        primary: {
          DEFAULT: 'hsl(var(--primary))',
          foreground: 'hsl(var(--primary-foreground))',
        },
        // ... más colores
      },
    },
  },
};
```

---

## 🔄 Criterios de Done

Para que esta épica se considere **COMPLETADA**:

- [ ] ✅ React + TypeScript + Vite setup completo
- [ ] ✅ Tailwind + shadcn/ui configurados
- [ ] ✅ Dashboard carga en <2 segundos
- [ ] ✅ Métricas display correctamente
- [ ] ✅ Charts rendering con Recharts
- [ ] ✅ Navigation completa (sidebar + routing)
- [ ] ✅ Breadcrumbs implementados
- [ ] ✅ Responsive design (320px+)
- [ ] ✅ Loading states implementados
- [ ] ✅ 100% tests en verde
- [ ] ✅ Performance: <200ms interactions
- [ ] ✅ Accesibilidad básica (ARIA labels)

---

## 🚀 Siguiente Épica

Una vez completada esta épica, proceder con:
**[Épica Web 2: Issue Management & Code Viewer](./EPIC-WEB-02-ISSUE_MANAGEMENT.md)**

---

## 📞 Contacto

**Frontend Lead:** [A definir]
**Epic Owner:** [A definir]
**Slack Channel:** #hodei-scan-frontend
**Figma Design:** [Link pendiente]

---

*Última actualización: 10 de noviembre de 2025*
