#!/bin/bash
# Script de ejecución de hodei-scan
# Archivo: scan-petclinic.sh

set -e  # Salir en caso de error

# Colores para output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuración
PROJECT_DIR="${1:-/tmp/spring-petclinic}"
RULES_DIR="rules"
CONFIG_FILE="config/quality-gates.yml"
OUTPUT_DIR="reports"

# Función para logging
log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Banner
cat << 'EOF'
╔═══════════════════════════════════════════════════════════════╗
║               HODEI SCAN - PETCLINIC EXAMPLE                  ║
║                                                               ║
║  Ejemplo práctico de escaneo de seguridad y calidad          ║
║  de código para aplicaciones Java/Spring                     ║
╚═══════════════════════════════════════════════════════════════╝

EOF

# Verificar directorio del proyecto
if [ ! -d "$PROJECT_DIR/src/main/java" ]; then
    error "Directorio de proyecto inválido: $PROJECT_DIR"
    error "Por favor, especifica un directorio válido con código Java"
    exit 1
fi

log "Iniciando escaneo de hodei-scan..."
log "Directorio del proyecto: $PROJECT_DIR"

# Crear directorio de reportes
mkdir -p "$OUTPUT_DIR"

# 1. Escaneo de Seguridad
log "🔍 Paso 1/5: Ejecutando escaneo de SEGURIDAD..."
echo "----------------------------------------"
hodei-scan scan \
    --rules "$RULES_DIR/security.rules" \
    --output "$OUTPUT_DIR/security-scan.json" \
    --format json \
    --severity-filter High,Critical \
    "$PROJECT_DIR/src/" 2>&1 | tee "$OUTPUT_DIR/security-scan.log"

if [ ${PIPESTATUS[0]} -eq 0 ]; then
    success "Escaneo de seguridad completado"
else
    error "Error en escaneo de seguridad"
fi

# 2. Escaneo de Calidad
log "📊 Paso 2/5: Ejecutando escaneo de CALIDAD..."
echo "----------------------------------------"
hodei-scan scan \
    --rules "$RULES_DIR/quality.rules" \
    --output "$OUTPUT_DIR/quality-scan.json" \
    --format json \
    "$PROJECT_DIR/src/" 2>&1 | tee "$OUTPUT_DIR/quality-scan.log"

if [ ${PIPESTATUS[0]} -eq 0 ]; then
    success "Escaneo de calidad completado"
else
    error "Error en escaneo de calidad"
fi

# 3. Escaneo de Testing
log "🧪 Paso 3/5: Ejecutando escaneo de TESTING..."
echo "----------------------------------------"
hodei-scan scan \
    --rules "$RULES_DIR/testing.rules" \
    --coverage-file "$OUTPUT_DIR/coverage.xml" \
    --output "$OUTPUT_DIR/testing-scan.json" \
    --format json \
    "$PROJECT_DIR/src/" 2>&1 | tee "$OUTPUT_DIR/testing-scan.log"

if [ ${PIPESTATUS[0]} -eq 0 ]; then
    success "Escaneo de testing completado"
else
    error "Error en escaneo de testing"
fi

# 4. Escaneo completo con Quality Gates
log "🚦 Paso 4/5: Ejecutando escaneo con QUALITY GATES..."
echo "----------------------------------------"
hodei-scan scan \
    --rules "$RULES_DIR"/*.rules \
    --config "$CONFIG_FILE" \
    --output "$OUTPUT_DIR/full-scan.json" \
    --format json \
    --format html \
    --format sarif \
    --report-summary \
    --fail-on-quality-gate \
    "$PROJECT_DIR/src/" 2>&1 | tee "$OUTPUT_DIR/full-scan.log"

GATE_STATUS=$?
if [ $GATE_STATUS -eq 0 ]; then
    success "Quality gates PASSED ✅"
else
    error "Quality gates FAILED ❌"
    warning "Revisa el reporte en: $OUTPUT_DIR/full-scan.html"
fi

# 5. Generar resumen
log "📋 Paso 5/5: Generando reporte de resumen..."
echo "----------------------------------------"

cat > "$OUTPUT_DIR/scan-summary.md" << EOF
# Hodei Scan - Resumen de Resultados - PetClinic

## Información del Escaneo
- **Fecha**: $(date)
- **Proyecto**: Spring PetClinic
- **Directorio escaneado**: $PROJECT_DIR/src/
- **Tecnologías**: Java, Spring Boot, Maven

## Reglas Aplicadas

### Security Rules (Reglas de Seguridad)
- SQL Injection in JPQL
- Insecure Random Number Generation
- Hardcoded Credentials

### Quality Rules (Reglas de Calidad)
- High Cyclomatic Complexity
- Long Method
- Too Many Parameters
- Empty Catch Block
- System.out.println in Production Code

### Testing Rules (Reglas de Testing)
- Low Test Coverage
- Missing Tests for Public Methods
- Test Method Without Assertions

## Quality Gates Validados
- Security Gate: Bloquea vulnerabilidades críticas
- Code Quality Gate: Controla complejidad y mantenibilidad
- Testing Gate: Asegura cobertura mínima (80%)
- Best Practices Gate: Verifica buenas prácticas

## Archivos Generados
- \`security-scan.json\`: Vulnerabilidades de seguridad
- \`quality-scan.json\`: Problemas de calidad de código
- \`testing-scan.json\`: Problemas de testing
- \`full-scan.json\`: Reporte completo en JSON
- \`full-scan.html\`: Reporte completo en HTML
- \`full-scan.sarif\`: Para GitHub Security tab

## Próximos Pasos
1. Revisar el reporte HTML para detalles
2. Abordar vulnerabilidades críticas primero
3. Refactorizar código con problemas de calidad
4. Mejorar cobertura de tests
5. Configurar CI/CD con quality gates

---
*Generado por hodei-scan v0.1.0*
EOF

success "Resumen generado: $OUTPUT_DIR/scan-summary.md"

# Mostrar estadísticas
echo ""
echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║                    RESUMEN FINAL                              ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo ""

if [ -f "$OUTPUT_DIR/full-scan.json" ]; then
    VULNS_CRITICAL=$(jq '.summary.critical // 0' "$OUTPUT_DIR/full-scan.json")
    VULNS_HIGH=$(jq '.summary.high // 0' "$OUTPUT_DIR/full-scan.json")
    VULNS_MEDIUM=$(jq '.summary.medium // 0' "$OUTPUT_DIR/full-scan.json")

    echo "📊 Estadísticas:"
    echo "   - Vulnerabilidades Críticas: $VULNS_CRITICAL"
    echo "   - Vulnerabilidades Altas: $VULNS_HIGH"
    echo "   - Vulnerabilidades Medias: $VULNS_MEDIUM"
    echo ""
fi

if [ $GATE_STATUS -eq 0 ]; then
    success "✅ TODOS LOS QUALITY GATES PASARON"
else
    error "❌ ALGUNOS QUALITY GATES FALLARON"
fi

echo ""
echo "📁 Reportes disponibles en: $OUTPUT_DIR/"
echo "🌐 Ver reporte HTML: $OUTPUT_DIR/full-scan.html"
echo "📖 Ver resumen: $OUTPUT_DIR/scan-summary.md"
echo ""

# Abrir reporte HTML automáticamente (si está en entorno gráfico)
if command -v xdg-open &> /dev/null; then
    if [ -f "$OUTPUT_DIR/full-scan.html" ]; then
        warning "Abriendo reporte HTML..."
        xdg-open "$OUTPUT_DIR/full-scan.html" &
    fi
elif command -v open &> /dev/null; then
    if [ -f "$OUTPUT_DIR/full-scan.html" ]; then
        warning "Abriendo reporte HTML..."
        open "$OUTPUT_DIR/full-scan.html" &
    fi
fi

success "¡Escaneo completado!"
exit $GATE_STATUS
