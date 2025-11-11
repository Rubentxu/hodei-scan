# **hodei-scan v3.3: Especificación Arquitectónica Mejorada**
## **Plataforma de Gobernanza de Calidad con Ecosistema Extensible y Correlación Inteligente**

**Versión:** 3.2.1 (Propuesta de Mejora)
**Fecha:** 2025-11-11
**Estado:** Propuesta para Desarrollo
**Autor:** Arquitectura hodei-scan (revisado por Gemini Solutions Architect)

---

## 📋 1. Resumen Ejecutivo (Visión Actualizada)

Este documento refina la arquitectura de **hodei-scan v3.3**, evolucionándola de un motor de análisis de alto rendimiento a una **plataforma de gobernanza de software completa y extensible**. Mantenemos los pilares de rendimiento extremo y seguridad por diseño de la v3.2, pero introducimos una estrategia de ecosistema abierto para acelerar drásticamente la adopción y la cobertura de análisis, sin sacrificar la profundidad ni la capacidad de correlación única que nos define.

### Objetivos Clave de la Arquitectura v3.3:

1.  **Cobertura Universal Acelerada:** Alcanzar una cobertura de análisis comparable a SonarQube/Semgrep en múltiples lenguajes en meses, no años, mediante una estrategia de extractores de 3 niveles.
2.  **Ecosistema Abierto Radical:** Permitir que cualquier herramienta externa y cualquier desarrollador (sin importar su lenguaje de programación) contribuya al ecosistema de `hodei-scan`.
3.  **Correlación Multi-Dominio Profunda:** Mantener y mejorar la capacidad de cruzar datos de SAST, SCA, Calidad y Cobertura como principal diferenciador competitivo.
4.  **Gobernanza con Estado:** Introducir un backend opcional para habilitar el análisis de tendencias, la gestión de deuda técnica y dashboards de alto nivel, cumpliendo la promesa de "Gobernanza".
5.  **Experiencia de Desarrollador (DX) de Primera Clase:** Facilitar al máximo la creación, prueba y depuración de reglas para fomentar una comunidad activa.

### Cambios Arquitectónicos Clave: v3.2 → v3.3

| Aspecto | v3.2 (Especificación Original) | v3.3 (Propuesta Mejorada) | Beneficio |
| :--- | :--- | :--- | :--- |
| **Extractores** | Plugins de Rust que implementan un `trait`. | **Procesos independientes** que se comunican vía `stdin/stdout` con el IR. | Ecosistema multi-lenguaje, integración de herramientas existentes. |
| **Esquema IR** | `enum` de Rust cerrado. | `enum` híbrido: tipos core nativos + **variante `Custom`** para plugins. | Extensibilidad infinita sin recompilar el core. |
| **Creación de Reglas** | Todas las reglas en DSL Cedar-like. | **Estrategia de 3 Niveles:** Adaptadores, Reglas YAML declarativas y DSL para correlaciones complejas. | 10x más rápido para cubrir el 80% de las reglas. |
| **Estado y Gobernanza** | Motor 100% stateless. | CLI stateless + **Backend de Gobernanza Stateful Opcional**. | Habilita análisis de tendencias, baselining y dashboards. |
| **Experiencia DSL** | No especificada. | **Language Server Protocol (LSP)**, framework de tests de reglas y herramientas de debug del IR. | Reduce la curva de aprendizaje y fomenta la adopción. |

---

## 🏗️ 2. Arquitectura del Sistema (Revisada)

La arquitectura de pipeline se mantiene, pero se redefine el contrato de la **Etapa 1 (Extracción)** para desacoplarla del core.

```
┌──────────────────────────────────────────────────────────────────┐
│                       hodei-scan v3.3                            │
│                  Plataforma de Gobernanza                        │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│ ┌────────────────────────────────────────────────────────────┐  │
│ │ ETAPA 1: EXTRACCIÓN (Ecosistema Abierto y Multi-Lenguaje)  │  │
│ │                                                            │  │
│ │   ┌─────────────────┐   ┌──────────────────┐   ┌───────────┐   │
│ │   │ NIVEL 1:        │   │ NIVEL 2:         │   │ NIVEL 3:  │   │
│ │   │ ADAPTADORES     │   │ EXTRACTORES      │   │ EXTRACTORES│  │
│ │   │ (Integración)   │   │ DECLARATIVOS     │   │ PROFUNDOS │   │
│ │   ├─────────────────┤   ├──────────────────┤   ├───────────┤   │
│ │   │ • sarif-to-hodei│   │ • Motor          │   │ • Motor de│   │
│ │   │ • ruff-adapter  │   │   tree-sitter    │   │   Análisis│   │
│ │   │ • eslint-adapter│   │   (lee YAMLs)    │   │   de Flujo│   │
│ │   └─────────┬───────┘   └─────────┬────────┘   └──────┬────┘   │
│ │             │                     │                  │         │
│ │             └─────────────┼──────────────────────────┘         │
│ │                           ▼                                  │
│ │    [ Contrato: Proceso CLI que emite IR (Cap'n Proto) a stdout ]   │
│ │                                                                  │
│ └────────────────────────────────────────────────────────────┘  │
│                            │                                     │
│                            ▼                                     │
│ ┌────────────────────────────────────────────────────────────┐  │
│ │ ETAPA 1.5: AGREGACIÓN Y VALIDACIÓN DEL IR                  │  │
│ │ • hodei-scan core recibe IR de todos los extractores.      │  │
│ │ • Valida cada IR contra el Esquema (incluyendo plugins).   │  │
│ │ • Fusiona los IRs en un único `facts.capnp`.               │  │
│ └────────────────────────────────────────────────────────────┘  │
│                            │                                     │
│                            ▼                                     │
│ [ Etapas 2 (Carga/Indexación) y 3 (Evaluación) se mantienen como en v3.2 ]│
│                            │                                     │
│                            ▼                                     │
│ ┌────────────────────────────────────────────────────────────┐  │
│ │ ETAPA 6: PUBLICACIÓN (Opcional)                            │  │
│ │ • `hodei-scan publish` envía el IR y los Hallazgos         │  │
│ │   al Backend de Gobernanza (vía API REST/gRPC).            │  │
│ └────────────────────────────────────────────────────────────┘  │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

---

## 🧩 3. El Ecosistema de Extractores: La Estrategia de 3 Niveles

Esta es la mejora más crítica para garantizar una adopción rápida y una cobertura amplia. Se abandona el modelo de "solo plugins de Rust" en favor de una arquitectura de procesos desacoplada.

### 3.1. Contrato del Extractor

*   **Definición:** Un "Extractor" es cualquier programa ejecutable que se adhiere al siguiente contrato:
    1.  **Entrada:** Acepta una configuración JSON por `stdin` que incluye la ruta al proyecto y configuraciones específicas del extractor.
    2.  **Salida:** Escribe el `IntermediateRepresentation` en formato binario Cap'n Proto a `stdout`.
    3.  **Logs:** Escribe logs legibles por humanos a `stderr`.
    4.  **Estado:** Finaliza con un código de salida `0` en caso de éxito.
*   **Orquestación:** El CLI `hodei-scan` gestiona la ejecución de estos procesos, definidos en un fichero `hodei.toml`, y agrega sus salidas.

*   **Historia de Usuario (Implementación):**
    > *Como desarrollador del core, quiero que el CLI `hodei-scan` pueda leer un fichero `hodei.toml`, ejecutar los comandos de los extractores definidos, y agregar sus salidas de `stdout` en un único fichero IR, para poder orquestar un ecosistema de herramientas externas.*

### 3.2. Nivel 1: Adaptadores (Cobertura Instantánea)

*   **Objetivo:** Integrar herramientas de análisis estático líderes en el mercado que ya son rápidas y maduras.
*   **Implementación:**
    1.  **Extractor SARIF:** Crear un extractor genérico `sarif-to-hodei` que convierta informes en formato SARIF a nuestro IR. Esto proporciona compatibilidad inmediata con docenas de herramientas.
    2.  **Extractores Específicos:** Para herramientas de alto rendimiento que no soportan SARIF (ej. `Ruff`), crear adaptadores ligeros que traduzcan su salida JSON a nuestro IR.
*   **Historias de Usuario:**
    > *Como usuario de Python, quiero poder ejecutar `hodei-scan` y ver los resultados del linter `Ruff`, para aprovechar su velocidad y sus cientos de reglas dentro del ecosistema `hodei-scan`.*
    >
    > *Como desarrollador de la plataforma, quiero implementar un adaptador SARIF para que `hodei-scan` pueda importar resultados de cualquier herramienta compatible, como las de GitHub Advanced Security.*

### 3.3. Nivel 2: Extractores Declarativos (Democratización de Reglas)

*   **Objetivo:** Permitir a los usuarios escribir reglas de patrones de código de forma rápida y sencilla sin necesidad de programar.
*   **Implementación:**
    1.  **Motor de Patrones:** Construir un extractor genérico y multi-lenguaje basado en **tree-sitter**.
    2.  **Formato de Regla YAML:** Definir un formato simple en YAML para describir patrones de código. El motor leerá estos ficheros `.hodei.yml` y buscará coincidencias.
        ```yaml
        id: JAVA-EMPTY-CATCH-BLOCK
        language: java
        message: "Bloque catch vacío detectado. El error se está ignorando silenciosamente."
        severity: Major
        pattern: |
          try { ... } catch ($EXCEPTION e) {
            // Comentario opcional
          }
        ```
    3.  **Generación de Hechos:** Cuando el motor encuentra una coincidencia, genera un `Fact` apropiado (ej. `FactType::CodeSmell`).
*   **Historias de Usuario:**
    > *Como ingeniero de seguridad, quiero poder definir una regla para un nuevo "code smell" en un fichero YAML en menos de 5 minutos, sin tener que escribir código Rust ni recompilar nada, para poder reaccionar rápidamente a nuevos patrones de riesgo.*
    >
    > *Como desarrollador del core, quiero construir un motor basado en tree-sitter que pueda parsear ficheros de reglas YAML y ejecutar esas búsquedas de patrones de forma eficiente sobre el código fuente.*

### 3.4. Nivel 3: Extractores Profundos (Análisis de Vanguardia)

*   **Objetivo:** Para vulnerabilidades complejas que requieren análisis de flujo de datos (Taint Analysis), construir extractores especializados.
*   **Implementación:**
    1.  **Librería Core de Análisis (`hodei-taint-engine`):** Crear una librería en Rust que abstraiga la complejidad de construir Grafos de Flujo de Control y propagar el "taint".
    2.  **Extractores por Lenguaje:** Construir extractores profundos que *usen* esta librería. Su trabajo se simplifica a definir las "Fuentes", "Sumideros" y "Sanitizers" específicos de cada lenguaje (ej. Java).
*   **Historias de Usuario:**
    > *Como desarrollador del extractor de Java, quiero usar una librería `hodei-taint-engine` para no tener que implementar el algoritmo de propagación de taint desde cero, permitiéndome enfocar solo en definir las APIs de Java que son fuentes y sumideros de datos.*

---

## 📐 4. IR Schema y Plugins (Revisión)

Para soportar la extensibilidad sin fricción, el esquema del IR debe evolucionar.

### 4.1. Esquema Híbrido

*   **Definición:** El `enum FactType` se modifica para incluir una variante `Custom`.
    ```rust
    pub enum FactType {
        // ... variantes core (TaintSource, UncoveredLine, etc.)

        Custom {
            discriminant: String, // ej. "terraform::aws::insecure_s3_bucket"
            data: HashMap<String, FactValue>,
        },
    }
    ```
*   **Registro de Esquemas:** El `PluginRegistry` (que ahora existe a nivel conceptual, no de `trait`) es responsable de conocer los esquemas de los tipos `Custom`. Esta información se carga desde la configuración del plugin.
*   **Validación:** El motor `hodei-scan`, al agregar los IRs, valida que los hechos `Custom` se adhieran al esquema que su plugin ha declarado.
*   **Historia de Usuario:**
    > *Como desarrollador de un plugin para Terraform, quiero poder definir mis propios tipos de hechos (como `InsecureS3Bucket`) y sus campos (`acl`, `public_access_enabled`) sin pedir permiso al equipo de `hodei-scan`, para poder innovar de forma independiente.*

---

## 🏛️ 5. Backend de Gobernanza (Nueva Funcionalidad)

Para ir más allá del análisis puntual y ofrecer una verdadera plataforma de gobernanza.

### 5.1. Componentes

*   **`hodei-scan` CLI (Stateless):** Se mantiene rápido y sin estado para el CI/CD.
*   **`hodei-server` (Stateful):** Un nuevo servicio que proporciona una API para almacenar y consultar resultados de análisis a lo largo del tiempo.
*   **Base de Datos:** Una base de datos optimizada para analíticas, como TimescaleDB o ClickHouse.

### 5.2. Funcionalidades

1.  **Almacenamiento Histórico:** El comando `hodei-scan publish` envía los resultados al servidor.
2.  **Análisis de Tendencias:** APIs que permiten comparar resultados entre dos commits o ramas (ej. `GET /api/projects/{id}/diff?base=main&head=feature-branch`).
3.  **Baselining y Gestión de Deuda:** APIs para marcar hallazgos existentes como "aceptados" o "won't fix", para que no fallen los builds de nuevas funcionalidades.
4.  **Dashboards:** Una interfaz web (que consume la API) para visualizar métricas de calidad, seguridad y riesgo a lo largo del tiempo.

*   **Historias de Usuario:**
    > *Como líder de equipo, quiero ver un gráfico que muestre la evolución de vulnerabilidades críticas en la rama `main` durante los últimos 6 meses, para entender si nuestras iniciativas de seguridad están funcionando.*
    >
    > *Como desarrollador, cuando mi pipeline falla, quiero que solo me notifique de los nuevos problemas que he introducido en mi rama, ignorando la deuda técnica preexistente, para poder enfocarme en mi trabajo.*

---

## 💻 6. Experiencia de Desarrollador (DX) para el DSL

Para que el potente DSL de correlación sea adoptado, debe ser fácil de usar.

### 6.1. Language Server Protocol (LSP)

*   **Implementación:** Crear un servidor LSP para el DSL (`hodei-dsl-lsp`).
*   **Funcionalidades:**
    *   Autocompletado de tipos de hechos (core y custom) y sus campos.
    *   Validación de sintaxis en tiempo real.
    *   Documentación emergente al pasar el ratón.
*   **Historia de Usuario:**
    > *Como analista de seguridad, mientras escribo una regla en VS Code, quiero que el editor me sugiera los campos disponibles para un `FactType::TaintSink` para no tener que consultar la documentación constantemente.*

### 6.2. Framework de Tests de Reglas

*   **Implementación:** Un comando `hodei-scan test-rule` que toma una regla y un fichero de caso de prueba.
*   **Formato del Caso de Prueba:** Un fichero YAML que contiene un fragmento de código y los hallazgos esperados.
*   **Historia de Usuario:**
    > *Como desarrollador de reglas, quiero poder escribir tests unitarios para mis reglas, proporcionando un trozo de código que debería activarla y otro que no, para asegurar que mis reglas son precisas y no tienen falsos positivos.*

### 6.3. Herramientas de Debug

*   **Implementación:** Un comando `hodei-scan ir-dump` que convierte el IR binario de Cap'n Proto a un formato legible como JSON o YAML.
*   **Historia de Usuario:**
    > *Cuando una de mis reglas no se activa como esperaba, quiero poder inspeccionar el IR que han generado los extractores en un formato legible, para entender qué hechos están disponibles y depurar mi lógica de correlación.*

---

## 🗓️ 7. Plan de Implementación Sugerido (Roadmap de Alto Nivel)

### Fase 1: Fundación y Cobertura Masiva (Meses 1-3)
1.  **Core:** Implementar el contrato de extractores por proceso y el agregador.
2.  **Nivel 1:** Construir el adaptador `sarif-to-hodei` e integrar 2-3 linters rápidos (ej. Ruff, ESLint).
3.  **IR:** Actualizar el esquema del IR para soportar la variante `Custom`.

### Fase 2: Empoderamiento y Personalización (Meses 4-6)
1.  **Nivel 2:** Construir el motor declarativo con tree-sitter y el formato YAML.
2.  **DX:** Desarrollar el `ir-dump` y la primera versión del framework de tests de reglas.
3.  **Comunidad:** Portar 20-30 reglas populares de Semgrep al formato YAML para demostrar el poder del sistema.

### Fase 3: Gobernanza y Profundidad (Meses 7-12)
1.  **Backend:** Diseñar e implementar la v1 del `hodei-server` con almacenamiento histórico y APIs de diff.
2.  **Nivel 3:** Comenzar el desarrollo del `hodei-taint-engine` y el primer extractor profundo para un lenguaje clave (ej. Java).
3.  **DX:** Lanzar la v1 del LSP para el DSL.

Este documento proporciona una hoja de ruta estratégica y técnica para hacer de `hodei-scan` no solo una herramienta de análisis superior, sino una plataforma líder en su categoría.