# Architecture Decision Records

## ¿Qué es un ADR?

Un Architecture Decision Record (ADR) es un documento corto que captura una decisión arquitectónica importante, incluyendo el contexto, la decisión y sus consecuencias.

## Formato

Cada ADR debe seguir el formato:
- **Título**: Descripción breve de la decisión
- **Contexto**: Situación que requiere la decisión
- **Decisión**: Qué hemos decidido hacer
- **Alternativas**: Otras opciones consideradas
- **Consecuencias**: Resultados de la decisión (positivos, negativos, riesgos)

## Índice de Decisiones

| ADR | Título | Estado | Fecha |
|-----|--------|--------|-------|
| [ADR-001](./ADR-001-rust-language.md) | Elección de Rust como Lenguaje Principal | Aceptado | 2025-01-10 |
| [ADR-002](./ADR-002-hexagonal-architecture.md) | Arquitectura Hexagonal (Ports & Adapters) | Aceptado | 2025-01-10 |

## Estados Posibles

- **Propuesto**: Pendiente de discusión
- **Aceptado**: Decisión tomada y en vigor
- **Deprecated**: Reemplazado por una nueva decisión
- **Superseded**: Reemplazado por un ADR posterior

## Cómo Contribuir

Para proponer una nueva decisión arquitectónica:

1. Crea un nuevo archivo `ADR-XXX-descriptive-name.md`
2. Sigue el formato establecido
3. Presenta el ADR en la siguiente reunión de arquitectura
4. Una vez aceptado, actualiza este INDEX.md

## Referencias

- [Michael Nygard - ADRs](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions)
- [ADR GitHub Organization](https://adr.github.io/)
