# Doctorado_UCAM

> Plataforma de investigación, gestión del conocimiento e inteligencia artificial para el desarrollo del doctorado.

Construyendo una plataforma de investigación donde el conocimiento, la automatización y la inteligencia artificial trabajan juntos para acelerar el proceso científico.

---

## Visión

Doctorado_UCAM es una plataforma diseñada para centralizar todo el conocimiento generado durante el doctorado, facilitar la investigación científica y automatizar tareas mediante inteligencia artificial.

El objetivo no es únicamente almacenar información, sino construir un sistema capaz de organizar, relacionar y explotar el conocimiento científico a lo largo de todo el proceso investigador.

Este repositorio constituye la fuente principal de documentación, arquitectura y desarrollo del proyecto.

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
Doctorado_UCAM/

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
└── AI_ARCHITECTURE.md
```

La estructura modular por directorios (architecture/, platform/, ai/…) se introducirá cuando comience la fase de implementación.

---

# Architecture Documents

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
AI Architecture
```

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

## Fase 2 · Plataforma

- [ ] Infraestructura
- [ ] Almacenamiento documental
- [ ] Base de datos
- [ ] APIs

## Fase 3 · IA

- [ ] Procesamiento documental
- [ ] Búsqueda semántica
- [ ] Embeddings
- [ ] Agentes especializados

## Fase 4 · Automatización

- [ ] Ingesta de documentos
- [ ] OCR
- [ ] Extracción de metadatos
- [ ] Clasificación automática

## Fase 5 · Investigación Asistida

- [ ] Asistente de investigación
- [ ] Comparación de artículos
- [ ] Revisión bibliográfica
- [ ] Generación de conocimiento

---

# Estado del proyecto

Actualmente el proyecto se encuentra en la fase de definición arquitectónica y modelado del dominio.

El objetivo inicial es establecer una base sólida sobre la que construir una plataforma de investigación sostenible, extensible y orientada al conocimiento.

---

# Licencia

Pendiente de definir.
