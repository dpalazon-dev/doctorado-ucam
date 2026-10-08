# Publicación y procedencia

## Alcance

Este repositorio reúne dos alcances relacionados pero independientes: la arquitectura futura de ResearchOS, documentada en la raíz, y una instantánea del prototipo local Research Workbench en [`../../prototypes/research-workbench/`](../../prototypes/research-workbench/). La instantánea no implementa las Specs de ResearchOS, no comparte su modelo canónico y no incorpora su sidecar cognitivo. La presencia de ambos en un repositorio no cambia contratos de producto ni convierte el prototipo en autoridad arquitectónica.

## Procedencia de Workbench

El código del prototipo procede del hito integrado `702d08b02f2d2655584a932455ac394fb026f8d2` de `integration/v0.1`, que incluye el lector y el backend PRE/P1. La UI PRE/P1 de workflow de T04c no forma parte del ejecutable publicado; su patch se conserva aparte en `pending-work/T04c-workflow-ui.patch` y no está integrado. Los documentos añadidos o modificados en `main` proceden de `6e20428fe8267dddac3f4a8adca41a158314fd05`; propuestas documentales proceden de `1f598cbe0bb36d5b0fd8f65c530337a7b36378e6`.

La carpeta es un snapshot reproducible del árbol de archivos, no una importación del historial Git remoto de Workbench. El historial local del origen se conserva en el bundle archivado fuera de esta publicación. Esta unificación conserva el historial de ResearchOS. Los documentos originales locales y el repositorio fuente permanecen intactos.

La importación excluye `.codex`, `docs/session`, binarios, releases, bibliotecas, cachés, `node_modules`, `target` y `work`. [`import-manifest.json`](import-manifest.json) registra la composición del snapshot; esta página describe sus fuentes y límites.

## Desarrollo del prototipo

Desde [`../../prototypes/research-workbench/`](../../prototypes/research-workbench/) en Windows 11 x64:

```powershell
npm.cmd ci
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/check.ps1
. .\scripts\development-env.ps1
npm.cmd run tauri:dev
```

El prototipo requiere Windows x64, Node.js/npm, Rust MSVC, Visual Studio C++ Build Tools con SDK de Windows y WebView2 Runtime. Los datos usados en las pruebas son sintéticos. Estos comandos describen el flujo de desarrollo; no acreditan instalación limpia ni finalización de v0.1. Consulta [`README del prototipo`](../../prototypes/research-workbench/README.md), [`estado del prototipo`](../../prototypes/research-workbench/docs/STATUS.md) y [`entorno de desarrollo`](../../prototypes/research-workbench/docs/development/ENVIRONMENT.md).

## Licencias

La licencia MIT de la raíz cubre el material propio de este repositorio. Las dependencias y otros componentes de terceros conservan sus licencias y avisos; consulta [`../../prototypes/research-workbench/THIRD_PARTY_NOTICES.md`](../../prototypes/research-workbench/THIRD_PARTY_NOTICES.md).
