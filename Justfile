#!/usr/bin/env just --justfile

# ==============================================================================
# HODEI-SCAN - Test Execution Profiles
# ==============================================================================
# Uso: just <comando>
#
# Perfiles disponibles:
# - quick: Tests rápidos sin infraestructura
# - full: Todos los tests incluyendo los ignorados
# - db: Solo tests de base de datos (PostgreSQL)
# - tdd: Solo tests TDD Red (no implementados)
# - integration: Tests de integración
# ==============================================================================

# Ejecutar tests rápidos (sin infraestructura)
@quick:
    cargo test --workspace --lib --bins --tests --exclude hodei-server

# Ejecutar TODOS los tests (incluyendo ignorados)
@full:
    cargo test --workspace

# Ejecutar solo tests que requieren base de datos
@db:
    cargo test --package hodei-server --lib -- --ignored

# Ejecutar tests de base de datos con contenedor Docker automático
@db-container:
    @echo "🐳 Iniciando contenedor PostgreSQL para tests..."
    ./scripts/test-containers.sh run-tests "cargo test --package hodei-server --lib modules::baseline::tests"

# Ejecutar solo tests TDD Red (implementaciones pendientes)
@tdd:
    @echo "🧪 Tests TDD Red (implementación pendiente)..."
    cargo test --package hodei-server --test dashboard_api -- --ignored
    cargo test --package hodei-server --test diff_api -- --ignored

# Ejecutar tests de integración
@integration:
    cargo test --package hodei-deep-analysis-engine --test integration
    cargo test --package hodei-deep-analysis-engine --test connascence
    cargo test --package hodei-deep-analysis-engine --test taint_analysis

# Ejecutar tests de un crate específico
@crate crate:
    cargo test --package {{crate}} --lib

# Ejecutar tests con coverage
@coverage:
    cargo test --workspace --verbose

# Limpiar archivos de test temporales
@clean:
    find /tmp -name "hodei-test-*" -type d -exec rm -rf {} + 2>/dev/null || true
    echo "✅ Limpieza completada"

# Verificar que el workspace compila
@check:
    cargo check --workspace

# Formatear código
@fmt:
    cargo fmt --all
    cargo clippy --workspace

# Verificar linting
@lint:
    cargo clippy --workspace -- -D warnings

# Ejecutar tests de un solo crate con output detallado
@test crate="":
    cargo test --package {{crate}} --lib -- --nocapture

# Gestionar contenedores de Docker para tests
@container-start:
    ./scripts/test-containers.sh start-postgres

@container-stop name:
    ./scripts/test-containers.sh stop {{name}}

@container-list:
    ./scripts/test-containers.sh list

@container-cleanup:
    ./scripts/test-containers.sh cleanup

@container-cleanup-force:
    @echo "🧹 LIMPIEZA FORZADA DE CONTAINERS"
    docker ps --filter "name=hodei-test" -aq | xargs -r docker rm -f || true
    docker ps --filter "name=postgres" -aq | xargs -r docker rm -f || true
    @echo "✅ Containers limpiados"

# Watch mode: ejecutar tests cuando cambien archivos
@watch:
    # Requiere cargo-watch: cargo install cargo-watch
    cargo watch -x "test --workspace --lib"

# Ayuda: mostrar todos los comandos disponibles
@help:
    @echo "Hodei-Scan Test Runner"
    @echo ""
    @echo "Comandos disponibles:"
    @echo "  quick      - Tests rápidos (sin infraestructura)"
    @echo "  full       - Todos los tests (incluyendo ignorados)"
    @echo "  db         - Solo tests de base de datos PostgreSQL"
    @echo "  db-container - Tests DB con contenedor Docker automático"
    @echo "  tdd        - Solo tests TDD Red (implementación pendiente)"
    @echo "  integration - Tests de integración"
    @echo "  crate      - Tests de un crate específico (ej: just crate hodei-extractors)"
    @echo "  coverage   - Tests con información de coverage"
    @echo "  clean      - Limpiar archivos temporales"
    @echo "  check      - Verificar que el workspace compila"
    @echo "  fmt        - Formatear código"
    @echo "  lint       - Verificar linting"
    @echo "  watch      - Modo watch (requiere cargo-watch)"
    @echo ""
    @echo "Gestión de Contenedores Docker:"
    @echo "  container-start  - Iniciar contenedor PostgreSQL para tests"
    @echo "  container-stop   - Detener contenedor (requiere nombre)"
    @echo "  container-list   - Listar contenedores activos"
    @echo "  container-cleanup - Limpiar todos los contenedores de test"
    @echo ""
    @echo "Ejemplos:"
    @echo "  just quick                    # Tests rápidos"
    @echo "  just db                       # Solo tests DB (requiere PostgreSQL)"
    @echo "  just db-container             # Tests DB con contenedor automático"
    @echo "  just container-start          # Iniciar contenedor PostgreSQL"
    @echo "  just crate hodei-extractors   # Tests de un crate"
    @echo "  just full                     # Todos los tests"
    @echo "  just help                     # Mostrar esta ayuda"
