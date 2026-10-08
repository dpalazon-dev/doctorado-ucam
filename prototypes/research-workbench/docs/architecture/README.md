# Research Workbench — base de arquitectura previa a implementación

Versión de diseño 0.2 · 1 de octubre de 2026 · **Baseline de ejecución aceptada por la instrucción del usuario del 1 de octubre de 2026.**

Este paquete define la arquitectura de la aplicación Windows instalable y el alcance funcional v0.1, antes de crear el repositorio de producto. No certifica código, rendimiento, instalación ni recuperación ya ejecutados. El método de desarrollo acordado sigue siendo Sol como coordinador y agentes Luna con responsabilidades acotadas.

## Resultado que se diseña

Una aplicación local para un investigador: instalar, abrir, importar un paper, leer su PDF, procesarlo de PRE a P2 y conservar conocimiento tipado con conceptos globales, procedencia, búsqueda, exportación y backup restaurable. Los agentes son herramientas de desarrollo; la aplicación inicial no ejecuta agentes ni LLMs.

El piloto 0.0.1 entrega biblioteca y lector **con instalador**. v0.1.0 añade el procesamiento PRE → P2 completo y las capacidades de seguridad de uso. P3/P4, IA, sincronización, fusión de conceptos y borrado definitivo se reservan para nuevas SPECs antes de implementarlos.

## Documentos normativos y orden de lectura

| Documento | Pregunta que resuelve |
|---|---|
| [ARCHITECTURE.md](ARCHITECTURE.md) | Cómo se organiza el programa, qué frameworks se usan, dependencias, patrones y ejecución |
| [DOMAIN.md](DOMAIN.md) | Qué significan las entidades, estados, límites e invariantes del conocimiento |
| [CONTRACTS.md](CONTRACTS.md) | Cómo se comunican funcionalidades y servicios; entradas, salidas, errores y consistencia |
| [DATA.md](DATA.md) | Modelo persistente, restricciones, transacciones, archivos, migraciones y recuperación |
| [SPECS.md](SPECS.md) | Comportamientos funcionales y criterios de aceptación de cada capacidad |
| [ADRS.md](ADRS.md) | Decisiones, alternativas, consecuencias y motivos para revisarlas |
| [QUALITY.md](QUALITY.md) | Requisitos no funcionales, verificación, trazabilidad y condiciones previas a implementación |

La especificación original de producto se conserva como [DOCX](../session/outputs/Research_Workbench_Arquitectura_y_Especificacion_v0.1.docx). El [plan multiagente](../plans/HISTORICAL_MULTIAGENT_PLAN.md) organiza ejecución; sus firmas abreviadas son antecedentes, no una segunda autoridad sobre los contratos.

## Autoridad y cambios respecto al diseño anterior

Cada asunto tiene una autoridad: semántica en DOMAIN, interfaz en CONTRACTS, almacenamiento en DATA, comportamiento observable en SPECS, decisiones en ADRS. ARCHITECTURE explica cómo encajan. Ante una discrepancia no se elige silenciosamente un documento: se corrigen los afectados antes de delegar implementación.

Este paquete concreta y sustituye estas ambigüedades anteriores:

- Instalador NSIS offline desde el primer piloto, identidad estable y datos fuera de los binarios.
- Monolito modular con puertos y adaptadores donde existen límites de infraestructura; contratos IPC locales, sin servidor HTTP.
- SQLite mediante rusqlite con una conexión propiedad de un hilo dedicado; transacciones de aplicación y concurrencia optimista.
- Texto plano como formato de cuerpo inicial; editor enriquecido y caché avanzada quedan para decisiones posteriores.
- Abrir un paper es una operación explícita que actualiza el último abierto; leer una ficha no produce ese efecto.
- Captura y procedencia inicial se confirman atómicamente. Evaluar un gate para mostrarlo no autoriza avanzar sin reevaluarlo en el backend.
- Archivar/restaurar es reversible; merge y borrado definitivo no forman parte de v0.1. El catálogo conceptual original conserva esas posibilidades para evolución futura.

Los enums incluyen estados reservados para evolución, pero la existencia de una columna o un valor no habilita un botón ni un comando fuera de alcance. La aceptación de esta baseline habilita la ejecución; no certifica el producto implementado. Los originales conservan su estado histórico.

## Condición para empezar a implementar

1. Aceptación registrada: la instrucción del usuario del 1 de octubre de 2026 autoriza entorno e implementación completa por fases sobre esta base.
2. Coordinador comprueba que SPECs, contratos, dominio y datos no tienen contradicciones críticas; actualiza el plan detallado con las firmas normativas.
3. Se fija el conjunto exacto de dependencias, toolchain y licencias mediante una revisión de compatibilidad; se registra antes de escribir funcionalidades. La selección de frameworks ya está definida; las versiones de parche se fijan en lockfiles y no se inventan en estos documentos.
4. Se autoriza e instala el requisito Rust que falta y se verifica el entorno de construcción. No se necesita instalar herramientas en un equipo que solo vaya a usar el programa.
5. Cada tarea recibe SPEC, ADRs, contrato, archivos propios, dependencias y pruebas de aceptación. Ningún worker redefine esos límites unilateralmente.

La revisión de este paquete es un hito de diseño. No se necesita cerrar ahora el comportamiento interno de P3/P4 o IA, que no se implementarán bajo estos contratos.

## Paso al repositorio futuro

Al iniciar el proyecto en C:\Users\david\Projects\Research-Workbench, copiar este paquete a docs/architecture/, la visión original a docs/PRODUCT.md y el plan a docs/plans/. El coordinador asignará un commit de baseline aceptada. ADRs cambian de Proposed a Accepted solo con aceptación registrada; una decisión reemplazada conserva su historial y enlaza el ADR sucesor.

Durante ejecución, docs/STATUS.md registra una sola lista de tareas y evidencia. El código debe generar o verificar los artefactos de contrato para evitar mantener tipos divergentes entre Rust y TypeScript. Todo cambio público de contrato actualiza su versión, las SPECs afectadas, las migraciones necesarias y los tests del consumidor.

## Revisión documental realizada

Dominio/contratos, SPECs/calidad y ADRs se redactaron con responsabilidades separadas. El coordinador integró arquitectura y datos; una revisión independiente contrastó los límites entre documentos. Se cerraron cinco ambigüedades materiales: la rama archive de P1, la representación persistente de candidatos P3, la revisión de procedencia, los selectores para backups externos y la invalidación determinista de fases.

La revisión final no encontró un bloqueo crítico adicional en esos cruces. Es evidencia de coherencia de la propuesta; los tests, medidas de rendimiento y comprobaciones del producto instalado siguen siendo requisitos de ejecución futura. Los ocho documentos y sus enlaces locales se comprueban antes de entregar el paquete.

