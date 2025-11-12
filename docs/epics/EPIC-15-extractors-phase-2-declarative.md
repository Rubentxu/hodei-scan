# EPIC-15: Extractores Declarativos - Fase 2

## La Fábrica de Reglas: Democratización del Análisis de Código

**Versión:** 1.0.0  
**Fecha Creación:** 2025-11-12  
**Estado:** Propuesta  
**Prioridad:** Alta  
**Fase:** v3.3 - Q1-Q2 2025  
**Dependencias:** EPIC-14 (Fase 1 completada)

---

## 📋 Resumen Ejecutivo

### Objetivo Estratégico

Empoderar a ingenieros de seguridad y desarrolladores para crear reglas personalizadas de análisis de código en <5 minutos, sin escribir código Rust ni recompilar nada, mediante un DSL declarativo basado en YAML y un motor universal tree-sitter.

### Propuesta de Valor

**Para usuarios**:

- Creación de reglas custom para patrones específicos de su organización
- No requiere conocimiento de AST internos ni parsers
- Feedback inmediato: escribir regla → ejecutar → ver resultados

**Para el proyecto**:

- Aceleración masiva en cobertura de reglas (de semanas a minutos por regla)
- Construcción de comunidad: usuarios contribuyen reglas
- Compatibilidad con catálogo de Semgrep (traducción semi-automática de 2000+ reglas)

### Métricas de Éxito

- ✅ **Velocidad de creación**: Regla simple en <5 minutos
- ✅ **Cobertura de lenguajes**: 10+ lenguajes soportados por tree-sitter
- ✅ **Performance**: Motor ejecuta 1000+ reglas YAML sobre 100K LOC en <10 segundos
- ✅ **Adopción**: Traducir 100+ reglas del repositorio de Semgrep en primer mes
- ✅ **Experiencia**: LSP con autocompletado y validación en tiempo real

---

## 🎯 Contexto y Motivación

### El Problema: Crear Reglas es Costoso

En la Fase 1 integramos herramientas existentes, pero estas tienen limitaciones:

- **Ruff, ESLint, Clippy**: Excelentes para reglas generales, pero no permiten patrones específicos de negocio
- **Extensibilidad limitada**: Requiere escribir plugins en el lenguaje del linter
- **No cross-language**: Una regla para Python no funciona para JavaScript

**Ejemplo real**: Una empresa quiere prohibir el uso de una API interna deprecated:

```python
# BAD: Uso de API deprecated
from mycompany.old_api import process_data

result = process_data(user_input)
```

Con linters tradicionales: Escribir plugin custom → 2-3 horas  
Con hodei-scan Fase 2: Escribir regla YAML → 5 minutos

### Benchmark de Competidores

| Herramienta | Enfoque | Curva de Aprendizaje | Tiempo por Regla | Lenguajes |
|-------------|---------|----------------------|------------------|-----------|
| **Semgrep** | DSL YAML + patrones | Media | 10-30 min | 30+ |
| **CodeQL** | Lenguaje propietario (QL) | Alta | 30-60 min | 12 |
| **PMD CPD** | XML + XPath | Alta | 20-40 min | Java, otros |
| **hodei-scan v3.3** | **YAML + tree-sitter** | **Baja** | **5-15 min** | **10+** |

### Estrategia: Motor Universal + Reglas Portables

```
┌──────────────────────────────────────────────────────────┐
│                MOTOR TREE-SITTER UNIVERSAL               │
├──────────────────────────────────────────────────────────┤
│                                                          │
│  [Código Python] → Parser → AST → Matcher de Patrones  │
│  [Código JavaScript] → Parser → AST → Matcher          │
│  [Código Rust] → Parser → AST → Matcher                │
│                            ↑                             │
│                            │                             │
│               Mismas Reglas YAML Declarativas            │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

**Ventaja clave**: Una sola regla puede funcionar en múltiples lenguajes si describen el mismo patrón semántico.

---

## 🏗️ Arquitectura del Sistema Declarativo

### Componentes Principales

```
┌──────────────────────────────────────────────────────────────┐
│                  EXTRACTOR DECLARATIVO                       │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌────────────────────────────────────────────────────┐    │
│  │  1. CARGADOR DE REGLAS                             │    │
│  │     • Lee ficheros .hodei.yml                      │    │
│  │     • Valida esquema                                │    │
│  │     • Compila a estructuras internas               │    │
│  └────────────────────┬───────────────────────────────┘    │
│                       │                                      │
│                       ▼                                      │
│  ┌────────────────────────────────────────────────────┐    │
│  │  2. MOTOR TREE-SITTER                              │    │
│  │     • Inicializa parsers por lenguaje              │    │
│  │     • Parsea código fuente → AST                   │    │
│  │     • Cachea ASTs para múltiples reglas            │    │
│  └────────────────────┬───────────────────────────────┘    │
│                       │                                      │
│                       ▼                                      │
│  ┌────────────────────────────────────────────────────┐    │
│  │  3. MATCHER DE PATRONES                            │    │
│  │     • Ejecuta queries S-expression                 │    │
│  │     • Soporta metavariables ($VAR)                 │    │
│  │     • Evalúa condiciones (where, not)              │    │
│  └────────────────────┬───────────────────────────────┘    │
│                       │                                      │
│                       ▼                                      │
│  ┌────────────────────────────────────────────────────┐    │
│  │  4. GENERADOR DE HECHOS                            │    │
│  │     • Match → Fact (CodeSmell/Vulnerability)       │    │
│  │     • Enriquece con metadata de regla              │    │
│  │     • Exporta IR Cap'n Proto                       │    │
│  └────────────────────────────────────────────────────┘    │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

### Formato de Reglas YAML

```yaml
# .hodei/rules/security/sql-injection-format.yml

id: HODEI-SEC-001
metadata:
  name: "SQL Injection via String Formatting"
  description: |
    Detecta construcción de queries SQL usando concatenación o 
    formateo de strings sin parámetros seguros.
  severity: critical
  confidence: high
  category: security
  cwe: [89]
  owasp: ["A03:2021 - Injection"]
  references:
    - https://owasp.org/www-community/attacks/SQL_Injection

languages:
  - python
  - javascript

patterns:
  # Patrón 1: .execute() con f-string
  - pattern: |
      $DB.execute(f"... {$VAR} ...")
    message: "SQL query construida con f-string. Use parámetros."
    
  # Patrón 2: .execute() con % formatting
  - pattern: |
      $SQL = "... %s ..."
      $DB.execute($SQL % $VAR)
    message: "SQL query con % formatting. Use parámetros."
    
  # Patrón 3: Concatenación directa
  - pattern: |
      $SQL = "SELECT * FROM users WHERE id = " + $VAR
      $DB.execute($SQL)
    message: "SQL query con concatenación. Use parámetros."

# Condiciones avanzadas
where:
  # $DB es un cursor/connection de base de datos
  - metavariable: $DB
    pattern: |
      $DB = $CONN.cursor()
  
  # $VAR no está sanitizado
  - metavariable: $VAR
    not:
      pattern: int($VAR)  # int() sanitiza

# Fix sugerido
fix:
  template: |
    $DB.execute("... WHERE id = ?", ($VAR,))
  message: "Use parámetros parametrizados"
```

### Sintaxis Simplificada para Patrones Comunes

Para acelerar aún más la creación, soportamos atajos:

```yaml
# Versión corta: Detectar hardcoded secrets
id: HODEI-SEC-002
name: "Hardcoded API Keys"
languages: [python, javascript, java]
pattern: |
  $VAR = "$SECRET"
where:
  - metavariable: $VAR
    regex: "(?i)(api_key|password|secret|token)"
  - metavariable: $SECRET
    regex: "[A-Za-z0-9+/]{20,}={0,2}"
severity: critical
```

---

## 📊 Historias de Usuario

### US-15.1: Motor Tree-sitter Multi-Lenguaje

**Como** desarrollador del core  
**Quiero** un motor que parsee código de múltiples lenguajes usando tree-sitter  
**Para** tener una base común para ejecutar reglas declarativas

**Criterios de Aceptación**:

- ✅ Soporta 10+ lenguajes: Python, JavaScript, TypeScript, Rust, Go, Java, C, C++, Ruby, PHP
- ✅ Inicializa parsers lazy (solo cuando se necesitan)
- ✅ Cachea ASTs entre múltiples reglas
- ✅ Rendimiento: <50ms para parsear fichero de 1K LOC
- ✅ Memory efficient: libera ASTs de ficheros ya procesados

**Estimación**: 5 Story Points (5-7 días)

---

### US-15.2: Cargador y Validador de Reglas YAML

**Como** usuario que escribe reglas  
**Quiero** que el sistema valide mi YAML en cuanto lo guardo  
**Para** detectar errores inmediatamente

**Criterios de Aceptación**:

- ✅ Parsea ficheros `.hodei.yml` con esquema JSON Schema
- ✅ Valida campos obligatorios (id, languages, patterns)
- ✅ Genera errores descriptivos con línea y columna
- ✅ Soporta carga de directorios completos de reglas
- ✅ Hot-reload: detecta cambios en ficheros y recarga

**Estimación**: 3 Story Points (3-4 días)

---

### US-15.3: Matcher de Patrones con Metavariables

**Como** usuario avanzado  
**Quiero** usar metavariables ($VAR) que capturen cualquier expresión  
**Para** escribir reglas genéricas que funcionen en múltiples contextos

**Criterios de Aceptación**:

- ✅ Soporta metavariables: `$VAR`, `$FUNC`, `$OBJ`, etc.
- ✅ Matching flexible: `$VAR` captura identificadores, literales, expresiones
- ✅ Backreferences: misma metavariable debe matchear mismo valor
- ✅ Condiciones sobre metavariables: `where: metavariable: $VAR; regex: ...`
- ✅ Negación: `not: pattern: ...`

**Estimación**: 5 Story Points (5-7 días)

---

### US-15.4: Traductor Semi-Automático de Reglas Semgrep

**Como** manager de proyecto  
**Quiero** importar reglas del repositorio de Semgrep automáticamente  
**Para** conseguir cobertura masiva rápidamente

**Criterios de Aceptación**:

- ✅ Script que lee ficheros YAML de Semgrep
- ✅ Traduce campos comunes (id, severity, message, pattern)
- ✅ Mapea diferencias de sintaxis tree-sitter vs Semgrep
- ✅ Genera warnings para patrones no soportados
- ✅ Output: reglas `.hodei.yml` listas para usar
- ✅ Tasa de traducción exitosa: >80% de reglas simples

**Estimación**: 3 Story Points (3-4 días)

---

### US-15.5: Language Server Protocol (LSP) para DSL

**Como** usuario escribiendo reglas en VSCode  
**Quiero** autocompletado, validación y quick fixes  
**Para** ser productivo inmediatamente

**Criterios de Aceptación**:

- ✅ LSP server para ficheros `.hodei.yml`
- ✅ Autocompletado de campos (id, languages, patterns, etc.)
- ✅ Validación en tiempo real (errores rojos)
- ✅ Hover: documentación de campos
- ✅ Go to definition: enlaces a CWEs, referencias
- ✅ Quick fixes: sugerencias de corrección

**Estimación**: 5 Story Points (5-7 días)

---

### US-15.6: Framework de Testing de Reglas

**Como** usuario que escribe reglas  
**Quiero** un framework para testear mis reglas con casos positivos y negativos  
**Para** estar seguro de que funcionan correctamente

**Criterios de Aceptación**:

- ✅ Formato de test en YAML:

```yaml
tests:
  - name: "Detecta f-string en SQL"
    code: |
      cursor.execute(f"SELECT * FROM users WHERE id = {user_id}")
    should_match: true
  
  - name: "No detecta parámetros seguros"
    code: |
      cursor.execute("SELECT * FROM users WHERE id = ?", (user_id,))
    should_match: false
```

- ✅ Comando CLI: `hodei-test-rules --rules-dir .hodei/rules`
- ✅ Output estilo pytest: ✓/✗ por test, summary
- ✅ CI integration: exit code 1 si algún test falla

**Estimación**: 3 Story Points (3-4 días)

---

### US-15.7: Biblioteca de Reglas de Seguridad OWASP Top 10

**Como** usuario nuevo  
**Quiero** un catálogo pre-instalado de reglas de seguridad  
**Para** empezar a analizar mi código inmediatamente

**Criterios de Aceptación**:

- ✅ Reglas para OWASP Top 10:
  - A01: Broken Access Control
  - A02: Cryptographic Failures
  - A03: Injection (SQL, Command, XSS)
  - A04: Insecure Design
  - A05: Security Misconfiguration
  - A06: Vulnerable Components
  - A07: Authentication Failures
  - A08: Software and Data Integrity Failures
  - A09: Security Logging Failures
  - A10: Server-Side Request Forgery (SSRF)

- ✅ Cobertura: 50+ reglas específicas
- ✅ Soporte multi-lenguaje (Python, JS, Java)
- ✅ Documentación: cada regla con ejemplo y fix

**Estimación**: 8 Story Points (8-10 días)

---

## 📈 Plan de Implementación

### Timeline Detallado

**Semana 1-2: Fundamentos**

- Días 1-3: US-15.1 (Motor tree-sitter) - Setup e inicialización
- Días 4-5: US-15.1 (Motor tree-sitter) - Tests + optimización
- Días 6-7: US-15.2 (Cargador YAML) - Implementación

**Semana 3-4: Matching Avanzado**

- Días 1-2: US-15.2 (Cargador YAML) - Completar + tests
- Días 3-7: US-15.3 (Matcher con metavariables) - Implementación completa

**Semana 5-6: Experiencia de Desarrollador**

- Días 1-2: US-15.3 (Matcher) - Optimización
- Días 3-7: US-15.5 (LSP) - Implementación completa
- Paralelo: US-15.6 (Framework testing) - 3 días

**Semana 7-8: Contenido y Traducción**

- Días 1-3: US-15.4 (Traductor Semgrep) - Script de traducción
- Días 4-10: US-15.7 (Biblioteca OWASP) - Creación de reglas

**Semana 9-10: Integración y Pulido**

- Tests end-to-end
- Documentación completa
- Guías de usuario
- Performance tuning

### Dependencias

```
EPIC-14 (Fase 1)
    └─> US-15.1 (Motor tree-sitter)
           ├─> US-15.2 (Cargador YAML)
           │      ├─> US-15.3 (Matcher)
           │      ├─> US-15.5 (LSP)
           │      └─> US-15.6 (Testing)
           ├─> US-15.4 (Traductor Semgrep)
           └─> US-15.7 (Biblioteca OWASP)
```

### Riesgos y Mitigaciones

| Riesgo | Prob. | Impacto | Mitigación |
|--------|-------|---------|------------|
| Tree-sitter no soporta lenguaje X | Media | Alto | Documentar lenguajes soportados claramente |
| Traducción Semgrep <80% éxito | Alta | Medio | Empezar con subset simple, iterar |
| Performance del matcher insuficiente | Baja | Alto | Benchmark early, optimizar algoritmos |
| LSP complejo de implementar | Media | Medio | Usar libraries existentes (tower-lsp) |

---

## 🎯 Criterios de Finalización de Épica

### Funcionales

- ✅ Motor tree-sitter parsea 10+ lenguajes
- ✅ Sistema carga y valida reglas YAML
- ✅ Matcher soporta metavariables y condiciones
- ✅ Traductor convierte >=80% de reglas simples de Semgrep
- ✅ LSP funciona en VSCode con autocompletado
- ✅ Framework de testing permite TDD de reglas
- ✅ Biblioteca con 50+ reglas OWASP lista para usar

### No Funcionales

- ✅ Rendimiento: 1000+ reglas sobre 100K LOC en <10 segundos
- ✅ Tests: Cobertura >=80% en motor y matcher
- ✅ Documentación:
  - Guía de usuario: "Escribiendo tu Primera Regla"
  - Referencia completa del DSL
  - Cookbook con 20+ ejemplos comunes
- ✅ CI/CD: Pipeline verde con tests de integración

### Métricas de Éxito

- **Velocidad de creación**: Usuario crea regla funcional en <5 minutos (medido con user testing)
- **Adopción de catálogo**: 100+ reglas traducidas de Semgrep en primer mes
- **Contribuciones**: >=5 reglas custom contribuidas por early adopters
- **Performance**: <10 segundos para análisis completo con 1000+ reglas

---

## 📚 Recursos y Referencias

### Especificaciones

- [Tree-sitter Documentation](https://tree-sitter.github.io/tree-sitter/)
- [Semgrep Rule Syntax](https://semgrep.dev/docs/writing-rules/rule-syntax/)
- [LSP Specification](https://microsoft.github.io/language-server-protocol/)

### Implementaciones de Referencia

- [Semgrep Open Source](https://github.com/returntocorp/semgrep) - Inspiración de DSL
- [tree-sitter-rust](https://github.com/tree-sitter/tree-sitter/tree/master/lib/binding_rust) - Bindings oficiales
- [tower-lsp](https://github.com/ebkalderon/tower-lsp) - Framework para LSP en Rust

### Catálogos de Reglas

- [Semgrep Registry](https://semgrep.dev/explore) - 2000+ reglas
- [OWASP Top 10](https://owasp.org/Top10/) - Guía de seguridad
- [CWE Top 25](https://cwe.mitre.org/top25/) - Vulnerabilidades comunes

---

**Próxima Épica**: EPIC-16 - Extractores Profundos (Fase 3 - Taint Analysis)
