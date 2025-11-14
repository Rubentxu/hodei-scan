# TODO: Tests Pendientes para Próximo PR

## Estado Actual ✅
- **64 tests** pasando en verde
- **13 test modules** implementados
- **5,125+ líneas** de tests
- Todos los tipos de testing implementados

## Tests Pendientes (4 tests ignorados)

### 1. Integration Tests (1 test)
- `test_jacoco_adapter_integration` en `integration_tests.rs:7`
  - **Motivo**: Depende de archivo real `/home/rubentxu/Proyectos/rust/hodei-scan/test-java-project/jacoco.xml`
  - **Solución**: Migrar a archivo temporal como los otros tests

### 2. JaCoCo Integration Tests (3 tests)
- `test_load_coverage_data_from_real_xml` en `jacoco_integration_test.rs:8`
  - **Motivo**: Usa archivo real en lugar de XML temporal
  - **Estado**: Ya arreglado parcialmente, necesita verificación

- `test_parse_coverage_metrics_correctly` en `jacoco_integration_test.rs:49`
  - **Motivo**: Usa archivo real en lugar de XML temporal
  - **Estado**: Ya arreglado parcialmente, necesita verificación

- `test_extract_source_id_correctly` en `jacoco_integration_test.rs:242`
  - **Motivo**: Usa archivo real en lugar de XML temporal
  - **Estado**: Ya arreglado parcialmente, necesita verificación

## Tests Excluidos Intentionally

### Tests de Alto Costo (No marcados como ignorados, pero no ejecutar en CI)
Los siguientes tests NO fueron incluidos porque requieren recursos significativos:

#### E2E GitHub Tests (`e2e_github_tests.rs`)
- Clonan repositorios reales desde GitHub
- Requieren conexión a internet
- Toman tiempo considerable
- **Recomendación**: Ejecutar manualmente o en CI específica

#### Testcontainers Integration (`testcontainers_integration_tests.rs`)
- Requieren Docker ejecutándose
- Levantan PostgreSQL, Redis, MinIO containers
- **Recomendación**: Ejecutar en CI con Docker o manualmente

#### Load Performance Tests (`load_performance_tests.rs`)
- Tests de estrés y carga
- Toman tiempo considerable (60+ segundos)
- **Recomendación**: Ejecutar nightly o bajo demanda

#### Chaos Tests (`chaos_tests.rs`)
- Simulan condiciones adversas
- Pueden ser no determinísticos
- **Recomendación**: Ejecutar en CI específica de chaos testing

#### Fuzz Tests (`fuzz_tests.rs`)
- Configurados con `#[cfg(fuzzing)]`
- Requieren cargo-fuzz
- **Recomendación**: Ejecutar con `cargo fuzz` específicamente

#### Visual Regression Tests (`visual_regression_tests.rs`)
- Generan screenshots/HTML
- Requieren verificación visual
- **Recomendación**: Ejecutar para releases

## Próximos Pasos

### Prioridad Alta (Siguiente PR)
1. Des-ignorar y verificar los 4 tests marcados con `#[ignore]`
2. Asegurar que todos pasan en verde
3. Ejecutar en CI para validar

### Prioridad Media
1. Configurar CI para ejecutar testcontainers con Docker
2. Configurar cargo-fuzz para fuzz tests
3. Configurar visual regression tests

### Prioridad Baja
1. Documentar cómo ejecutar tests de alto costo manualmente
2. Crear scripts de conveniencia para tests complejos
3. Añadir badges en README para status de tests

## Cómo Ejecutar Tests

### Tests Rápidos (Todos los días)
```bash
cargo test -p hodei-java-extractor --lib --test property_tests --test contract_tests --test integration_tests --test mutation_tests
```

### Tests Completos (Release)
```bash
cargo test -p hodei-java-extractor
```

### Tests de Alto Costo (Bajo Demanda)
```bash
# Testcontainers
cargo test -p hodei-java-extractor --test testcontainers_integration_tests

# Load Performance
cargo test -p hodei-java-extractor --test load_performance_tests

# Chaos
cargo test -p hodei-java-extractor --test chaos_tests

# Fuzz (requiere cargo-fuzz)
cargo install cargo-fuzz
cargo fuzz run jacoco_parse_xml

# E2E GitHub
cargo test -p hodei-java-extractor --test e2e_github_tests
```

## Métricas de Calidad

### Objetivos Alcanzados ✅
- [x] 64 tests unitarios pasando
- [x] Property-based testing (proptest)
- [x] Contract testing (interface compliance)
- [x] Mutation testing (15 escenarios)
- [x] Performance benchmarks
- [x] Security fuzzing (12 targets)
- [x] Chaos engineering
- [x] Testcontainers integration
- [x] E2E integration tests
- [x] Snapshot/visual regression

### Próximos Objetivos
- [ ] 4 tests ignorados → pasando
- [ ] 90%+ coverage
- [ ] Mutation score > 80%
- [ ] E2E tests en CI
- [ ] Fuzzing en CI

---
**Fecha**: 2025-11-14
**Responsable**: Development Team
**Estado**: Tests base completados ✅
