# Épica Web 10: Mobile Responsive & Accessibility
## Diseño Responsive y Cumplimiento de Accesibilidad

**Versión:** 1.0
**Fecha:** 10 de noviembre de 2025
**Estado:** 🚧 Planning
**Época:** Fase 3 (Meses 13-24)
**Prioridad:** 🟡 High

---

## 📋 Resumen Ejecutivo

Implementar diseño responsive completo y cumplimiento WCAG 2.1 Level AA para asegurar accesibilidad universal.

**Objetivos:**
- ✅ Mobile-first responsive design
- ✅ WCAG 2.1 Level AA compliance
- ✅ Keyboard navigation
- ✅ Screen reader support
- ✅ Focus management
- ✅ ARIA labels
- ✅ High contrast mode
- ✅ Touch-friendly interfaces

---

## 👥 Historias de Usuario

### US-WEB-16: Como user con discapacidad, quiero access usando screen reader

**Prioridad:** 🔴 Critical
**Story Points:** 8

```gherkin
Feature: Accessibility
  Como user con screen reader
  Quiero navigate hodei-scan easily
  Para access all features

  Scenario: Screen reader navigation
    Given user con NVDA/JAWS
    When navigates dashboard
    Then debería:
      And announce page title
      And read headings hierarchy
      And announce interactive elements
      And provide skip links
      And describe charts y metrics

  Scenario: Keyboard navigation
    Given user usando only keyboard
    When navigates Issues table
    Then debería:
      And tab through all elements
      And use arrow keys en table
      And activate buttons con Enter
      And show visible focus indicators
```

**Tareas:**

1. **TASK-WEB-10-01: ARIA Implementation** (2 días)
2. **TASK-WEB-10-02: Keyboard Navigation** (2 días)
3. **TASK-WEB-10-03: Focus Management** (2 días)
4. **TASK-WEB-10-04: Screen Reader Testing** (1 día)
5. **TASK-WEB-10-05: Mobile Responsive** (3 días)

**Tests:**

```typescript
// tests/accessibility/axe.test.tsx
import { render } from '@testing-library/react';
import { axe, toHaveNoViolations } from 'jest-axe';
import { Dashboard } from '../../pages/Dashboard';

expect.extend(toHaveNoViolations);

describe('Accessibility', () => {
  it('should not have accessibility violations', async () => {
    const { container } = render(<Dashboard />);
    const results = await axe(container);
    
    expect(results).toHaveNoViolations();
  });

  it('should have proper heading hierarchy', () => {
    render(<Dashboard />);
    
    const h1 = screen.getByRole('heading', { level: 1 });
    expect(h1).toBeInTheDocument();
    
    const h2s = screen.getAllByRole('heading', { level: 2 });
    expect(h2s.length).toBeGreaterThan(0);
  });

  it('should support keyboard navigation', () => {
    render(<IssuesTable issues={mockIssues} />);
    
    // Test Tab navigation
    fireEvent.keyDown(screen.getByText('File'), { key: 'Tab' });
    
    // Test Enter activation
    fireEvent.keyDown(screen.getByText('Mark as Fixed'), { key: 'Enter' });
    
    expect(screen.getByText('Issue marked as fixed')).toBeInTheDocument();
  });
});

// tests/accessibility/lighthouse.test.ts
describe('Lighthouse Accessibility', () => {
  it('should pass Lighthouse accessibility audit', async () => {
    // This would run in CI with Lighthouse
    const score = await runLighthouseAudit('accessibility');
    expect(score).toBeGreaterThanOrEqual(90);
  });
});
```

---

## 🏗️ Accessibility Implementation

```typescript
// components/AccessibleButton.tsx
interface AccessibleButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  children: React.ReactNode;
  ariaLabel?: string;
  description?: string;
}

export const AccessibleButton: React.FC<AccessibleButtonProps> = ({
  children,
  ariaLabel,
  description,
  ...props
}) => {
  return (
    <button
      {...props}
      aria-label={ariaLabel}
      aria-describedby={description ? `${props.id}-description` : undefined}
      className={cn(
        'focus:outline-none focus:ring-2 focus:ring-blue-500',
        'px-4 py-2 rounded-md',
        props.className
      )}
    >
      {children}
      {description && (
        <span id={`${props.id}-description`} className="sr-only">
          {description}
        </span>
      )}
    </button>
  );
};

// hooks/useKeyboardNavigation.ts
export const useKeyboardNavigation = (items: any[], onSelect: (item: any) => void) => {
  const [focusedIndex, setFocusedIndex] = useState(0);

  const handleKeyDown = (event: KeyboardEvent) => {
    switch (event.key) {
      case 'ArrowDown':
        event.preventDefault();
        setFocusedIndex((prev) => Math.min(prev + 1, items.length - 1));
        break;
      case 'ArrowUp':
        event.preventDefault();
        setFocusedIndex((prev) => Math.max(prev - 1, 0));
        break;
      case 'Enter':
        event.preventDefault();
        onSelect(items[focusedIndex]);
        break;
    }
  };

  return { focusedIndex, handleKeyDown };
};
```

### Mobile Responsive Design

```typescript
// components/ResponsiveTable.tsx
export const ResponsiveTable: React.FC<{ issues: Issue[] }> = ({ issues }) => {
  const [isMobile, setIsMobile] = useState(false);

  useEffect(() => {
    const checkMobile = () => {
      setIsMobile(window.innerWidth < 768);
    };

    checkMobile();
    window.addEventListener('resize', checkMobile);

    return () => window.removeEventListener('resize', checkMobile);
  }, []);

  if (isMobile) {
    return (
      <div className="space-y-4">
        {issues.map((issue) => (
          <Card key={issue.id} className="p-4">
            <div className="flex justify-between items-start">
              <div>
                <p className="font-semibold">{issue.file}</p>
                <p className="text-sm text-gray-600">Line {issue.line}</p>
              </div>
              <Badge severity={issue.severity}>{issue.severity}</Badge>
            </div>
            <p className="mt-2 text-sm">{issue.message}</p>
          </Card>
        ))}
      </div>
    );
  }

  return <Table data={issues} columns={columns} />;
};
```

---

## 🔄 Criterios de Done

- [ ] ✅ WCAG 2.1 Level AA compliance
- [ ] ✅ Keyboard navigation complete
- [ ] ✅ Screen reader support
- [ ] ✅ ARIA labels on all interactive elements
- [ ] ✅ Focus management
- [ ] ✅ Mobile responsive (320px+)
- [ ] ✅ Touch-friendly interfaces
- [ ] ✅ High contrast mode
- [ ] ✅ Lighthouse accessibility score > 90
- [ ] ✅ 100% tests including a11y tests

**Total Story Points:** 39 | **Duración:** 5 semanas
