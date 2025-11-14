# Plan de Adopción y Pruebas

: `hodei-scan` para el Ecosistema Java**

**Objetivo del Proyecto:** Validar y demostrar el poder de `hodei-scan` analizando una aplicación Java real. Al final de este plan, debemos ser capaces de:

1. Ejecutar un análisis completo (Nivel 1, 2 y 3) sobre un proyecto Java.
2. Detectar vulnerabilidades de seguridad (OWASP Top 10) y "malos olores" arquitectónicos (Connascence).
3. Correlacionar estos hallazgos con datos de cobertura de tests.
4. Generar un informe de `Findings` accionable.

---

## **Fase 0: Preparación del "Campo de Batalla" (Duración: 1 Semana)**

**Objetivo:** Tener todas las herramientas, configuraciones y el proyecto de prueba listos antes de escribir la primera línea de código de un extractor.

### Tarea 0.1: Elegir la Aplicación Java Objetivo

No podemos probar en el vacío. Necesitamos una "víctima".

* **Acción:** Seleccionar un proyecto Java.
* **Opciones:**
  1. **Ideal (Proyecto Interno):** Una aplicación de tamaño medio (50k - 200k líneas) de tu empresa que sea conocida, con su historial de Git y, preferiblemente, con algunos problemas de seguridad o calidad ya identificados.
  2. **Excelente (Proyecto Open Source Vulnerable):** **OWASP WebGoat**. Esta es la mejor opción para probar la seguridad. Es una aplicación **deliberadamente vulnerable** diseñada para enseñar y probar herramientas de seguridad.
  3. **Realista (Proyecto Open Source Conocido):** Un proyecto popular como `spring-petclinic`. Es un buen ejemplo de una aplicación Spring Boot bien estructurada.
* **Decisión para este plan:** **Usaremos OWASP WebGoat.** Nos permite verificar de forma objetiva si nuestro análisis de Taint funciona.

### Tarea 0.2: Ensamblar la "Caja de Herramientas"

* **Acción:** Instalar y configurar todas las herramientas de terceros que nuestros extractores de Nivel 1 y 3 necesitarán.
* **Checklist de Herramientas:**
  1. **JDK (Java Development Kit):** Necesitamos compilar el proyecto Java. Instalar una versión estándar (ej. JDK 17).
  2. **Maven o Gradle:** El sistema de build del proyecto Java. WebGoat usa Maven.
  3. **JaCoCo:** La herramienta estándar en el ecosistema Java para medir la cobertura de tests.
     * **Configuración:** Modificar el `pom.xml` de WebGoat para integrar el plugin de JaCoCo. El objetivo es que al ejecutar `mvn clean install`, se genere un fichero `jacoco.exec` y un informe XML en el directorio `target/site/jacoco/`.
  4. **Spoon (o la herramienta elegida en `HU-22.00`):** Nuestro "escáner de resonancia magnética".
     * **Acción:** Crear un pequeño proyecto Java/Maven independiente que tome como dependencia a Spoon. Este proyecto contendrá un programa simple `main` que:
       * Toma la ruta al código fuente de WebGoat como argumento.
       * Usa Spoon para parsear todo el proyecto.
       * Por ahora, solo imprime "Éxito al parsear" o recorre el AST e imprime los nombres de las clases para verificar que funciona.
       * Este pequeño programa será el núcleo de nuestro "Servicio de Análisis Semántico".

### Tarea 0.3: Definir las Políticas Iniciales (El "Libro de Leyes" v1)

* **Acción:** Crear la estructura de carpetas en nuestro repositorio `hodei-policies` y escribir las primeras reglas que queremos probar.
* **Ficheros a Crear:**
  1. **`rules/java/security/patterns.hodei.yml` (Nivel 2):**
     ```yaml
     - id: JAVA-HARDCODED-PASSWORD-001
       language: java
       name: "Contraseña Hardcodeada Detectada"
       message: "Se encontró un valor que parece una contraseña en una variable. Evita almacenar secretos en el código."
       severity: "Critical"
       tags: ["security", "cwe-798"]
       pattern: |
         String $VAR = "..."; # Un patrón simple para empezar
     ```
  2. **`rules/java/security/sqli.hodei` (Nivel 3 - DSL):**
     ```cedar
     forbid(
       rule: "JAVA-SQL-INJECTION-001",
       severity: "Critical",
       description: "Flujo de datos no sanitizado detectado desde una entrada HTTP a una consulta SQL."
     ) on {
       exists(Fact { type: "TaintSource", "data.tags": contains("http-input"), flow_id: $f }) &&
       exists(Fact { type: "TaintSink", "data.category": "SqlQuery", consumes_flow: $f })
     }
     ```
  3. **`policies/java-sqli-policy.toml` (Nivel 3 - Configuración de Taint):**
     ```toml
     [[sources]]
     type = "method_parameter"
     class_name = "org.springframework.web.bind.annotation.RequestParam"
     description = "Parámetro de una petición web."
     tags = ["http-input", "user-input"]

     [[sinks]]
     type = "method_call"
     class_name = "java.sql.Statement"
     method_name = "executeQuery"
     parameter_index = 0
     description = "Ejecución de una consulta SQL."
     category = "SqlQuery"
     ```

---

## **Fase 1: Implementación de los Extractores (Duración: 4-6 Sprints)**

**Objetivo:** Construir los tres niveles de extractores para Java.

### Tarea 1.1: El "Escucha de Radio" (Extractor de Nivel 1 para Cobertura)

* **Epica/HU:** `HU-XX.XX (Nueva): Crear Adaptador para JaCoCo.`
* **Acción:** Construir el `jacoco-adapter`.
  1. Será un script (Python/Go/Rust) que toma como entrada la ruta al fichero `jacoco.xml`.
  2. Parsea el XML, que contiene información detallada de qué líneas están cubiertas, parcialmente cubiertas o no cubiertas.
  3. Por cada línea no cubierta (`type="LINE" missed_instructions="1"`), genera un `Fact: UncoveredLine`.
  4. Emite su resultado como un IR Parcial.
* **TDD:** Preparar un `jacoco.xml` de ejemplo y escribir un test que verifique que se generan el número y tipo correctos de `Facts`.

### Tarea 1.2: La "Vigilancia por Satélite" (Extractor de Nivel 2)

* **Epica/HU:** `HU-22.01 (revisada): Implementar el Extractor Declarativo para Java.`
* **Acción:**
  1. Integrar `tree-sitter-java` en el `declarative-extractor` genérico.
  2. Probarlo con la regla `JAVA-HARDCODED-PASSWORD-001` que ya definimos.
* **TDD:** Usar `hodei-scan test-rule` con un fichero Java que contenga una contraseña y otro que no.

### Tarea 1.3: El "Agente Encubierto" (Extractor de Nivel 3)

* **Epica/HU:** `EPIC-22` completa.
* **Acción:** Este es el trabajo principal.
  1. **(`HU-22.01`)** Integrar la herramienta Java (Spoon) en el "Servicio de Análisis Semántico". El servicio debe tomar una ruta de proyecto y producir un `SemanticModel.sem.capnp`.
  2. **(`HU-22.02`)** Implementar la lógica del orquestador en Rust que:
     * Invoca al servicio anterior (manejando el caché).
     * Carga el `SemanticModel` resultante.
     * Invoca al `TaintPropagator` de `hodei-deep-analysis-engine` usando la política `java-sqli-policy.toml`.
     * Traduce los `TaintFlow` encontrados a Hechos `TaintSource` y `TaintSink`.
* **TDD:** Utilizar el proyecto WebGoat como el caso de prueba de integración de extremo a extremo. El test final debe ejecutar el extractor completo y verificar que se detectan las vulnerabilidades de SQL Injection conocidas de WebGoat.

---

## **Fase 2: La Primera Ejecución Completa (Duración: 1 Sprint)**

**Objetivo:** Juntar todas las piezas y realizar el primer análisis completo de `hodei-scan` sobre WebGoat.

### Tarea 2.1: Configurar la Orquestación

* **Acción:** Crear el fichero `hodei.toml` final para el proyecto WebGoat.
  ```toml
  # hodei.toml en la raíz de WebGoat

  # El orden de ejecución puede ser importante
  [[extractors]]
  name = "java-declarative"
  command = "hodei-declarative-extractor --lang java"

  [[extractors]]
  name = "jacoco-coverage"
  command = "python3 ./extractors/jacoco-adapter.py --input ./target/site/jacoco/jacoco.xml"

  [[extractors]]
  name = "java-deep-analysis"
  command = "hodei-java-deep-extractor"
  ```

### Tarea 2.2: Ejecutar y Depurar

* **Acción:** Ejecutar `$ hodei-scan analyze . --ir-output webgoat.ir`
* **Proceso:**
  1. Verificar que la ejecución termina sin errores.
  2. Usar `$ hodei-scan ir-dump --input webgoat.ir` para inspeccionar el informe de evidencia. ¿Están ahí todos los tipos de `Facts` que esperábamos? ¿De Nivel 1, 2 y 3?
  3. Ejecutar el motor de reglas con una política de correlación:
     ```cedar
     // test-correlation.hodei
     forbid(...) on {
       exists(Fact { type: "TaintSink", "data.category": "SqlQuery", file: $f, line: $l }) &&
       exists(Fact { type: "UncoveredLine", file: $f, line: $l })
     }
     ```
  4. Verificar si se generan `Findings` de correlación. Depurar por qué sí o por qué no.

### Tarea 2.3: Medir y Optimizar el Rendimiento

* **Acción:** Realizar benchmarks.
* **Métricas a Medir:**
  * Tiempo total de ejecución de `hodei-scan analyze` (en frío, sin caché).
  * Tiempo de ejecución de cada extractor individualmente.
  * Tiempo de ejecución en caliente (con caché). Debería ser drásticamente más rápido.
  * Uso de memoria pico del `java-deep-extractor`.
* **Resultado:** Identificar cuellos de botella y crear nuevas tareas de optimización.

Este plan te lleva desde la preparación inicial hasta la validación completa. Cada fase y tarea es medible y se basa en las anteriores. Siguiendo este roadmap, no solo probarás `hodei-scan` en Java, sino que construirás el ecosistema necesario para que sea un éxito.
