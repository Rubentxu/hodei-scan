# Épica Web 5: Reports & Export
## Generación y Exportación de Reportes

**Versión:** 1.0
**Fecha:** 10 de noviembre de 2025
**Estado:** 🚧 Planning
**Época:** Fase 2 (Meses 7-12)
**Prioridad:** 🟡 High

---

## 📋 Resumen Ejecutivo

Sistema completo de generación, visualización y exportación de reportes para stakeholders técnicos y no-técnicos.

**Objetivos:**
- ✅ PDF report generation
- ✅ Executive summaries
- ✅ Custom report builder
- ✅ Scheduled reports
- ✅ Export formats (PDF, CSV, JSON, Excel)
- ✅ Email report delivery
- ✅ Branded reports con logo

---

## 👥 Historias de Usuario

### US-WEB-11: Como manager, quiero executive summary report

**Prioridad:** 🔴 Critical
**Story Points:** 8

```gherkin
Feature: Executive Report
  Como engineering manager
  Quiero executive summary de code quality
  Para report to leadership

  Scenario: Generate executive report
    Given project analysis completa
    When hace click "Generate Report"
    And selecciona "Executive Summary"
    Then debería crear PDF con:
      And overall health score
      And security posture
      And technical debt trend
      And top 5 issues
      And recommendations
      And charts y graphs
      And no technical jargon
```

**Tareas:**

1. **TASK-WEB-05-01: Report Templates** (2 días)
2. **TASK-WEB-05-02: PDF Generation** (3 días)
3. **TASK-WEB-05-03: Export Formats** (2 días)
4. **TASK-WEB-05-04: Scheduled Reports** (2 días)
5. **TASK-WEB-05-05: Email Delivery** (1 día)

**Tests:**

```typescript
describe('Reports', () => {
  it('should generate PDF report', async () => {
    render(<ReportGenerator />);
    
    fireEvent.click(screen.getByText('Generate PDF'));
    
    await waitFor(() => {
      expect(screen.getByText('Downloading...')).toBeInTheDocument();
    });
  });

  it('should create custom report', async () => {
    render(<CustomReportBuilder />);
    
    fireEvent.click(screen.getByText('Add Section'));
    fireEvent.selectOptions(screen.getByLabelText('Metric'), ['Security', 'Quality']);
    
    fireEvent.click(screen.getByText('Generate'));
    
    await waitFor(() => {
      expect(screen.getByText('Report generated')).toBeInTheDocument();
    });
  });
});
```

---

## 🏗️ Report Builder

```typescript
// components/reports/ReportBuilder.tsx
export const ReportBuilder: React.FC = () => {
  const [sections, setSections] = useState<ReportSection[]>([]);

  const addSection = (type: ReportType) => {
    setSections([...sections, createSection(type)]);
  };

  const generateReport = async () => {
    const reportData = await fetchReportData(sections);
    const pdf = await generatePDF(reportData);
    downloadPDF(pdf);
  };

  return (
    <div>
      <div className="mb-4">
        <Button onClick={() => addSection('security')}>Add Security Section</Button>
        <Button onClick={() => addSection('quality')}>Add Quality Section</Button>
        <Button onClick={() => addSection('debt')}>Add Debt Section</Button>
      </div>

      <SortableContext items={sections}>
        {sections.map((section) => (
          <ReportSection key={section.id} section={section} />
        ))}
      </SortableContext>

      <Button onClick={generateReport} className="w-full">
        Generate Report
      </Button>
    </div>
  );
};
```

---

## 🔄 Criterios de Done

- [ ] ✅ PDF report generation
- [ ] ✅ Executive summary template
- [ ] ✅ Custom report builder
- [ ] ✅ Export CSV, JSON
- [ ] ✅ Scheduled reports
- [ ] ✅ Email delivery
- [ ] ✅ Branded reports
- [ ] ✅ 100% tests

**Total Story Points:** 39 | **Duración:** 5 semanas
