# ✅ GUÍA: Usar hodei-scan en Tu Proyecto Real

## 🚀 Pasos para Implementar

### 1. Copiar Configuración
```bash
# Copiar la estructura a tu proyecto
cp -r examples/petclinic-scan/* /tu/proyecto/

# Adaptar reglas
vim rules/security.rules
vim rules/quality.rules
vim rules/testing.rules

# Configurar quality gates
vim config/quality-gates.yml
```

### 2. Ejecutar Análisis
```bash
# Opción A: Con el analizador simple
./simple-analyze.sh /ruta/a/tu/proyecto

# Opción B: Con hodei-scan real
hodei-scan scan \
  --rules rules/*.rules \
  --config config/quality-gates.yml \
  --output reports/scan.json \
  --format json \
  --format html \
  --fail-on-quality-gate \
  /tu/proyecto/src/
```

### 3. Configurar GitHub Actions
```bash
# El pipeline está listo en .github/workflows/hodei-scan.yml
# Solo haz commit y push:

git add .
git commit -m "feat: add hodei-scan quality gates"
git push origin main

# GitHub ejecutará automáticamente el análisis
```

### 4. Ver Resultados
```bash
# Ver resumen
cat reports/petclinic-summary.md

# Ver reporte visual
open reports/petclinic-analysis.html  # macOS
xdg-open reports/petclinic-analysis.html  # Linux

# Ver JSON
jq '.' reports/petclinic-analysis-results.json
```

## 📋 Personalizar Rules

### Ejemplo: Agregar Regla SQL Injection
```python
# rules/security.rules
rule "SQL Injection" {
    description: "Detecta concatenación en SQL"
    severity: "Critical"
    tags: ["security", "sql"]
    
    match {
        func: Function {
            source_code ~= "String\s+\w+\s*=\s*.*\+\s*.*"
        }
    }
    
    emit Finding {
        message: "SQL injection risk in {func.name}"
        confidence: "High"
    }
}
```

### Ejemplo: Regla Complejidad
```python
# rules/quality.rules
rule "High Cyclomatic Complexity" {
    description: "Complejidad > 15"
    severity: "High"
    
    match {
        func: Function {
            complexity > 15
        }
    }
    
    emit Finding {
        message: "Complejidad alta: {func.complexity} in {func.name}"
    }
}
```

## 🚦 Modificar Quality Gates

### Hacer Más Estricto
```yaml
# config/quality-gates.yml
quality_gates:
  - name: "Security Gate"
    fail_conditions:
      - severity: "Critical"
        count: 0    # No permite ninguna
      - severity: "High"
        count: 1    # Solo 1 permitido (era 3)
```

### Hacer Menos Estricto
```yaml
quality_gates:
  - name: "Code Quality Gate"
    fail_conditions:
      - severity: "High"
        count: 10   # Permite hasta 10 (era 5)
      - severity: "Major"
        count: 20   # Permite hasta 20 (era 10)
```

## 🐳 Usar con Docker
```bash
# Construir imagen
docker build -t hodei-scan:latest .

# Escanear tu proyecto
docker run --rm \
  -v /tu/proyecto:/app/src \
  -v $(pwd)/reports:/app/reports \
  hodei-scan:latest scan

# Solo seguridad
docker run --rm \
  -v /tu/proyecto:/app/src \
  hodei-scan:latest scan-security
```

## 🔄 Integrar en CI/CD

### GitHub Actions (ya incluido)
```yaml
# .github/workflows/hodei-scan.yml
- name: Run hodei-scan
  run: |
    ./simple-analyze.sh src/
    
- name: Check Quality Gates
  run: |
    if [ $? -ne 0 ]; then
      echo "❌ Quality gates failed"
      exit 1
    fi
```

### Jenkins
```groovy
pipeline {
    stages {
        stage('Hodei Scan') {
            steps {
                sh './simple-analyze.sh src/'
            }
            post {
                always {
                    archiveArtifacts artifacts: 'reports/**', allowEmptyArchive: true
                }
            }
        }
    }
}
```

### GitLab CI
```yaml
hodei-scan:
  stage: quality
  script:
    - ./simple-analyze.sh src/
  artifacts:
    reports:
      when: always
      paths:
        - reports/
```

## 📊 Interpretar Resultados

### Códigos de Salida
- `0` = Quality gates pasaron ✅
- `1` = Quality gates fallaron ❌

### Severidades
- **Critical**: Bloquear siempre
- **High**: Revisar cuidadosamente
- **Major**: Mejorar en refactoring
- **Minor**: Sugerencias
- **Info**: Informativo

## ❓ FAQ

**P: ¿Cómo agregar más archivos a escanear?**
R: Modifica la línea en simple-analyze.sh:
```bash
find "$PROJECT_DIR/src" -name "*.java"
```

**P: ¿Cómo cambiar el directorio de salida?**
R: Exporta la variable:
```bash
export OUTPUT_DIR="mi-reporte"
./simple-analyze.sh
```

**P: ¿Cómo escanear solo archivos específicos?**
R: Modifica el patrón:
```bash
find "$PROJECT_DIR/src" -name "Controller.java"
```

**P: ¿Cómo integrar con SonarQube?**
R: Usa el output SARIF (cuando esté implementado):
```bash
hodei-scan scan --format sarif
```

## 🎉 ¡Listo!

```bash
# Comando mágico
cp -r examples/petclinic-scan/* /mi/proyecto/
cd /mi/proyecto
./simple-analyze.sh

# ¡Y listo! Quality gates configurados
```

---
**📚 Documentación completa**: README.md  
**⚡ Quick Start**: QUICKSTART.md  
**🐙 GitHub**: [hodei-scan/hodei-scan](https://github.com/hodei-scan)
