# Intención del producto

> **Nota de publicación (8 de octubre de 2026):** este documento conserva la intención histórica de Research Workbench. El prototipo publicado es parcial, está pausado y no tiene mantenimiento activo por falta de tiempo de su autor. La intención de completar v0.1 que sigue no describe un resultado alcanzado ni un compromiso vigente. Workbench es una aplicación Windows local independiente, sin IA; no implementa ResearchOS ni comparte su modelo canónico. Para procedencia y límites, consulta [la guía de publicación](../../docs/publication/README.md).

Research Workbench es una aplicación Windows local para David, un investigador que transforma la lectura manual de papers en conocimiento estructurado, recuperable y trazable. Se instala y abre como una aplicación de escritorio; el equipo de uso no necesita herramientas de desarrollo.

Paper es la unidad de trabajo; Concept organiza conocimiento; KnowledgeItem captura una unidad semántica; Evidence y Provenance conservan trazabilidad; Relation expresa relaciones explícitas; Workflow guía el trabajo intelectual. La aplicación conserva decisiones humanas, incertidumbre y límites, sin inventar evidencias ni convertir hipótesis en resultados.

## Resultado autorizado
Implementar v0.1 por fases hasta poder importar PDFs, leerlos, completar PRE/P1/P2, capturar elementos tipados y conceptos globales, registrar localizadores y relaciones, buscar, exportar y recuperar una biblioteca desde backups. Entregar instalador NSIS Windows x64 con WebView2 offline, datos fuera del directorio del programa y pruebas de las capacidades realizadas.

El primer piloto 0.0.1 incluye instalación, biblioteca y lector. v0.1.0 añade workflow y conocimiento, búsqueda, exportación, backup y restauración segura. Cada fase se integra después de revisión y validación; no se posterga la integridad de datos para ganar funcionalidades.

## Límites
Un usuario, una biblioteca activa y un escritor; uso offline; sin login, servidor HTTP, cloud obligatoria, sincronización, OCR o integración LLM. P3/P4, fusión y borrado definitivo se diseñarán después. Archivar/restaurar es reversible. El programa no ejecuta los agentes que lo desarrollan.

## Principios
Persistencia fiable; procedencia explícita; control del investigador; contratos comprobables; interfaz en español clara; mínima complejidad suficiente; portabilidad sin dependencia del programa; recuperación antes de cambios de esquema. No afirmar una capacidad sin demostrarla.

## Fuente y aceptación
El usuario pidió arquitectura antes de implementar y, el 1 de octubre de 2026, autorizó crear el entorno, traer todos los archivos e implementar completamente por fases con este chat como orquestador. Se adopta la arquitectura v0.2 de `docs/architecture/` como baseline de ejecución. El DOCX y los planes originales se conservan íntegros en `docs/session/`; sus formulaciones anteriores no reemplazan contratos normativos posteriores.

Cambiar alcance o decisiones exige actualizar documentos afectados y registrar razón, impacto, migración y verificación en un ADR. La aceptación del diseño no demuestra aún la implementación ni sus prestaciones.
