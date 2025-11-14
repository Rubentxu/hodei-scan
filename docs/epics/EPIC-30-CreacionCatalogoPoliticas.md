Aquí tienes un plan de batalla completo y ambicioso, estructurado en Épicas y Fases. Está diseñado para ser pragmático, entregando valor incrementalmente, pero sin perder de vista el objetivo final: una cobertura de clase mundial.

---

# **ÉPICA Maestra: "Génesis" - Creación del Catálogo de Políticas Multi-Lenguaje**

**Como** un nuevo usuario de `hodei-scan`,
**Quiero** que la plataforma venga con un catálogo de políticas completo y de alta calidad "out-of-the-box",
**Para que** pueda obtener valor inmediato y una protección exhaustiva para mis proyectos sin tener que convertirme en un experto en la escritura de reglas.

**Objetivo Estratégico:** Alcanzar y superar la cobertura de reglas de seguridad y calidad de competidores líderes como SonarQube (Community Edition) y Semgrep (Reglas Comunitarias) para los 5 principales lenguajes en un roadmap de 18 meses, diferenciándonos con nuestras capacidades únicas de correlación y análisis arquitectónico.

---

## **Fase 1: La Fundación y la Conquista Rápida (Meses 1-3)**

**Objetivo:** Establecer la infraestructura del catálogo y lograr una cobertura masiva para **JavaScript/TypeScript** y **Python** usando las tácticas más rápidas (Nivel 1 y 2). Al final de esta fase, `hodei-scan` debe ser percibido como una alternativa viable y potente para estos lenguajes.

### **ÉPICA-30: Infraestructura del Catálogo de Políticas**

*   **HU-30.01: Diseñar y Estructurar el Repositorio `hodei-policies`.**
    > **Como** desarrollador del core, **quiero** una estructura de carpetas lógica y un esquema de metadatos estandarizado para todas las reglas, **para que** el catálogo sea mantenible, escalable y fácil de navegar.
    *   **TDD:** Escribir un test que valide un fichero de regla de ejemplo contra un esquema JSON, verificando que todos los campos requeridos (`id`, `name`, `severity`, `tags`) existan.
    *   **Investigación:** Analizar la estructura de los repositorios `semgrep-rules` y `eslint-rules` para inspirarse.

*   **HU-30.02: Implementar el Concepto de `Policy Packs`.**
    > **Como** usuario, **quiero** poder activar conjuntos de reglas predefinidos (ej. `owasp-top-10`) con una sola línea de configuración, **para que** no tenga que seleccionar reglas una por una.
    *   **TDD:** Crear un test que cargue un `.hodei-pack` de ejemplo y verifique que el motor de reglas se prepara para ejecutar el número correcto de reglas listadas.

### **ÉPICA-31: Cobertura Masiva para JavaScript/TypeScript**

*   **HU-31.01 (Investigación): Mapear el Ecosistema de Linters de JS/TS.**
    > **Como** desarrollador de extractores, **quiero** investigar las 100 reglas más populares de **ESLint** y sus plugins (`typescript-eslint`, `eslint-plugin-security`), y crear un "fichero de mapeo" que las traduzca a nuestro esquema de metadatos, **para que** sepamos qué cobertura obtendremos al crear el adaptador.

*   **HU-31.02 (Implementación Nivel 1): Crear el Adaptador para ESLint.**
    > **Como** usuario de JS/TS, **quiero** que `hodei-scan` pueda ejecutar ESLint, parsear su salida JSON y convertir cada `linting error` en un Hecho Atómico de `hodei-scan` (`CodeSmell` o `Vulnerability`), **para que** obtenga cientos de reglas de calidad y seguridad al instante.
    *   **TDD:**
        1.  *Red:* Escribir un test que ejecute el adaptador sobre un fichero JSON de ESLint de ejemplo y falle porque no se generan los `Facts` esperados.
        2.  *Green:* Implementar la lógica del adaptador para que el test pase.
        3.  *Refactor:* Optimizar el parseo y añadir manejo de errores.

*   **HU-31.03 (Implementación Nivel 2): Portar 20 Reglas de Seguridad Clave de Semgrep para JS/TS.**
    > **Como** Ingeniero de Seguridad, **quiero** portar los 20 patrones de seguridad más importantes de Semgrep para JS/TS (ej. uso de `eval`, `dangerouslySetInnerHTML`) a nuestro formato `.hodei.yml`, **para que** tengamos una capa de seguridad basada en patrones que ESLint no cubre.
    *   **TDD:** Para cada regla portada, crear un fichero de test con un fragmento de código vulnerable y otro seguro, y verificar que `hodei-scan test-rule` pasa para ambos.

### **ÉPICA-32: Cobertura Masiva para Python**

*   **HU-32.01 (Investigación): Mapear el Ecosistema de Linters de Python.**
    > **Como** desarrollador de extractores, **quiero** investigar las reglas de **Ruff** y **Bandit** (para seguridad) y crear sus correspondientes "ficheros de mapeo", **para que** sepamos qué cobertura obtendremos.

*   **HU-32.02 (Implementación Nivel 1): Crear el Adaptador para Ruff.**
    > **Como** usuario de Python, **quiero** un adaptador para `Ruff`, **para que** obtenga una cobertura masiva de reglas de calidad y estilo con un rendimiento excepcional.
    *   **(Sigue TDD como en HU-31.02)**

*   **HU-32.03 (Implementación Nivel 1): Crear el Adaptador para Bandit.**
    > **Como** usuario de Python, **quiero** un adaptador para `Bandit`, **para que** obtenga una cobertura inicial de vulnerabilidades de seguridad comunes en Python.

---

## **Fase 2: La Conquista de la Empresa (Meses 4-9)**

**Objetivo:** Añadir soporte de alto nivel para **Java** y **Go**, los pilares del desarrollo empresarial y de infraestructura cloud. Empezar a construir la fundación de nuestro análisis profundo (Nivel 3).

### **ÉPICA-20 (Continuación): Fundación del Análisis Profundo**

*   *(Las historias de usuario `HU-20.01` a `HU-20.05` de la respuesta anterior encajan aquí, comenzando el desarrollo de `hodei-deep-analysis-engine`).*

### **ÉPICA-33: Cobertura Amplia para Java**

*   **HU-33.01 (Investigación): Mapear el Ecosistema de Java.**
    > **Como** desarrollador de extractores, **quiero** investigar las reglas de herramientas como **Checkstyle** (estilo), **SpotBugs** (bugs) y las reglas comunitarias de SonarQube para Java, y crear sus "ficheros de mapeo", **para que** tengamos una hoja de ruta clara para la cobertura.

*   **HU-33.02 (Implementación Nivel 1): Crear el Adaptador para Checkstyle y SpotBugs (vía SARIF).**
    > **Como** usuario de Java, **quiero** que `hodei-scan` pueda importar los resultados de Checkstyle y SpotBugs (idealmente usando nuestro adaptador SARIF genérico), **para que** obtenga una cobertura básica de calidad y bugs.
    *   **(Sigue TDD como en HU-31.02)**

*   **HU-33.03 (Implementación Nivel 3 - Inicio): Iniciar el Extractor Profundo de Java.**
    > **Como** desarrollador del core, **quiero** empezar la implementación del `java-deep-extractor`, enfocándome en la **HU-22.01** (construir el traductor a `SemanticModel`), **para que** podamos sentar las bases para el futuro análisis de Taint.
    *   **TDD:** Escribir un test que parsea una clase Java simple y verifica que el `SemanticModel` resultante contiene la estructura correcta (nodos de clase, métodos, etc.).

### **ÉPICA-34: Cobertura Amplia para Go**

*   **HU-34.01 (Implementación Nivel 1): Crear el Adaptador para `staticcheck`.**
    > **Como** usuario de Go, **quiero** que `hodei-scan` se integre con `staticcheck`, **para que** obtenga una cobertura de primera clase para errores, simplificaciones y problemas de rendimiento.

*   **HU-34.02 (Implementación Nivel 2): Portar Reglas de Semgrep para Go.**
    > **Como** Ingeniero de Seguridad, **quiero** portar 15 patrones de seguridad clave de Semgrep para Go (ej. inyecciones SQL, uso de `unsafe`) a nuestro formato `.hodei.yml`, **para que** tengamos una buena base de seguridad para el ecosistema de Go.

---

## **Fase 3: La Cima de la Seguridad y la Arquitectura (Meses 10-18)**

**Objetivo:** Desplegar completamente el poder de nuestro análisis de Nivel 3. Alcanzar y superar a la competencia en la detección de vulnerabilidades profundas para JS/TS y Java. Introducir las políticas arquitectónicas únicas.

### **ÉPICA-21 y 22 (Continuación): Finalización de Extractores Profundos**

*   **HU-21.02, HU-22.02 (Implementación): Implementar y Desplegar el Análisis de Taint.**
    > **Como** Ingeniero de Seguridad, **quiero** que los extractores de JS y Java detecten el **OWASP Top 10** completo usando el motor de Taint Analysis, **para que** `hodei-scan` se convierta en una herramienta de SAST de élite.
    *   **TDD:** Para cada clase de vulnerabilidad (SQLi, XSS, etc.), crear un test de integración con un micro-proyecto vulnerable y verificar que se generan los `Facts` de `TaintSource` y `TaintSink` correctos.
    *   **Investigación:** Analizar las consultas QL de CodeQL para identificar las `Sources` y `Sinks` más relevantes para cada framework (Express, Spring).

*   **HU-21.03, HU-22.03 (Implementación): Desplegar el Análisis Arquitectónico.**
    > **Como** Arquitecto de Software, **quiero** que los extractores de JS y Java detecten y reporten **Hechos de Connascence**, **para que** pueda empezar a escribir políticas de gobernanza de diseño.
    *   **TDD:** Crear tests que verifiquen la detección correcta de CoP y CoM en fragmentos de código específicos.

### **ÉPICA-35: Políticas de Correlación Avanzada (El Diferenciador Final)**

*   **HU-35.01: Crear el "Policy Pack" de Riesgo Contextual.**
    > **Como** Ingeniero de Seguridad, **quiero** escribir y agrupar un conjunto de políticas DSL `.hodei` que **correlacionen Hechos de diferentes dominios**, **para que** `hodei-scan` pueda priorizar automáticamente los riesgos más importantes.
    *   **TDD:** Para cada regla de correlación, crear un test que genere un IR sintético con los `Facts` necesarios (ej. un `TaintSink` y un `UncoveredLine` en la misma ubicación) y verifique que se produce el `Finding` esperado.
    *   **Reglas a Implementar:**
        *   `RISK-UNTESTED-VULNERABILITY`: `TaintSink` + `UncoveredLine`.
        *   `RISK-BUG-IN-PERFORMANCE-HOTSPOT`: `CodeSmell` + `PerformanceHotspot`.
        *   `RISK-CRITICAL-DEBT-IN-LEGACY-CODE`: `Coupling (CoP/CoM)` + `GitCommitInfo (age > 365 days)`.

Este plan te da un camino claro. Empiezas con una explosión de valor (Fase 1), luego te expandes a mercados clave (Fase 2), y finalmente construyes tu foso competitivo inexpugnable (Fase 3), todo mientras desarrollas la infraestructura fundamental en paralelo. Es ambicioso, pero cada fase es una victoria por sí misma.