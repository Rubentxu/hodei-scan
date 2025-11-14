# Manual de Metodología de Pruebas E2E

## Visión General

Las pruebas End-to-End (E2E) validan que hodei-scan pueda escanear, analizar y extraer facts de **proyectos Java reales** obtenidos directamente de GitHub. A diferencia de las pruebas unitarias que usan código simulado o simplificado, las pruebas E2E trabajan con bases de código de producción reales.

---

## Qué Hacen las Pruebas E2E

### Proceso Central

Cada prueba E2E sigue este flujo de trabajo:

```
1. Clonar Repositorio Real ──┐
                            ├──> Descubrir Archivos Java
                            ├──> Ejecutar Extracción hodei-scan
                            ├──> Validar Salida
                            └──> Verificar Facts Extraídos
```

### Ejecución Paso a Paso

#### Paso 1: Adquisición del Repositorio
```bash
git clone --depth 1 --shallow-submodules <repo_url> <temp_dir>
```

- **Depth 1**: Clon superficial para acelerar descarga (solo último commit)
- **Shallow submodules**: Inicialización más rápida de submódulos
- **Red Requerida**: Las pruebas requieren acceso a internet

#### Paso 2: Descubrimiento de Archivos
```rust
fn discover_java_files(project_path: &Path) -> Vec<PathBuf> {
    walk_dir(project_path)
        .filter(|p| p.extension() == Some("java".as_ref()))
        .collect()
}
```

**Qué hace:**
- Escanea recursivamente todos los directorios del proyecto clonado
- Recopila rutas absolutas de todos los archivos `.java`
- Filtra archivos no Java (ej. `.kt`, `.scala`)

#### Paso 3: Extraer Facts
```rust
fn validate_scan_capability(project_path: &Path, java_files: &[PathBuf]) {
    let ir = hodei_java_extractor::extract(
        project_path,
        &ExtractionConfig::default()
    ).expect("Extracción fallida");

    assert!(!ir.get_facts().is_empty(), "No se extrajeron facts");
    assert!(ir.get_facts().len() > java_files.len(), "Muy pocos facts");
}
```

**Qué hace:**
- Invoca hodei-java-extractor en el proyecto
- Parsea todos los archivos Java descubiertos
- Extrae facts (Function, Variable, TaintSink, etc.)
- Genera Representación Intermedia (IR)

#### Paso 4: Validación
```rust
assert!(extraction_successful);
assert!(facts_extracted > 0);
assert!(facts_per_file_ratio > MIN_EXPECTED_RATIO);
```

**Validaciones realizadas:**
- Extracción completada sin errores
- Se extrajeron facts realmente
- Ratio de facts a archivos es razonable
- Estructura IR es válida

---

## Pruebas E2E Específicas Explicadas

### 1. test_simple_java_library_extraction (Google Guava)

**Repositorio:** https://github.com/google/guava.git

**Qué es Guava:**
- Librerías centrales de Google para Java
- Utilidades de colecciones, caché, primitivas de concurrencia
- Código Java bien estructurado y limpio
- ~1,000+ archivos Java

**Qué prueba:**
```
✅ Parsing básico de Java (clases, interfaces, enums)
✅ Extracción de métodos y firmas
✅ Detección de campos y variables
✅ Parsing de declaraciones import
✅ Parámetros de tipos genéricos
✅ Procesamiento de anotaciones (si están presentes)
```

**Salida Esperada:**
- 800-1,500 facts extraídos
- 1-2 minutos tiempo de ejecución
- Tasa de éxito: 95%+ archivos procesados

**Patrones Objetivo:**
- Clases utilitarias (final, constructores privados)
- Uso de métodos estáticos
- Colecciones genéricas
- Patrones builder
- Precondiciones y validación

**Criterios de Éxito:**
```
Descubrimiento de Archivos: 800+ archivos .java
Facts Extraídos: 1,000+ facts
Tiempo de Extracción: < 120 segundos
Tasa de Errores: < 5%
```

---

### 2. test_spring_boot_application (Spring Boot)

**Repositorio:** https://github.com/spring-projects/spring-boot.git

**Qué es Spring Boot:**
- Framework para construir aplicaciones web Java
- Código de calidad empresarial, listo para producción
- 500+ archivos Java a través de múltiples módulos
- Uso extensivo de anotaciones y AOP

**Qué prueba:**
```
✅ Procesamiento complejo de anotaciones (@RestController, @Autowired, @RequestMapping)
✅ Configuración de inyección de dependencias
✅ Escaneo de classpath y descubrimiento de beans
✅ Propiedades de configuración (@ConfigurationProperties)
✅ Programación orientada a aspectos (AOP)
✅ Estructura de proyecto multi-módulo
✅ Patrones empresariales (MVC, Data Access, Security)
```

**Qué lo hace desafiante:**
- **Anotaciones**: Metadatos complejos que necesitan parsing
- **Generics**: Uso pesado de tipos parametrizados
- **Dependencias Circulares**: Spring maneja grafos de dependencia complejos
- **Proxies**: Spring crea objetos proxy que obscurecen el código real
- **Reflection**: Procesamiento de anotaciones en tiempo de ejecución

**Patrones Objetivo:**
```java
@RestController
@RequestMapping("/api/users")
public class UserController {
    @Autowired
    private UserService userService;

    @GetMapping("/{id}")
    public User getUser(@PathVariable Long id) {
        return userService.findById(id);
    }
}

@SpringBootApplication
@EnableJpaRepositories
@EntityScan("com.example.model")
public class Application {
    public static void main(String[] args) {
        SpringApplication.run(Application.class, args);
    }
}
```

**Salida Esperada:**
- 2,000-5,000 facts extraídos
- 2-3 minutos tiempo de ejecución
- Tasa de éxito: 90%+ archivos procesados

**Criterios de Éxito:**
```
Descubrimiento de Archivos: 2,000+ archivos .java
Facts Extraídos: 3,000+ facts
Facts de Anotaciones: 500+ instancias de anotaciones
Definiciones de Beans: 100+ beans Spring
Endpoints Web: 50+ endpoints REST
Tiempo de Extracción: < 180 segundos
Tasa de Errores: < 10%
```

**Qué validamos:**
- Clases Controller marcadas con `@RestController`
- Dependencias Autowired reconocidas apropiadamente
- Request mappings extraídos
- Abstracciones de Service layer identificadas
- Clases de configuración detectadas
- Dependencias multi-módulo comprendidas

---

### 3. test_multimodule_maven_project (Apache Camel)

**Repositorio:** https://github.com/apache/camel.git

**Qué es Apache Camel:**
- Framework de Enterprise Integration Patterns
- Middleware orientado a mensajes
- 5,000+ archivos Java a través de 20+ módulos
- Uso pesado de DSLs (Domain-Specific Languages)

**Qué prueba:**
```
✅ Navegación de proyecto multi-módulo
✅ Reconocimiento de patrones DSL (RouteBuilder, Endpoint DSL)
✅ Enterprise Integration Patterns (EIP)
✅ Flujos de transformación de mensajes
✅ Adaptadores de protocolo (HTTP, JMS, Kafka, etc.)
✅ Integración de beans y mensajería POJO
✅ Gestión compleja de dependencias entre módulos
```

**Qué lo hace desafiante:**
- **Multi-Módulo**: Necesita navegar relaciones padre/hijo
- **Patrones DSL**: Lenguajes personalizados embebidos en Java
- **Dependencias Pesadas**: Classpath complejo con cientos de JARs
- **Gran Escala**: 5,000+ archivos = prueba de memoria y rendimiento
- **Patrones EIP**: Integración empresarial requiere comprensión semántica

**Patrones Objetivo:**
```java
public class MyRouteBuilder extends RouteBuilder {
    @Override
    public void configure() throws Exception {
        from("file:data/inbox")
            .choice()
                .when().xpath("/order/@type='widget'")
                    .to("jms:widgetQueue")
                .otherwise()
                    .to("jms:gadgetQueue");

        from("direct:start")
            .to("rest:get:/api/users/{id}")
            .split().xpath("//orders/order")
            .choice()
                .when().method(MyBean.class, "isLargeOrder")
                    .to("mock:largeOrders")
                .otherwise()
                    .to("mock:otherOrders");
    }
}
```

**Salida Esperada:**
- 10,000-20,000 facts extraídos
- 3-5 minutos tiempo de ejecución
- Tasa de éxito: 85%+ archivos procesados

**Criterios de Éxito:**
```
Descubrimiento de Archivos: 5,000+ archivos .java
Facts Extraídos: 15,000+ facts
Definiciones de Rutas: 200+ rutas Camel
Endpoints: 300+ referencias de endpoints
Transformaciones: 400+ transformaciones de mensajes
Cobertura de Módulos: 15+ módulos Maven
Tiempo de Extracción: < 300 segundos
Tasa de Errores: < 15%
```

**Qué validamos:**
- Definiciones de rutas reconocidas
- URIs de endpoints extraídos
- Patrones EIP identificados (choice, split, transform)
- Referencias a beans detectadas
- Límites de módulos comprendidos
- Enlaces de protocolos catalogados

---

### 4. test_concurrent_project_analysis

**Qué prueba:**
```
✅ Thread safety de hodei-scan
✅ Acceso concurrente a recursos compartidos
✅ Extracción paralela de múltiples proyectos
✅ Gestión de memoria bajo carga concurrente
✅ Escalabilidad de rendimiento
```

**Ejecución:**
```rust
async fn test_concurrent_project_analysis() {
    let projects = vec![
        ("guava", "https://github.com/google/guava.git"),
        ("gson", "https://github.com/google/gson.git"),
        ("okhttp", "https://github.com/square/okhttp.git"),
    ];

    let handles: Vec<_> = projects
        .into_iter()
        .map(|(name, url)| {
            tokio::spawn(async move {
                clone_and_extract(url, name).await
            })
        })
        .collect();

    let results = futures::future::join_all(handles).await;

    assert_eq!(results.len(), 3);
    assert!(results.iter().all(|r| r.is_ok()));
}
```

**Qué validamos:**
- Los tres proyectos se extraen exitosamente
- No hay condiciones de carrera en estado compartido
- Uso de memoria se mantiene dentro de límites
- Tiempo total < suma de tiempos individuales (paralelización funciona)

---

## Por Qué Se Eligieron Estos Proyectos

### Google Guava
**Por qué:** Código limpio y bien estructurado
- Perfecto para probar extracción básica
- Excelente prueba de calidad de parsing
- Patrones y convenciones conocidas
- Rápido de clonar y procesar

### Spring Boot
**Por qué:** Características empresariales complejas
- Las anotaciones se usan intensivamente
- Patrones de inyección de dependencias
- Proyectos multi-módulo
- Calidad de código de nivel producción
- Patrones de aplicación web del mundo real

### Apache Camel
**Por qué:** Escala y complejidad
- Base de código masiva (5,000+ archivos)
- Patrones de integración complejos
- Estructura Maven multi-módulo
- Desafíos de parsing DSL
- Requisitos de rendimiento de nivel empresarial

### Proyectos Adicionales (Gson, OkHttp, Commons)
**Por qué:** Diversidad de patrones
- **Gson**: Parsing JSON, reflection
- **OkHttp**: Networking, clientes HTTP
- **Commons**: Manipulación de cadenas, patrones utilitarios

---

## Qué Extrae hodei-scan Realmente

### Tipos de Facts

Cada prueba E2E valida la extracción de estos tipos de facts:

#### 1. Facts de Función
```json
{
  "fact_type": "Function",
  "name": "findById",
  "class": "UserService",
  "visibility": "public",
  "parameters": ["Long"],
  "return_type": "User",
  "line_number": 42
}
```

#### 2. Facts de Variable
```json
{
  "fact_type": "Variable",
  "name": "userRepository",
  "class": "UserController",
  "type": "UserRepository",
  "annotations": ["@Autowired"]
}
```

#### 3. Facts de Taint Sink
```json
{
  "fact_type": "TaintSink",
  "method": "executeQuery",
  "class": "UserDao",
  "vulnerability_type": "SQL_INJECTION",
  "line_number": 87
}
```

#### 4. Facts de Cobertura
```json
{
  "fact_type": "CoverageStats",
  "class": "UserService",
  "line_coverage": 85.5,
  "branch_coverage": 72.3
}
```

---

## Flujo de Ejecución de Pruebas

### Sin Comandos Just (Manual)
```bash
# Clonar repo manualmente
git clone https://github.com/spring-projects/spring-boot.git /tmp/spring-boot

# Ejecutar prueba
cd crates/hodei-java-extractor
cargo test test_spring_boot_application -- --ignored --nocapture

# Limpiar
rm -rf /tmp/spring-boot
```

### Con Comandos Just (Automatizado)
```bash
# Comando único - todo automatizado
just test-e2e-spring

# La salida incluye:
# - Clonación del repositorio
# - Proceso de descubrimiento de archivos
# - Proceso de extracción
# - Resultados de validación
# - Limpieza
```

---

## Métricas de Éxito

### Métricas de Rendimiento
- **Tiempo de Clonado**: < 120 segundos por proyecto
- **Tiempo de Extracción**: < 5 minutos total
- **Uso de Memoria**: < 2GB pico
- **Tasa de Procesamiento de Archivos**: > 50 archivos/segundo

### Métricas de Calidad
- **Facts Extraídos**: Debe exceder el conteo de archivos
- **Tasa de Errores**: < 10% archivos fallidos
- **Completitud**: > 90% código analizado
- **Precisión**: Facts deben pasar validación de esquema

### Métricas de Cobertura
- **Clases**: > 80% clases reconocidas
- **Métodos**: > 70% métodos extraídos
- **Anotaciones**: > 85% anotaciones procesadas
- **Campos**: > 75% campos catalogados

---

## Solución de Problemas

### Problemas Comunes

#### 1. Timeout de Clonado
```
Error: git clone failed after 120 seconds
Solución: Incrementar timeout o usar --depth 1
```

#### 2. Sin Memoria
```
Error: allocation failed
Solución: Incrementar heap Java o procesar menos archivos
```

#### 3. Errores de Parse
```
Error: Failed to parse Java file
Solución: Verificar errores de sintaxis en archivo fuente
```

#### 4. Problemas de Red
```
Error: Failed to fetch repository
Solución: Verificar conexión a internet
```

### Comandos de Depuración

```bash
# Ver qué archivos se descubrieron
find /tmp/test-*/ -name "*.java" | wc -l

# Ver salida de extracción
just test-e2e-guava-verbose

# Limpiar caché
just clean-e2e-cache

# Verificar estado de pruebas
just test-e2e-status
```

---

## Integración CI/CD

### Ejemplo de GitHub Actions
```yaml
name: Pruebas E2E

on:
  schedule:
    - cron: '0 6 * * *'  # Diario a las 6 AM
  workflow_dispatch:      # Trigger manual

jobs:
  e2e-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Instalar Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Ejecutar Pruebas E2E
        run: just test-e2e-all
        env:
          CARGO_NET_GIT_FETCH_WITH_CLI: true
```

### Desarrollo Local
```bash
# Ciclo de retroalimentación rápida
just test-e2e-guava

# Antes de commit
just test-e2e-all

# Depurar problema específico
just test-e2e-spring-verbose
```

---

## Conclusión

Las pruebas E2E prueban que hodei-scan funciona en **código de producción real**, no solo ejemplos diseñados. Validan:

1. **Escalabilidad**: Puede manejar proyectos de 5,000+ archivos
2. **Precisión**: Extrae facts correctas de código complejo
3. **Robustez**: Maneja casos extremos y entrada malformada
4. **Rendimiento**: Se completa en tiempo razonable
5. **Confiabilidad**: Resultados consistentes entre ejecuciones

La combinación de **pruebas unitarias** (valida lógica) y **pruebas E2E** (valida uso del mundo real) asegura que hodei-scan esté listo para producción.

---

## Referencia Rápida

| Prueba | Comando | Archivos | Tiempo | Enfoque |
|--------|---------|----------|--------|---------|
| Guava | `just test-e2e-guava` | ~50 | 1-2 min | Extracción básica |
| Spring | `just test-e2e-spring` | 500+ | 2-3 min | Anotaciones, DI |
| Camel | `just test-e2e-camel` | 2000+ | 3-5 min | Multi-módulo, DSL |
| Gson | `just test-e2e-gson` | ~50 | 1-2 min | Reflection |
| Todos | `just test-e2e-all` | 3000+ | 5-7 min | Validación completa |

**Recuerda:** Todas las pruebas E2E están marcadas `#[ignore]` y requieren flag `--ignored` para ejecutarse.
