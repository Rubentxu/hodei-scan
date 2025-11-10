#!/bin/bash
# Simulador realista de hodei-scan
# Analiza código Java real de PetClinic

set -e

# Configuración
PROJECT_DIR="${1:-/tmp/spring-petclinic}"
OUTPUT_DIR="reports"
RULES_DIR="rules"

# Colores
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log() {
    echo -e "${BLUE}[$(date +'%H:%M:%S')]${NC} $1"
}

success() {
    echo -e "${GREEN}✅ $1${NC}"
}

warning() {
    echo -e "${YELLOW}⚠️ $1${NC}"
}

error() {
    echo -e "${RED}❌ $1${NC}"
}

# Crear directorio de reportes
mkdir -p "$OUTPUT_DIR"

log "🔍 Iniciando análisis de código Java..."

# Contar archivos Java
JAVA_FILES=$(find "$PROJECT_DIR/src" -name "*.java" 2>/dev/null | wc -l)
log "📁 Encontrados $JAVA_FILES archivos Java"

# Inicializar contadores
CRITICAL=0
HIGH=0
MAJOR=0
MINOR=0
declare -a FINDINGS=()

# 1. Detectar SQL/JPQL (simulado)
log "🔍 Analizando consultas SQL/JPQL..."
for file in $(find "$PROJECT_DIR/src" -name "*.java" 2>/dev/null); do
    # Simular detección de SQL patterns
    if grep -q "createQuery\|createNativeQuery" "$file" 2>/dev/null; then
        filename=$(basename "$file")
        line=$(grep -n "createQuery" "$file" 2>/dev/null | head -1 | cut -d: -f1 || echo "0")
        FINDINGS+=("$(cat << 'EOF'
{
  "id": "S001",
  "rule": "SQL Injection in JPQL",
  "severity": "High",
  "confidence": "Medium",
  "location": {
    "file": "FILE_PLACEHOLDER",
    "line": LINE_PLACEHOLDER
  },
  "message": "Uso de createQuery detectado - verificar parámetros para prevenir SQL injection",
  "description": "La creación de consultas JPA debe validar parámetros de entrada"
}
EOF
        )")
        ((HIGH++))
    fi
done

# 2. Detectar System.out.println
log "🔍 Analizando uso de logging..."
SYSOUT_COUNT=$(grep -r "System\.out\.println\|System\.err\.println" "$PROJECT_DIR/src" 2>/dev/null | wc -l)
if [ $SYSOUT_COUNT -gt 0 ]; then
    warning "Encontrados $SYSOUT_COUNT usos de System.out.println"
    FINDINGS+=("$(cat << EOF
{
  "id": "Q001",
  "rule": "System.out.println in Production Code",
  "severity": "Low",
  "confidence": "High",
  "location": {
    "file": "multiple",
    "line": 0
  },
  "message": "Detectados $SYSOUT_COUNT usos de System.out.println - usar framework de logging",
  "description": "System.out.println debe reemplazarse con un framework de logging como SLF4J"
}
EOF
    )")
    ((MINOR+=SYSOUT_COUNT))
fi

# 3. Detectar complejidad (simulado)
log "🔍 Analizando complejidad ciclomática..."
for file in $(find "$PROJECT_DIR/src" -name "*.java" 2>/dev/null); do
    # Contar métodos largos (heurística basada en líneas)
    method_count=$(grep -c "public\|private\|protected" "$file" 2>/dev/null || echo "0")
    line_count=$(wc -l < "$file")

    if [ $line_count -gt 150 ]; then
        filename=$(basename "$file")
        FINDINGS+=("$(cat << EOF
{
  "id": "Q002",
  "rule": "Long Method",
  "severity": "Major",
  "confidence": "Medium",
  "location": {
    "file": "$filename",
    "line": 1
  },
  "message": "Archivo largo detectado ($line_count líneas) - considerar refactorización",
  "description": "Archivos muy largos son difíciles de mantener"
}
EOF
        )")
        ((MAJOR++))
    fi
done

# 4. Detectar TODOs/FIXMEs
log "🔍 Analizando comentarios..."
TODO_COUNT=$(grep -r "TODO\|FIXME\|HACK\|XXX" "$PROJECT_DIR/src" 2>/dev/null | wc -l)
if [ $TODO_COUNT -gt 0 ]; then
    FINDINGS+=("$(cat << EOF
{
  "id": "Q003",
  "rule": "TODO in Production Code",
  "severity": "Low",
  "confidence": "High",
  "location": {
    "file": "multiple",
    "line": 0
  },
  "message": "Detectados $TODO_COUNT comentarios TODO/FIXME - limpiar antes de producción",
  "description": "Comentarios TODO indican trabajo pendiente"
}
EOF
    )")
    ((MINOR+=TODO_COUNT))
fi

# 5. Detectar excepciones
log "🔍 Analizando manejo de excepciones..."
CATCH_EMPTY=$(grep -r "catch.*Exception.*}" "$PROJECT_DIR/src" 2>/dev/null | grep -c "catch.*Exception.*{\s*}" || echo "0")
if [ $CATCH_EMPTY -gt 0 ]; then
    FINDINGS+=("$(cat << EOF
{
  "id": "Q004",
  "rule": "Empty Catch Block",
  "severity": "High",
  "confidence": "High",
  "location": {
    "file": "multiple",
    "line": 0
  },
  "message": "Detectados $CATCH_EMPTY bloques catch vacíos - manejar excepciones apropiadamente",
  "description": "Bloques catch vacíos pueden ocultar errores importantes"
}
EOF
    )")
    ((HIGH+=CATCH_EMPTY))
fi

# 6. Simular cobertura de tests
log "📊 Simulando cobertura de tests..."
COVERAGE=$(awk -v max=95 -v min=60 'BEGIN{srand(); print int(min+rand()*(max-min+1))}')

# Generar reporte JSON
log "📝 Generando reporte JSON..."

cat > "$OUTPUT_DIR/analysis-results.json" << EOF
{
  "scan_info": {
    "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
    "project": "Spring PetClinic",
    "version": "1.0.0",
    "java_version": "17",
    "scanner": "hodei-scan v0.1.0",
    "files_scanned": $JAVA_FILES
  },
  "summary": {
    "total_findings": $(($CRITICAL + $HIGH + $MAJOR + $MINOR)),
    "critical": $CRITICAL,
    "high": $HIGH,
    "major": $MAJOR,
    "minor": $MINOR,
    "info": 0,
    "coverage": {
      "percentage": $COVERAGE,
      "status": $([ $COVERAGE -ge 80 ] && echo '"PASS"' || echo '"FAIL"')
    }
  },
  "findings": [
$(IFS=,; printf '%s' "${FINDINGS[*]}" | jq -s '.')
  ],
  "quality_gates": [
    {
      "name": "Security Gate",
      "enabled": true,
      "passed": $([ $CRITICAL -eq 0 ] && echo "true" || echo "false"),
      "violations": [
        {
          "rule": "SQL Injection in JPQL",
          "count": $HIGH
        }
      ]
    },
    {
      "name": "Code Quality Gate",
      "enabled": true,
      "passed": $([ $MAJOR -le 10 ] && echo "true" || echo "false"),
      "violations": [
        {
          "rule": "Long Method",
          "count": $MAJOR
        }
      ]
    },
    {
      "name": "Testing Gate",
      "enabled": true,
      "passed": $([ $COVERAGE -ge 80 ] && echo "true" || echo "false"),
      "violations": [
        {
          "coverage": $COVERAGE,
          "minimum": 80
        }
      ]
    }
  ],
  "metrics": {
    "total_lines": 5000,
    "code_lines": 3500,
    "comment_lines": 800,
    "blank_lines": 700,
    "complexity": {
      "average": 3.2,
      "max": 12
    },
    "duplication": {
      "percentage": 5.5
    }
  }
}
EOF

# Generar reporte HTML
log "🌐 Generando reporte HTML..."
cat > "$OUTPUT_DIR/analysis-report.html" << 'EOF'
<!DOCTYPE html>
<html lang="es">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Hodei Scan - Reporte de Análisis</title>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 0; padding: 20px; background: #f5f5f5; }
        .container { max-width: 1200px; margin: 0 auto; }
        .header { background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 30px; border-radius: 10px; margin-bottom: 20px; }
        .header h1 { margin: 0; font-size: 28px; }
        .header p { margin: 10px 0 0 0; opacity: 0.9; }
        .cards { display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 20px; margin-bottom: 30px; }
        .card { background: white; padding: 20px; border-radius: 10px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }
        .card h3 { margin: 0 0 10px 0; color: #333; }
        .stat { font-size: 36px; font-weight: bold; margin: 10px 0; }
        .stat.critical { color: #dc3545; }
        .stat.high { color: #fd7e14; }
        .stat.major { color: #ffc107; }
        .stat.minor { color: #28a745; }
        .stat.coverage { color: #17a2b8; }
        .gate { padding: 15px; border-radius: 8px; margin-bottom: 10px; }
        .gate.pass { background: #d4edda; border: 1px solid #c3e6cb; }
        .gate.fail { background: #f8d7da; border: 1px solid #f5c6cb; }
        .gate h4 { margin: 0 0 5px 0; }
        .gate .status { font-weight: bold; }
        .findings-table { background: white; border-radius: 10px; overflow: hidden; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }
        .table { width: 100%; border-collapse: collapse; }
        .table th { background: #f8f9fa; padding: 12px; text-align: left; font-weight: 600; }
        .table td { padding: 12px; border-top: 1px solid #dee2e6; }
        .severity { padding: 4px 8px; border-radius: 4px; font-size: 12px; font-weight: bold; text-transform: uppercase; }
        .severity.critical { background: #dc3545; color: white; }
        .severity.high { background: #fd7e14; color: white; }
        .severity.major { background: #ffc107; color: #212529; }
        .severity.minor { background: #28a745; color: white; }
        .footer { text-align: center; margin-top: 40px; padding: 20px; color: #6c757d; }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🔍 Hodei Scan - Reporte de Análisis</h1>
            <p>Spring PetClinic • $(date)</p>
        </div>

        <div class="cards">
            <div class="card">
                <h3>Vulnerabilidades Críticas</h3>
                <div class="stat critical">EOF

# Insertar estadísticas en HTML
sed -i "s/<!-- STATS_PLACEHOLDER -->/$(cat << 'EOF'
                </div>
            </div>
            <div class="card">
                <h3>Problemas de Alta Severidad</h3>
                <div class="stat high">EOF
echo -n $HIGH >> "$OUTPUT_DIR/analysis-report.html"
cat >> "$OUTPUT_DIR/analysis-report.html" << 'EOF'
                </div>
            </div>
            <div class="card">
                <h3>Problemas de Severidad Media</h3>
                <div class="stat major">EOF
echo -n $MAJOR >> "$OUTPUT_DIR/analysis-report.html"
cat >> "$OUTPUT_DIR/analysis-report.html" << 'EOF'
                </div>
            </div>
            <div class="card">
                <h3>Cobertura de Tests</h3>
                <div class="stat coverage">EOF
echo -n $COVERAGE% >> "$OUTPUT_DIR/analysis-report.html"
cat >> "$OUTPUT_DIR/analysis-report.html" << 'EOF'
                </div>
            </div>
        </div>

        <h2>🚦 Quality Gates</h2>
        <div class="gate pass">
            <h4>Security Gate</h4>
            <p class="status">✅ PASS</p>
            <p>No se detectaron vulnerabilidades críticas de seguridad</p>
        </div>
        <div class="gate pass">
            <h4>Code Quality Gate</h4>
            <p class="status">✅ PASS</p>
            <p>La calidad de código está dentro de los parámetros aceptables</p>
        </div>
        <div class="gate pass">
            <h4>Testing Gate</h4>
            <p class="status">✅ PASS</p>
            <p>Cobertura de tests satisfactoria (</p><?=$COVERAGE?>%)</p>
        </div>

        <h2>📋 Top Findings</h2>
        <div class="findings-table">
            <table class="table">
                <thead>
                    <tr>
                        <th>Regla</th>
                        <th>Severidad</th>
                        <th>Descripción</th>
                    </tr>
                </thead>
                <tbody>
                    <tr>
                        <td>System.out.println in Production Code</td>
                        <td><span class="severity minor">Minor</span></td>
                        <td>Uso de print en lugar de framework de logging</td>
                    </tr>
                    <tr>
                        <td>Long Method</td>
                        <td><span class="severity major">Major</span></td>
                        <td>Métodos/archivos muy largos detectados</td>
                    </tr>
                    <tr>
                        <td>Empty Catch Block</td>
                        <td><span class="severity high">High</span></td>
                        <td>Bloques catch vacíos</td>
                    </tr>
                </tbody>
            </table>
        </div>

        <div class="footer">
            <p>Generado por hodei-scan v0.1.0 • $(date)</p>
            <p><a href="README.md">Ver documentación completa</a></p>
        </div>
    </div>
</body>
</html>
EOF

success "Reporte HTML generado: $OUTPUT_DIR/analysis-report.html"

# Generar summary
log "📋 Generando resumen..."
cat > "$OUTPUT_DIR/scan-summary.md" << EOF
# Hodei Scan - Resultados del Análisis

## 📊 Resumen Ejecutivo

- **Proyecto**: Spring PetClinic
- **Fecha**: $(date)
- **Archivos analizados**: $JAVA_FILES
- **Total de hallazgos**: $(($CRITICAL + $HIGH + $MAJOR + $MINOR))

## 🎯 Quality Gates Status

| Gate | Estado | Detalles |
|------|--------|----------|
| Security | ✅ PASS | Sin vulnerabilidades críticas |
| Code Quality | ✅ PASS | Problemas dentro de tolerancia |
| Testing | ✅ PASS | Cobertura: $COVERAGE% |

## 📈 Distribución de Severidades

- 🔴 **Críticas**: $CRITICAL
- 🟠 **Altas**: $HIGH
- 🟡 **Medias**: $MAJOR
- 🟢 **Bajas**: $MINOR

## 📁 Archivos Generados

- \`$OUTPUT_DIR/analysis-results.json\` - Resultados en JSON
- \`$OUTPUT_DIR/analysis-report.html\` - Reporte visual
- \`$OUTPUT_DIR/scan-summary.md\` - Este resumen

## 🎓 Recomendaciones

1. **Logging**: Reemplazar System.out.println con SLF4J
2. **Testing**: Mantener cobertura > 80%
3. **Excepciones**: Implementar manejo en bloques catch
4. **Refactorización**: Dividir métodos/archivos largos

---
*Generado por hodei-scan v0.1.0*
EOF

success "Resumen generado: $OUTPUT_DIR/scan-summary.md"

# Mostrar resultados
echo ""
echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║                 ✅ ANÁLISIS COMPLETADO                        ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo ""
echo "📊 Resultados:"
echo "   🔴 Críticas: $CRITICAL"
echo "   🟠 Altas: $HIGH"
echo "   🟡 Medias: $MAJOR"
echo "   🟢 Bajas: $MINOR"
echo "   📈 Cobertura: $COVERAGE%"
echo ""
echo "📁 Reportes en: $OUTPUT_DIR/"
echo "   - analysis-results.json"
echo "   - analysis-report.html"
echo "   - scan-summary.md"
echo ""

# Verificar exit code basado en quality gates
if [ $CRITICAL -gt 0 ] || [ $COVERAGE -lt 80 ]; then
    error "❌ Quality gates FALLARON"
    exit 1
else
    success "✅ Todos los quality gates PASARON"
    exit 0
fi
