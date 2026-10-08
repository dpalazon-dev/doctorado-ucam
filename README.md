# ResearchOS y Research Workbench

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

Este repositorio reúne el diseño de **ResearchOS**, una visión futura de entorno personal de trabajo del conocimiento, y un **prototipo parcial** de Research Workbench que puede servir como referencia o base para forks. El desarrollo de estos proyectos está pausado y no tiene mantenimiento activo porque su autor no dispone actualmente de tiempo. La publicación no implica soporte ni respuesta a issues.

## Dos alcances

- **ResearchOS**, en la raíz, contiene la arquitectura canónica y el catálogo de Specs de una aplicación futura multiplataforma. No hay aquí una implementación de producción de ResearchOS.
- **Research Workbench**, en [`prototypes/research-workbench/`](prototypes/research-workbench/), es una aplicación local para Windows 11 x64 construida con Tauri, React/TypeScript, Rust y SQLite. Es una implementación independiente e incompleta: no implementa las Specs de ResearchOS, no comparte su modelo canónico y no incluye IA.

La visión futura de ResearchOS admite procesamiento cognitivo Python opcional. Esa capacidad no forma parte del prototipo Workbench. Consulta [`docs/publication/`](docs/publication/) para los límites y fuentes de la instantánea.

## Prototipo Workbench

Consulta [`prototypes/research-workbench/`](prototypes/research-workbench/) para el README, estado, intención histórica y arquitectura. El prototipo requiere Windows x64, Node.js/npm, Rust MSVC, Visual Studio C++ Build Tools con SDK de Windows y WebView2 Runtime. Desde PowerShell en el directorio del prototipo:

```powershell
npm.cmd ci
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/check.ps1
. .\scripts\development-env.ps1
npm.cmd run tauri:dev
```

Los datos de prueba son sintéticos. Estos comandos de desarrollo no demuestran instalación en equipo limpio ni que v0.1 esté completa.

## ResearchOS

La arquitectura y el estado de diseño se describen en [VISION.md](VISION.md), [DOCUMENTATION_ARCHITECTURE.md](DOCUMENTATION_ARCHITECTURE.md), [ROADMAP.md](ROADMAP.md) y [SPEC_CATALOG.md](SPEC_CATALOG.md). El README es una guía de navegación; esos documentos conservan sus ámbitos de autoridad.

## Licencia

El material propio se distribuye bajo [MIT](LICENSE). Componentes de terceros conservan sus licencias y avisos en [`prototypes/research-workbench/`](prototypes/research-workbench/). Consulta también [`docs/publication/`](docs/publication/) para la procedencia del snapshot.
