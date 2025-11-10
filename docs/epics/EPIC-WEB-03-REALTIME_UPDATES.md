# Épica Web 3: Real-time Updates & WebSockets
## Actualizaciones en Tiempo Real via WebSockets

**Versión:** 1.0
**Fecha:** 10 de noviembre de 2025
**Estado:** 🚧 Planning
**Época:** Fase 1 (Meses 1-6)
**Prioridad:** 🔴 Critical

---

## 📋 Resumen Ejecutivo

Implementar sistema de actualizaciones en tiempo real usando WebSockets para mostrar progreso de análisis, notifications de nuevos issues, y updates de estado sin reload de página.

**Objetivos:**
- ✅ WebSocket client con React Query integration
- ✅ Real-time analysis progress
- ✅ Push notifications para new issues
- ✅ Live updates en dashboard
- ✅ Reconnection logic con exponential backoff
- ✅ Offline queue para missed updates

**Tecnologías:**
- **WebSocket**: Native WebSocket API
- **State**: TanStack Query + WebSocket integration
- **Notifications**: React Hot Toast
- **Reconnection**: ReconnectingWebSocket o custom

---

## 👥 Historias de Usuario

### US-WEB-08: Como developer, quiero ver progress del analysis en tiempo real

**Prioridad:** 🔴 Critical
**Story Points:** 5

```gherkin
Feature: Real-time Analysis Progress
  Como developer ejecutando analysis
  Quiero ver progress en tiempo real
  Para know cuando termina

  Scenario: Analysis in progress
    Given inicia analysis de proyecto
    When está en dashboard
    Then debería ver:
      And progress bar con % complete
      And "Analyzing..." status
      And files processed count
      And time elapsed
      And estimated time remaining

  Scenario: Analysis completed
    Given analysis reaches 100%
    Then debería:
      And show "Analysis Complete" toast
      And auto-refresh metrics
      And show summary of findings
      And stop progress bar
```

**Tareas:**

1. **TASK-WEB-03-01: WebSocket Client Setup** (2 días)
2. **TASK-WEB-03-02: Analysis Progress Component** (2 días)
3. **TASK-WEB-03-03: Reconnection Logic** (1 día)

**Tests:**

```typescript
describe('WebSocket Client', () => {
  it('should connect and receive progress updates', async () => {
    const ws = new WebSocket('ws://localhost:8080/ws');
    render(<AnalysisProgress />);

    ws.onmessage = (event) => {
      const data = JSON.parse(event.data);
      expect(data.type).toBe('analysis_progress');
    };
  });

  it('should reconnect on connection loss', async () => {
    // Test reconnection logic
  });
});
```

---

### US-WEB-09: Como security team, quiero notifications para critical vulnerabilities

**Prioridad:** 🔴 Critical
**Story Points:** 5

```gherkin
Feature: Push Notifications
  Como security team
  Quiero recibir notifications inmediatas
  Para act on critical issues rápido

  Scenario: Critical vulnerability found
    Given nuevo critical security issue
    When issue es detected
    Then debería show:
      And browser notification (if allowed)
      And toast notification in-app
      And badge en Issues menu
      And "Security Alert" con red color
```

**Tareas:**

1. **TASK-WEB-03-04: Notification System** (2 días)
2. **TASK-WEB-03-05: Browser Notifications** (1 día)

---

## 🏗️ WebSocket Architecture

```typescript
// hooks/useWebSocket.ts
import { useQueryClient } from '@tanstack/react-query';
import { useEffect, useRef } from 'react';

export const useWebSocket = (url: string) => {
  const ws = useRef<WebSocket | null>(null);
  const queryClient = useQueryClient();

  useEffect(() => {
    const connect = () => {
      ws.current = new WebSocket(url);

      ws.current.onopen = () => {
        console.log('WebSocket connected');
      };

      ws.current.onmessage = (event) => {
        const data = JSON.parse(event.data);

        switch (data.type) {
          case 'analysis_progress':
            queryClient.setQueryData(['analysis', data.analysisId], data);
            break;
          case 'new_issue':
            queryClient.invalidateQueries(['issues']);
            break;
          case 'notification':
            showToast(data.message, 'info');
            break;
        }
      };

      ws.current.onclose = () => {
        // Reconnect with exponential backoff
        setTimeout(connect, 1000);
      };
    };

    connect();

    return () => {
      ws.current?.close();
    };
  }, [url]);

  return ws.current;
};
```

---

## 🔄 Criterios de Done

- [ ] ✅ WebSocket connection estable
- [ ] ✅ Real-time progress updates
- [ ] ✅ Notifications para critical issues
- [ ] ✅ Auto-reconnection working
- [ ] ✅ Offline queue para missed updates
- [ ] ✅ 100% tests

**Total Story Points:** 18 | **Duración:** 3 semanas
