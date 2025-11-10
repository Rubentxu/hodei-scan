# ÉPICA-03: SOFTWARE COMPOSITION ANALYSIS (SCA)

**Versión:** 2.0
**Fecha:** 10 de noviembre de 2025
**Story Points:** 58 SP
**Sprint Estimado:** 4 sprints
**Dependencias:** EPIC-01-CORE_STATIC_ANALYSIS_ENGINE
**Estado:** 🚀 Ready for Development

---

## 📋 Descripción de la Épica

Esta épica implementa el **motor de análisis de composición de software (SCA) basado en IR** que detecta dependencias con vulnerabilidades conocidas, genera SBOMs (Software Bill of Materials), y verifica compliance de licencias. Utiliza la arquitectura IR para correlacionar vulnerabilities con código y cobertura.

**Objetivo Principal:** Implementar análisis completo de dependencias que detecta CVEs, genera SBOMs, verifica licencias, y proporciona supply chain security con correlación cross-domain via IR.

---

## 🎯 Objetivos y Alcance

### Objetivos Estratégicos
1. **CVE Detection** - Dependency → Vulnerability facts con CVSS scoring
2. **SBOM Generation** - SPDX 2.3 y CycloneDX 1.4 compliance
3. **License Compliance** - License facts → compliance rules
4. **Supply Chain Security** - Dependency tree analysis + IR correlation
5. **Package Manager Coverage** - npm, yarn, pnpm, pip, poetry, cargo, go mod, Maven, Gradle, NuGet
6. **<30s Scan Time** - para proyecto típico
7. **<5% False Positives** - en CVE detection

### Alcance Funcional
- ✅ **Vulnerability Detection**: CVE database integration con IR facts
- ✅ **SBOM Generation**: SPDX 2.3, CycloneDX 1.4, customizable formats
- ✅ **License Analysis**: Dependency license detection + compatibility checking
- ✅ **Supply Chain**: Dependency tree visualization + risk assessment
- ✅ **Package Managers**: 10+ ecosystems support
- ✅ **Version Comparison**: Vulnerability por version ranges
- ✅ **Remediation Guidance**: Upgrade paths + fix suggestions
- ✅ **IR Correlation**: Vulnerabilities + Uncovered code (risk score)

### Fuera de Alcance
- ❌ Runtime dependency monitoring - Static analysis only
- ❌ Private vulnerability feeds - Public NVD integration
- ❌ License conflicts resolution - Detection only

---

## 👥 Historias de Usuario

### US-01: JavaScript Package Analysis (npm/yarn/pnpm)
**Como** developer
**Quiero** que el sistema analice dependencias JavaScript/TypeScript
**Para** detectar vulnerabilidades en npm packages

**Criterios de Aceptación:**
```
GIVEN un package.json con dependencies
WHEN se analiza
THEN se extraen todas las dependencias con versiones exactas

GIVEN una dependencia con CVE-2024-1234 conocido
WHEN se evalúa
THEN se reporta vulnerability con CVSS score

GIVEN lock file (package-lock.json, yarn.lock, pnpm-lock.yaml)
WHEN se analiza
THEN se obtienen versiones lockeadas

GIVEN transitive dependency vulnerable
WHEN se analiza
THEN se reporta vulnerability con path completo
```

**Tareas Técnicas:**
- [ ] Implementar package.json parser
- [ ] Crear lock file parsers (npm, yarn, pnpm)
- [ ] Implementar dependency tree resolution
- [ ] Integrar npm registry API
- [ ] Crear version comparison logic
- [ ] Implementar transitive dependency resolution
- [ ] Crear IR facts mapping (dependency, version, scope)
- [ ] Escribir tests con proyectos reales

**TDD Tests:**
```rust
#[cfg(test)]
mod js_sca_tests {
    #[test]
    fn should_extract_dependencies_from_package_json() {
        // Given: package.json con dependencies
        // When: Se analiza
        // Then: Se extraen todas las dependencias
    }

    #[test]
    fn should_detect_known_cve() {
        // Given: Dependencia con CVE-2024-1234
        // When: Se evalúa
        // Then: Finding con CVSS score
    }

    #[test]
    fn should_resolve_transitive_dependencies() {
        // Given: Dependencia que usa otra vulnerable
        // When: Se resuelve tree
        // Then: Se reporta vulnerability con path
    }

    #[test]
    fn should_handle_lock_files() {
        // Given: package-lock.json
        // When: Se analiza
        // Then: Versiones lockeadas extraídas
    }
}
```

---

### US-02: Python Package Analysis (pip/poetry/pipenv)
**Como** developer
**Quiero** que el sistema analice dependencias Python
**Para** detectar vulnerabilidades en PyPI packages

**Criterios de Aceptación:**
```
GIVEN un requirements.txt
WHEN se analiza
THEN se extraen dependencias con version constraints

GIVEN poetry.lock o Pipfile.lock
WHEN se evalúa
THEN se obtienen versiones exactas

GIVEN un package con vulnerabilidad conocida en PyPI
WHEN se compara
THEN se reporta CVE con severidad

GIVEN dependency sin versión específica
WHEN se resuelve
THEN se sugiere upgrade a versión segura
```

**Tareas Técnicas:**
- [ ] Implementar requirements.txt parser
- [ ] Crear poetry.lock parser
- [ ] Implementar Pipfile/Pipfile.lock parser
- [ ] Integrar PyPI API
- [ ] Crear version constraint resolver
- [ ] Implementar dependency resolver (similar a pip)
- [ ] Crear IR facts para Python dependencies
- [ ] Escribir tests con proyectos Python

**TDD Tests:**
```rust
#[cfg(test)]
mod python_sca_tests {
    #[test]
    fn should_parse_requirements_txt() {
        // Given: requirements.txt
        // When: Se parsea
        // Then: Dependencias con constraints extraídas
    }

    #[test]
    fn should_parse_poetry_lock() {
        // Given: poetry.lock
        // When: Se parsea
        // Then: Versiones exactas extraídas
    }

    #[test]
    fn should_detect_pypi_vulnerability() {
        // Given: Paquete vulnerable en PyPI
        // When: Se evalúa
        // Then: CVE reportado
    }

    #[test]
    fn should_suggest_secure_version() {
        // Given: Versión sin constraint
        // When: Se sugiere upgrade
        // Then: Versión segura recomendada
    }
}
```

---

### US-03: Rust Package Analysis (Cargo)
**Como** developer
**Quiero** que el sistema analice dependencias Rust
**Para** detectar vulnerabilidades en crates.io packages

**Criterios de Aceptación:**
```
GIVEN un Cargo.toml
WHEN se analiza
THEN se extraen dependencies y dev-dependencies

GIVEN un Cargo.lock
WHEN se evalúa
THEN se obtienen versiones exactas lockeadas

GIVEN crate con vulnerabilidad conocida
WHEN se compara
THEN se reporta vulnerability con advisory details

GIVEN feature flags habilitados
WHEN se analiza
THEN se incluyen dependencies de features
```

**Tareas Técnicas:**
- [ ] Implementar Cargo.toml parser
- [ ] Crear Cargo.lock parser
- [ ] Integrar crates.io API
- [ ] Implementar feature resolution
- [ ] Crear dependency tree resolver
- [ ] Implementar version comparison
- [ ] Crear IR facts para Rust dependencies
- [ ] Escribir tests con proyectos Rust

**TDD Tests:**
```rust
#[cfg(test)]
mod rust_sca_tests {
    #[test]
    fn should_parse_cargo_toml() {
        // Given: Cargo.toml
        // When: Se parsea
        // Then: Dependencies y dev-dependencies extraídas
    }

    #[test]
    fn should_resolve_features() {
        // Given: Feature flags en Cargo.toml
        // When: Se resuelven
        // Then: Feature dependencies incluidas
    }

    #[test]
    fn should_detect_crate_vulnerability() {
        // Given: Crate vulnerable
        // When: Se evalúa
        // Then: Advisory details reportados
    }
}
```

---

### US-04: Go Module Analysis
**Como** developer
**Quiero** que el sistema analice dependencias Go
**Para** detectar vulnerabilidades en Go packages

**Criterios de Aceptación:**
```
GIVEN un go.mod file
WHEN se analiza
THEN se extraen direct dependencies con versiones

GIVEN un go.sum file
WHEN se evalúa
THEN se validan checksums y versiones

GIVEN módulo con vulnerabilidad conocida
WHEN se compara
THEN se reporta CVE con module path

GIVEN replace directive
WHEN se analiza
THEN se sigue replace para análisis
```

**Tareas Técnicas:**
- [ ] Implementar go.mod parser
- [ ] Crear go.sum validator
- [ ] Integrar Go vulnerability database
- [ ] Implementar module proxy support
- [ ] Crear replace directive handler
- [ ] Implementar version resolution
- [ ] Crear IR facts para Go modules
- [ ] Escribir tests con proyectos Go

**TDD Tests:**
```rust
#[cfg(test)]
mod go_sca_tests {
    #[test]
    fn should_parse_go_mod() {
        // Given: go.mod file
        // When: Se parsea
        // Then: Direct dependencies extraídas
    }

    #[test]
    fn should_validate_go_sum() {
        // Given: go.sum file
        // When: Se valida
        // Then: Checksums verificados
    }

    #[test]
    fn should_handle_replace_directive() {
        // Given: replace directive
        // When: Se sigue
        // Then: Módulo replaced analizado
    }
}
```

---

### US-05: Java Package Analysis (Maven/Gradle)
**Como** developer
**Quiero** que el sistema analice dependencias Java
**Para** detectar vulnerabilidades en Maven Central packages

**Criterios de Aceptación:**
```
GIVEN un pom.xml
WHEN se analiza
THEN se extraen dependencies con scopes (compile, test, provided)

GIVEN un build.gradle(.kts)
WHEN se evalúa
THEN se extraen dependencies y configurations

GIVEN dependency con vulnerability en Maven Central
WHEN se compara
THEN se reporta CVE con GAV coordinates

GIVEN transitive dependency
WHEN se resuelve
THEN se muestra dependency tree completo
```

**Tareas Técnicas:**
- [ ] Implementar pom.xml parser
- [ ] Crear build.gradle parser
- [ ] Integrar Maven Central API
- [ ] Implementar dependency tree resolver
- [ ] Crear scope handling (compile, test, provided, runtime)
- [ ] Implementar version comparison
- [ ] Crear IR facts para Java dependencies
- [ ] Escribir tests con proyectos Java

**TDD Tests:**
```rust
#[cfg(test)]
mod java_sca_tests {
    #[test]
    fn should_parse_pom_xml() {
        // Given: pom.xml
        // When: Se parsea
        // Then: Dependencies con scopes extraídas
    }

    #[test]
    fn should_parse_gradle_build() {
        // Given: build.gradle
        // When: Se parsea
        // Then: Dependencies y configurations extraídas
    }

    #[test]
    fn should_resolve_dependency_tree() {
        // Given: pom.xml con transitive deps
        // When: Se resuelve tree
        // Then: Tree completo mostrado
    }
}
```

---

### US-06: .NET Package Analysis (NuGet)
**Como** developer
**Quiero** que el sistema analice dependencias .NET
**Para** detectar vulnerabilidades en NuGet packages

**Criterios de Aceptación:**
```
GIVEN un .csproj file
WHEN se analiza
THEN se extraen PackageReference items

GIVEN un packages.config
WHEN se evalúa
THEN se obtienen packages con versiones

GIVEN un .assets.json (lock file)
WHEN se analiza
THEN se validan versiones lockeadas

GIVEN package vulnerable en NuGet Gallery
WHEN se compara
THEN se reporta CVE con package details
```

**Tareas Técnicas:**
- [ ] Implementar .csproj parser
- [ ] Crear packages.config parser
- [ ] Integrar NuGet Gallery API
- [ ] Implementar lock file validation
- [ ] Crear framework-specific handling
- [ ] Implementar version range resolution
- [ ] Crear IR facts para .NET dependencies
- [ ] Escribir tests con proyectos .NET

**TDD Tests:**
```rust
#[cfg(test)]
mod dotnet_sca_tests {
    #[test]
    fn should_parse_csproj() {
        // Given: .csproj con PackageReference
        // When: Se parsea
        // Then: Packages extraídos
    }

    #[test]
    fn should_parse_packages_config() {
        // Given: packages.config
        // When: Se parsea
        // Then: Packages con versiones extraídos
    }

    #[test]
    fn should_validate_lock_file() {
        // Given: .assets.json
        // When: Se valida
        // Then: Versiones lockeadas verificadas
    }
}
```

---

### US-07: CVE Database Integration
**Como** security engineer
**Quiero** que el sistema detecte CVEs en dependencies
**Para** identificar vulnerabilidades conocidas

**Criterios de Aceptación:**
```
GIVEN dependency con CVE asignado
WHEN se busca en NVD
THEN se retorna vulnerability details con CVSS score

GIVEN versión específica de dependency
WHEN se compara contra CVE ranges
THEN se determina si está afectada

GIVEN dependency con múltiples CVEs
WHEN se analizan
THEN se reportan todos los CVEs

GIVEN nueva CVE publicada
WHEN se actualiza database
THEN próxima scan la detecta
```

**Tareas Técnicas:**
- [ ] Integrar National Vulnerability Database (NVD) API
- [ ] Implementar CVE search por package
- [ ] Crear version range comparison
- [ ] Implementar CVSS score calculation
- [ ] Crear vulnerability severity classification
- [ ] Implementar local CVE cache
- [ ] Crear database update mechanism
- [ ] Escribir tests con CVEs reales

**TDD Tests:**
```rust
#[cfg(test)]
mod cve_tests {
    #[test]
    fn should_find_cve_for_package() {
        // Given: Package vulnerable
        // When: Se busca en NVD
        // Then: CVE details retornados
    }

    #[test]
    fn should_check_version_affected() {
        // Given: CVE con version range
        // When: Se compara versión
        // Then: Se determina si afectada
    }

    #[test]
    fn should_handle_multiple_cves() {
        // Given: Package con 3 CVEs
        // When: Se analizan
        // Then: 3 CVEs reportados
    }

    #[test]
    fn should_cache_cve_data() {
        // Given: CVE lookup
        // When: Se cachea
        // Then: Próxima lookup usa cache
    }
}
```

---

### US-08: SBOM Generation (SPDX/CycloneDX)
**Como** compliance officer
**Quiero** que el sistema genere SBOMs estándar
**Para** cumplir con Executive Order 14028

**Criterios de Aceptación:**
```
GIVEN proyecto con dependencies
WHEN se genera SBOM
THEN se produce formato SPDX 2.3 válido

GIVEN SBOM en formato CycloneDX 1.4
WHEN se valida
THEN pasa schema validation

GIVEN SBOM generado
WHEN se incluye en security report
THEN se pueden identificar dependencies rápidamente

GIVEN SBOM para diferentes proyectos
WHEN se comparan
THEN se identifican diferencias
```

**Tareas Técnicas:**
- [ ] Implementar SBOM generation engine
- [ ] Crear SPDX 2.3 formatter
- [ ] Implementar CycloneDX 1.4 formatter
- [ ] Crear schema validation
- [ ] Implementar metadata enrichment
- [ ] Crear SBOM diff/comparison
- [ ] Implementar SBOM export (JSON, XML, SPDX tag-value)
- [ ] Escribir tests de SBOM generation

**TDD Tests:**
```rust
#[cfg(test)]
mod sbom_tests {
    #[test]
    fn should_generate_spdx_sbom() {
        // Given: Proyecto con dependencies
        // When: Se genera SBOM
        // Then: Formato SPDX 2.3 válido
    }

    #[test]
    fn should_generate_cyclonedx_sbom() {
        // Given: Proyecto con dependencies
        // When: Se genera SBOM
        // Then: Formato CycloneDX 1.4 válido
    }

    #[test]
    fn should_validate_sbom_schema() {
        // Given: SBOM generado
        // When: Se valida
        // Then: Pasa schema validation
    }

    #[test]
    fn should_compare_sboms() {
        // Given: 2 SBOMs
        // When: Se comparan
        // Then: Diferencias identificadas
    }
}
```

---

### US-09: License Compliance
**Como** legal team
**Quiero** que el sistema verifique license compliance
**Para** evitar infracciones de copyright

**Criterios de Aceptación:**
```
GIVEN dependency con license MIT
WHEN se evalúa contra policy MIT/Apache-2.0
THEN se marca como compatible

GIVEN dependency con license GPL-3.0
WHEN se evalúa contra policy MIT-only
THEN se marca como incompatible

GIVEN dependency sin license declarada
WHEN se analiza
THEN se marca como unknown

GIVEN proyecto con license incompatible
WHEN se genera report
THEN se lista dependency + reason
```

**Tareas Técnicas:**
- [ ] Implementar license detection
- [ ] Crear license database (SPDX identifiers)
- [ ] Implementar compatibility rules engine
- [ ] Crear policy configuration
- [ ] Implementar license scanning (package metadata)
- [ ] Crear compatibility matrix
- [ ] Implementar license exception handling
- [ ] Escribir tests de licenses

**TDD Tests:**
```rust
#[cfg(test)]
mod license_tests {
    #[test]
    fn should_detect_mit_license() {
        // Given: Dependencia con MIT
        // When: Se detecta license
        // Then: MIT reportado
    }

    #[test]
    fn should_mark_license_compatible() {
        // Given: MIT license + policy MIT/Apache
        // When: Se evalúa
        // Then: Compatible marcado
    }

    #[test]
    fn should_mark_license_incompatible() {
        // Given: GPL-3.0 + policy MIT-only
        // When: Se evalúa
        // Then: Incompatible marcado
    }

    #[test]
    fn should_handle_unknown_license() {
        // Given: Dependencia sin license
        // When: Se analiza
        // Then: Unknown marcado
    }
}
```

---

### US-10: Supply Chain Security
**Como** security engineer
**Quiero** que el sistema analice supply chain risks
**Para** detectar dependencias comprometedas o malicious

**Criterios de Aceptación:**
```
GIVEN dependency con maintainer reputation baja
WHEN se evalúa
THEN se marca como supply chain risk

GIVEN dependency que no se actualiza en 2+ años
WHEN se analiza
THEN se marca como outdated

GIVEN dependency con muchos maintainers
WHEN se evalúa
THEN se calcula reputation score

GIVEN dependency recién creado con few downloads
WHEN se analiza
THEN se marca como suspicious
```

**Tareas Técnicas:**
- [ ] Implementar maintainer reputation tracking
- [ ] Crear download statistics integration
- [ ] Implementar last update detection
- [ ] Crear supply chain risk scoring
- [ ] Implementar suspicious package detection
- [ ] Crear dependency age analysis
- [ ] Implementar ecosystem health metrics
- [ ] Escribir tests de supply chain

**TDD Tests:**
```rust
#[cfg(test)]
mod supply_chain_tests {
    #[test]
    fn should_detect_outdated_dependency() {
        // Given: Dependencia sin update en 3 años
        // When: Se analiza
        // Then: Outdated marcado
    }

    #[test]
    fn should_calculate_reputation_score() {
        // Given: Maintainer con historial
        // When: Se calcula score
        // Then: Score retornado
    }

    #[test]
    fn should_detect_suspicious_package() {
        // Given: Package nuevo con pocos downloads
        // When: Se analiza
        // Then: Suspicious marcado
    }
}
```

---

## ✅ Criterios de Validación

### Funcionales
- [ ] **10 Package Managers**: npm, yarn, pnpm, pip, poetry, cargo, go mod, Maven, Gradle, NuGet
- [ ] **CVE Detection**: Integration completa con NVD
- [ ] **SBOM Generation**: SPDX 2.3 + CycloneDX 1.4
- [ ] **License Compliance**: SPDX license database
- [ ] **Supply Chain**: Risk scoring + reputation

### Performance
- [ ] **Scan Time**: <30s para proyecto típico
- [ ] **CVE Lookup**: <100ms cached, <2s uncached
- [ ] **SBOM Generation**: <5s para 1000 dependencies
- [ ] **False Positives**: <5% en CVE detection
- [ ] **False Negatives**: <2% (missed CVEs)

### Calidad
- [ ] **Package Coverage**: 100% ecosystems principales
- [ ] **Test Coverage**: >90%
- [ ] **Documentation**: 100% KDoc
- [ ] **Schema Validation**: 100% SBOMs válidos

---

## 📊 Métricas de Éxito

| Métrica | Target | Actual | Status |
|---------|--------|--------|--------|
| **Scan Time** | <30s | - | ⏳ |
| **CVE Accuracy** | >95% | - | ⏳ |
| **False Positives** | <5% | - | ⏳ |
| **False Negatives** | <2% | - | ⏳ |
| **Package Managers** | 10/10 | - | ⏳ |
| **SBOM Generation** | <5s | - | ⏳ |
| **License Detection** | >98% | - | ⏳ |

---

## 🔗 Dependencias

### Internas
- **EPIC-01-CORE_STATIC_ANALYSIS_ENGINE**: IR Schema, extractors

### Externas
- **NVD API**: National Vulnerability Database
- **PyPI API**: Python Package Index
- **npm Registry**: Node Package Manager
- **crates.io API**: Rust Package Registry
- **Maven Central**: Java Repository
- **NuGet Gallery**: .NET Package Repository
- **SPDX License List**: License database

---

## ⚠️ Riesgos y Mitigación

| Riesgo | Probabilidad | Impacto | Mitigación |
|--------|-------------|---------|------------|
| **API rate limits** | Alta | Medio | Local cache + batch requests |
| **False positives en CVEs** | Media | Alto | Version range validation |
| **Database updates** | Media | Medio | Automated sync + delta updates |
| **Registry outages** | Baja | Alto | Offline mode + retry logic |
| **License detection errors** | Media | Medio | Multiple sources + heuristics |

---

## 🚀 Plan de Implementación

### Sprint 1 (2 semanas): JavaScript + Python
- Implementar npm/yarn/pnpm analyzer
- Implementar pip/poetry analyzer
- Integrar registries APIs
- CVE detection básico

### Sprint 2 (2 semanas): Rust + Go + Java
- Implementar Cargo analyzer
- Implementar Go modules analyzer
- Implementar Maven/Gradle analyzer
- Dependency tree resolution

### Sprint 3 (2 semanas): .NET + CVE Enhancement
- Implementar NuGet analyzer
- Enhanced CVE database integration
- CVSS scoring + classification
- License compliance engine

### Sprint 4 (2 semanas): SBOM + Supply Chain
- Implementar SBOM generation (SPDX, CycloneDX)
- Supply chain security analysis
- Performance optimization
- Documentation + tests

---

## 📚 Referencias Técnicas

- [SPDX 2.3 Specification](https://spdx.github.io/spdx-spec/v2.3/)
- [CycloneDX 1.4 Specification](https://cyclonedx.org/specification/1.4/)
- [National Vulnerability Database](https://nvd.nist.gov/)
- [Executive Order 14028 - SBOM](https://www.whitehouse.gov/briefing-room/presidential-actions/2021/05/12/executive-order-on-improving-the-nations-cybersecurity/)
- [Package Registries APIs](https://docs.npmjs.com/cli/v8/using-npm/registry)

---

**Estado:** ✅ Documentación Completa - Ready for Development
**Próximos Pasos:** Crear EPIC-04-CODE_COVERAGE_INTEGRATION.md
