# **ÉPICA-22 (v3.0 - Detallada): "El Titán" - Extractor Profundo para Java**

**Estado**: ✅ **Ready for Development**
**Versión**: 3.0
**Épica padre**: `Análisis de Flujo de Datos y Diseño Arquitectónico (Nivel 3)`
**Dependencias**: `EPIC-20` (`hodei-deep-analysis-engine`)
**Owner**: Java Analysis Team
**Prioridad**: **Critical Path**

---

## 1. Resumen Ejecutivo y Visión

Esta épica detalla la construcción de **`java-deep-extractor`**, nuestro "Agente de Élite" para el ecosistema Java. Su misión es ir más allá del análisis de patrones superficiales para entender la semántica profunda del código Java, incluyendo su complejo sistema de tipos, el polimorfismo, el uso de frameworks empresariales como Spring, y las sutilezas de la JVM.

El resultado final será un extractor que, utilizando nuestra librería central `hodei-deep-analysis-engine`, producirá un **`SemanticModel` de una riqueza sin precedentes para el código Java**. A partir de este modelo, generará Hechos Atómicos para **vulnerabilidades de seguridad de flujo de datos (OWASP Top 10)** y para **métricas de salud arquitectónica (Connascence)**.

### Objetivo de Negocio
Posicionar a `hodei-scan` como la herramienta de análisis estático **superior para aplicaciones Java empresariales**, combinando la profundidad de análisis de herramientas especializadas (como CodeQL o SonarQube Commercial) con el rendimiento, la flexibilidad y las capacidades de correlación únicas de nuestra plataforma.

### Métricas de Éxito
-   **Cobertura de Seguridad**: Detectar con éxito al menos 8 de las 10 vulnerabilidades del OWASP Top 10 en un proyecto de prueba deliberadamente vulnerable (ej. OWASP WebGoat).
-   **Precisión**: Lograr una tasa de falsos positivos < 15% en el análisis de Taint para SQL Injection en proyectos de código abierto bien conocidos.
-   **Cobertura Arquitectónica**: Detectar correctamente al menos 3 tipos de Connascence estática (CoT, CoP, CoM) en proyectos con "anti-patrones" conocidos.
-   **Rendimiento**: Analizar un proyecto Java de tamaño medio (250k líneas de código) y generar su `SemanticModel` y Hechos en < 5 minutos en una máquina estándar.

---

## 2. Contexto Técnico y Estrategia de Implementación

### 2.1. El Desafío: Java es un Gigante
Analizar Java es difícil. No es solo el lenguaje, es el ecosistema:
*   **Compilado:** El código fuente no es suficiente; a menudo se necesita información del bytecode o de las dependencias compiladas.
*   **Sistema de Tipos Complejo:** Herencia, interfaces, genéricos, polimorfismo.
*   **Frameworks "Mágicos":** Spring, Jakarta EE, etc., usan inyección de dependencias y reflexión, ocultando el flujo de control y de datos.

### 2.2. Nuestra Estrategia: "No Reinventar el Compilador"
Nuestra estrategia no es construir un compilador de Java en Rust. Eso sería un proyecto de una década. En su lugar, adoptaremos un **enfoque de dos etapas**:

1.  **Etapa de Extracción de Datos Crudos:** Usaremos una herramienta madura del ecosistema Java para que haga el trabajo pesado de compilación y análisis inicial. Esta herramienta actuará como nuestro "escáner de resonancia magnética", exportando una representación detallada del código.
2.  **Etapa de Construcción del `SemanticModel` en Rust:** Nuestro extractor de Rust consumirá esta representación intermedia y la "traducirá" a nuestro `SemanticModel` genérico. Aquí es donde aplicaremos nuestra inteligencia y la conectaremos con nuestro `hodei-deep-analysis-engine`.

## Estrategia de Implementación (Revisada):
Adoptaremos un modelo de análisis híbrido. El extractor de Rust orquestará dos motores:
Motor Sintáctico Rápido (basado en tree-sitter-java): Para todos los análisis que no requieran resolución de tipos completa (Nivel 2).
Motor Semántico Profundo (basado en un "headless compiler"): Que se invoca solo para construir el SemanticModel necesario para el análisis de Nivel 3.

**Herramienta Candidata para la Etapa 1:**
*   **Investigación Inicial (primera tarea):** Se debe realizar una investigación para elegir la mejor herramienta. Candidatos:
    *   **Spoon:** Una librería de análisis de Java que puede transformar código fuente en un AST rico y bien tipado, y exportarlo.
    *   **Eclipse JDT:** El compilador de Java de Eclipse, que puede ser usado como librería y tiene APIs para recorrer el AST y la información de tipos.
    *   **Compilador de Errores de Google:** Proporciona un AST y un análisis semántico simple.

---

## 3. Historias de Usuario Detalladas (Impulsadas por TDD)

### **Funcionalidad Clave 1: Construcción del `SemanticModel` para Java**

*   **HU-22.00 (Investigación): Seleccionar la Herramienta de Interfaz con Java.**
    > **Como** desarrollador del extractor, **quiero** investigar y elegir la mejor librería/compilador del ecosistema Java para parsear el código y exportar su estructura (AST, tipos, etc.), **para que** tengamos una base fiable sobre la que construir nuestro `SemanticModel` sin reinventar la rueda.
    *   *Tareas:* Crear un PoC con Spoon y Eclipse JDT. Evaluar la riqueza de la información exportada, el rendimiento y la facilidad de integración.

### **ÉPICA-22 (v4.0 - Optimizada para Rendimiento): Extractor Híbrido para Java**

**Estrategia de Implementación (Revisada):**
Adoptaremos un modelo de análisis híbrido. El extractor de Rust **orquestará** dos motores:
1.  **Motor Sintáctico Rápido (basado en `tree-sitter-java`):** Para todos los análisis que no requieran resolución de tipos completa (Nivel 2).
2.  **Motor Semántico Profundo (basado en un "headless compiler"):** Que se invoca solo para construir el `SemanticModel` necesario para el análisis de Nivel 3.

El extractor de Rust `java-deep-extractor` se convierte en un orquestador inteligente.

### Historias de Usuario (Redefinidas con el Enfoque Híbrido)

#### **Funcionalidad Clave 1: El Extractor Sintáctico de Alta Velocidad**

*   **HU-22.01 (Reemplaza la anterior): Implementar un Extractor de Nivel 2 para Java usando `tree-sitter`.**
    > **Como** desarrollador de nuestro ecosistema, **quiero** tener un `java-declarative-extractor` ultra-rápido basado en `tree-sitter` que pueda ejecutar las reglas de patrones YAML, **para que** podamos detectar cientos of "code smells" y anti-patrones comunes en Java en segundos, sin el overhead de una compilación.
    *   **TDD:**
        1.  *Red:* Crear un test con una regla YAML simple (ej. `pattern: System.out.println(...)`) y un fichero Java. El test falla porque no se genera el `Fact`.
        2.  *Green:* Implementar la lógica que invoca a `tree-sitter-java`, recorre el AST y lo compara con los patrones YAML.
    *   **Rendimiento:** Este extractor debe ser capaz de analizar un proyecto de 1M de líneas en < 30 segundos.

#### **Funcionalidad Clave 2: El Motor Semántico "A Demanda"**

*   **HU-22.02 (Reemplaza la HU-22.01 anterior): Crear un "Servicio de Análisis Semántico" para Java.**
    > **Como** desarrollador del extractor profundo, **quiero** construir un "servicio" (que puede ser un proceso separado o una librería invocada) que utilice una herramienta como Spoon/JDT para realizar una compilación completa y **exportar un `SemanticModel` serializado (en Cap'n Proto)**, **para que** la parte lenta y dependiente de Java esté aislada.
    *   **TDD:**
        1.  *Red:* Crear un test de integración que invoque este servicio con un proyecto Java simple y falle porque el fichero de salida del `SemanticModel` no se crea o es incorrecto.
        2.  *Green:* Implementar la lógica en Java (usando Spoon) que construye una representación de nuestro `SemanticModel` (clases, métodos, CFG, DFG) y la serializa usando Cap'n Proto (que tiene librerías para Java).
    *   **Optimización del Rendimiento:** Este servicio debe ser **incremental por sí mismo**. Herramientas como el compilador de Eclipse ya tienen mecanismos de compilación incremental. Debemos aprovecharlo al máximo.

#### **Funcionalidad Clave 3: El Orquestador Inteligente en Rust**

*   **HU-22.03 (Reemplaza la HU-22.02 anterior): Implementar el Extractor Profundo como un Orquestador.**
    > **Como** desarrollador del `java-deep-extractor` en Rust, **quiero** que mi extractor primero invoque al "Servicio de Análisis Semántico" (HU-22.02) para obtener el `SemanticModel` serializado, y luego cargue este modelo en memoria **sin coste de parsing (zero-copy)**, **para que** la parte de Rust pueda ejecutar los análisis de Taint y Connascence a velocidad nativa sin tocar una línea de código Java ni la JVM.
    *   **TDD:**
        1.  *Red:* Escribir un test que simule la salida del servicio (un fichero `.sem.capnp`) y falle porque el extractor de Rust no lo carga correctamente en sus estructuras de `hodei-deep-analysis-engine`.
        2.  *Green:* Implementar la lógica de carga "zero-copy" del `SemanticModel` en Rust.

*   **HU-22.04 (Nueva): Integrar el Caching Diferencial a Nivel de `SemanticModel`.**
    > **Como** Ingeniero de Rendimiento, **quiero** que el orquestador `java-deep-extractor` sea lo suficientemente inteligente como para no volver a invocar el costoso "Servicio de Análisis Semántico" si el código fuente del proyecto no ha cambiado, **para que** las ejecuciones sucesivas sean casi instantáneas.
    *   **TDD:** Escribir un test que ejecute el extractor dos veces sobre el mismo código. Usar `mocking` para verificar que el "Servicio de Análisis Semántico" solo es invocado en la primera ejecución.
    *   **Implementación:** Usar un sistema de caché (como el que definimos para los IRs Parciales) donde la clave es un hash de todo el código fuente del proyecto y el valor es el `SemanticModel` serializado.


*   **(Historias sucesivas): HU-22.05 (XSS), HU-22.06 (Deserialización Insegura), etc.**
    > *Siguen el mismo patrón, expandiendo iterativamente el fichero de políticas de Taint y añadiendo tests de integración para cada clase de vulnerabilidad.*

### **Funcionalidad Clave 3: Implementación de Análisis Arquitectónico**

*   **HU-22.07: Detectar y Reportar Connascence Estática en Java.**
    > **Como** Arquitecto de Software, **quiero** que el extractor de Java utilice el `ConnascenceAnalyzer` del motor central para detectar y generar Hechos `Coupling` para problemas como la **Connascence de Posición** en constructores con >5 parámetros de tipo primitivo y la **Connascence de Significado** por el uso de `enums` comparados con strings, **para que** podamos enforzar buenas prácticas de diseño y reducir el acoplamiento.
    *   **TDD - Flujo de Trabajo:**
        1.  *Red:* Crear un proyecto de test con clases que exhiban explícitamente CoP y CoM. Escribir un test de integración que ejecute el extractor y falle porque no se generan los `Facts` de tipo `Coupling` esperados.
        2.  *Green:* Implementar la orquestación que llama al `ConnascenceAnalyzer` y traduce sus resultados a Hechos IR.
        3.  *Refactor:* Optimizar el análisis para que no degrade significativamente el rendimiento.
        *   **Commit:** `feat(java-extractor): add detection of static connascence (CoP, CoM)`

*   **HU-22.08: Detectar y Reportar la Exposición de Endpoints.**
    > **Como** Arquitecto, **quiero** que el extractor reconozca las anotaciones de frameworks web como Spring (`@RestController`, `@GetMapping`) y Jakarta EE (`@Path`) y genere Hechos `ApiEndpoint`, **para que** podamos escribir políticas de correlación que prioricen los riesgos en el código públicamente expuesto.
    *   **TDD:** Crear un test con un controlador Spring de ejemplo y verificar que se genera un `Fact: ApiEndpoint` con la ruta (`/api/users`) y el método HTTP (`GET`) correctos.

Este plan de Épica es un proyecto grande, pero cada Historia de Usuario es una unidad de trabajo verificable que entrega valor incremental. El uso riguroso de TDD garantiza que, al final, el extractor sea no solo potente, sino también robusto, fiable y bien documentado a través de sus tests.


---

Vamos a desglosar por qué, herramienta por herramienta.

---

### El Papel de Cada Herramienta en la "Construcción" de Nuestro `SemanticModel`

Imagina de nuevo que construir el `SemanticModel` es como crear una maqueta 3D ultra-detallada de una ciudad (tu código Java).

#### 1. `tree-sitter` (El Topógrafo)

*   **¿Qué hace?:** Es nuestro topógrafo experto. Puede tomar una foto aérea de cualquier edificio (`.java`) y, usando su conocimiento de la "gramática de la arquitectura" de Java, dibujar un **mapa estructural perfecto de ese edificio individual (el AST)**. Sabe "esto es un muro", "esto es una ventana", "esto es una puerta".
*   **¿Cuál es su limitación fundamental?:** Es un experto en **sintaxis**, no en **semántica**.
    *   Ve una puerta que dice `"Lleva a la oficina del Sr. Smith"`, pero **no tiene ni idea de dónde está la oficina del Sr. Smith**. Podría estar en el edificio de al lado, o en otra ciudad. (`tree-sitter` ve `miObjeto.hacerAlgo()`, pero no sabe dónde se define `hacerAlgo()`).
    *   Ve un cable eléctrico, pero **no sabe si es de 110V o 220V**. (`tree-sitter` ve una variable, pero no sabe si su tipo es `String` o `Integer`).
    *   No puede leer los planos de otros edificios (`.jar` de las dependencias).

#### 2. `petgraph` (El Software de Modelado 3D)

*   **¿Qué hace?:** Es nuestro software CAD (`AutoCAD`, `Blender`). Proporciona las herramientas para dibujar grafos: nodos, aristas, y algoritmos para trabajar con ellos (como "encontrar el camino más corto").
*   **¿Cuál es su limitación fundamental?:** Es una caja de herramientas vacía. Es increíblemente potente para **representar relaciones**, pero no tiene ni idea de **qué relaciones representar**. Necesita que alguien le diga "dibuja una línea de aquí a aquí". No puede mirar el código y deducir el Grafo de Flujo de Control por sí solo.

#### 3. `datafrog` (El Motor de Lógica y Deducción)

*   **¿Qué hace?:** Es nuestro "Súper Cerebro". Si le damos un conjunto de hechos y reglas, puede deducir nueva información a una velocidad increíble.
*   **¿Cuál es su limitación fundamental?:** "Basura entra, basura sale" (Garbage in, garbage out). Su capacidad de deducción depende enteramente de la **calidad de los hechos iniciales** que le proporcionamos. Si no le podemos decir con certeza que "el valor de la variable `x` fluye a la variable `y`", entonces no puede deducir que la "contaminación" de `x` se propaga a `y`.

### El Eslabón Perdido: La "Resolución de Símbolos"

Aquí está el trabajo que ninguna de esas tres herramientas hace, y que es la especialidad de un compilador como **Spoon/JDT**: la **Resolución de Símbolos y Tipos**.

Este es el proceso de conectar los puntos. Es el trabajo de un detective que no solo tiene los mapas de los edificios, sino que también tiene acceso al registro de la propiedad, al censo, y puede hacer llamadas para verificar identidades.

**Para hacer un Taint Analysis preciso en Java, necesitas responder a preguntas como:**
> "En la línea `miLista.add(datoSensible)`, ¿`miLista` es un `ArrayList`, un `LinkedList`, o una clase personalizada `MiListaSegura` que sanitiza los datos al añadirlos?"

*   `tree-sitter` no lo sabe. Solo ve `miLista`.
*   Para saberlo, necesitas:
    1.  Encontrar dónde se declaró `miLista`.
    2.  Resolver su tipo.
    3.  Si es un tipo genérico, resolver sus parámetros.
    4.  Cargar la definición de ese tipo, que podría estar en otro fichero de tu proyecto o en un `.jar` de una dependencia.
    5.  Analizar la jerarquía de herencia para ver si `add()` es un método sobreescrito.

**Este proceso de "resolución" es, en esencia, lo que hace un compilador.** Es computacionalmente caro y requiere una visión global de todo el proyecto y sus dependencias.

### Conclusión: ¿Por Qué el Enfoque Híbrido Sigue Siendo el Óptimo?

Tienes toda la razón en querer maximizar el rendimiento. Por eso, el enfoque híbrido sigue siendo la mejor estrategia:

1.  **Aprovechamos `tree-sitter` al máximo (Extractor de Nivel 2):** Para el 80% de las reglas (patrones, "code smells", etc.) que **no necesitan** esta información semántica profunda, usamos `tree-sitter` directamente. Esto nos da un feedback casi instantáneo para la mayoría de los problemas de calidad.

2.  **Aislamos el "Trabajo Lento" (Extractor de Nivel 3):** Para el 20% de las reglas que **sí necesitan** la inteligencia de un compilador, delegamos esa fase de "Resolución de Símbolos" a una herramienta especializada (Spoon/JDT). Lo hacemos de forma inteligente:
    *   **Lo encapsulamos** en un "Servicio de Análisis Semántico" que se ejecuta por separado.
    *   **Lo cacheamos agresivamente**, de modo que el coste de esta operación lenta solo se pague una vez por cada estado del código.

3.  **Usamos Nuestras Herramientas de Rust para lo que son Mejores:**
    *   Una vez que el servicio de Java nos ha dado el `SemanticModel` enriquecido (el resultado de la Resolución de Símbolos), **ahora sí** podemos desatar el poder de nuestras herramientas de Rust.
    *   Cargamos ese modelo en **`petgraph`**.
    *   Ejecutamos los análisis de Taint sobre el grafo usando **`datafrog`**.
    *   Todo este trabajo final, que es donde se ejecuta la lógica de seguridad, ocurre a la **velocidad nativa de Rust**.

En resumen: usas `tree-sitter`, `petgraph`, y `datafrog` para construir el **motor de análisis más rápido posible**, pero necesitas alimentarlo con **datos de alta calidad**. Para Java, la forma más pragmática y fiable de obtener esos datos de alta calidad es aprovechando el trabajo que ya ha hecho el ecosistema de compiladores de Java, en lugar de intentar reconstruirlo desde cero en Rust.

