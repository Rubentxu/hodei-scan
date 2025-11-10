# Épica 3: Software Composition Analysis (SCA)
## Análisis de Dependencias y Detección de Vulnerabilidades

**Versión:** 1.0
**Fecha:** 10 de noviembre de 2025
**Estado:** 🚧 Planning
**Época:** Fase 2 (Meses 7-12)
**Prioridad:** 🔴 Crítica

---

## 📋 Resumen Ejecutivo

Desarrollar el motor de análisis de composición de software (SCA) para hodei-scan, que detecte vulnerabilidades en dependencias, genere Software Bill of Materials (SBOM) y asegure compliance de licencias. Este motor proporcionará análisis de supply chain security con base de datos de CVE actualizada automáticamente.

### Objetivos Principales
- ✅ Detección automática de dependencias vulnerables
- ✅ Generación de SBOM (Software Bill of Materials) en formatos estándar
- ✅ Management de compliance de licencias
- ✅ Análisis de supply chain security
- ✅ Integración con base de datos CVE (actualizada automáticamente)
- ✅ Assessment de riesgo de third-party libraries

### Métricas de Éxito
- **Performance**: <30s análisis de dependencias de proyecto típico
- **Coverage**: 100% package managers (npm, cargo, go mod, pip, maven)
- **CVE Detection**: >95% accuracy en detección de CVEs
- **SBOM Generation**: Formatos SPDX, CycloneDX
- **License Compliance**: 100% detección de licencias
- **False Positives**: <5% en detección de CVEs

---

## 👥 Historias de Usuario

### US-10: Como developer, quiero saber si mis dependencias tienen vulnerabilidades conocidas

**Prioridad:** 🔴 Critical
**Story Points:** 8
**Criterios de Aceptación:**

```gherkin
Feature: Detección de CVEs en dependencias
  Como developer con proyecto con dependencias
  Quiero que hodei-scan detecte CVEs en mis dependencies
  Para prevenir supply chain attacks

  Scenario: Proyecto Node.js con dependencia vulnerable
    Given package.json con lodash@4.17.15
    And CVE-2019-10744 afecta lodash < 4.17.19
    When hodei-scan ejecuta sca scan
    Then debería detectar CVE-2019-10744
    And debería reportar severidad crítica
    And debería sugerir actualización a 4.17.21

  Scenario: Proyecto Rust con dependencia vulnerable
    Given Cargo.toml con serde_json@1.0
    And CVE-2020-11060 afecta serde_json < 1.0.40
    When hodei-scan ejecuta sca scan
    Then debería detectar la vulnerabilidad
    And debería mostrar CVSS score
    And debería proporcionar fix suggestion

  Scenario: Sin vulnerabilidades
    Given proyecto con todas las dependencias actualizadas
    When hodei-scan ejecuta sca scan
    Then debería reportar "No vulnerabilities found"
    And debería mostrar green status

  Scenario: Proyecto Go con go.mod
    Given go.mod con dependencias específicas
    When hodei-scan ejecuta sca scan
    Then debería parsear go.mod correctamente
    And debería resolver todas las dependencias transitivas
    And debería detectar vulnerabilidades en cualquier nivel
```

**Tareas de Desarrollo:**

1. **TASK-03-01: Implementar Dependency Resolver base**
   - Criterio: Tests en verde
   - Estimación: 3 días
   - Dependencias: Épica 1 - TASK-01-06
   - Deliverable: DependencyResolver trait

   ```rust
   // Implementación mínima requerida:
   #[test]
   fn test_dependency_resolver_npm() {
       let resolver = NPMResolver::new();
       let deps = resolver.parse_lockfile("package-lock.json").unwrap();

       assert!(deps.contains(&Dependency {
           name: "lodash".to_string(),
           version: "4.17.21".to_string(),
           ecosystem: "npm".to_string(),
       }));
   }
   ```

2. **TASK-03-02: Implementar package managers (npm, cargo, go mod)**
   - Criterio: Tests en verde
   - Estimación: 5 días
   - Dependencias: TASK-03-01
   - Deliverable: PackageManager enum con implementaciones

3. **TASK-03-03: Implementar CVE Scanner**
   - Criterio: Tests en verde
   - Estimación: 4 días
   - Dependencias: TASK-03-02
   - Deliverable: CVEDatabase con API integration

4. **TASK-03-04: Implementar automatic database updates**
   - Criterio: Tests en verde
   - Estimación: 3 días
   - Dependencias: TASK-03-03
   - Deliverable: CVEDatabaseUpdater

**Tests de Validación:**

```rust
// TEST-03-01: CVE detection accuracy
#[test]
fn test_cve_detection_accuracy() {
    let project = Project::parse("./tests/fixtures/vulnerable-npm").unwrap();
    let scanner = CVEScanner::new();

    let findings = scanner.scan(&project).unwrap();
    assert!(findings.iter().any(|f| {
        f.cve_id == "CVE-2019-10744" &&
        f.severity == Severity::Critical &&
        f.affected_package == "lodash"
    }));
}

// TEST-03-02: False positive prevention
#[test]
fn test_no_false_positives_updated_deps() {
    let project = Project::parse("./tests/fixtures/secure-npm").unwrap();
    let scanner = CVEScanner::new();

    let findings = scanner.scan(&project).unwrap();
    assert!(findings.is_empty());
}

// TEST-03-03: Transitive dependency scanning
#[test]
fn test_transitive_dependency_cve() {
    let project = Project::parse("./tests/fixtures/go-project").unwrap();
    let scanner = CVEScanner::new();

    // Esta dependencia tiene una vulnerabilidad en una dependency transitiva
    let findings = scanner.scan(&project).unwrap();
    assert!(findings.iter().any(|f| {
        f.is_transitive == true &&
        f.depth >= 2
    }));
}
```

---

### US-11: Como security officer, quiero generar SBOM para compliance

**Prioridad:** 🔴 Critical
**Story Points:** 8
**Criterios de Aceptación:**

```gherkin
Feature: SBOM Generation
  Como security officer para compliance
  Quiero generar Software Bill of Materials (SBOM)
  Para cumplir con Executive Order 14028 y regulatory requirements

  Scenario: Generar SBOM en formato SPDX
    Given proyecto con dependencias diversas
    When ejecuto hodei-scan sca sbom --format spdx
    Then debería generar archivo .spdx.json válido
    And debería incluir todos los componentes directos y transitivos
    And debería incluir licenses y copyright information

  Scenario: Generar SBOM en formato CycloneDX
    Given proyecto multi-lenguaje
    When ejecuto hodei-scan sca sbom --format cyclonedx
    Then debería generar archivo .cdx.json válido
    And debería ser compatible con CycloneDX schema
    And debería incluir metadata del proyecto

  Scenario: Validar SBOM generado
    Given SBOM generado previamente
    When ejecuto hodei-scan sca sbom validate file.sbom
    Then debería validar contra schema correspondiente
    And debería reportar si es válido o no
    And debería mostrar errores específicos si es inválido
```

**Tareas de Desarrollo:**

1. **TASK-03-05: Implementar SBOM Generator**
   - Criterio: Tests en verde
   - Estimación: 4 días
   - Dependencias: TASK-03-02
   - Deliverable: SBOMGenerator con formatos

2. **TASK-03-06: Implementar SPDX format**
   - Criterio: Tests en verde
   - Estimación: 3 días
   - Dependencias: TASK-03-05
   - Deliverable: SPDXGenerator

3. **TASK-03-07: Implementar CycloneDX format**
   - Criterio: Tests en verde
   - Estimación: 3 días
   - Dependencias: TASK-03-05
   - Deliverable: CycloneDXGenerator

4. **TASK-03-08: Implementar SBOM validator**
   - Criterio: Tests en verde
   - Estimación: 2 días
   - Dependencias: TASK-03-06, TASK-03-07
   - Deliverable: SBOMValidator

**Tests de Validación:**

```rust
// TEST-03-04: SPDX generation
#[test]
fn test_spdx_generation() {
    let project = Project::parse("./tests/fixtures/npm-project").unwrap();
    let generator = SPDXGenerator::new();

    let sbom = generator.generate(&project).unwrap();
    assert_eq!(sbom.spdx_version, "SPDX-2.3");
    assert_eq!(sbom.document_namespace, "https://hodei-scan.dev/spdxdocs/");
    assert!(!sbom.packages.is_empty());
}

// TEST-03-05: CycloneDX generation
#[test]
fn test_cyclonedx_generation() {
    let project = Project::parse("./tests/fixtures/rust-project").unwrap();
    let generator = CycloneDXGenerator::new();

    let sbom = generator.generate(&project).unwrap();
    assert_eq!(sbom.bom_format_version, "1.4");
    assert_eq!(sbom.spec_version, "1.4");
    assert!(!sbom.components.is_empty());
}

// TEST-03-06: SBOM validation
#[test]
fn test_spdx_validation_valid() {
    let valid_spdx = load_test_file("valid.spdx.json");
    let validator = SBOMValidator::new();

    let result = validator.validate(&valid_spdx, Format::SPDX);
    assert!(result.is_valid());
}

#[test]
fn test_spdx_validation_invalid() {
    let invalid_spdx = load_test_file("invalid.spdx.json");
    let validator = SBOMValidator::new();

    let result = validator.validate(&invalid_spdx, Format::SPDX);
    assert!(!result.is_valid());
    assert!(!result.errors.is_empty());
}
```

---

### US-12: Como legal team, quiero verificar license compliance

**Prioridad:** 🔴 Critical
**Story Points:** 5
**Criterios de Aceptación:**

```gherkin
Feature: License Compliance
  Como legal team manejando compliance
  Quiero verificar licenses de todas las dependencies
  Para evitar legal issues por incompatibilidad

  Scenario: Detectar licencia MIT
    Given dependencia con license MIT
    When hodei-scan analiza dependencies
    Then debería identificar license como MIT
    And debería reportar como "Approved for commercial use"

  Scenario: Detectar licencia GPL
    Given dependencia con licencia GPL-3.0
    When hodei-scan analiza dependencies
    Then debería identificar licencia como GPL-3.0
    And debería reportar como "Copyleft - Commercial use restricted"
    And debería mostrar warning

  Scenario: Incompatible licenses
    Given proyecto con dependencies con MIT y GPL-3.0
    When hodei-scan analiza dependencies
    Then debería detectar license conflict
    And debería reportar "License incompatibility detected"
    And debería sugerir alternativas

  Scenario: Unknown license
    Given dependencia sin license claramente definida
    When hodei-scan analiza dependencies
    Then debería reportar "License unknown"
    And debería sugerir revisar manualmente
    And debería marcar como "Potential risk"
```

**Tareas de Desarrollo:**

1. **TASK-03-09: Implementar License Detector**
   - Criterio: Tests en verde
   - Estimación: 3 días
   - Dependencias: TASK-03-02
   - Deliverable: LicenseDetector con base de datos

2. **TASK-03-10: Implementar License Compatibility Checker**
   - Criterio: Tests en verde
   - Estimación: 2 días
   - Dependencias: TASK-03-09
   - Deliverable: LicenseCompatibilityMatrix

**Tests de Validación:**

```rust
// TEST-03-07: MIT license detection
#[test]
fn test_mit_license_detection() {
    let detector = LicenseDetector::new();
    let license = detector.detect("tests/fixtures/mit-license-package").unwrap();

    assert_eq!(license.spdx_id, "MIT");
    assert_eq!(license.commercial_use, Permission::Allowed);
    assert_eq!(license.modifications, Permission::Allowed);
}

// TEST-03-08: GPL license detection
#[test]
fn test_gpl_license_detection() {
    let detector = LicenseDetector::new();
    let license = detector.detect("tests/fixtures/gpl-license-package").unwrap();

    assert_eq!(license.spdx_id, "GPL-3.0");
    assert_eq!(license.commercial_use, Permission::Restricted);
    assert_eq!(license.copyleft, true);
}

// TEST-03-09: License compatibility check
#[test]
fn test_license_incompatibility() {
    let project = Project::parse("./tests/fixtures/mixed-licenses").unwrap();
    let checker = LicenseCompatibilityChecker::new();

    let report = checker.check_compatibility(&project).unwrap();
    assert!(report.has_conflicts());
    assert!(report.messages.iter().any(|m| {
        m.contains("MIT") && m.contains("GPL-3.0")
    }));
}
```

---

### US-13: Como DevOps, quiero monitoreo continuo de dependencies

**Prioridad:** 🟡 Medium
**Story Points:** 8
**Criterios de Aceptación:**

```gherkin
Feature: Continuous Dependency Monitoring
  Como DevOps configurando security monitoring
  Quiero que hodei-scan monitoree dependencies continuamente
  Para recibir alerts cuando nuevas CVEs se descubren

  Scenario: Scheduled scan
    Given configuración de scheduled scan daily
    When hodei-scan ejecuta scan en schedule
    Then debería verificar todas las dependencies
    And debería comparar contra latest CVE database
    And debería enviar alerts si nuevas CVEs encontradas

  Scenario: New CVE discovered
    Given dependency marcada como "monitored"
    When nueva CVE se agrega a database
    Then hodei-scan debería detectar en next scan
    And debería generar alert
    And debería reportar a configured channels (email, Slack)

  Scenario: Dependency update available
    Given dependency con nueva versión disponible
    When hodei-scan detecta update
    Then debería reportar "Update available"
    And debería mostrar changelog si disponible
    And debería sugerir testing en staging
```

**Tareas de Desarrollo:**

1. **TASK-03-11: Implementar Scheduled Scanner**
   - Criterio: Tests en verde
   - Estimación: 3 días
   - Dependencias: TASK-03-03
   - Deliverable: ScheduledScanner con cron support

2. **TASK-03-12: Implementar Alert System**
   - Criterio: Tests en verde
   - Estimación: 2 días
   - Dependencias: TASK-03-11
   - Deliverable: AlertManager (email, Slack, webhooks)

**Tests de Validación:**

```rust
// TEST-03-10: Scheduled scan execution
#[test]
fn test_scheduled_scan() {
    let config = ScheduledScanConfig {
        schedule: "0 9 * * *".to_string(), // Daily at 9 AM
        projects: vec!["project-1".to_string(), "project-2".to_string()],
    };

    let scanner = ScheduledScanner::new(config);
    let result = scanner.run_scheduled_scan().unwrap();

    assert_eq!(result.scanned_projects.len(), 2);
    assert!(result.has_vulnerabilities == true);
}

// TEST-03-11: Alert system
#[test]
fn test_email_alert() {
    let alert_manager = AlertManager::new();
    let finding = CVEFinding {
        cve_id: "CVE-2023-12345".to_string(),
        package: "example-package".to_string(),
        severity: Severity::Critical,
    };

    let result = alert_manager.send_alert(
        Channel::Email,
        "security@example.com",
        &finding
    ).unwrap();

    assert!(result.is_delivered());
}
```

---

### US-14: Como developer, quiero alternative suggestions para vulnerable dependencies

**Prioridad:** 🟡 Medium
**Story Points:** 5
**Criterios de Aceptación:**

```gherkin
Feature: Alternative Suggestions
  Como developer con dependencia vulnerable
  Quiero sugerencias de alternativas seguras
  Para reemplazar fácilmente la dependency

  Scenario: Suggest secure alternative
    Given dependency vulnerable lodash
    When hodei-scan detecta CVE
    Then debería sugerir alternativas como "ramda" o "underscore"
    And debería verificar que alternativas no tienen same CVE
    And debería mostrar migration guide si disponible

  Scenario: Suggest version update
    Given dependency con versión vulnerable
    When hodei-scan detecta CVE
    Then debería sugerir minimum safe version
    And debería mostrar changelog highlights
    And debería warn si breaking changes

  Scenario: No alternative available
    Given dependency vulnerable sin alternativas
    When hodei-scan detecta CVE
    Then debería reportar "No alternative available"
    And debería sugerir mitigation strategies
    And debería recomendar monitor closely
```

**Tareas de Desarrollo:**

1. **TASK-03-13: Implementar Alternative Finder**
   - Criterio: Tests en verde
   - Estimación: 3 días
   - Dependencias: TASK-03-03
   - Deliverable: AlternativeFinder con database

2. **TASK-03-14: Implementar Migration Helper**
   - Criterio: Tests en verde
   - Estimación: 2 días
   - Dependencias: TASK-03-13
   - Deliverable: MigrationHelper

**Tests de Validación:**

```rust
// TEST-03-12: Secure alternative suggestion
#[test]
fn test_alternative_suggestion() {
    let finder = AlternativeFinder::new();
    let vuln_dep = VulnerableDependency {
        name: "lodash".to_string(),
        version: "4.17.15".to_string(),
        ecosystem: "npm".to_string(),
        cve: "CVE-2019-10744".to_string(),
    };

    let alternatives = finder.find_alternatives(&vuln_dep).unwrap();
    assert!(!alternatives.is_empty());
    assert!(alternatives.iter().any(|alt| alt.name == "ramda"));
    assert!(alternatives.iter().all(|alt| alt.is_secure == true));
}

// TEST-03-13: Safe version suggestion
#[test]
fn test_safe_version_suggestion() {
    let finder = AlternativeFinder::new();
    let vuln_dep = VulnerableDependency {
        name: "lodash".to_string(),
        version: "4.17.15".to_string(),
        ecosystem: "npm".to_string(),
    };

    let safe_version = finder.find_safe_version(&vuln_dep).unwrap();
    assert!(Version::parse(&safe_version) >= Version::parse("4.17.19"));
}
```

---

## 🏗️ Arquitectura Técnica

### SCA Engine Architecture

```
┌─────────────────────────────────────────┐
│        SCA (Software Composition        │
│           Analysis Engine)              │
├─────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────────┐  │
│  │  Package    │  │  Dependency     │  │
│  │  Managers   │  │  Resolver       │  │
│  │  (npm,      │  │  (Transitive)   │  │
│  │   cargo,    │  │                 │  │
│  │   go mod)   │  │                 │  │
│  └─────────────┘  └─────────────────┘  │
│         │                 │             │
│  ┌──────▼──────┐  ┌──────▼──────┐     │
│  │  Ecosystem  │  │  Version    │     │
│  │  Detector   │  │  Comparator │     │
│  └─────────────┘  └─────────────┘     │
│         │                 │             │
│  ┌──────▼─────────────────▼──────┐      │
│  │  CVE Database & Scanner       │      │
│  │  • Real-time updates          │      │
│  │  • CVSS scoring               │      │
│  │  • False positive filtering   │      │
│  └───────────────────────────────┘      │
│                 │                       │
│  ┌──────────────▼──────────────┐        │
│  │  SBOM Generator             │        │
│  │  • SPDX 2.3                 │        │
│  │  • CycloneDX 1.4            │        │
│  │  • Validation               │        │
│  └─────────────────────────────┘        │
│                 │                       │
│  ┌──────────────▼──────────────┐        │
│  │  License Compliance         │        │
│  │  • License detection        │        │
│  │  • Compatibility matrix     │        │
│  │  • Alternative finder       │        │
│  └─────────────────────────────┘        │
└─────────────────────────────────────────┘
```

### Dependencias SCA

```toml
[dependencies]
# HTTP client para CVE database
reqwest = { version = "0.11", features = ["json"] }
tokio = { version = "1", features = ["full"] }

# JSON parsing
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Version comparison
semver = "1.0"

# Package ecosystem support
cargo-edit = "0.12"  # For Cargo.toml parsing
npm-api = "0.10"     # For npm registry
```

---

## 📊 Estimación y Plan de Entrega

### Cronograma Épica 3 (3 meses, Fase 2)

| Semana | Tareas | Story Points | Entregables |
|--------|--------|--------------|-------------|
| 1-3 | TASK-03-01: Dependency Resolver | 8 | Base resolver |
| 4-8 | TASK-03-02: Package Managers | 13 | npm, cargo, go mod |
| 9-12 | TASK-03-03: CVE Scanner | 10 | CVE detection |
| 13-15 | TASK-03-04: DB Updates | 8 | Auto-updates |
| 16-19 | TASK-03-05 a 03-08: SBOM | 21 | SBOM generation |
| 20-22 | TASK-03-09, 03-10: License | 13 | License compliance |
| 23-25 | TASK-03-11, 03-12: Monitoring | 13 | Continuous monitoring |
| 26-27 | TASK-03-13, 03-14: Alternatives | 13 | Alternative finder |

**Total Story Points:** 100
**Sprints Necesarios:** 3 sprints (27 semanas)
**Duración:** ~3 meses

---

## 🧪 Estrategia de Testing

### SCA Testing Strategy

1. **Unit Tests (60%)**
   - Package manager parser tests
   - CVE detection tests
   - SBOM generation tests
   - License detection tests

2. **Integration Tests (30%)**
   - Real-world project tests
   - Multi-language projects
   - CVE database integration tests
   - SBOM validation tests

3. **Vulnerability Tests (10%)**
   - Known vulnerable projects
   - False positive detection
   - Regression testing

### Test Fixtures

```toml
[dev-dependencies]
# Test utilities
tempfile = "3.0"
mockall = "0.12"  # Para mocking CVE database
wiremock = "0.6"  # Para mocking HTTP endpoints
```

---

## 📚 Formatos de SBOM Soportados

### SPDX (Software Package Data Exchange) 2.3

**Características:**
- Formato JSON estándar
- Identificación única con SPDXRef
- License information completa
- Copyright notices
- File checksums

**Ejemplo:**
```json
{
  "spdxVersion": "SPDX-2.3",
  "dataLicense": "CC0-1.0",
  "SPDXID": "SPDXRef-DOCUMENT",
  "name": "hodei-scan-project",
  "documentNamespace": "https://hodei-scan.dev/spdxdocs/...",
  "creationInfo": {
    "created": "2025-11-10T00:00:00Z",
    "creators": ["Tool: hodei-scan v1.0"]
  },
  "packages": [
    {
      "SPDXID": "SPDXRef-Package-lodash",
      "name": "lodash",
      "downloadLocation": "https://registry.npmjs.org/lodash/-/lodash-4.17.21.tgz",
      "filesAnalyzed": false,
      "licenseConcluded": "MIT",
      "licenseDeclared": "MIT",
      "copyrightText": "Copyright JS Foundation and other contributors"
    }
  ]
}
```

### CycloneDX 1.4

**Características:**
- Formato XML/JSON compatible
- BomMetaData con timestamp
- Component hierarchy
- External references
- Vulnerabilities

**Ejemplo:**
```json
{
  "bomFormat": "CycloneDX",
  "specVersion": "1.4",
  "serialNumber": "urn:uuid:...",
  "version": 1,
  "metadata": {
    "timestamp": "2025-11-10T00:00:00.000Z",
    "component": {
      "type": "application",
      "bom-ref": "hodei-scan",
      "name": "hodei-scan-project"
    }
  },
  "components": [
    {
      "type": "library",
      "bom-ref": "pkg:npm/lodash@4.17.21",
      "name": "lodash",
      "version": "4.17.21",
      "licenses": [
        {
          "expression": "MIT"
        }
      ]
    }
  ]
}
```

---

## 🔄 Criterios de Done

Para que esta épica se considere **COMPLETADA**:

- [ ] ✅ 3 package managers soportados (npm, cargo, go mod)
- [ ] ✅ CVE database con 95%+ accuracy
- [ ] ✅ <30s scan time para proyecto típico
- [ ] ✅ SBOM generation en formatos SPDX y CycloneDX
- [ ] ✅ SBOM validation funcional
- [ ] ✅ License compliance checker completo
- [ ] ✅ Continuous monitoring con alerts
- [ ] ✅ Alternative suggestions functional
- [ ] ✅ 100% tests en verde
- [ ] ✅ Integration con real-world projects
- [ ] ✅ Performance benchmarks passed

---

## 🚀 Siguiente Épica

Una vez completada esta épica, proceder con:
**[Épica 4: Code Coverage Integration](./EPIC-04-CODE_COVERAGE_INTEGRATION.md)**

---

## 📞 Contacto y Soporte

**SCA Lead:** [A definir]
**Epic Owner:** [A definir]
**Slack Channel:** #hodei-scan-sca
**CVE Database:** cve@hodei-scan.dev

---

*Última actualización: 10 de noviembre de 2025*
