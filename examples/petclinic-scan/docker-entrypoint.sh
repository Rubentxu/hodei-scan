#!/bin/bash
# Docker entrypoint para hodei-scan
# Permite ejecutar escaneos con configuraciones predefinidas

set -e

# Colores
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

log() {
    echo -e "${BLUE}[hodei-scan]${NC} $1"
}

success() {
    echo -e "${GREEN}[hodei-scan]${NC} $1"
}

# Función para mostrar ayuda
show_help() {
    cat << 'EOF'
Hodei Scan - Docker Entrypoint

Uso:
  docker run --rm [opciones] hodei-scan [comando]

Comandos:
  scan              Ejecutar escaneo completo
  scan-security     Solo escaneo de seguridad
  scan-quality      Solo escaneo de calidad
  scan-testing      Solo escaneo de testing
  gate-check        Verificar quality gates
  help              Mostrar esta ayuda

Ejemplos:
  # Escaneo completo
  docker run --rm -v $(pwd):/app/src hodei-scan:latest scan

  # Solo seguridad
  docker run --rm -v $(pwd):/app/src hodei-scan:latest scan-security

  # Con quality gates
  docker run --rm -v $(pwd):/app/src hodei-scan:latest gate-check

Ambiente:
  HODEI_RULES_DIR    Directorio de rules (default: /app/rules)
  HODEI_CONFIG       Archivo de config (default: /app/config/quality-gates.yml)
  HODEI_OUTPUT       Directorio de salida (default: /app/reports)

EOF
}

# Ejecutar escaneo
run_scan() {
    local scan_type="${1:-full}"

    log "Iniciando escaneo: $scan_type"

    case $scan_type in
        "security")
            hodei-scan scan \
                --rules "${HODEI_RULES_DIR:-/app/rules}/security.rules" \
                --output "/app/reports/security.json" \
                --format json \
                /app/src/
            ;;
        "quality")
            hodei-scan scan \
                --rules "${HODEI_RULES_DIR:-/app/rules}/quality.rules" \
                --output "/app/reports/quality.json" \
                --format json \
                /app/src/
            ;;
        "testing")
            hodei-scan scan \
                --rules "${HODEI_RULES_DIR:-/app/rules}/testing.rules" \
                --output "/app/reports/testing.json" \
                --format json \
                /app/src/
            ;;
        "full"|*)
            hodei-scan scan \
                --rules "${HODEI_RULES_DIR:-/app/rules}"/*.rules \
                --config "${HODEI_CONFIG:-/app/config/quality-gates.yml}" \
                --output "/app/reports/full.json" \
                --format json \
                --format html \
                --format sarif \
                --report-summary \
                --fail-on-quality-gate \
                /app/src/
            ;;
    esac
}

# Verificar quality gates
check_gates() {
    log "Verificando quality gates..."
    run_scan full
}

# Main
log "Hodei Scan Docker Container v0.1.0"

# Verificar si se proporcionan argumentos
if [ $# -eq 0 ]; then
    show_help
    exit 0
fi

# Procesar comando
case $1 in
    "scan")
        run_scan "${2:-full}"
        ;;
    "scan-security")
        run_scan "security"
        ;;
    "scan-quality")
        run_scan "quality"
        ;;
    "scan-testing")
        run_scan "testing"
        ;;
    "gate-check")
        check_gates
        ;;
    "help"|"-h"|"--help")
        show_help
        ;;
    *)
        log "Comando desconocido: $1"
        show_help
        exit 1
        ;;
esac

success "Escaneo completado"
