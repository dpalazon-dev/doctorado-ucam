# ResearchOS

## Document Contract

| Concern | Contract |
|---|---|
| **Level** | Level 4 — Navigation, Delivery and Process |
| **Normative status** | Repository front door; non-authoritative for system behavior |
| **Authoritative for** | Concise repository identity, current documentation map, links and current high-level status. |
| **Not authoritative for** | Product requirements, architectural decisions, domain semantics, implementation contracts or phase details. |
| **Required reading** | `VISION.md`, `DOCUMENTATION_ARCHITECTURE.md`, `ROADMAP.md`, `SPEC_CATALOG.md`. |
| **Downstream documents** | New-contributor orientation and repository navigation. |

> Scope and conflict rules are defined in `DOCUMENTATION_ARCHITECTURE.md`.

---

> Sistema operativo personal, local-first y gobernado para trabajo de conocimiento, investigación y continuidad operativa.

ResearchOS unifica conocimiento, proyectos, documentos, tareas, actividades, personas, recursos e inteligencia artificial dentro de un único estado coherente.

## Objetivo del producto

ResearchOS se distribuye como una aplicación de escritorio instalable para Windows, macOS y Linux. La primera versión está orientada a una sola persona y mantiene localmente su estado canónico, documentos y proyecciones.

La baseline técnica actual es:

```text
Tauri + React + TypeScript
        ↓
runtime autoritativo en Rust
        ↓ IPC versionado
sidecar cognitivo en Python
        ↓
SQLite + filesystem local + proyecciones reconstruibles
```

Los proveedores remotos de IA son adapters opcionales. No constituyen la autoridad del sistema ni son necesarios para iniciar la aplicación.

---

# Estado actual

La arquitectura conceptual, lógica, de sistema, software, componentes, datos, ejecución cognitiva y tecnología está cerrada para comenzar desarrollo. `CANONICAL_DATA_MODEL.md` define la forma estructural común que deberán extender las Specs verticales.

La fase actual es **Development Specifications**. El siguiente trabajo es elaborar e implementar las Specs registradas en [SPEC_CATALOG.md](SPEC_CATALOG.md), comenzando por la fundación del repositorio y del producto desktop.

No existe todavía una implementación de producción. El código solo debe introducirse contra una Spec aprobada o como experimento explícitamente delimitado por `IMPLEMENTATION_PLAN.md`.

---

# Arquitectura documental

## Nivel 1 · Fundamentos canónicos

- [Vision](VISION.md)
- [System Principles](SYSTEM_PRINCIPLES.md)
- [System Architecture](SYSTEM_ARCHITECTURE.md)
- [Logical Domain Model](LOGICAL_DOMAIN_MODEL.md)
- [Software Architecture](SOFTWARE_ARCHITECTURE.md)

## Nivel 2 · Modelos especializados

- [System Model](SYSTEM_MODEL.md)
- [Researcher Operational Model](RESEARCHER_OPERATIONAL_MODEL.md)
- [System Responsibilities](SYSTEM_RESPONSIBILITIES.md)
- [Domain Map](DOMAIN_MAP.md)
- [Domain Model](DOMAIN_MODEL.md)
- [Knowledge Model](KNOWLEDGE_MODEL.md)
- [AI Architecture](AI_ARCHITECTURE.md)
- [Memory Model](MEMORY_MODEL.md)
- [Context Model](CONTEXT_MODEL.md)
- [Event Model](EVENT_MODEL.md)
- [Interaction Model](INTERACTION_MODEL.md)
- [Component Model](COMPONENT_MODEL.md)
- [Canonical Data Model](CANONICAL_DATA_MODEL.md)
- [Data Architecture](DATA_ARCHITECTURE.md)
- [Agent Runtime](AGENT_RUNTIME.md)
- [Technical Architecture](TECHNICAL_ARCHITECTURE.md)

## Nivel 3 · Especificaciones funcionales, verticales y de desarrollo

- [System Capabilities](SYSTEM_CAPABILITIES.md)
- [Use Cases](USE_CASES.md)
- [Domain Verticals](DOMAIN_VERTICALS.md)
- [Research Vertical](RESEARCH_VERTICAL.md)
- [Teaching Vertical](TEACHING_VERTICAL.md)
- [Administration Vertical](ADMINISTRATION_VERTICAL.md)
- [Organization Vertical](ORGANIZATION_VERTICAL.md)
- Futuras Specs bajo `specs/`, gobernadas por [SPEC_CATALOG.md](SPEC_CATALOG.md)

## Nivel 4 · Navegación, entrega y proceso

- [Roadmap](ROADMAP.md)
- [Architecture Decisions](DECISIONS.md)
- [Specification Catalog](SPEC_CATALOG.md)
- [Implementation Plan](IMPLEMENTATION_PLAN.md)
- [Documentation Architecture](DOCUMENTATION_ARCHITECTURE.md)
- [Implementation Contexts](IMPLEMENTATION_CONTEXTS.md)
- [Claude Code Guidance](CLAUDE.md)

La regla de autoridad es simple: cada preocupación tiene un único propietario. Un documento downstream puede aplicar una regla, pero no redefinirla.

---

# Secuencia de desarrollo

```text
SPEC-001 Repository and Build System
        ↓
fundación desktop y runtime local
        ↓
almacenamiento canónico y jobs duraderos
        ↓
IPC Rust–Python y sidecar cognitivo
        ↓
ingesta Document → Knowledge
        ↓
retrieval, propuestas y Workspace
        ↓
primer vertical Research
```

El detalle, las dependencias y el estado de cada Spec viven exclusivamente en [SPEC_CATALOG.md](SPEC_CATALOG.md). El orden de fases y milestones vive en [ROADMAP.md](ROADMAP.md).

---

# Verticales

Las siete Core Entities se especializan en seis verticales operativas:

1. Personal;
2. Daily Work;
3. Administration;
4. Teaching;
5. Research;
6. Organization.

No existen modelos de datos independientes por vertical. Todos extienden el mismo modelo canónico mediante Derived Types, perfiles tipados, relaciones, invariantes y proyecciones.

---

# Reglas para contribuir

- Leer [CLAUDE.md](CLAUDE.md), [ROADMAP.md](ROADMAP.md) y [SPEC_CATALOG.md](SPEC_CATALOG.md).
- Seleccionar un bundle en [IMPLEMENTATION_CONTEXTS.md](IMPLEMENTATION_CONTEXTS.md).
- Implementar solo dentro de una Spec aprobada o de un experimento autorizado.
- Mantener autoridad, procedencia, aprobación humana y proyecciones reconstruibles.
- Registrar mediante ADR cualquier cambio arquitectónico significativo.

---

# Licencia

Pendiente de definir.
