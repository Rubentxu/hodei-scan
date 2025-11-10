# Épica Web 8: Integrations (PR, IDE, CI/CD)
## Integraciones con Git Providers y CI/CD

**Versión:** 1.0
**Fecha:** 10 de noviembre de 2025
**Estado:** 🚧 Planning
**Época:** Fase 3 (Meses 13-24)
**Prioridad:** 🔴 High

---

## 📋 Resumen Ejecutivo

Sistema de integraciones con GitHub, GitLab, Bitbucket para PR decoration, y CI/CD platforms para automated analysis.

**Objetivos:**
- ✅ GitHub/GitLab/Bitbucket integration
- ✅ PR decoration con inline comments
- ✅ CI/CD pipeline integration
- ✅ IDE extensions status
- ✅ Git hooks setup
- ✅ Webhook management
- ✅ Integration health monitoring

---

## 👥 Historias de Usuario

### US-WEB-14: Como developer, quiero ver analysis results en GitHub PR

**Prioridad:** 🔴 Critical
**Story Points:** 8

```gherkin
Feature: PR Decoration
  Como developer
  Quiero see analysis results en PR
  Para review code quality en context

  Scenario: PR con new issues
    Given PR que introduces new issues
    When GitHub Actions runs hodei-scan
    Then debería add:
      And check run "hodei-scan" con status
      And inline comments en problematic lines
      And summary comment con:
        And new issues count
        And fixed issues count
        And coverage change
        And quality gate status

  Scenario: PR passes quality gate
    Given PR con no new issues
    When completes analysis
    Then debería show:
      And "All checks passed" status
      And green checkmark
      And summary of maintained quality
```

**Tareas:**

1. **TASK-WEB-08-01: GitHub App Integration** (3 días)
2. **TASK-WEB-08-02: PR Decoration Logic** (2 días)
3. **TASK-WEB-08-03: GitLab Integration** (2 días)
4. **TASK-WEB-08-04: CI/CD Pipeline Setup** (2 días)
5. **TASK-WEB-08-05: Webhook Handler** (1 día)

**Tests:**

```typescript
describe('PR Integration', () => {
  it('should add check run to GitHub PR', async () => {
    const pr = createMockPR();
    const analysis = createMockAnalysis();
    
    const result = await addPRCheck(pr, analysis);
    
    expect(result.status).toBe('completed');
    expect(result.conclusion).toBe('success');
  });

  it('should add inline comments for new issues', async () => {
    const pr = createMockPR();
    const issues = [{ line: 42, issue: 'SQL injection' }];
    
    await addInlineComments(pr, issues);
    
    expect(mockGitHubAPI.addComment).toHaveBeenCalledWith(
      expect.objectContaining({
        line: 42,
        body: expect.stringContaining('SQL injection'),
      })
    );
  });
});
```

---

## 🏗️ Integration Architecture

```typescript
// services/github/GitHubService.ts
export class GitHubService {
  async createCheckRun(
    repo: string,
    sha: string,
    analysis: AnalysisResult
  ) {
    const response = await fetch(`/api/integrations/github/check-run`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        repo,
        sha,
        analysis,
      }),
    });

    return response.json();
  }

  async addPRComment(prNumber: number, comment: PRComment) {
    return fetch(`/api/integrations/github/pr-comment`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ prNumber, comment }),
    });
  }
}

// components/integrations/IntegrationStatus.tsx
export const IntegrationStatus: React.FC = () => {
  const { data: integrations } = useQuery(['integrations']);

  return (
    <div className="space-y-4">
      <h2 className="text-2xl font-bold">Integrations</h2>
      
      <Card>
        <CardHeader>
          <CardTitle>GitHub</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center justify-between">
            <span>Status: {integrations.github.connected ? 'Connected' : 'Disconnected'}</span>
            <Badge variant={integrations.github.connected ? 'success' : 'destructive'}>
              {integrations.github.connected ? 'Connected' : 'Connect'}
            </Badge>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};
```

---

## 🔄 Criterios de Done

- [ ] ✅ GitHub App integration
- [ ] ✅ PR decoration working
- [ ] ✅ GitLab integration
- [ ] ✅ CI/CD pipeline config
- [ ] ✅ Webhook handling
- [ ] ✅ Integration health monitoring
- [ ] ✅ IDE extensions status
- [ ] ✅ 100% tests

**Total Story Points:** 39 | **Duración:** 5 semanas
