# Épica Web 4: Settings & Configuration
## Gestión de Configuraciones y Preferencias

**Versión:** 1.0
**Fecha:** 10 de noviembre de 2025
**Estado:** 🚧 Planning
**Época:** Fase 1 (Meses 1-6)
**Prioridad:** 🟡 High

---

## 📋 Resumen Ejecutivo

Crear interfaz para configurar análisis, reglas, quality gates y preferencias de usuario.

**Objetivos:**
- ✅ Project settings (languages, exclusions)
- ✅ Rule configuration (enable/disable)
- ✅ Quality gate thresholds
- ✅ User preferences
- ✅ Notification settings
- ✅ Theme selection (light/dark)

---

## 👥 Historias de Usuario

### US-WEB-10: Como project admin, quiero configure project settings

**Prioridad:** 🔴 Critical
**Story Points:** 8

```gherkin
Feature: Project Settings
  Como project admin
  Quiero configure project parameters
  Para customize analysis behavior

  Scenario: Configure languages
    Given project multi-language
    When navega a Settings > Languages
    Then debería poder:
      And enable/disable specific languages
      And set language priorities
      And exclude directories (node_modules, dist)

  Scenario: Configure quality gates
    Given project con custom standards
    When navega a Settings > Quality Gates
    Then debería set:
      And minimum coverage threshold (e.g., 80%)
      And max issues por severity
      And max technical debt hours
```

**Tareas:**

1. **TASK-WEB-04-01: Settings Layout** (1 día)
2. **TASK-WEB-04-02: Project Settings Form** (2 días)
3. **TASK-WEB-04-03: Quality Gates Config** (2 días)
4. **TASK-WEB-04-04: Rule Toggles** (2 días)
5. **TASK-WEB-04-05: User Preferences** (1 día)

**Tests:**

```typescript
describe('Settings', () => {
  it('should save project settings', async () => {
    render(<ProjectSettings />);
    
    fireEvent.change(screen.getByLabelText('Coverage Threshold'), {
      target: { value: '80' },
    });
    
    fireEvent.click(screen.getByText('Save'));
    
    await waitFor(() => {
      expect(screen.getByText('Settings saved')).toBeInTheDocument();
    });
  });

  it('should toggle rule on/off', () => {
    render(<RuleConfig />);
    
    expect(screen.getByText('SQL Injection Rule')).toBeInTheDocument();
    
    fireEvent.click(screen.getByRole('switch'));
    
    expect(screen.getByRole('switch')).not.toBeChecked();
  });
});
```

---

## 🏗️ Settings Architecture

```typescript
// components/settings/SettingsLayout.tsx
export const SettingsLayout: React.FC = () => {
  const [activeTab, setActiveTab] = useState('project');

  return (
    <div className="flex h-full">
      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList className="w-48">
          <TabsTrigger value="project">Project</TabsTrigger>
          <TabsTrigger value="rules">Rules</TabsTrigger>
          <TabsTrigger value="gates">Quality Gates</TabsTrigger>
          <TabsTrigger value="notifications">Notifications</TabsTrigger>
          <TabsTrigger value="preferences">User Preferences</TabsTrigger>
        </TabsList>

        <div className="flex-1 ml-8">
          <TabsContent value="project">
            <ProjectSettings />
          </TabsContent>
          <TabsContent value="rules">
            <RuleConfiguration />
          </TabsContent>
          <TabsContent value="gates">
            <QualityGatesConfig />
          </TabsContent>
        </div>
      </Tabs>
    </div>
  );
};
```

---

## 🔄 Criterios de Done

- [ ] ✅ Project settings form
- [ ] ✅ Language selection
- [ ] ✅ Rule toggles
- [ ] ✅ Quality gates configuration
- [ ] ✅ User preferences (theme, notifications)
- [ ] ✅ Settings persistence
- [ ] ✅ 100% tests

**Total Story Points:** 34 | **Duración:** 4 semanas
