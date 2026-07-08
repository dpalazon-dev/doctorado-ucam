# ResearchOS

## Document Contract

| Concern | Contract |
|---|---|
| **Level** | Level 4 — Navigation, Delivery and Process |
| **Normative status** | Repository front door; non-authoritative for system behavior |
| **Authoritative for** | Concise repository identity, current documentation map, links and current high-level status. |
| **Not authoritative for** | Product requirements, architectural decisions, domain semantics, implementation contracts or phase details. |
| **Required reading** | `VISION.md`, `DOCUMENTATION_ARCHITECTURE.md`, `ROADMAP.md`. |
| **Downstream documents** | New-contributor orientation and repository navigation. |

> Scope and conflict rules are defined in `DOCUMENTATION_ARCHITECTURE.md`.

---

> Sistema operativo personal para trabajo de conocimiento, investigación y continuidad operativa.

ResearchOS unifica conocimiento, proyectos, documentos, tareas, personas, recursos e inteligencia artificial dentro de un único estado coherente y gobernado.

---

## Visión

ResearchOS es un sistema operativo personal para la realidad completa del investigador: investigación, docencia, administración, colaboración, organización y trabajo cotidiano.

El objetivo no es almacenar información ni reunir herramientas desconectadas. Es mantener continuidad de contexto, conocimiento conectado y capacidad operativa a lo largo del tiempo y entre dominios.

Este repositorio constituye la fuente principal de documentación, arquitectura y desarrollo del proyecto.

> **Nota sobre nomenclatura:** `Doctorado_UCAM` es el nombre del repositorio y del proyecto. El sistema en ejecución que se construye —la plataforma en sí— se denomina **ResearchOS** en la documentación conceptual.

---

# Objetivos

- Mantener un único estado operativo coherente para todas las áreas de trabajo.
- Preservar contexto entre proyectos, documentos, tareas, personas, recursos y conocimiento.
- Reducir la carga cognitiva causada por herramientas y flujos fragmentados.
- Automatizar trabajo repetitivo sin ceder la autoridad del investigador.
- Incorporar inteligencia artificial como una capa operativa gobernada y reemplazable.
- Conservar trazabilidad, procedencia, historial y capacidad de recuperación.
- Validar la arquitectura mediante flujos reales antes de ampliar el sistema.

---

# Principios del proyecto

## El conocimiento es el activo principal

Toda la información debe poder localizarse, reutilizarse y relacionarse.
Guardar información por sí misma no sirve de nada si no tenemos un sistema para operar sobre ella.

## La automatización debe ahorrar tiempo

Las tareas repetitivas deberán automatizarse siempre que sea posible.

## La IA es una herramienta

Los modelos de IA deben ayudar al investigador, nunca sustituir el razonamiento científico.

## Todo debe ser reproducible

Las decisiones, experimentos y resultados deben quedar documentados.

## Arquitectura evolutiva

El sistema crecerá de forma incremental sin comprometer su mantenibilidad.

---

# Estructura del repositorio

Actualmente el repositorio contiene la documentación de arquitectura del proyecto en su fase de diseño. La estructura real es plana:

```
ResearchOS/

├── README.md
├── VISION.md
├── SYSTEM_PRINCIPLES.md
├── SYSTEM_MODEL.md
├── RESEARCHER_OPERATIONAL_MODEL.md
├── SYSTEM_RESPONSIBILITIES.md
├── DOMAIN_MAP.md
├── DOMAIN_MODEL.md
├── KNOWLEDGE_MODEL.md
├── SYSTEM_CAPABILITIES.md
├── USE_CASES.md
├── DOMAIN_VERTICALS.md
├── RESEARCH_VERTICAL.md
├── TEACHING_VERTICAL.md
├── ADMINISTRATION_VERTICAL.md
├── ORGANIZATION_VERTICAL.md
├── AI_ARCHITECTURE.md
├── MEMORY_MODEL.md
├── CONTEXT_MODEL.md
├── EVENT_MODEL.md
├── LOGICAL_DOMAIN_MODEL.md
├── INTERACTION_MODEL.md
├── SYSTEM_ARCHITECTURE.md
├── SOFTWARE_ARCHITECTURE.md
├── DOCUMENTATION_ARCHITECTURE.md
├── IMPLEMENTATION_CONTEXTS.md
├── IMPLEMENTATION_PLAN.md
├── ROADMAP.md
└── CLAUDE.md
```

La estructura modular por directorios (architecture/, platform/, ai/…) se introducirá cuando comience la fase de implementación.

---

# Arquitectura documental

La documentación ya no se trata como una única cadena que un agente deba cargar completa. Está gobernada mediante cuatro niveles y un propietario explícito para cada preocupación.

La especificación completa se encuentra en **[DOCUMENTATION_ARCHITECTURE.md](DOCUMENTATION_ARCHITECTURE.md)**. Para tareas de implementación, **[IMPLEMENTATION_CONTEXTS.md](IMPLEMENTATION_CONTEXTS.md)** define qué documentos y secciones debe cargar un coding agent.

## Nivel 1 · Fundamentos canónicos

Definen el núcleo estable y las reglas globales.

- [Vision](VISION.md)
- [System Principles](SYSTEM_PRINCIPLES.md)
- [System Architecture](SYSTEM_ARCHITECTURE.md)
- [Logical Domain Model](LOGICAL_DOMAIN_MODEL.md)
- [Software Architecture](SOFTWARE_ARCHITECTURE.md)

## Nivel 2 · Modelos especializados

Cada documento es canónico únicamente dentro de una preocupación delimitada.

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

## Nivel 3 · Especificaciones funcionales y verticales

Definen vocabulario de comportamiento, casos de uso y amplitud del dominio.

- [System Capabilities](SYSTEM_CAPABILITIES.md)
- [Use Cases](USE_CASES.md)
- [Domain Verticals](DOMAIN_VERTICALS.md)
- [Research Vertical](RESEARCH_VERTICAL.md)
- [Teaching Vertical](TEACHING_VERTICAL.md)
- [Administration Vertical](ADMINISTRATION_VERTICAL.md)
- [Organization Vertical](ORGANIZATION_VERTICAL.md)

## Nivel 4 · Navegación, entrega y proceso

Orientan el trabajo, pero no introducen comportamiento ni arquitectura del sistema.

- [README](README.md)
- [Roadmap](ROADMAP.md)
- [Implementation Plan](IMPLEMENTATION_PLAN.md)
- [Claude Code guidance](CLAUDE.md)
- [Documentation Architecture](DOCUMENTATION_ARCHITECTURE.md)
- [Implementation Contexts](IMPLEMENTATION_CONTEXTS.md)

## Regla de autoridad

Cada decisión tiene un único documento propietario. Los demás documentos pueden resumirla, aplicarla o derivar consecuencias, pero no redefinirla. Cada archivo declara al principio:

- de qué es autoridad;
- de qué no es autoridad;
- qué lectura requiere antes de modificarlo;
- qué documentos deben revisarse si cambia.

Un coding agent debe seleccionar un *context bundle* en `IMPLEMENTATION_CONTEXTS.md`; no debe leer todos los Markdown por defecto.

---

# Verticales de dominio

Las siete Core Entities se especializan en seis verticales operativas. Cuatro tienen entidad suficiente para contar con su propio documento; dos permanecen deliberadamente ligeras dentro del índice.

- [Domain Verticals](DOMAIN_VERTICALS.md) — índice de las seis
- [Research Vertical](RESEARCH_VERTICAL.md)
- [Teaching Vertical](TEACHING_VERTICAL.md)
- [Administration Vertical](ADMINISTRATION_VERTICAL.md)
- [Organization Vertical](ORGANIZATION_VERTICAL.md)

Personal y Daily Work se documentan de forma inline dentro de [DOMAIN_VERTICALS.md](DOMAIN_VERTICALS.md).

---

# Áreas funcionales

El proyecto se divide en varios dominios principales:

- Gestión documental
- Gestión bibliográfica
- Base de conocimiento
- Investigación
- Escritura científica
- Automatización
- Inteligencia Artificial
- Infraestructura

Cada dominio evolucionará de forma independiente siguiendo una arquitectura modular.

---

# Hoja de ruta

El plan detallado y vigente vive en **[ROADMAP.md](ROADMAP.md)**. En resumen, dos fases:

## Fase A · Completar el dominio — Completada

Las seis verticales operativas —personal, vida diaria, investigación, docencia, administración y organización— ya cuentan con *Derived Types* y casos de uso, sin introducir nuevas entidades raíz. Las siete entidades del dominio sostienen las seis capas de la vida del investigador. Detalle en [ROADMAP.md](ROADMAP.md) y [DOMAIN_VERTICALS.md](DOMAIN_VERTICALS.md).

## Fase B · Construir el sistema

Materializar la arquitectura de referencia ya definida: derivar las vistas de implementación, registrar las decisiones arquitectónicas, seleccionar la arquitectura técnica y validar el sistema con una rebanada vertical de extremo a extremo.

---

# Estado del proyecto

El proyecto ha completado la arquitectura conceptual, la arquitectura del sistema y la arquitectura software de referencia. El siguiente paso es derivar las vistas de implementación, registrar las decisiones mediante ADRs y seleccionar la arquitectura técnica antes de construir la primera rebanada vertical.

El objetivo inicial de implementación es validar que la arquitectura puede operar sobre datos reales sin perder autoridad del dominio, trazabilidad, control humano ni capacidad de recuperación.

---

# Licencia

Pendiente de definir.
