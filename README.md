# ResearchOS

> Plataforma de investigación, gestión del conocimiento e inteligencia artificial para el desarrollo del doctorado.

Construyendo una plataforma de investigación donde el conocimiento, la automatización y la inteligencia artificial trabajan juntos para acelerar el proceso científico.

---

## Visión

ResearchOS es una plataforma diseñada para centralizar todo el conocimiento generado durante el doctorado, facilitar la investigación científica y automatizar tareas mediante inteligencia artificial.

El objetivo no es únicamente almacenar información, sino construir un sistema capaz de organizar, relacionar y explotar el conocimiento científico a lo largo de todo el proceso investigador.

Este repositorio constituye la fuente principal de documentación, arquitectura y desarrollo del proyecto.

> **Nota sobre nomenclatura:** `Doctorado_UCAM` es el nombre del repositorio y del proyecto. El sistema en ejecución que se construye —la plataforma en sí— se denomina **ResearchOS** en la documentación conceptual.

---

# Objetivos

- Centralizar toda la documentación del doctorado.
- Gestionar artículos científicos, notas y bibliografía.
- Diseñar una base de conocimiento estructurada.
- Automatizar procesos de investigación.
- Incorporar herramientas basadas en IA.
- Facilitar la escritura de la tesis y publicaciones.
- Mantener la trazabilidad de todas las decisiones tomadas durante el proyecto.

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
└── ROADMAP.md
```

La estructura modular por directorios (architecture/, platform/, ai/…) se introducirá cuando comience la fase de implementación.

---

# Documentos de arquitectura

Los documentos de arquitectura trabajan a distintos niveles conceptuales y deben leerse en el siguiente orden. Cada uno depende conceptualmente del anterior:

```
Vision
    ↓
Principles
    ↓
System Model
    ↓
Operational Model
    ↓
Responsibilities
    ↓
Domain Map
    ↓
Domain Model
    ↓
Knowledge Model
    ↓
System Capabilities
    ↓
Use Cases
    ↓
AI Architecture ─┬─ Memory Model
                 ├─ Context Model
                 └─ Event Model
    ↓
Logical Domain Model
    ↓
Interaction Model
```

Los documentos 11–14 conforman la **Arquitectura Cognitiva**: la capa operativa de IA y los tres modelos del sustrato sobre el que opera — lo que el sistema recuerda (Memory), lo que ensambla por tarea (Context) y aquello a lo que reacciona (Events).

El **Logical Domain Model** es la especificación lógica del dominio: la definición canónica y neutral respecto a la tecnología de su estructura, de la que se derivan todos los esquemas, diagramas y tipos.

El **Interaction Model** cierra la documentación conceptual (la cadena 1–16): cómo colaboran el investigador y el sistema, con independencia de cualquier tecnología de interfaz. Es, además, el primer paso de la Fase B y precede a la Software Architecture.

1. [Vision](VISION.md)
2. [Principles](SYSTEM_PRINCIPLES.md)
3. [System Model](SYSTEM_MODEL.md)
4. [Operational Model](RESEARCHER_OPERATIONAL_MODEL.md)
5. [Responsibilities](SYSTEM_RESPONSIBILITIES.md)
6. [Domain Map](DOMAIN_MAP.md)
7. [Domain Model](DOMAIN_MODEL.md)
8. [Knowledge Model](KNOWLEDGE_MODEL.md)
9. [System Capabilities](SYSTEM_CAPABILITIES.md)
10. [Use Cases](USE_CASES.md)
11. [AI Architecture](AI_ARCHITECTURE.md)
12. [Memory Model](MEMORY_MODEL.md)
13. [Context Model](CONTEXT_MODEL.md)
14. [Event Model](EVENT_MODEL.md)
15. [Logical Domain Model](LOGICAL_DOMAIN_MODEL.md)
16. [Interaction Model](INTERACTION_MODEL.md)

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

Materializar la arquitectura: Software Architecture, servicios, Memory/Context Engine, Knowledge Graph, Event Bus, capa MCP e interfaces, validados con una rebanada vertical de extremo a extremo.

---

# Estado del proyecto

El proyecto ha completado la definición arquitectónica y el modelado del dominio, y se dispone a iniciar la fase de diseño del sistema (Software Architecture).

El objetivo inicial es establecer una base sólida sobre la que construir una plataforma de investigación sostenible, extensible y orientada al conocimiento.

---

# Licencia

Pendiente de definir.
