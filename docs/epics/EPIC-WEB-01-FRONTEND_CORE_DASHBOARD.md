# ÉPICA-WEB-01: FRONTEND CORE & DASHBOARD

**Versión:** 2.0
**Fecha:** 10 de noviembre de 2025
**Story Points:** 45 SP
**Sprint Estimado:** 4 sprints (paralelo con backend)
**Dependencias:** EPIC-01-CORE_STATIC_ANALYSIS_ENGINE (backend API)
**Estado:** 🚀 Ready for Development

---

## 📋 Descripción de la Épica

Esta épica implementa el **core del frontend** usando React 18 + TypeScript + Vite con un dashboard principal que muestra métricas en tiempo real, quality score, trends, y cross-domain correlations. Establece la foundation para toda la aplicación web.

**Objetivo Principal:** Crear la aplicación frontend base con dashboard en tiempo real, routing, state management, y integrations con backend IR para mostrar métricas, trends, y quality scores de manera intuitiva y performante.

---

## 🎯 Objetivos y Alcance

### Objetivos Estratégicos
1. **React 18 + TypeScript** - Modern stack con mejor performance
2. **Real-time Dashboard** - Métricas en tiempo real via WebSocket
3. **Quality Score Visualization** - Executive-level metrics
4. **Cross-domain Correlation** - Security + Coverage + SCA combined
5. **Responsive Design** - Desktop + tablet support
6. **Performance <3s Load** - Bundle <500KB
7. **Dark/Light Theme** - User preference

### Alcance Funcional
- ✅ **Dashboard Principal**: Métricas en tiempo real, quality score, trends
- ✅ **React 18 + TypeScript**: Modern stack con Vite
- ✅ **State Management**: Zustand + TanStack Query
- ✅ **UI Components**: shadcn/ui + Tailwind CSS
- ✅ **Charts**: Recharts para visualizations
- ✅ **Routing**: React Router con lazy loading
- ✅ **Real-time Updates**: WebSocket integration
- ✅ **Theme System**: Dark/Light mode
- ✅ **Performance**: Bundle splitting + code optimization

### Fuera de Alcance
- ❌ Issue Management detallada (EPIC-WEB-02)
- ❌ Auth/RBAC (EPIC-WEB-06)
- ❌ Advanced Settings (EPIC-WEB-04)
- ❌ Reports/Export (EPIC-WEB-05)

---

## 👥 Historias de Usuario

### US-01: Dashboard Overview
**Como** engineering manager
**Quiero** ver overview del proyecto en dashboard
**Para** entender health del código de un vistazo

**Criterios de Aceptación:**
```
GIVEN usuario accede al dashboard
WHEN carga la página
THEN ve: quality score, security issues, coverage %, technical debt

GIVEN nuevos datos disponibles
WHEN se actualizan via WebSocket
THEN dashboard se actualiza en <2s

GIVEN proyecto con 0 issues
WHEN ve dashboard
THEN ve green status con "Excellent" message

GIVEN proyecto con issues críticos
WHEN carga dashboard
THEN ve red status con critical issues count
```

**Tareas Técnicas:**
- [ ] Configurar React 18 + Vite + TypeScript
- [ ] Instalar y configurar Tailwind CSS + shadcn/ui
- [ ] Implementar Zustand para state management
- [ ] Configurar React Router con lazy routes
- [ ] Crear dashboard layout principal
- [ ] Implementar quality score component
- [ ] Integrar Recharts para charts
- [ ] Configurar WebSocket client
- [ ] Escribir tests unitarios y E2E

**TDD Tests:**
```typescript
// Dashboard.test.tsx
describe('Dashboard', () => {
  it('should display quality score', async () => {
    // Given: Project con quality score 85
    // When: Dashboard renders
    // Then: Muestra score 85 con color
  });

  it('should update in real-time', async () => {
    // Given: Dashboard con quality score 80
    // When: WebSocket envía update a 90
    // Then: UI se actualiza en <2s
  });

  it('should show critical status for high issues', async () => {
    // Given: 10 critical issues
    // When: Dashboard renders
    // Then: Status rojo con critical badge
  });
});
```

---

### US-02: Real-time Metrics
**Como** developer
**Quiero** ver metrics actualizándose en tiempo real
**Para** trackear changes immediately después de commit

**Criterios de Aceptación:**
```
GIVEN análisis corriendo
WHEN se completan results
THEN métricas se actualizan automáticamente en dashboard

GIVEN WebSocket disconnected
WHEN se pierde conexión
THEN muestra "Reconnecting..." indicator

GIVEN nueva vulnerability detectada
WHEN se envía via WebSocket
THEN aparece en dashboard con notification

GIVEN coverage update
WHEN se recibe
THEN chart se actualiza smooth animation
```

**Tareas Técnicas:**
- [ ] Implementar WebSocket connection manager
- [ ] Crear real-time metrics store (Zustand)
- [ ] Implementar reconnect logic con exponential backoff
- [ ] Crear notification system para updates
- [ ] Implementar chart animations con Recharts
- [ ] Crear loading states para updates
- [ ] Implementar error handling
- [ ] Escribir tests de WebSocket integration

**TDD Tests:**
```typescript
// WebSocketManager.test.ts
describe('WebSocket Manager', () => {
  it('should reconnect on disconnect', async () => {
    // Given: WebSocket conectado
    // When: Se disconnecta
    // Then: Intenta reconectar con backoff
  });

  it('should update metrics on message', async () => {
    // Given: WebSocket con message de nueva metric
    // When: Recibe message
    // Then: Store se actualiza
  });
});
```

---

### US-03: Security Metrics Visualization
**Como** security engineer
**Quiero** ver security metrics visualizadas
**Para** entender security posture rápidamente

**Criterios de Aceptación:**
```
GIVEN dashboard principal
WHEN se carga
THEN muestra: Critical (0), High (3), Medium (12), Low (8)

GIVEN click en critical count
WHEN se navega
THEN va a detailed security view con filtros

GIVEN 0 vulnerabilities
WHEN se muestra dashboard
THEN muestra green shield icon con "Secure" message

GIVEN nueva vulnerability crítica
WHEN aparece
THEN muestra red alert con sound notification (optional)
```

**Tareas Técnicas:**
- [ ] Crear security metrics component
- [ ] Implementar severity badges (Critical, High, Medium, Low)
- [ ] Crear shield icon con estado colors
- [ ] Implementar drill-down navigation
- [ ] Crear alert system para critical issues
- [ ] Integrar con security API endpoints
- [ ] Implementar sound notifications (opt-in)
- [ ] Escribir tests de security visualization

**TDD Tests:**
```typescript
// SecurityMetrics.test.tsx
describe('Security Metrics', () => {
  it('should display severity breakdown', () => {
    // Given: 3 critical, 5 high, 12 medium
    // When: Render component
    // Then: Shows breakdown correctly
  });

  it('should navigate on click', () => {
    // Given: Security metrics component
    // When: Click en critical count
    // Then: Navigate to security details
  });
});
```

---

### US-04: Coverage Trends Chart
**Como** tech lead
**Quiero** ver coverage trends over time
**Para** identify regression patterns

**Criterios de Aceptación:**
```
GIVEN dashboard
WHEN se carga
THEN muestra line chart con coverage % over last 30 days

GIVEN coverage drops >5%
WHEN se detecta
THEN muestra red indicator en chart

GIVEN hover sobre data point
WHEN se muestra tooltip
THEN muestra exact % y date

GIVEN coverage mejora
WHEN se actualiza
THEN muestra green indicator
```

**Tareas Técnicas:**
- [ ] Implementar coverage trends chart (Recharts LineChart)
- [ ] Crear tooltip personalizado
- [ ] Implementar regression detection
- [ ] Crear indicators para drops/improvements
- [ ] Implementar date range selection (7d, 30d, 90d, 1y)
- [ ] Integrar con coverage API
- [ ] Crear smooth animations
- [ ] Escribir tests de chart

**TDD Tests:**
```typescript
// CoverageChart.test.tsx
describe('Coverage Chart', () => {
  it('should display trend over time', () => {
    // Given: 30 días de data
    // When: Render chart
    // Then: Line graph with points
  });

  it('should highlight drops', () => {
    // Given: Coverage drop del 85% al 75%
    // When: Render chart
    // Then: Red indicator en drop point
  });
});
```

---

### US-05: Quality Gate Status
**Como** engineering manager
**Quiero** ver status de quality gates
**Para** saber si proyecto pasa thresholds

**Criterios de Aceptación:**
```
GIVEN quality gate: 80% coverage
WHEN coverage actual es 85%
THEN muestra green checkmark "PASSED"

GIVEN quality gate: <5 critical issues
WHEN tiene 7 critical
THEN muestra red X "FAILED" con details

GIVEN gate con threshold configurable
WHEN se configuran valores
THEN se actualiza status inmediatamente

GIVEN todos los gates pass
WHEN se muestra
THEN muestra "All Quality Gates PASSED" con celebration
```

**Tareas Técnicas:**
- [ ] Crear quality gates component
- [ ] Implementar pass/fail indicators
- [ ] Crear gate configuration modal
- [ ] Implementar real-time status updates
- [ ] Crear celebration animation (confetti)
- [ ] Integrar con quality gates API
- [ ] Implementar gate details drill-down
- [ ] Escribir tests de quality gates

**TDD Tests:**
```typescript
// QualityGates.test.tsx
describe('Quality Gates', () => {
  it('should show PASSED status', () => {
    // Given: Coverage 85% vs threshold 80%
    // When: Render gates
    // Then: Green checkmark "PASSED"
  });

  it('should show FAILED status', () => {
    // Given: 7 critical vs threshold 5
    // When: Render gates
    // Then: Red X "FAILED"
  });
});
```

---

### US-06: Technical Debt Visualization
**Como** engineering manager
**Quiero** ver technical debt hours y trend
**Para** planificar refactoring sprints

**Criterios de Aceptación:**
```
GIVEN dashboard
WHEN se carga
THEN muestra: 45 horas debt, trend +5h este mes

GIVEN click en debt hours
WHEN se navega
THEN va a detailed debt breakdown por category

GIVEN debt reduction
WHEN se actualiza
THEN muestra green arrow down con "-10h"

GIVEN debt increase
WHEN se actualiza
THEN muestra red arrow up con "+15h"
```

**Tareas Técnicas:**
- [ ] Crear technical debt component
- [ ] Implementar debt hours display
- [ ] Crear trend indicators
- [ ] Implementar drill-down navigation
- [ ] Crear debt breakdown view
- [ ] Integrar con technical debt API
- [ ] Implementar debt categories
- [ ] Escribir tests de debt visualization

**TDD Tests:**
```typescript
// TechnicalDebt.test.tsx
describe('Technical Debt', () => {
  it('should display debt hours', () => {
    // Given: 45 horas debt
    // When: Render component
    // Then: Shows "45h" con icon
  });

  it('should show positive trend', () => {
    // Given: +5h este mes
    // When: Render
    // Then: Red arrow up "+5h"
  });
});
```

---

### US-07: Theme System (Dark/Light)
**Como** developer
**Quiero** switch entre dark y light theme
**Para** work comfortably en different environments

**Criterios de Aceptación:**
```
GIVEN usuario en light theme
WHEN click en toggle
THEN cambia a dark theme inmediatamente

GIVEN theme selection
WHEN se guarda en localStorage
THEN persiste entre browser sessions

GIVEN system theme cambia
WHEN se detecta (prefers-color-scheme)
THEN sugiere switch to system theme

GIVEN theme applied
WHEN se navega entre pages
THEN mantiene theme consistency
```

**Tareas Técnicas:**
- [ ] Implementar theme provider (Zustand)
- [ ] Crear theme toggle component
- [ ] Configurar Tailwind dark mode
- [ ] Implementar localStorage persistence
- [ ] Detectar system theme preference
- [ ] Crear theme switcher UI
- [ ] Aplicar themes a todos components
- [ ] Escribir tests de theme system

**TDD Tests:**
```typescript
// ThemeProvider.test.tsx
describe('Theme Provider', () => {
  it('should toggle theme', () => {
    // Given: Light theme
    // When: Toggle to dark
    // Then: Theme changes to dark
  });

  it('should persist selection', () => {
    // Given: Dark theme selected
    // When: Reload page
    // Then: Still dark theme
  });
});
```

---

## ✅ Criterios de Validación

### Funcionales
- [ ] Dashboard principal con métricas en tiempo real
- [ ] Quality score visualization
- [ ] Security metrics breakdown
- [ ] Coverage trends chart
- [ ] Quality gates status
- [ ] Technical debt display
- [ ] Theme system (dark/light)
- [ ] WebSocket real-time updates

### Performance
- [ ] **Initial Load**: <3s
- [ ] **Bundle Size**: <500KB (gzipped)
- [ ] **Time to Interactive**: <2s
- [ ] **WebSocket Latency**: <500ms
- [ ] **Chart Rendering**: <100ms
- [ ] **Theme Switch**: <200ms

### Calidad
- [ ] **TypeScript Coverage**: 100%
- [ ] **Test Coverage**: >85%
- [ ] **Lighthouse Score**: >90
- [ ] **Accessibility**: WCAG 2.1 AA
- [ ] **Responsive**: Desktop + tablet

---

## 📊 Métricas de Éxito

| Métrica | Target | Actual | Status |
|---------|--------|--------|--------|
| **Initial Load Time** | <3s | - | ⏳ |
| **Bundle Size** | <500KB | - | ⏳ |
| **WebSocket Latency** | <500ms | - | ⏳ |
| **Test Coverage** | >85% | - | ⏳ |
| **Lighthouse Performance** | >90 | - | ⏳ |
| **TypeScript Coverage** | 100% | - | ⏳ |

---

## 🔗 Dependencias

### Internas
- **EPIC-01-CORE_STATIC_ANALYSIS_ENGINE**: Backend IR API
- Backend WebSocket para real-time updates

### Externas
- **React 18**: UI library
- **TypeScript**: Type safety
- **Vite**: Build tool
- **Tailwind CSS**: Styling
- **shadcn/ui**: Component library
- **Zustand**: State management
- **TanStack Query**: Data fetching
- **Recharts**: Charts library
- **React Router**: Routing

---

## ⚠️ Riesgos y Mitigación

| Riesgo | Probabilidad | Impacto | Mitigación |
|--------|-------------|---------|------------|
| **WebSocket disconnections** | Media | Medio | Auto-reconnect + offline indicators |
| **Bundle size >500KB** | Media | Alto | Code splitting + tree shaking |
| **Chart performance** | Baja | Medio | React.memo + virtualization |
| **Theme inconsistencies** | Media | Medio | Comprehensive theming system |
| **Real-time data sync** | Media | Alto | Buffer + delta updates |

---

## 🚀 Plan de Implementación

### Sprint 1 (1.5 semanas): Foundation
- Configurar React 18 + Vite + TypeScript
- Instalar y configurar Tailwind + shadcn/ui
- Implementar routing y lazy loading
- Crear base layout y theme system
- Escribir tests foundation

### Sprint 2 (1.5 semanas): Dashboard Core
- Implementar dashboard layout
- Crear quality score component
- Implementar security metrics visualization
- Integrar con backend API (mock data inicial)
- Escribir tests componentes

### Sprint 3 (1 semana): Real-time + Charts
- Implementar WebSocket client
- Crear coverage trends chart (Recharts)
- Implementar real-time updates
- Add loading states y error handling
- Escribir tests WebSocket

### Sprint 4 (1 semana): Quality Gates + Debt
- Implementar quality gates status
- Crear technical debt visualization
- Implementar drill-down navigation
- Performance optimization
- E2E tests + documentation

---

## 📚 Referencias Técnicas

- [React 18 Documentation](https://react.dev/)
- [Vite Guide](https://vitejs.dev/)
- [TypeScript Handbook](https://www.typescriptlang.org/docs/)
- [Tailwind CSS](https://tailwindcss.com/)
- [shadcn/ui](https://ui.shadcn.com/)
- [Zustand Documentation](https://docs.pmnd.rs/zustand/)
- [TanStack Query](https://tanstack.com/query/)
- [Recharts Documentation](https://recharts.org/)
- [React Router](https://reactrouter.com/)

---

**Estado:** ✅ Documentación Completa - Ready for Development
**Próximos Pasos:** Crear EPIC-WEB-02-ISSUE_MANAGEMENT.md
