## **Hodei-Scan (Parte 2): De la Teoría a la Práctica - Tu Agencia de Inteligencia en Acción**

En la primera parte, presentamos a hodei-scan como tu propia agencia de inteligencia para código. Hablamos de agentes de campo, una central de análisis y la diferencia entre ver "problemas" y entender el "riesgo".

Ahora, vamos a bajar a la tierra. ¿Cómo funciona esto en el día a día? ¿Cómo puedes, como desarrollador o líder de equipo, usar este poder para construir mejor software?

### La Anatomía de una Alerta Inteligente

Recordemos nuestra alerta mágica: *"Se encontró una contraseña justo en un fichero que no tiene tests, que fue modificado ayer y que pertenece al módulo de pagos."*

¿Cómo llega hodei-scan a esta conclusión? No es magia, es un proceso de 3 pasos: **Observar, Correlacionar y Decidir.**

```mermaid
flowchart LR
    subgraph Paso 1: Observar (Los Agentes Reportan)
        A["Agente de Seguridad (Nivel 2)<br/>Encuentra 'db_pass = ...' en `payments.py`"] --> B(Hecho 1: HardcodedSecret)
        C["Agente de Tests (Nivel 1)<br/>Adapta informe de Cobertura"] --> D(Hecho 2: UncoveredLine en `payments.py`)
        E["Agente de Git (Plugin)<br/>Analiza 'git blame'"] --> F(Hecho 3: GitCommitInfo<br/>`payments.py` modificado ayer)
        G["Agente de Propiedad (Nivel 2)<br/>Lee fichero CODEOWNERS"] --> H(Hecho 4: CodeOwner<br/>`payments.py` pertenece a 'equipo-pagos')
    end

    subgraph Paso 2: Correlacionar (Los Analistas Conectan los Puntos)
        I[Sala de Estrategia: Tu DSL de Políticas]
        B & D & F & H --> I
    end

    subgraph Paso 3: Decidir (El Director Actúa)
        J[Finding: Alerta de Riesgo Máximo]
        I --> J
    end

    style A fill:#f9f,stroke:#333,stroke-width:2px
    style C fill:#f9f,stroke:#333,stroke-width:2px
    style E fill:#f9f,stroke:#333,stroke-width:2px
    style G fill:#f9f,stroke:#333,stroke-width:2px
```

#### Paso 1: Observar - Los Agentes Rellenan sus Formularios

Cuando hodei-scan se ejecuta, todos los agentes se despliegan y empiezan a reportar hechos simples y objetivos:

*   **Agente de Seguridad:** "He encontrado algo que parece una contraseña en el fichero `payments.py`, línea 42". → Genera un `Fact: HardcodedSecret`.
*   **Agente de Tests:** "He leído el informe de cobertura y la línea 42 de `payments.py` no fue ejecutada por ningún test". → Genera un `Fact: UncoveredLine`.
*   **Agente de Git:** "He consultado el historial de Git y la línea 42 de `payments.py` fue modificada ayer por 'dev@acme.com'". → Genera un `Fact: GitCommitInfo`.
*   **Agente de Propiedad:** "He leído el fichero `CODEOWNERS` y el fichero `payments.py` pertenece al '@equipo-pagos'". → Genera un `Fact: CodeOwner`.

En este punto, solo tenemos una lista de observaciones inconexas.

#### Paso 2: Correlacionar - La Sala de Estrategia se Activa

Aquí es donde entra en juego tu **DSL de Políticas**. Un analista de seguridad de tu empresa ha escrito una regla que define lo que significa "riesgo crítico de negocio":

```cedar
// Archivo: politicas/riesgo-critico.hodei
forbid(
  rule: "BUSINESS-CRITICAL-RISK-001",
  severity: "Blocker",
  description: "Se ha detectado un secreto expuesto en código de pagos sensible, sin cobertura de tests y modificado recientemente."
) on {
  // Condición 1: Hay un secreto expuesto...
  exists(Fact { type: "HardcodedSecret", file: $fichero_sensible, line: $linea_sensible }) &&

  // Condición 2: ...en un fichero que pertenece al equipo de pagos...
  exists(Fact { type: "CodeOwner", file_pattern: $fichero_sensible, owner_team: "equipo-pagos" }) &&

  // Condición 3: ...en una línea sin tests...
  exists(Fact { type: "UncoveredLine", file: $fichero_sensible, line: $linea_sensible }) &&

  // Condición 4: ...que fue modificada en los últimos 3 días.
  exists(Fact { type: "GitCommitInfo", file: $fichero_sensible, line: $linea_sensible, age_in_days < 3 })
}
```

El motor de hodei-scan coge esta regla y la usa como un plano para buscar conexiones en la lista de hechos. Gracias a las variables (`$fichero_sensible`, `$linea_sensible`), se asegura de que todos los hechos se refieren **al mismo punto exacto del código**.

#### Paso 3: Decidir - Se Genera el Hallazgo (Finding)

El motor encuentra que los cuatro hechos coinciden con el patrón de la regla. ¡Bingo! Se genera un **Hallazgo (Finding)** de severidad `Blocker`. Este hallazgo no es solo una observación, es una **conclusión inteligente** que contiene referencias a los cuatro hechos que la demuestran.

Cuando el CI/CD recibe este hallazgo, el Quality Gate (el "Director") dice: "Esto es un Blocker. Detengan el despliegue. Y llamen al jefe del equipo de pagos".

### Tú Tienes el Poder: Un Ecosistema Abierto

"Todo esto suena genial", podrías pensar, "pero ¿tengo que esperar a que el equipo de hodei-scan construya todos esos agentes para mi lenguaje o mis herramientas?"

**¡NO!** Y esta es la segunda idea más importante de hodei-scan: **la plataforma es abierta por diseño.**

#### ¿Quieres Soportar una Nueva Herramienta? ¡Construye un Adaptador!

Imagina que tu equipo usa una herramienta de análisis de infraestructura increíble llamada `TerraScan`. Quieres que sus resultados participen en la correlación.

1.  **Ejecutas `TerraScan`:** `terrascan -f json > terrascan_report.json`
2.  **Escribes un Adaptador:** Creas un pequeño script (en Python, Go, Rust, ¡lo que quieras!) llamado `terrascan-adapter`. Su único trabajo es leer `terrascan_report.json` y convertir cada problema en un **Hecho Atómico** de hodei-scan (probablemente de tipo `IaCSecurityMisconfiguration`).
3.  **Lo Añades a tu Configuración:**
    ```toml
    # hodei.toml
    [[extractors]]
    name = "Terraform Security"
    command = "./terrascan-adapter"
    ```
¡Listo! Acabas de integrar `TerraScan` en tu agencia de inteligencia. Sus informes ahora pueden ser correlacionados con la cobertura de tests, el historial de Git y todo lo demás.

#### ¿Tienes una Regla Específica de tu Empresa? ¡Escríbela en YAML!

Tu empresa tiene una regla interna: "Nunca se debe llamar a la función `legacy_billing_api()`".

No tienes que pedirle a nadie que implemente esto. Simplemente creas un fichero:

```yaml
# reglas_internas/no-legacy-billing.hodei.yml
id: ACME-DEPRECATED-001
language: python
message: "Se está usando la API de facturación legacy. Por favor, migra a `new_billing_service()`."
severity: Major
pattern: |
  legacy_billing_api(...)
```
Lo guardas en tu repositorio y el "Agente de Vigilancia por Satélite" de hodei-scan lo detectará automáticamente y empezará a reportar `Hechos` sobre su uso.

### Conclusión: El Futuro de la Gobernanza de Código

Hodei-scan representa un cambio de mentalidad:

*   **De la Cobertura a la Correlación:** Dejamos de obsesionarnos con métricas aisladas y empezamos a entender cómo interactúan.
*   **De un Monolito a un Ecosistema:** Dejamos de depender de un único proveedor y empezamos a construir una plataforma abierta que integra las mejores herramientas para cada trabajo.
*   **De Reglas Genéricas a Inteligencia Contextual:** Dejamos de aplicar las mismas reglas a todo el mundo y empezamos a crear políticas que entienden el contexto de nuestro negocio.

No estamos construyendo un linter más rápido. Estamos construyendo una plataforma para tomar decisiones de ingeniería más inteligentes.

**¿Estás listo para montar tu propia agencia de inteligencia?**