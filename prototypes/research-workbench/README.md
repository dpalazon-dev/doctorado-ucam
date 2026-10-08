# Research Workbench

> **Prototipo parcial pausado.** Este proyecto se publica como referencia reutilizable y base para forks; actualmente no tiene mantenimiento activo porque su autor no dispone de tiempo. La publicación no implica compromiso de soporte o respuesta a issues. Es un prototipo Windows 11 x64 independiente, no una implementación de las Specs ResearchOS. Véase la [procedencia y límites del snapshot](../../docs/publication/README.md).

Aplicación de escritorio local para leer papers y convertirlos en conocimiento estructurado con procedencia. Windows 11 x64 · Tauri 2 · React/TypeScript · Rust · SQLite.

El estado de ejecución conserva la evidencia e historia de desarrollo local. Consulta [estado](docs/STATUS.md), [intención](INTENT.md), [arquitectura](docs/architecture/README.md) y [plan](docs/plans/IMPLEMENTATION.md); la pausa de mantenimiento se describe arriba.

## Desarrollo

El directorio contiene el proyecto completo del prototipo para inspección y forks. En Windows x64 se requieren Node.js/npm, Rust con target MSVC, Visual Studio C++ Build Tools y SDK de Windows, y WebView2 Runtime. Desde PowerShell abierto en este directorio:

```powershell
npm.cmd ci
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/check.ps1
. .\scripts\development-env.ps1
npm.cmd run tauri:dev
```

Los checks y el modo debug deben usar datos sintéticos. La guía de [`docs/development/ENVIRONMENT.md`](docs/development/ENVIRONMENT.md) contiene detalles de toolchain y límites de validación.

## Fuentes de la instantánea

La carpeta `docs/session/` y los materiales de trabajo auxiliares fueron excluidos de la publicación. Los originales permanecen únicamente en el repositorio fuente local; no forman parte de este snapshot. La especificación original editable está disponible en [`docs/PRODUCT.md`](docs/PRODUCT.md).
