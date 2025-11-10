### Fase 1: 🚀 Construir el Núcleo (v3.1) - "Hacer que Funcione"

**Objetivo:** Implementar la arquitectura `v3.1` de forma funcional, probando que el concepto (IR + Motor DSL) es viable de principio a fin. El rendimiento aún no es la prioridad; la *corrección* sí.

1.  **Implementar el `hodei-ir` (Schema v3.1):**
    * **Acción:** Crear la biblioteca de Rust con todos los tipos seguros (`ProjectPath`, `LineNumber`, `FlowId`, `FactType`, etc.).
    * **Justificación:** Este es el "contrato" de todo el sistema. Debe ser lo primero y estar 100% probado (especialmente `ProjectPath` contra Path Traversal).

2.  **Construir 1 Extractor de Nivel 2 (SAST Profundo):**
    * **Acción:** Elegir **un solo lenguaje** (ej. Java o TypeScript) y construir el extractor *completo*, incluyendo el Taint Analysis (DFA/CFG) que emite `TaintSource` y `TaintSink`.
    * **Justificación:** Esta es la parte más difícil. Si no podemos hacer esto, el proyecto fracasa. Hay que validar el mayor riesgo técnico primero.

3.  **Construir 2 Extractores de Nivel 1 (Linting Rápido):**
    * **Acción:** Implementar extractores basados en `tree-sitter` para otros lenguajes (ej. Python, Go) que *solo* emitan hechos de Nivel 1 (`Function`, `Complexity`, `UnsafeCall`).
    * **Justificación:** Demuestra la extensibilidad de la Etapa 1.

4.  **Implementar el `IndexedFactStore` (Versión Simple):**
    * **Acción:** Implementar el motor de indexación. **Importante:** Usar `std::HashMap` (o `AHashMap` si es fácil) por ahora. No implementar *todavía* el `SpatialIndex` completo.
    * **Justificación:** Necesitamos una indexación funcional antes de poder optimizarla.

5.  **Implementar el Motor DSL (v1.0):**
    * **Acción:** Usar `pest` para implementar la gramática PEG. Construir el evaluador que pueda manejar las reglas básicas (ej. `exists(Fact { type: "..." })`) y la correlación `by_flow`.
    * **Justificación:** Probar que el DSL puede generar `Findings` a partir de los `Hechos`.

**Resultado de la Fase 1:** Un prototipo funcional que puede ejecutar un análisis completo (`Extract -> Index -> Evaluate`) sobre 3 lenguajes. Es lento, pero funciona y demuestra el concepto.

---

### Fase 2: ⚡ Optimizar el Núcleo (v3.2) - "Hacer que sea Rápido"

**Objetivo:** Atacar los cuellos de botella de la Fase 1. Ahora que funciona, lo hacemos rápido. Estas son optimizaciones *dirigidas por profiling*.

1.  **Optimización de Carga (Cap'n Proto):**
    * **Acción:** Reemplazar el formato de serialización del IR (probablemente JSON o similar en la Fase 1) por **Cap'n Proto y `mmap`**.
    * **Justificación:** Reduce el tiempo de carga del motor de evaluación (Etapa 2) de segundos a microsegundos.

2.  **Optimización de CI/CD (Caching Incremental):**
    * **Acción:** Implementar la **Optimización 1 (Caching por Archivo)**. El CLI debe poder hacer `git diff` y solo re-extraer los archivos modificados.
    * **Justificación:** Esta es la mejora de rendimiento **más importante** para el usuario. Reduce los análisis de PR de minutos a segundos.

3.  **Optimización de Índices (Estructuras de Datos):**
    * **Acción:** Implementar la **Optimización 2 (String Interning y EnumMap)**. Reemplazar `AHashMap<FactType, ...>` por `Box<[Vec<...>]>` (un EnumMap). Reemplazar `Arc<ProjectPath>` por claves `u32` de un interner.
    * **Justificación:** Reduce drásticamente el uso de memoria y la sobrecarga de CPU en la indexación.

4.  **Optimización de Correlación (Índice Espacial):**
    * **Acción:** Implementar el `SpatialIndex` (`by_location`) que faltaba en la Fase 1.
    * **Justificación:** Permite que las reglas de correlación multi-dominio (ej. SAST + Cobertura) se ejecuten en O(k\*m) en lugar de O(N²).

5.  **Optimización de Evaluación (Poda de Reglas):**
    * **Acción:** Implementar la **Optimización 3 (Rule Pruning)**.
    * **Justificación:** Reduce el trabajo de la CPU en la Etapa 3 al no evaluar reglas irrelevantes.

**Resultado de la Fase 2:** Una aplicación v1.0 lista para producción. Es extremadamente rápida, eficiente en memoria y supera a la competencia en el rendimiento de CI/CD.

---

### Fase 3: 🧠 Expandir la Visión (v4.0) - "Hacer que sea Inteligente"

**Objetivo:** Usar la plataforma estable y rápida de la Fase 2 para implementar las características conceptuales que nos diferencian.

1.  **Grafo de Riesgo (Kauffman):**
    * **Acción:** Usar los `FlowId` y los grafos de llamadas de los extractores para construir un **Grafo de Propagación de Riesgo**.
    * **Justificación:** Se convierte en una nueva característica de la UI: "Vista de Radio de Explosión" (Blast Radius), mostrando cómo un `TaintSource` se propaga por el sistema.

2.  **IA de Descubrimiento de Reglas (Kauffman):**
    * **Acción:** Iniciar un proyecto de I+D (R&D). Empezar a recolectar (anónimamente, con *opt-in*) los IRs generados por los usuarios.
    * **Justificación:** Usar Algoritmos Genéticos o ML sobre este conjunto de datos para *descubrir* nuevas correlaciones (`K > 1`) que se conviertan en las reglas de seguridad del futuro.

3.  **Optimización de Hardware (SIMD / io_uring):**
    * **Acción:** Solo ahora, en la Fase 3, implementar las optimizaciones de Nivel 4 (SIMD y `io_uring`).
    * **Justificación:** Son optimizaciones de "última milla", complejas y específicas de plataforma, que solo tienen sentido cuando todo lo demás ya está optimizado.

Este enfoque por fases es la única forma viable de construir un sistema de esta complejidad y ambición.