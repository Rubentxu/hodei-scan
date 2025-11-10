# 🚀 QUICK START - Hodei Scan en 5 Minutos

## ⚡ Inicio Rápido

### Opción 1: Con Make (Más Fácil)

```bash
# 1. Configurar entorno
make setup

# 2. Ejecutar escaneo completo
make scan

# 3. Ver resultados
make show-report
```

### Opción 2: Con Script

```bash
# 1. Hacer ejecutable
chmod +x scan-petclinic.sh

# 2. Ejecutar
./scan-petclinic.sh /tmp/spring-petclinic
```

### Opción 3: Con Docker

```bash
# 1. Construir imagen
make build-docker

# 2. Ejecutar escaneo
make run-docker
```

## 📁 Estructura Creada

```
petclinic-scan/
├── README.md                    # 📚 Documentación completa
├── QUICKSTART.md               # ⚡ Esta guía
├── Makefile                    # 🔨 Comandos fáciles
├── scan-petclinic.sh           # 📜 Script de escaneo
├── Dockerfile                  # 🐳 Contenedor
├── docker-entrypoint.sh        # 🚀 Entry point
├── rules/                      # 📋 Reglas
│   ├── security.rules          # 🔒 Seguridad
│   ├── quality.rules           # 📊 Calidad
│   └── testing.rules           # 🧪 Testing
├── config/                     # ⚙️ Configuración
│   └── quality-gates.yml       # 🚦 Quality Gates
└── .github/workflows/          # 🤖 CI/CD
    └── hodei-scan.yml          # Pipeline automático
```

## 🎯 Comandos Útiles

```bash
make help                      # Ver todos los comandos
make scan-security             # Solo seguridad
make scan-quality              # Solo calidad
make scan-testing              # Solo testing
make test                      # Ejecutar tests de PetClinic
make validate-gates            # Validar configuración
make clean                     # Limpiar
make ci-test                   # Simular CI
```

## 🔍 Lo que Hace Cada Comando

### 1. make scan
- ✅ Escanea todo el código Java
- ✅ Aplica rules de seguridad, calidad y testing
- ✅ Valida quality gates
- ✅ Genera reportes JSON, HTML y SARIF
- ❌ Falla si quality gates no pasan

### 2. make scan-security
- ✅ Solo escaneo de vulnerabilidades
- ✅ Detecta SQL Injection
- ✅ Detecta credenciales hardcodeadas
- ✅ Detecta random inseguro

### 3. make test
- ✅ Compila PetClinic
- ✅ Ejecuta tests unitarios
- ✅ Genera reporte de cobertura

### 4. make run-docker
- ✅ Escaneo en contenedor aislado
- ✅ No requiere instalación local
- ✅ Mismo resultado que escaneo local

## 📊 Interpretar Resultados

### Ver Resumen
```bash
cat reports/scan-summary.md
```

### Ver Reporte HTML
```bash
open reports/full-scan.html  # macOS
xdg-open reports/full-scan.html  # Linux
```

### Ver JSON
```bash
jq '.' reports/full-scan.json
```

### Ver Solo Críticos
```bash
jq '.findings[] | select(.severity == "Critical")' reports/full-scan.json
```

## 🚦 Quality Gates

### Security Gate
- ❌ **BLOCKS** si hay vulnerabilidades críticas
- ❌ **BLOCKS** si hay >3 vulnerabilidades altas

### Code Quality Gate
- ❌ **BLOCKS** si hay >5 problemas de alta severidad
- ❌ **BLOCKS** si hay >10 problemas de severidad media

### Testing Gate
- ❌ **BLOCKS** si cobertura < 80%
- ❌ **BLOCKS** si hay clases críticas sin tests

## 🐙 GitHub Actions

### Activación Automática
```yaml
# Se ejecuta en:
on:
  push:          # Cada push
  pull_request:  # Cada PR
  schedule:      # Diariamente a las 2 AM
  workflow_dispatch:  # Manual
```

### Resultado
- ✅ **Pass**: Quality gates pasaron → Merge allowed
- ❌ **Fail**: Quality gates fallaron → Merge blocked

## 🛠️ Personalizar

### Agregar Nueva Regla
```bash
# Editar rules/security.rules
vim rules/security.rules

# Agregar regla:
rule "Mi Regla" {
    description: "Descripción"
    severity: "High"
    match { ... }
    emit Finding { ... }
}
```

### Cambiar Quality Gate
```bash
# Editar config/quality-gates.yml
vim config/quality-gates.yml

# Modificar fail_conditions
fail_conditions:
  - severity: "High"
    count: 1  # Más estricto
```

### Usar en Tu Proyecto
```bash
# 1. Copiar a tu repositorio
cp -r petclinic-scan/* /tu/proyecto/

# 2. Modificar reglas
vim rules/*.rules

# 3. Ejecutar
make scan

# 4. Configurar GitHub Actions
make github-setup
```

## 📈 Ejemplo de Output

```
🔍 Paso 1/5: Ejecutando escaneo de SEGURIDAD...
✅ Escaneo de seguridad completado

📊 Paso 2/5: Ejecutando escaneo de CALIDAD...
✅ Escaneo de calidad completado

🧪 Paso 3/5: Ejecutando escaneo de TESTING...
✅ Escaneo de testing completado

🚦 Paso 4/5: Ejecutando escaneo con QUALITY GATES...
✅ Quality gates PASSED ✅

📋 Paso 5/5: Generando reporte de resumen...
✅ Resumen generado: reports/scan-summary.md

╔═══════════════════════════════════════════════════════════════╗
║                    RESUMEN FINAL                              ║
╚═══════════════════════════════════════════════════════════════╝

📊 Estadísticas:
   - Vulnerabilidades Críticas: 0
   - Vulnerabilidades Altas: 2
   - Vulnerabilidades Medias: 8

✅ TODOS LOS QUALITY GATES PASARON
```

## 🎓 Casos de Uso Reales

### 1. Pre-Release Check
```bash
# Antes de hacer release
make ci-test  # Debe pasar
make scan     # Debe pasar
```

### 2. CI/CD Integration
```yaml
# En tu pipeline
- name: Hodei Scan
  run: |
    make setup
    make scan
```

### 3. Docker Scan
```bash
# Sin instalar nada
make build-docker
make run-docker
```

### 4. Security Audit
```bash
# Solo seguridad
make scan-security
```

## ❓ FAQ

**P: ¿Cómo escanear mi proyecto?**
R: `make scan PROJECT_DIR=/ruta/a/mi/proyecto`

**P: ¿Cómo hacer más estricto?**
R: Edita `config/quality-gates.yml` y reduce los `count` limits

**P: ¿Cómo desactivar un gate?**
R: Cambia `enabled: false` en `config/quality-gates.yml`

**P: ¿Cómo agregar más rules?**
R: Crea archivo `.rules` en `rules/` o edita existente

**P: ¿Cómo integrar con SonarQube?**
R: Usa output SARIF: `--format sarif`

## 🎉 ¡Listo!

```bash
# Comando mágico
make setup && make scan && make show-report
```

**¡En 3 comandos tienes un escaneo completo con reportes!**

---
**Documentación completa**: README.md
**Comandos**: make help
**Issues**: /r/hodei-scan
