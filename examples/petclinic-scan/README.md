# Hodei Scan - Ejemplo Práctico con Spring PetClinic

## 🎯 Objetivo

Este ejemplo demuestra cómo usar **hodei-scan** para escanear una aplicación Java/Spring real, crear rules personalizadas, configurar quality gates y automatizar todo en un pipeline de CI/CD.

## 🏗️ Estructura del Proyecto

```
petclinic-scan/
├── README.md                    # Este archivo
├── scan-petclinic.sh           # Script de ejecución manual
├── rules/                      # Rules de escaneo
│   ├── security.rules          # Reglas de seguridad
│   ├── quality.rules           # Reglas de calidad
│   └── testing.rules           # Reglas de testing
├── config/                     # Configuración
│   └── quality-gates.yml       # Configuración de quality gates
└── .github/                    # GitHub Actions
    └── workflows/
        └── hodei-scan.yml      # Pipeline de CI/CD
```

## 🚀 Inicio Rápido

### 1. Clonar PetClinic
```bash
# Opción A: Usar el script directamente
./scan-petclinic.sh /tmp/spring-petclinic

# Opción B: Clonar manualmente
git clone https://github.com/spring-projects/spring-petclinic.git
cd spring-petclinic
../petclinic-scan/scan-petclinic.sh
```

### 2. Ver Resultados
```bash
# Ver resumen
cat reports/scan-summary.md

# Ver reporte HTML
open reports/full-scan.html  # macOS
xdg-open reports/full-scan.html  # Linux
```

## 📋 Reglas Incluidas

### 🔒 Security Rules (`rules/security.rules`)

| Regla | Descripción | Severidad | Tags |
|-------|-------------|-----------|------|
| SQL Injection in JPQL | Detecta concatenación de strings en consultas JPQL | High | security, sqli, jpa |
| Insecure Random Number Generation | Uso inseguro de Random/SystemRandom | Medium | security, crypto |
| Hardcoded Credentials | Credenciales hardcodeadas en el código | Critical | security, secrets |

### 📊 Quality Rules (`rules/quality.rules`)

| Regla | Descripción | Severidad | Tags |
|-------|-------------|-----------|------|
| High Cyclomatic Complexity | Complejidad > 10 | High | quality, complexity |
| Long Method | Métodos > 100 líneas | Medium | quality, maintainability |
| Too Many Parameters | Funciones con > 5 parámetros | Medium | quality, design |
| Empty Catch Block | Bloques catch vacíos | High | quality, error-handling |
| System.out.println | Uso de print en lugar de logging | Low | quality, logging |

### 🧪 Testing Rules (`rules/testing.rules`)

| Regla | Descripción | Severidad | Tags |
|-------|-------------|-----------|------|
| Low Test Coverage | Cobertura < 80% | Critical | testing, coverage |
| Missing Tests for Public Methods | Clases públicas sin tests | High | testing |
| Test Method Without Assertions | Tests sin assertions | Medium | testing |

## 🚦 Quality Gates

### Configuración (`config/quality-gates.yml`)

```yaml
quality_gates:
  - name: "Security Gate"
    rules: ["SQL Injection", "Hardcoded Credentials"]
    fail_conditions:
      - severity: "Critical"
        count: 0  # No permite ninguna vulnerabilidad crítica
      - severity: "High"
        count: 3  # Permite máximo 3 vulnerabilidades altas
```

### Gates Definidos

1. **Security Gate** 🔒
   - Bloquea vulnerabilidades críticas
   - Máximo 3 vulnerabilidades altas
   - Enforced en PRs

2. **Code Quality Gate** 📊
   - Controla complejidad y mantenibilidad
   - Máximo 5 problemas de alta severidad
   - Máximo 10 de severidad media

3. **Testing Gate** 🧪
   - Cobertura mínima: 80%
   - No permite clases críticas sin tests

4. **Best Practices Gate** ✅
   - Verifica logging y buenas prácticas
   - Actualmente deshabilitado (opcional)

## �� GitHub Actions Pipeline

### Workflow (`.github/workflows/hodei-scan.yml`)

El pipeline incluye:

1. **Setup** - Preparar entorno Java
2. **Build** - Compilar proyecto con Maven
3. **Test** - Ejecutar tests unitarios
4. **Hodei Scan** - Ejecutar escaneos
5. **Quality Gate Validation** - Validar gates
6. **Notifications** - Notificar resultados
7. **Security Tab** - Subir SARIF a GitHub

### Activación

```bash
# El pipeline se ejecuta automáticamente en:
# - Push a main/develop
# - Pull Request
# - Programado (diario a las 2 AM)
# - Manual (workflow_dispatch)
```

### Resultado en GitHub

![GitHub Check](https://img.shields.io/badge/GitHub%20Check-hodei--scan-blue)
- ✅ **Pass**: Quality gates pasaron
- ❌ **Fail**: Quality gates fallaron
- 📊 **Artifacts**: Reportes disponibles para descarga

## 🛠️ Personalización

### Crear Nueva Regla

```python
rule "Mi Regla Personalizada" {
    description: "Descripción de la regla"
    severity: "High"  # Critical, High, Major, Minor, Info
    tags: ["tag1", "tag2"]

    match {
        pattern: FactType {
            condition
        }
    }

    emit Finding {
        message: "Mensaje descriptivo"
        confidence: "High"  # High, Medium, Low
        metadata: {
            key1 = value1,
            key2 = value2
        }
    }
}
```

### Ejemplo: Regla para Detectar TODO/FIXME

```python
rule "TODO in Production Code" {
    description: "Detecta comentarios TODO/FIXME en código"
    severity: "Low"
    tags: ["maintainability", "java"]

    match {
        code: CodeSmell {
            smell_type == "TODO"
        }
    }

    emit Finding {
        message: "Comentario TODO encontrado: {code.message}"
        confidence: "Low"
    }
}
```

### Modificar Quality Gate

```yaml
# config/quality-gates.yml
quality_gates:
  - name: "Custom Gate"
    enabled: true
    rules:
      - "Mi Regla Personalizada"
    fail_conditions:
      - severity: "High"
        count: 0  # No permite ningún problema de alta severidad
      - coverage_below: 85  # Cobertura mínima 85%
```

## 📊 Interpretación de Resultados

### Archivo JSON (`full-scan.json`)

```json
{
  "summary": {
    "total_findings": 42,
    "critical": 0,
    "high": 3,
    "major": 12,
    "minor": 27,
    "coverage": {
      "percentage": 85.3,
      "total_lines": 5000,
      "covered_lines": 4265
    }
  },
  "findings": [
    {
      "id": "F001",
      "rule": "High Cyclomatic Complexity",
      "severity": "High",
      "confidence": "High",
      "location": {
        "file": "src/main/java/Example.java",
        "line": 42
      },
      "message": "Complejidad alta (12) en método processData"
    }
  ],
  "quality_gates": [
    {
      "name": "Security Gate",
      "passed": true,
      "violations": []
    }
  ]
}
```

### Archivo HTML (`full-scan.html`)

Reporte visual con:
- 📊 Dashboard de métricas
- 📋 Tabla de findings
- 🔍 Filtros por severidad/tag
- 📈 Gráficos de tendencias

### Archivo SARIF (`full-scan.sarif`)

Formato estándar para herramientas de análisis estático, compatible con:
- GitHub Security tab
- Azure DevOps
- VS Code

## 🏃‍♂️ Ejecución Manual Paso a Paso

```bash
# 1. Preparar entorno
export PROJECT_DIR=/tmp/spring-petclinic

# 2. Escaneo de seguridad
hodei-scan scan \
  --rules rules/security.rules \
  --output reports/security.json \
  src/

# 3. Escaneo de calidad
hodei-scan scan \
  --rules rules/quality.rules \
  --output reports/quality.json \
  src/

# 4. Escaneo con quality gates
hodei-scan scan \
  --rules rules/*.rules \
  --config config/quality-gates.yml \
  --output reports/full.json \
  --format json \
  --format html \
  --fail-on-quality-gate \
  src/

# 5. Verificar exit code
if [ $? -eq 0 ]; then
  echo "✅ Todos los quality gates pasaron"
else
  echo "❌ Quality gates fallaron"
fi
```

## 📈 Métricas y Monitoreo

### Dashboard Local

```bash
# Instalar dashboard (opcional)
npm install -g hodei-dashboard

# Ejecutar dashboard
hodei-dashboard --port 8080 --input reports/full.json
# Abrir: http://localhost:8080
```

### Integración con Prometheus

```yaml
# config/prometheus.yml
scrape_configs:
  - job_name: 'hodei-scan'
    static_configs:
      - targets: ['localhost:9090']
    metrics_path: /metrics
```

## ❓ FAQ

**P: ¿Cómo agregar más reglas?**
R: Edita los archivos en `rules/` o crea nuevos archivos .rules

**P: ¿Cómo cambiar la severidad de una regla?**
R: Modifica el campo `severity` en la regla

**P: ¿Cómo deshabilitar un quality gate?**
R: Cambia `enabled: false` en `config/quality-gates.yml`

**P: ¿Cómo ver solo problemas críticos?**
R: Usa `--severity-filter Critical`

**P: ¿Cómo integrar con SonarQube?**
R: Exporta resultados en formato SARIF y configura SonarQube para consumirlo

## 🎓 Casos de Uso Avanzados

### 1. Escaneo en Múltiples Ramas

```yaml
# .github/workflows/hodei-scan-branches.yml
on:
  push:
    branches: ['**']  # Todas las ramas
```

### 2. Escaneo Programado Nocturno

```yaml
schedule:
  - cron: '0 2 * * *'  # 2 AM UTC diario
```

### 3. Escaneo de Dependencias

```yaml
# rules/dependencies.rules
rule "Vulnerable Dependency" {
  match {
    dep: Dependency {
      cve_id != null
    }
  }
  emit Finding {
    message: "Dependencia vulnerable: {dep.name} {dep.version}"
  }
}
```

### 4. Escaneo de Licencias

```yaml
# rules/licenses.rules
rule "Incompatible License" {
  match {
    license: License {
      compatible == false
    }
  }
  emit Finding {
    message: "Licencia incompatible: {license.license_type}"
  }
}
```

## 🚀 Siguiente Paso

1. **Ejecuta el script**: `./scan-petclinic.sh`
2. **Revisa los reportes**: `reports/full-scan.html`
3. **Personaliza las rules**: Edita `rules/*.rules`
4. **Adapta a tu proyecto**: Copia a tu repositorio
5. **Configura CI/CD**: Usa `.github/workflows/hodei-scan.yml`

## 🤝 Contribuir

¿Tienes reglas útiles? ¡Compártelas!

```bash
# Crear fork y pull request
# O enviar reglas a: rules@hodei-scan.dev
```

---

**¡Gracias por usar hodei-scan!** 🎉

Para más información: [documentación oficial](https://hodei-scan.dev)
