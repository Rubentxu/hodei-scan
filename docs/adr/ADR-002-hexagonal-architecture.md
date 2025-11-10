# ADR-002: Arquitectura Hexagonal (Ports & Adapters)

## Contexto

hodei-scan v3.0 necesita una arquitectura que permita:
- Múltiples fuentes de análisis (extractors)
- Múltiples formatos de salida
- Facilidad de testing con mocks
- Escalabilidad para nuevos lenguajes y herramientas
- Separación clara entre lógica de negocio e infraestructura

La arquitectura debe soportar la evolución del sistema sin requerir refactorización masiva.

## Decisión

**Adoptamos Arquitectura Hexagonal (Ports & Adapters)** para organizar el código en capas claramente definidas:

- **Domain (Core)**: Lógica de negocio pura sin dependencias externas
- **Application**: Casos de uso que coordinan la lógica de dominio
- **Infrastructure**: Adaptadores para herramientas externas (parsers, databases, etc.)

## Alternativas Consideradas

### Opción 1: Layered Architecture (tradicional)

**Pros:**
- Familiar para la mayoría de desarrolladores
- Estructura clara (presentation, business logic, data access)
- Documentación extensa y ejemplos abundantes

**Contras:**
- Acoplamiento fuerte entre capas
- Difícil de testear sin herramientas de mocking
- La capa de presentación puede filtrarse a las capas inferiores
- Cambios en infraestructura afectan la lógica de negocio

### Opción 2: Microservices

**Pros:**
- Escalabilidad independiente por servicio
- Tecnologías diversas por servicio
- Deployment independiente

**Contras:**
- Overhead operacional significativo
- Complejidad de comunicación entre servicios
- Transacciones distribuidas difíciles
- Demasiado complejo para un monolito modular
- Latencia de red entre servicios

## Consecuencias

### Positivas

1. **Testabilidad**: Interfaces claras permiten mocking fácil
2. **Flexibilidad**: Cambiar adaptadores sin afectar la lógica de negocio
3. **Separation of Concerns**: Cada módulo tiene una responsabilidad clara
4. **Extensibilidad**: Agregar nuevos extractors o formatters sin modificar core
5. **Independencia de Framework**: El dominio no depende de librerías externas
6. **Claridad Arquitectónica**: Móduloscrates reflejan la separación de capas

### Negativas

1. **Curva de Aprendizaje**: Concepto de ports & adapters puede ser nuevo
2. **Boilerplate**: Interfaces adicionales requieren más código
3. **Complejidad Inicial**: Más estructura que un monolito simple

## Estructura Propuesta

```
crates/
├── hodei-ir/          # Domain: Tipos core y entidades
├── hodei-engine/      # Application: Casos de uso y reglas
├── hodei-dsl/         # Application: Parser para DSL
├── hodei-extractors/  # Infrastructure: Adaptadores para parsers
└── hodei-cli/         # Presentation: Interface de línea de comandos
```

## Referencias

- [Hexagonal Architecture by Alistair Cockburn](https://alistair.cockburn.us/hexagonal-architecture/)
- [Ports & Adapters Pattern](https://codurance.com/2015/05/12/ports-and-adapters/)
- [Clean Architecture by Robert C. Martin](https://8thlight.com/blog/uncle-bob/2012/08/13/the-clean-architecture.html)
