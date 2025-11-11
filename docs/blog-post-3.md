## **Hodei-Scan (Parte 3): El Manual de Campo del Agente - Expandiendo tu Inteligencia**

En las partes anteriores, establecimos nuestra misión: construir una agencia de inteligencia para nuestro código. Vimos cómo el motor de hodei-scan actúa como una central que conecta los informes de sus agentes de campo.

Pero, ¿qué tipo de información pueden recoger estos agentes? ¿Y cómo puedes, como desarrollador, convertirte en un agente y empezar a enviar tus propios informes?

### El Universo de la Inteligencia: El Catálogo de Hechos

El poder de nuestra agencia reside en la variedad y riqueza de la información que recoge. No solo buscamos "bugs" o "vulnerabilidades". Buscamos cualquier pieza de información objetiva sobre nuestro ecosistema de software que pueda ser relevante.

A esto lo llamamos el **Catálogo de Hechos Atómicos**. Es el "vocabulario" que usan nuestros agentes. Aquí tienes una muestra del tipo de inteligencia que recogemos:

*   **🛡️ Inteligencia de Seguridad (SAST):**
    *   `TaintSource`: ¿Entraron datos del enemigo (usuario) por aquí?
    *   `TaintSink`: ¿Estos datos llegaron a un punto vital (base de datos, terminal) sin ser verificados?
    *   `HardcodedSecret`: ¿Hay una "llave maestra" (contraseña) tirada a la vista?
    *   `CryptographicOperation`: ¿Se está usando un código de encriptación obsoleto de la Guerra Fría (MD5)?

*   **📦 Inteligencia de Suministros (SCA):**
    *   `DependencyVulnerability`: ¿Alguno de nuestros proveedores de armas (librerías externas) tiene un defecto de fábrica conocido (un CVE)?
    *   `License`: ¿El contrato con nuestro proveedor de armas nos obliga a hacer público todo nuestro arsenal (una licencia GPL)?

*   **🏛️ Inteligencia de Infraestructura (IaC):**
    *   `IaCSecurityMisconfiguration`: ¿Está la puerta principal de nuestra base (un bucket S3) abierta al público?
    *   `OverprovisionedResource`: ¿Estamos pagando por un tanque para vigilar un aparcamiento de bicicletas (un servidor sobredimensionado)?

*   **✅ Inteligencia de Preparación (Testing):**
    *   `UncoveredLine`: ¿Hay algún pasillo en nuestra base que los guardias nunca patrullan (código sin tests)?
    *   `FlakyTest`: ¿Tenemos un guardia que a veces se duerme en su puesto (un test inestable)?

*   **🔭 Inteligencia de Campo (Runtime y Observabilidad):**
    *   `ProductionError`: ¿Estamos recibiendo informes de que una de nuestras defensas está fallando **ahora mismo** en el campo de batalla (un error en producción)?
    *   `PerformanceHotspot`: ¿Se está formando un atasco monumental en la puerta oeste de nuestra base cada día a las 5 PM (un cuello de botella de rendimiento)?

Y muchos, muchos más, cubriendo desde el pipeline de CI/CD hasta la documentación. Cada uno de estos "hechos" es una pieza de un puzzle gigante. Por sí solos pueden no significar mucho. Juntos, revelan la imagen completa del riesgo.

### Tu Primera Misión: Cómo Construir un Extractor desde Cero

Ahora, la parte más emocionante. No tienes que ser un empleado de la "central" de hodei-scan para contribuir. Cualquiera puede convertirse en un agente de campo.

Vamos a construir un extractor simple pero increíblemente útil desde cero.

**La Misión:** Nuestra agencia necesita identificar código que podría ser difícil de mantener en el futuro. Una señal clásica de esto son los comentarios `TODO` o `FIXME` que se quedan olvidados durante meses. Queremos un agente que los encuentre.

**Nombre del Agente:** `todo-extractor`
**Lenguaje del Agente:** Vamos a usar **Python**, para demostrar que no necesitas saber Rust.

#### Paso 1: El Plan del Agente

Nuestro agente `todo-extractor.py` hará lo siguiente:
1.  Recibirá la ruta del proyecto que tiene que analizar.
2.  Recorrerá todos los ficheros de código (ej. `.py`, `.java`, `.js`).
3.  En cada fichero, leerá línea por línea.
4.  Si una línea contiene "TODO" o "FIXME", generará un **informe de Hecho Atómico**.
5.  Imprimirá todos estos informes en la consola en el formato estándar de hodei-scan (JSON en este ejemplo, aunque en la realidad sería Cap'n Proto).

#### Paso 2: El Código del Agente (`todo-extractor.py`)

```python
import os
import sys
import json

# Definimos el tipo de hecho que vamos a generar
FACT_TYPE = "TODOComment"

def scan_file(file_path, project_root):
    """Escanea un único fichero en busca de comentarios TODO/FIXME."""
    facts = []
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            for line_number, line_content in enumerate(f, 1):
                if "TODO" in line_content or "FIXME" in line_content:
                    # ¡Hemos encontrado algo! Preparamos el informe (el Hecho).
                    message = line_content.strip()
                    relative_path = os.path.relpath(file_path, project_root)

                    fact = {
                        "fact_type": {
                            "discriminant": FACT_TYPE,
                            "data": {
                                "message": message
                            }
                        },
                        "location": {
                            "file": relative_path,
                            "line": line_number
                        }
                        # Hodei-scan añadirá otros metadatos como el ID y el timestamp.
                    }
                    facts.append(fact)
    except Exception:
        # Ignoramos ficheros que no podemos leer
        pass
    return facts

def main():
    """Punto de entrada del extractor."""
    # Hodei-scan nos pasará la ruta del proyecto como argumento.
    # En una implementación real, esto vendría por stdin como JSON.
    project_root = sys.argv[1]
    all_facts = []

    for root, _, files in os.walk(project_root):
        for file in files:
            # Solo escaneamos extensiones de código comunes
            if file.endswith(('.py', '.js', '.java', '.go', '.rs')):
                file_path = os.path.join(root, file)
                facts_from_file = scan_file(file_path, project_root)
                all_facts.extend(facts_from_file)

    # El contrato final: imprimimos la lista de hechos a stdout en formato JSON.
    # El motor de hodei-scan recogerá esta salida.
    print(json.dumps({"facts": all_facts}))

if __name__ == "__main__":
    main()
```

#### Paso 3: Desplegando a Nuestro Nuevo Agente

1.  **Guardamos el script** como `todo-extractor.py` y lo hacemos ejecutable.
2.  **Registramos al agente** en la configuración de hodei-scan, para que la central sepa que existe:

    ```toml
    # hodei.toml
    [[extractors]]
    name = "Buscador de TODOs"
    # El comando que la central ejecutará. Pasa la ruta del proyecto como argumento.
    command = "python3 ./extractors/todo-extractor.py ${project_root}"
    ```

¡Y ya está! Acabas de expandir las capacidades de tu agencia de inteligencia. Ahora, cuando ejecutes `hodei-scan`, este nuevo agente se activará y su informe se incluirá en el análisis global.

#### Paso 4: Usando la Nueva Inteligencia

Ahora, un líder de equipo puede ir a la "Sala de Estrategia" y escribir una política que use esta nueva información:

```cedar
// politica/deuda-tecnica.hodei
forbid(
  rule: "TECH-DEBT-STALE-TODO",
  severity: "Minor",
  description: "Se encontró un comentario TODO en código que no ha sido modificado en más de 180 días."
) on {
  // Encuentra un comentario TODO en un fichero y línea
  exists(Fact { type: "TODOComment", file: $f, line: $l }) &&

  // Y correlaciónalo con la información de Git para ver si ese código es antiguo
  exists(Fact { type: "GitCommitInfo", file: $f, line: $l, age_in_days > 180 })
}
```
Esta es una regla que antes era imposible. Acabamos de darle a nuestra agencia la capacidad de **distinguir entre deuda técnica reciente y deuda técnica olvidada y peligrosa.**

### El Futuro es Abierto

Este ejemplo simple es solo el principio. Imagina a la comunidad construyendo agentes para:
*   Analizar la complejidad de las consultas de GraphQL.
*   Detectar sesgos en los datasets de Machine Learning.
*   Verificar que la documentación esté sincronizada con el código.
*   Analizar la factura de la nube y sugerir ahorros de costes.

Hodei-scan no es solo un producto; es una **plataforma**. No te damos solo el pescado; te damos la mejor caña de pescar del mundo y te invitamos a ti y a toda la comunidad a pescar juntos.

**La pregunta ya no es qué puede hacer hodei-scan por ti, sino qué inteligencia increíble podemos descubrir juntos.**