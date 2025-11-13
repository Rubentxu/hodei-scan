# **ÉPICA Maestra: Análisis de Flujo de Datos y Diseño Arquitectónico (Nivel 3)**

**Visión Actualizada:** No solo detectaremos vulnerabilidades de seguridad, sino que también analizaremos la **estructura profunda y la salud arquitectónica** del software, proporcionando a los desarrolladores y arquitectos una inteligencia sin precedentes sobre el acoplamiento y el diseño de su código.

Esta épica detalla la creación de **`hodei-deep-analysis-engine`**, la librería central en Rust que servirá como el "cerebro" para todos nuestros Extractores Profundos (Nivel 3). Su misión es abstraer la ciencia computacional compleja del análisis de programas, permitiendo que los extractores de cada lenguaje se enfoquen en la semántica de su ecosistema en lugar de reinventar algoritmos de grafos.

Esta librería no solo proporcionará un motor de **Taint Analysis** de clase mundial, sino que, de forma innovadora, integrará el **Análisis de Diseño Arquitectónico** basado en el concepto de Connascence. Es la "Academia" donde entrenamos a nuestros "Agentes de Élite" para ser tanto detectives de seguridad como arquitectos de software.

### 📊 Estado Actual (Noviembre 2025)

**INVESTIGACIÓN COMPLETADA:** Se ha realizado una evaluación exhaustiva de las tecnologías y del estado actual del código. Los resultados muestran que **el proyecto ya tiene una base sólida implementada**.

**Ver documentos de referencia:**
- `/docs/INVENTARIO-NIVEL3-ACTUAL.md` - Análisis detallado del código existente
- `/docs/ANALYSIS-CONNASCENCE-EPIC20.md` - Investigación de tecnologías y propuesta arquitectónica

### Objetivo de Negocio
Desarrollar un activo tecnológico reutilizable que **reduzca drásticamente el coste y el tiempo** para construir analizadores profundos para nuevos lenguajes. Esto nos permitirá escalar nuestra oferta de análisis de alta gama y proporcionar una inteligencia sobre el código (tanto de seguridad como de diseño) que es inalcanzable para nuestros competidores.

### Métricas de Éxito
-   **Reutilización**: >80% del código de análisis de flujo y acoplamiento debe residir en esta librería, no en los extractores específicos del lenguaje.
-   **Rendimiento**: El motor de Taint Analysis debe ser capaz de procesar un grafo de 100,000 nodos en < 5 segundos.
-   **Flexibilidad**: Añadir una nueva clase de vulnerabilidad de Taint (ej. "Inyección LDAP") debe ser posible modificando solo un fichero de configuración TOML, sin recompilar el motor.
-   **Expresividad**: El motor debe ser capaz de detectar al menos 4 tipos de Connascence estática (Posición, Significado, Tipo, Nombre).

### 🛠️ Tecnologías Evaluadas (2025)

| Tecnología | Versión | Estado | Evaluación |
|------------|---------|--------|------------|
| **datafrog** | v2.0.1 | ✅ Óptimo | Motor Datalog mantenido por rust-lang, >5,800 dependientes |
| **petgraph** | v0.6.5 | ✅ Excelente | Soporta 100k+ nodos, algoritmos completos (astar, dijkstra) |
| **tree-sitter** | v0.25.10 | ✅ Madura | 22,600+ stars, grammars para JS/TS/Java/Python/Rust |

**Conclusión:** Las tecnologías están en su mejor momento y perfectamente adaptadas al caso de uso.


---

## 2. Contexto Técnico

### 2.1. Problema Actual
Construir un analizador de flujo de datos desde cero para cada lenguaje es un proyecto de años. Los algoritmos para construir grafos de flujo, propagar información y analizar acoplamiento son complejos y propensos a errores. Sin una fundación común, cada equipo de extractores duplicaría esfuerzos, introduciría inconsistencias y se movería a una velocidad glacial.

### 2.2. Solución: Una Librería de Primitivas de Análisis

Crearemos una librería en Rust agnóstica al lenguaje que proporcionará las "herramientas del detective" ya listas para usar. El trabajo de un nuevo extractor de Nivel 3 se convierte en un "trabajo de integración": tomar la representación específica de un lenguaje (de `tree-sitter` o similar) y "traducirla" a las estructuras de datos genéricas de esta librería.

```mermaid
graph TD
    subgraph "Extractor Profundo de Java<br>Cliente de la Librería"
        A[Parser de Java] --> B{Traductor Específico de Java};
        B --> C[Llamadas a hodei-deep-analysis-engine];
    end

    subgraph "hodei-deep-analysis-engine (Librería Central)"
        D[Definición del SemanticModel]
        E[Motor de Taint Analysis con datafrog]
        F[Motor de Análisis de Connascence]
    end

    subgraph "Salida del Extractor"
        G((IR Parcial con Hechos Profundos<br/><i>TaintSource, Coupling...</i>))
    end
    
    C -- Usa --> D & E & F;
    C -- Genera --> G;

    style D fill:#cde,stroke:#333,stroke-width:2px
    style E fill:#cde,stroke:#333,stroke-width:2px
    style F fill:#cde,stroke:#333,stroke-width:2px
```

---

## 2.5. **Estado Real de Implementación** (Actualizado 2025-11-13)

### 📋 Inventario de Componentes Existentes

**PROGRESO ACTUAL:** 40-50% ya implementado

#### ✅ **Lo que YA EXISTE (Reutilizable)**

1. **hodei-pattern-engine** (100% completo)
   - Ubicación: `crates/hodei-pattern-engine/`
   - **Funcional:** Tree-sitter integration con cache LRU, YAML rules, multi-lenguaje
   - **Reutilizar:** ✅ Listo para usar como base de parsing

2. **FlowIndex con petgraph** (90% completo)
   - Ubicación: `crates/hodei-engine/src/store/flow_index.rs`
   - **Funcional:** DiGraph con algoritmos astar, dijkstra, reachable queries
   - **Reutilizar:** ✅ Base perfecta para Taint Analysis (solo falta datafrog overlay)

3. **IR Schema Cap'n Proto** (100% completo)
   - Ubicación: `crates/hodei-ir/schema/facts.capnp`
   - **Funcional:** TaintSource, TaintSink, Sanitization ya definidos
   - **Reutilizar:** ✅ Schema perfecto, listo para usar

4. **hodei-declarative-extractors** (30% completo)
   - Ubicación: `crates/hodei-declarative-extractors/`
   - **Estado:** Estructura lista, pero usa AST stub (no tree-sitter real)
   - **Reutilizar:** ⚠️ Solo estructura, necesita conexión tree-sitter real

#### ❌ **Lo que FALTA (Por implementar)**

1. **hodei-deep-analysis-engine** - Crate principal **NO EXISTE**
2. **TaintPropagator con datafrog** - 0% implementado
3. **ConnascenceAnalyzer** - 0% implementado
4. **SemanticModel Builder** - 0% implementado
5. **Policy TOML parser** - 0% implementado
6. **CFG/DFG desde AST** - 0% implementado

### 🎯 **Plan de Implementación Optimizado**

**ESTRATEGIA:** Aprovechar lo existente (5-6 semanas vs 12-16 semanas)

#### **Paso 1: Crear hodei-deep-analysis-engine (Semana 1)**
```bash
cargo new --lib crates/hodei-deep-analysis-engine
[dependencies]
datafrog = "2.0.1"                    # NUEVO
petgraph = { workspace = true }        # REUTILIZAR
hodei-ir = { path = "../hodei-ir" }    # REUTILIZAR
```

#### **Paso 2: Integrar datafrog sobre FlowIndex (Semana 1-2)**
```rust
// Overlay de datafrog en FlowIndex existente
pub struct TaintPropagator {
    flow_index: FlowIndex,  // Ya existe
    iteration: Iteration,   // NUEVO con datafrog
    sources: Variable<(FlowId, VariableName)>,
    sinks: Variable<(FlowId, SinkCategory)>,
    // ...
}
```

#### **Paso 3: Conectar tree-sitter real (Semana 2-3)**
```rust
// Reemplazar AST stub con tree-sitter real
impl MultiLanguageParser {
    pub fn parse_real(&self, lang: Language, code: &str) -> Result<ParseResult> {
        let parser = Parser::new().set_language(language)?;
        let tree = parser.parse(code, None)?;
        Ok(self.tree_to_ast(tree.root_node()))
    }
}
```

#### **Paso 4: ConnascenceAnalyzer con TDD (Semana 3-4)**
```rust
impl ConnascenceAnalyzer {
    pub fn detect_positional(&self) -> Vec<CouplingFinding> {
        // Heurística: 3+ parámetros mismo tipo = CoP
    }
}
```

#### **Paso 5: SemanticModel Builder (Semana 4-5)**
```rust
impl SemanticModel {
    pub fn from_ast(ast: &ASTNode) -> Result<Self> {
        let cfg = self.build_cfg(ast)?;
        let dfg = self.build_dfg(&cfg)?;
        Ok(SemanticModel { cfg, dfg, ... })
    }
}
```

#### **Paso 6: Policy TOML + Integración (Semana 5-6)**
```toml
# política.toml
[[sources]]
pattern = "request.*"
source_type = "HttpRequest"
tags = ["PII", "UserInput"]

[[sinks]]
pattern = "executeQuery"
category = "SqlQuery"
severity = "critical"
```

### 💡 **Ventajas de la Estrategia**

1. **Máxima reutilización:** 40-50% del trabajo ya hecho
2. **Riesgo mínimo:** Componentes probados en producción
3. **Time-to-market:** 5-6 semanas en lugar de 12-16
4. **Calidad:** Tests y documentación ya existentes

### 📊 **Métricas Revisadas**

| Métrica | Objetivo Original | Estimación Revisada |
|---------|-------------------|---------------------|
| **Tiempo desarrollo** | 12-16 semanas | **5-6 semanas** |
| **Código reutilizable** | 80% | **40-50% ya implementado** |
| **Nuevos tests** | 100% | **60% reutilizables** |
| **Riesgo técnico** | Alto | **Bajo** (tecnologías maduras) |

**CONCLUSIÓN:** El proyecto está **mucho más avanzado** de lo esperado. La base existente permite un desarrollo mucho más rápido y seguro.

---

### **ÉPICA-20 (v2.0): La Fundación del Análisis Profundo y Arquitectónico (`hodei-deep-analysis-engine`)**

**Objetivo (Actualizado):** Crear la librería central que contenga la lógica para el análisis de flujo de datos **Y** para el análisis de acoplamiento estructural, basado en el concepto de Connascence. Es la "Academia de Espías y Arquitectos".

*   **HU-20.01: Diseñar un `SemanticModel` Rico en Contexto.** ⚠️ **REVISADO**
    > **Como** desarrollador del motor, **quiero** que las estructuras de datos del `SemanticModel` incluyan no solo grafos de flujo, sino también una representación explícita de **ámbitos (scopes), jerarquías de tipos y un grafo de acoplamiento**, **para que** podamos realizar análisis que entiendan el contexto completo del código.
    **Estado Real:** Base existente en `hodei-declarative-extractors` (ASTNode), necesita conexión con tree-sitter real y petgraph.
    **Criterios de Aceptación:**
      *   ✅ `struct ASTNode` ya definida (en hodei-declarative-extractors)
      *   ⚠️ `struct SemanticModel` por crear (basado en ASTNode)
      *   ⚠️ `struct ControlFlowGraph` por crear (usando petgraph::Graph)
      *   ⚠️ `struct DataFlowGraph` por crear (usando petgraph::Csr)
      *   ⚠️ `struct ScopeTree` por crear (nuevo)
      *   ⚠️ `struct CouplingGraph` por crear (petgraph::Graph con ConnascenceEdge)
      *   [ ] Todas las estructuras son agnósticas a cualquier lenguaje de programación específico.
    **Reutilización:** hodei-declarative-extractors provee base, petgraph ya disponible en workspace.

*   **HU-20.02: Implementar el Motor de Taint Analysis.** ✅ **REVISADO**

    **Como** desarrollador del motor, **quiero** integrar `datafrog` para crear un `TaintPropagator` genérico, declarativo y ultra-performante, **para que** podamos modelar problemas complejos de flujo de datos como un conjunto de reglas lógicas, separando la "física" de la propagación de la semántica específica del lenguaje.

    **Estado Real:** FlowIndex con petgraph ya implementado (crates/hodei-engine/src/store/flow_index.rs), solo necesita overlay de datafrog.
    
*   **Criterios de Aceptación:**
    *   ⚠️ Nueva dependencia `datafrog = "2.0.1"` por añadir
    *   ⚠️ Reglas Datalog por definir (ej. `Tainted(Y) :- FlowsTo(Y, X), Tainted(X).`)
    *   ✅ `FlowIndex` con `DiGraph`, `astar`, `dijkstra` ya implementado
    *   ⚠️ Función `run_taint_analysis` por crear (combinando FlowIndex + datafrog)
    *   ⚠️ Soporte `Sanitizers` por implementar
    *   [ ] Sanitizers actúan como "retract" en Datalog, deteniendo propagación
    **Reutilización:** FlowIndex provee 70% del trabajo (grafo, algoritmos, queries)

*   **HU-20.03: Definir el Formato de Políticas de Taint (Sin Cambios).**
    
    **Como** Ingeniero de Seguridad, **quiero** un formato de fichero TOML intuitivo para definir qué son las `Sources` (fuentes), `Sinks` (sumideros) y `Sanitizers` (sanitizadores), y que me permita añadir **etiquetas de dominio de datos** (ej. `PII`, `Finance`, `Credentials`), **para que** el análisis de flujo pueda rastrear no solo la contaminación, sino también el **tipo** de datos sensibles, y para que pueda añadir nuevas definiciones de vulnerabilidades sin recompilar el motor.

*   **Criterios de Aceptación:**
    *   [ ] `structs` de Rust (`TaintPolicy`, `SourceDefinition`, etc.) que pueden ser deserializadas desde un fichero TOML.
    *   [ ] La `SourceDefinition` debe incluir un campo opcional `tags: Vec<String>`.
    *   [ ] El `TaintPropagator` (`HU-20.02`) debe ser modificado para propagar estas etiquetas junto con el estado de "taint".
    *   [ ] La estructura de resultado `TaintFlow` debe incluir las etiquetas que llegaron desde la fuente hasta el sumidero.


### **HU-20.04: Implementar el Motor de Análisis de Connascence.** 🆕 **NUEVO**

**Como** desarrollador del motor, **quiero** crear un módulo `ConnascenceAnalyzer` que opere sobre el `SemanticModel` y pueda identificar y clasificar diferentes tipos de acoplamiento (Connascence), **para que** los extractores puedan detectar automáticamente "malos olores" arquitectónicos y de diseño.

**Estado Real:** ❌ NO IMPLEMENTADO - Requiere implementación completa desde cero

*   **Criterios de Aceptación:**
    #### **Connascence Estática (Acoplamiento de Código Fuente)**

    Estos tipos son visibles analizando el código sin ejecutarlo. Nuestro extractor de Nivel 3 está **perfectamente posicionado** para detectarlos.
    
    1.  **Connascence de Nombre (CoN):**
        *   **¿Qué es?:** Un componente se refiere a otro por su nombre. `moduloA.funcionX()`.
        *   **Viabilidad de Detección:** **Trivial.** Es la forma más básica de dependencia. Nuestra Tabla de Símbolos y Grafo de Llamadas ya modelan esto.
        *   **Veredicto:** ✅ **Implementado por defecto.**
    
    2.  **Connascence de Tipo (CoT):**
        *   **¿Qué es?:** Dos componentes deben estar de acuerdo en un tipo de dato. `function procesarUsuario(usuario: User)`.
        *   **Viabilidad de Detección:** **Trivial** para lenguajes de tipado estático (Java, Rust). Más complejo pero **viable** para lenguajes dinámicos (Python, JS) usando análisis de inferencia de tipos.
        *   **Veredicto:** ✅ **Viable y Muy Valioso.**
    
    3.  **Connascence de Significado (CoM):**
        *   **¿Qué es?:** Dos componentes deben estar de acuerdo en el significado de un valor. `if (estado == 2) // 2 significa 'Completado'`.
        *   **Viabilidad de Detección:** **Viable con heurísticas.** Podemos detectar "valores mágicos" (literales hardcodeados) que se repiten en diferentes partes del sistema. Es una señal fuerte de CoM.
        *   **Veredicto:** ✅ **Viable y Muy Valioso.** Es una de las detecciones más importantes para la mantenibilidad.
    
    4.  **Connascence de Posición (CoP):**
        *   **¿Qué es?:** El orden de los argumentos importa. `crearUsuario("Juan", "Pérez", 30)`.
        *   **Viabilidad de Detección:** **Viable y Relativamente Fácil.** Podemos detectar llamadas a funciones/constructores con múltiples parámetros del mismo tipo primitivo.
        *   **Veredicto:** ✅ **Viable y Muy Valioso.** Otro "quick win".
    
    5.  **Connascence de Algoritmo (CoA):**
        *   **¿Qué es?:** Dos componentes deben usar el mismo algoritmo para ser compatibles (ej. el cliente y el servidor deben usar el mismo algoritmo de compresión/encriptación).
        *   **Viabilidad de Detección:** **Difícil pero Parcialmente Viable.** No podemos "entender" un algoritmo complejo de forma abstracta. PERO, podemos detectar patrones como:
            *   Ambos lados llaman a la misma función criptográfica (`sha256(...)`).
            *   Ambos lados usan la misma librería de serialización (ej. `JSON.stringify` en JS y `ObjectMapper` en Java).
            *   Detectar que dos implementaciones manuales del mismo algoritmo son compatibles es un problema a nivel de investigación (casi imposible de forma genérica).
        *   **Veredicto:** ⚠️ **Parcialmente Viable.** Podemos detectar el uso de algoritmos *conocidos*, pero no verificar la compatibilidad de algoritmos *implementados a mano*. Debemos enfocarnos en lo primero.


### **HU-20.05: Definir los Hechos Atómicos Arquitectónicos (`Coupling`, `ApiEndpoint`).**

**Como** arquitecto de la plataforma, **quiero** que el Esquema del IR (`.capnp`) se extienda para incluir nuevos `FactTypes` como `Coupling` y `ApiEndpoint`, **para que** los resultados de los análisis arquitectónicos puedan ser reportados de forma estándar y consumidos por el motor de políticas DSL, permitiendo la creación de reglas de gobernanza de diseño.

*   **Criterios de Aceptación:**
    *   [ ] El fichero `schema.capnp` define la nueva estructura `Coupling` con campos como `entity_a`, `entity_b`, `connascence_type`, `strength`.
    *   [ ] El fichero `schema.capnp` define la nueva estructura `ApiEndpoint` con campos como `function_name`, `route`, `http_method`.
    *   [ ] Las estructuras de Rust correspondientes se generan y se integran en el `enum FactType`.

---

### **ÉPICA-21 (v2.0): Extractor Profundo para JavaScript/TypeScript**

**Objetivo (Actualizado):** Implementar el primer "Agente de Élite" que no solo encuentre vulnerabilidades, sino que también evalúe la calidad del diseño en el ecosistema dinámico de JavaScript.

*   **HU-21.01: Construir el Traductor a `SemanticModel` (Incluyendo Scopes).**
    > **Como** desarrollador del extractor de JS, **quiero** que el traductor de `tree-sitter` a `SemanticModel` modele correctamente los **ámbitos léxicos (lexical scopes) y el `hoisting` de JavaScript**, **para que** el análisis de flujo de datos y de Connascence sea preciso.
    *   *Tareas:* Implementar la lógica para rastrear `var`, `let`, `const`, clausuras (closures) y el `this` contextual.

*   **HU-21.02: Implementar el Análisis de Taint (Sin Cambios).**
    > *Esta historia de usuario se mantiene igual.*

*   **HU-21.03 (NUEVA): Detectar y Reportar Connascence en JavaScript.**
    > **Como** desarrollador senior de JavaScript, **quiero** que el extractor detecte patrones de acoplamiento fuerte comunes en JS, como la dependencia de argumentos posicionales en funciones (`CoP`) o el uso compartido de objetos globales (`CoI`), **para que** podamos promover un código más modular y mantenible.
    *   *Tareas:*
        *   Orquestar la ejecución del `ConnascenceAnalyzer` de la librería central sobre el `SemanticModel` de JS.
        *   Generar Hechos `FactType::Coupling` para los problemas detectados.
        *   Crear una política YAML de Nivel 2 para detectar el uso de "strings mágicos" en `switch` o `if/else if` (`CoM`).

---

### **ÉPICA-22 (v2.0): Extractor Profundo para Java**

**Objetivo (Actualizado):** Proveer análisis de seguridad y arquitectónico de primer nivel para el ecosistema empresarial de Java, entendiendo su sistema de tipos y patrones de diseño.

*   **HU-22.01: Construir el Traductor a `SemanticModel` (Incluyendo Jerarquía de Tipos).**
    > **Como** desarrollador del extractor de Java, **quiero** que el `SemanticModel` para Java represente con precisión la **jerarquía de herencia de clases y la implementación de interfaces**, **para que** el análisis pueda entender el polimorfismo y las dependencias de tipo.
    *   *Tareas:* El traductor debe ser capaz de resolver la herencia (`extends`), las interfaces (`implements`) y las anotaciones.

*   **HU-22.02: Implementar el Análisis de Taint (Sin Cambios).**
    > *Esta historia de usuario se mantiene igual.*

*   **HU-22.03 (NUEVA): Analizar Patrones de Diseño y Connascence en Java.**
    > **Como** arquitecto de Java, **quiero** que el extractor detecte "anti-patrones" que generan acoplamiento fuerte, como la **Connascence de Posición** en constructores con muchos parámetros, o la **Connascence de Algoritmo** donde dos clases dependen de una implementación de `hashCode()` compatible, **para que** podamos guiar a los equipos hacia un diseño más limpio.
    *   *Tareas:*
        *   Ejecutar el `ConnascenceAnalyzer`.
        *   Crear una política TOML para el análisis de Taint que modele la serialización de Java como un `Sink` peligroso (un ejemplo de `CoA`).

---

### **Épicas 23 (Python) y 24 (Rust) - Actualización similar**

Las Épicas para Python y Rust seguirían el mismo patrón de actualización:

1.  **La Historia de Usuario del `SemanticModel` se enriquece** para incluir las características específicas del lenguaje (ej. ámbitos dinámicos para Python, lifetimes y traits para Rust).
2.  **Se añade una nueva Historia de Usuario** para ejecutar el `ConnascenceAnalyzer` y detectar patrones de acoplamiento específicos de ese ecosistema, generando Hechos `FactType::Coupling`.

### El Resultado Final: Políticas Arquitectónicas en Acción

Esta actualización de las Épicas asegura que, al final de la implementación, no solo podremos escribir políticas de seguridad como esta:
```cedar
// Política de Seguridad (ya posible antes)
forbid(...) on { exists(TaintSink { category: "SqlQuery" }) }
```
Sino que también podremos escribir **políticas de diseño arquitectónico** como esta:
```cedar
// Política Arquitectónica (posible gracias a la actualización)
forbid(
  rule: "DESIGN-001-AVOID-POSITIONAL-COUPLING",
  severity: "Major",
  description: "Se detectó Connascence de Posición. Refactoriza usando un objeto de parámetros (Builder Pattern o un DTO) para mejorar la legibilidad y reducir errores."
) on {
  exists(Fact { type: "Coupling", connascence_type: "Position" })
}
```

Al integrar la Connascence y el análisis de ámbitos en el núcleo de nuestros extractores más potentes, `hodei-scan` cumple su promesa de ser una verdadera plataforma de **inteligencia de ingeniería.**


