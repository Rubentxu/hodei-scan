### **Glosario Universal de hodei-scan**

#### A

*   **Adaptador (Extractor de Nivel 1):** Un tipo de extractor cuyo único trabajo es ejecutar una herramienta de análisis de terceros (ej. `Ruff`, `ESLint`) y **traducir** su informe a Hechos Atómicos en formato IR de `hodei-scan`. Es la forma más rápida de añadir cobertura de reglas para un nuevo lenguaje o herramienta.
*   **Análisis Diferencial:** El proceso de analizar solo los ficheros que han cambiado entre dos puntos en el tiempo (ej. entre dos commits), utilizando un caché para evitar re-analizar el código que no ha sido modificado.
*   **Análisis de Flujo de Datos (DFA - Data-Flow Analysis):** Una técnica de análisis profundo que modela cómo los datos (valores) fluyen a través de las variables y funciones de un programa. Es la base del Taint Analysis. En nuestra analogía, es el "mapa de tuberías".
*   **Arena Allocator:** Una estrategia de gestión de memoria que aloca grandes bloques de memoria de una sola vez y luego reparte pequeños trozos de ese bloque. Es mucho más rápido que pedir memoria al sistema operativo para cada objeto pequeño. Lo usamos para optimizar la creación del `SemanticModel`.
*   **AST (Abstract Syntax Tree / Árbol de Sintaxis Abstracta):** Una representación en forma de árbol de la estructura gramatical del código fuente. Es el "plano 3D" básico de un fichero de código.

#### B

*   **Backend de Gobernanza (`hodei-server`):** El componente de servidor centralizado y stateful de la plataforma. Actúa como el "Archivo Central" y el "Centro Estratégico", gestionando el caché central, las políticas, el historial de análisis y los dashboards.
*   **Baselining (Línea Base):** El proceso de marcar un conjunto de `Findings` existentes en un punto del tiempo (ej. en la rama `main`) como "deuda técnica aceptada". Esto permite que los análisis futuros en ramas de funcionalidad solo fallen por los **nuevos** problemas introducidos.

#### C

*   **Caché Central:** Un almacén de resultados de análisis (IRs Parciales) gestionado por el `hodei-server`. Permite compartir el trabajo de análisis entre todo el equipo y los pipelines de CI/CD.
*   **Caché Híbrido:** La estrategia del CLI `hodei-scan` que combina un caché local (en la máquina del usuario, para velocidad instantánea) y el caché central (para compartir trabajo). El orden es: `Local -> Central -> Ejecutar`.
*   **Caché Local:** Un almacén de IRs Parciales en la máquina local del usuario (ej. en `~/.cache/hodei`), utilizado para acelerar ejecuciones repetidas del mismo análisis.
*   **Cap'n Proto:** Un protocolo de serialización de datos extremadamente rápido que permite la **deserialización "zero-copy"**. Es la tecnología que usamos para nuestro formato de IR, permitiendo leer el informe de evidencia sin tener que cargarlo y parsearlo completamente en memoria.
*   **Correlación Multi-Dominio:** La capacidad única de `hodei-scan` de crear reglas que conectan Hechos Atómicos de diferentes dominios de análisis (ej. un Hecho de Seguridad + un Hecho de Cobertura de Tests + un Hecho de Git) para descubrir riesgos contextuales.

#### D

*   **DSL (Domain-Specific Language / Lenguaje de Dominio Específico):** Un lenguaje de programación diseñado para un propósito muy concreto. En nuestro caso, el DSL Cedar-like es el lenguaje del "Fiscal", diseñado para escribir políticas y correlacionar Hechos Atómicos del IR.

#### E

*   **Épica:** En desarrollo ágil, una historia de usuario grande y compleja que debe ser desglosada en historias más pequeñas. Nuestra Épica del Backend define la visión completa del `hodei-server`.
*   **Extractor:** Un programa o componente responsable de analizar el código fuente (o informes de otras herramientas) y producir una lista de Hechos Atómicos en formato IR. Son los "agentes de campo" de nuestra agencia de inteligencia.
*   **Extractor Declarativo (Nivel 2):** Un tipo de extractor (nuestra "Vigilancia por Satélite") que utiliza `tree-sitter` para encontrar patrones de código definidos en ficheros de reglas simples (ej. YAML), sin necesidad de escribir código de análisis complejo.
*   **Extractor Profundo (Nivel 3):** Un tipo de extractor (nuestro "Agente Encubierto") que construye un `SemanticModel` completo del código para realizar análisis complejos como el Taint Analysis. Es el más potente pero también el más costoso de desarrollar.

#### F

*   **Fact (Hecho Atómico):** La unidad mínima e indivisible de información objetiva extraída del código. Es un "informe de una sola línea" de un agente. Ejemplos: `TaintSource`, `UncoveredLine`, `CodeSmell`.
*   **Finding (Hallazgo):** El **resultado** de una regla `forbid` del DSL que se ha cumplido. Es una **conclusión inteligente** o "veredicto" que se genera al correlacionar uno o más Hechos. Los `Findings` son lo que el usuario final ve en el informe.

#### G

*   **Grafo de Flujo de Control (CFG - Control-Flow Graph):** Una representación del código que modela todos los caminos posibles que puede tomar la ejecución a través de una función (bucles, condicionales, etc.). Es el "mapa de carreteras".

#### H

*   **Hecho Atómico:** Ver **Fact**.

#### I

*   **IR (Intermediate Representation / Representación Intermedia):** Nuestro "lenguaje universal". Es el formato de datos estándar (definido con Cap'n Proto) que todos los extractores usan para reportar sus Hechos Atómicos. El IR Final es el "Informe de Evidencia" completo que se pasa al motor de reglas.
*   **IR Parcial:** Una instancia del IR que contiene solo los Hechos Atómicos correspondientes a **un único fichero de código fuente**. Es la "unidad de trabajo cacheable".

#### L

*   **LSP (Language Server Protocol):** Un protocolo estándar que permite a los "servidores de lenguaje" (como el que crearemos para nuestro DSL) proporcionar funcionalidades inteligentes (autocompletado, detección de errores) a cualquier editor de código compatible (VS Code, Neovim, etc.).

#### M

*   **`mmap` (Memory-mapped file):** Una técnica del sistema operativo que permite tratar un fichero en disco como si fuera una sección de la memoria RAM. Es la tecnología que, junto a Cap'n Proto, nos permite acceder al IR sin el coste de leerlo y parsearlo.

#### P

*   **Planificador de Consultas (Query Planner):** El componente del motor de reglas que, antes de evaluar una política, analiza su estructura y los índices disponibles para decidir la forma más rápida y eficiente de ejecutarla.
*   **Policy Pack (Paquete de Políticas):** Un conjunto de reglas (DSL y/o YAML) agrupadas lógicamente (ej. "OWASP Top 10", "Estándares de Código de Acme Corp."). Son gestionados por el `hodei-server`.

#### R

*   **REPL (Read-Eval-Print Loop):** Un entorno de consola interactivo. `hodei-scan query` proporcionará un REPL para explorar el IR y hacer "investigación forense".

#### S

*   **SARIF (Static Analysis Results Interchange Format):** Un formato estándar de la industria para los informes de herramientas de análisis estático. Nuestro **Adaptador de Nivel 1** `sarif-to-hodei` nos da compatibilidad instantánea con docenas de herramientas.
*   **Semantic Model (Modelo Semántico):** Una representación interna y muy rica del código fuente que construye un Extractor Profundo (Nivel 3). Contiene el AST, CFG, DFG, etc. Es nuestra "maqueta 3D" de la ciudad.
*   **Stateless (Sin Estado):** Una operación o componente que no guarda información de ejecuciones anteriores. El CLI de `hodei-scan` es fundamentalmente stateless, lo que lo hace predecible y fiable para CI/CD.
*   **Stateful (Con Estado):** Un componente que mantiene un registro de interacciones pasadas. El `hodei-server` es stateful porque almacena el historial de análisis.

#### T

*   **Taint Analysis (Análisis de Contaminación):** Una técnica de análisis profundo para seguir el flujo de datos no confiables ("taint" o "contaminación") desde su punto de entrada (`TaintSource`) hasta un punto de ejecución peligroso (`TaintSink`).
*   **tree-sitter:** Un generador de parsers y librería de parsing incremental extremadamente rápido. Es la tecnología que impulsa nuestro Extractor Declarativo de Nivel 2.

#### Z

*   **Zero-Copy Deserialization:** La capacidad de acceder a los datos de una estructura serializada (como un fichero Cap'n Proto) sin tener que copiarla y reconstruirla en la memoria. Es una de las optimizaciones de rendimiento más importantes de `hodei-scan`.