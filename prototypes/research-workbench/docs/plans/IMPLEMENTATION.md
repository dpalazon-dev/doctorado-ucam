# Research Workbench v0.1 Implementation Plan

> **For agentic workers:** ejecutar mediante `superpowers:subagent-driven-development` conforme al método ya autorizado: Sol coordina, un único worker de producto implementa cada tarea en worktree y Sol revisa antes de integrar. T01 por complejidad usa worker Sol; siguientes tareas acotadas usan Luna. Research/review pueden coexistir, nunca dos implementadores de producto. Las instrucciones directas del usuario y AGENTS prevalecen sobre defaults de skills.

**Goal:** entregar Windows 11 x64 instalable, local y offline, desde biblioteca/lector piloto hasta PRE–P2, conocimiento trazable, búsqueda y portabilidad v0.1 verificados.

**Architecture:** monolito modular Tauri; dominio Rust puro, casos de uso/UoW y puertos, adaptadores SQLite/filesystem/OS, IPC tipado a React. SQLite canónico y PDFs administrados; FTS y exports derivados. Una conexión por hilo DB y coordinador de mantenimiento evitan escritores cruzados.

**Tech Stack:** Tauri 2, Rust stable MSVC, rusqlite bundled/backup/FTS5, Serde/ts-rs, React TypeScript strict/Vite, Tailwind/shadcn/Radix, RHF/Zod, PDF.js, Vitest/Testing Library, NSIS/WebView2 offline.

**Spec:** `docs/architecture/{ARCHITECTURE,DOMAIN,CONTRACTS,DATA,SPECS,ADRS,QUALITY}.md`. `HISTORICAL_MULTIAGENT_PLAN.md` conserva antecedentes; sus firmas/paths abreviados no gobiernan. Este plan se preparó tras aceptación del usuario comunicada por Sol; Sol registra aceptación en arquitectura y STATUS.

## Global Constraints

- Enums y firmas exactas CONTRACTS v1; UUID canónico, UTC RFC3339, DTO camelCase, SQL snake_case; artefactos TS generados/verificados desde Rust. Cambios públicos requieren resolución de Sol y actualizar todos los consumidores/tests/documentos antes de delegar.
- Actor DB cola **64**, una Connection; UoW `BEGIN IMMEDIATE`, FKs ON, WAL, synchronous FULL, busy_timeout **5000 ms**. Ningún SQL/hash/copia larga en hilo ventana ni transacción abierta durante copia PDF.
- Toda mutación persistente confirma receipt, auditoría y revisiones con cambios. requestId+hash igual retorna resultado previo; payload distinto Conflict. expectedRevision impide lost update; no-op canónico no invalida fases.
- Límites: title **1000**, answer/body **20000**, snippet/quote **10000**, relation/context **5000**, autores **100**, PDF **500 MiB**, search query **1000**, excerpt **500**, listas **100** por defecto/**500** máximo. Rechazar desconocidos/no finitos/oversize, no truncar.
- Picker otorga token ligado a operación/biblioteca con **24 h** de vida; UI no entrega paths libres. Protocolo PDF por UUID registrado, raíz canónica y reparse points validados; CSP/capabilities restringidas. Sin contenido privado en logs/export compartible.
- Datos `%LOCALAPPDATA%/ResearchWorkbench/library` separados de instalación/cwd; snapshots SQLite backup con WAL y PDFs, backup pre-migración, checksum ledger, diagnóstico read-only ante esquema futuro. Desinstalar conserva datos/backups.
- Piloto **0.0.1** ya tiene instalador offline; **v0.1.0** solo tras todas las capacidades y gates. P3/P4 reservadas, P2 completada conserva Paper ACTIVE; sin LLM/IA, OCR, sync/red, hard delete, TRASHED visible, merge, rich text ni store canónico frontend.
- No modificar configuración global ni instalar SDKs/frameworks globales. Dependencias de proyecto/versions reales son responsabilidad de Sol y T01; lockfiles prueban selección, no docs con parches inventados.
- No commits/publicación/PR/remoto de worker hasta autorización de Sol. Sol controla baseline, worktrees, integración y STATUS; workers solo código propio y reporte. No revertir trabajo ajeno.

## Review Focus

- Reintento tras cierre en frontera filesystem/SQL: T02 fault injection y recibo durable evitan doble paper/borrado ambiguo.
- Respuesta tardía de guardado tras cambiar documento: T03 serializa por documento y descarta renders anteriores; Guardado exige commit.
- Cambiar respuesta/artefacto tras gate visible: T04/T06/T07 prueban reevaluación bajo transacción, revisión y NEEDS_REVIEW sin pérdida.
- Export parcial con fuente de otro paper y extremos compartidos: T08 prueba cierre de referencias completo, no leaks ni FKs huérfanas.
- Restore/switch con corte después de cerrar DB anterior: T08/T09 journal fuera de raíz y reabrir proceso prueban recuperación y ubicación real.

## Ownership y preflight obligatorio

El [preflight de los 45 pares](PREFLIGHT.md) concreta archivos compartidos, interfaces producer/consumer y pruebas dependientes de cada cruce; Sol lo recorre antes de dispatch y reasigna el subconjunto real de integración a un worker.

Cada brief es consumible por separado e incluye requisitos, paths, consume/produce, tests y reporte. Antes de asignar, Sol comprueba `git status --short`, baseline commit, archivos reales y firmas T01; actualiza paths si la base aprobada los cambió. Asigna **un único escritor por archivo**, comprueba migraciones/DTOs aún sin drift y registra worktree/branch en STATUS. Un worktree no elimina conflictos de archivos compartidos.

Compartidos bajo responsabilidad de Sol tras T01: `src-tauri/src/lib.rs`, `*/mod.rs` de nivel superior, `src-tauri/src/transport/dto.rs`, `src/shared/contracts/{generated/,wire.ts,ports.ts}`, `src/shared/adapters/tauri/{client.ts,index.ts}`, `src/app/{App.tsx,ShellView.ts,tokens.css}`, `src-tauri/src/adapters/sqlite/{actor.rs,migrations.rs,receipts.rs}`, `src-tauri/src/desktop/`, manifiestos/lockfiles, capabilities/permissions/CSP, scripts/check/generate-contracts, docs/STATUS y docs/architecture. Los workers entregan módulo/command file y descripción precisa de integración; **Sol asigna explícitamente esas integraciones de código al siguiente worker**, decide/revisa y registra el ownership antes de escribir. Sol no implementa código de producto durante SDD. Un módulo aislado no equivale a función integrada.

| Tarea / brief | Ownership principal / complejidad | Consume / depende | Compartidos y pruebas que exigen preflight | Produce / gate |
|---|---|---|---|---|
| [01](tasks/task-01-brief.md) Base | Scaffold/actor/DTO/0001/lifecycle/config; alta | Baseline y toolchain verificado | Inicialmente tiene compartidos; Sol congela exports/identity; ninguna prueba depende de features futuras | Ventana, DB, wire, scripts y NSIS config |
| [02](tasks/task-02-brief.md) Library backend | library + documents/import + commands/library; alta | T01 actor/UoW/schema/DTO | Registro IPC/permissions Sol; actor tests T01 | Import recuperable y metadata/archive/restore reales |
| [03](tasks/task-03-brief.md) Library/Reader | features library/reader + reader backend/protocol; alta | T01; integración real T02 | lib/adapter/App Sol; protocol consulta documents T02; fixtures únicos T03 | Recorrido biblioteca/lector persistido |
| [04](tasks/task-04-brief.md) PRE/P1 | workflow/domain/migration0002/UI PRE/P1; alta | T02/T03 | Import inicialización: change request a Sol; migrator registry; P1 archive usa Paper tx T02 | Gates versionados y ramas P1 |
| [05](tasks/task-05-brief.md) Knowledge/Concept | knowledge/concepts + migration0003; alta | T04 invalidación; T02 documents | 0003 incluye relations/provenance esquema para T07; no dos editores; FTS hooks Sol T08 | 12 tipos, conceptos compartidos y captura atómica |
| [06](tasks/task-06-brief.md) P2 y captura UI | p2/candidates + features knowledge/workflow P2; alta | T04/T05 | Worker asignado integra extensión workflow; tests P2 final requieren T07, fixtures de policy no sustituyen journey | P2/cola P3 sin workspace P3; gate completo pendiente T07 |
| [07](tasks/task-07-brief.md) Relations/Provenance | domain/services/repos + features relations/provenance; alta | T04/T05; T06 para integración P2 | 0003 existente T05; invalidation.rs T04 Sol; ningún rewrite de migration publicada | 13 relaciones y localizadores navegables/stale |
| [08](tasks/task-08-brief.md) Search/Portability | search/portability + 0004/files jobs; muy alta | T02/T04/T05/T07 | Actor/migrator/maintenance/switch Sol; tests export necesitan P2 T06 y provenance T07 | FTS/export/backup/restore/switch servicios reales |
| [09](tasks/task-09-brief.md) UI/integración | search/settings/portability + integration tests; alta | T01–T08 integradas | App/adapter/capability Sol; componentes T03/T06/T07 sin edición concurrente | Viaje v0.1 completo, accesibilidad/NFR medidos |
| [10](tasks/task-10-brief.md) Release instalado | release script/installer/docs/QA; alta | Gate piloto T01–T03; final T01–T09 | Sol tauri.conf/version/lockfiles y build sobre árbol inmóvil; no instalar en biblioteca personal | NSIS/checksum/evidencia limpia offline/upgrade |

Cada worker produce `docs/reports/task-NN-report.md` con base SHA, branch/worktree, ownership real, requerimientos/tests por ID, comandos y exit codes, fallos pendientes, cambios de contrato solicitados y límites de evidencia. Sol actualiza única lista en STATUS (`pendiente|en curso|en revisión|completada|bloqueada`). Los reportes documentan evidencia; no constituyen otra lista de tareas.

## Orden y cortes entregables

1. **Fase A — infraestructura y piloto:** T01 → revisión/integración → T02 → revisión/integración → T03. Worker T03 asignado por Sol conecta comandos y shell. **T10-piloto obligatorio inmediatamente después de T03 y antes de T04** verifica **0.0.1 instalado**. T10 no espera al final para primer instalador.
2. **Fase B — PRE/P1:** T04 integra migración protegida/definiciones y UI sobre piloto. Fixture de upgrade prueba papers existentes sin cambiar UUID. Versión de desarrollo intermedia, aún no v0.1 final. Tras el preflight del piloto se divide en tres cortes secuenciales, con un autor nuevo y revisión por corte: **T04a** aporta prueba documental tipada preservando Reader y guardas Windows; **T04b** implementa dominio, migración, repositorio, import e IPC PRE/P1; **T04c** conecta formularios y navegación. [TASK04_PORTS](TASK04_PORTS.md)/ADR-018 fija contratos internos antes de implementar. T04 sólo se declara completada cuando los tres cortes y sus pruebas de integración pasan; no se activan capacidades de formularios ausentes por completar T04a.
3. **Fase C — conocimiento:** T05 → revisión/integración → T06 → revisión/integración → T07. Sol asigna a worker cada integración de invalidación por transacción. Pruebas fixture de T06 verifican policy/candidatos; el gate de journey P2 completo queda pendiente hasta artefactos/relaciones/provenance T07, sin excepción de dominio.
4. **Fase D — portabilidad:** T08 inicialmente preparar FTS/projection/export; su integración exige T06/T07 completos. Si la tarea excede una sesión, Sol entrega subcortes 08a FTS, 08b export/snapshot, 08c backup/restore/switch dentro del mismo ownership, con revisión por corte y pruebas independientes; no delegar todo sin pausa de revisión.
5. **Fase E — producto:** T09 conecta todas las capacidades verdaderas, full checks y revisión; T10-final congela/release/QA instalado **0.1.0**. Sol entrega setup, SHA256, notas y limitaciones solo después de evidencia.

## Estrategia de verificación

En cada task: tests de comportamiento/fallo pertinentes antes de implementación, fallo observado cuando framework existe, implementación mínima, suite afectada, typecheck/fmt/clippy y revisión real. No tests que solo repitan código ni cobertura porcentual arbitraria. Backend tests con SQLite/filesystem reales temporales; fixtures sintéticos, nunca biblioteca personal.

Gate de integración: `npm run typecheck`, `npm run test`, `npm run build`; `cargo fmt --manifest-path src-tauri/Cargo.toml --check`, `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml`; generación de contratos sin diff, dependency boundaries y registry completo. `scripts/check.ps1` ejecuta la secuencia y conserva exit codes. Tests web/mocks, IPC real, executable nativo y app instalada se reportan separados.

Gate final: todos SPEC-001…007 y QUALITY cubiertos, FKs/UUID/hashes intactos tras reopen/restore/upgrade, sin paths privados compartidos ni acceso fuera de raíz, sin guardado antes de commit. Perfil nuevo de desarrollo no sustituye Windows limpio sin herramientas ni WebView2. Hardware/corpus y NFR se miden; un umbral propuesto no se declara cumplido sin medir. Si falta VM/offline/automatización nativa se deja QA pendiente y se continúa todo trabajo independiente.

## Riesgos y mitigaciones

- **SQLite + filesystem:** intención durable, hash, staging propio y fault injection/reopen (T02/T08), ninguna eliminación ambigua.
- **Luna frente a contrato grande:** scopes tipados, contratos extraídos en briefs, reporte y revisión por task/subcorte; Sol decide cambios públicos.
- **Ediciones compartidas:** tabla preflight, composition roots Sol, patches/requerimientos de integración explícitos, no merges sin revisión.
- **Versión/instalación:** config desde T01, primer QA instalado T10-piloto, final T10; lockfiles/artefacto SHA identificados y actualizaciones fixture.
- **Confundir progreso con verdad:** UI muestra origin/confidence/provenance y P2 completada, no score automático ni Paper COMPLETED.

## Success Criteria

- [ ] Piloto NSIS offline real probado, sin dependencias de herramientas/source/cwd; datos conservados al uninstall/reinstall.
- [ ] PRE→P1→P2 con versiones fijadas, gates backend reevaluados, ramas P1 exactas e invalidación conservando contenido.
- [ ] Doce tipos core y conceptos globales, trece relaciones válidas, captura/procedencia atómica, PENDING/STALE visibles y navegación fuente.
- [ ] FTS filtros/literales/index rebuild consistente; export JSONL/Markdown sin edges huérfanos ni paths privados.
- [ ] Backup íntegro/restore nueva raíz/switch recuperable/reopen conservan IDs/FKs/PDFs/posición/fases.
- [ ] Checks, revisiones, accesibilidad, rendimiento y QA instalado registrados con alcance y ningún bloqueo de pérdida/integridad pendiente.

### Precisión de cortes T06 (ADR-023)

Tras T05a/b y T04c integradas: T06a backend/candidatos/policy y T06b captura/cola UI, autores nuevos secuenciales y revisión por corte. Todos save/evaluate/advance P2 de producción esperan T07; ninguna habilitación parcial por key. T07 incorpora resolutores completos y el journey P2 final. TASK06_DECISIONS fija semántica; ABI del preflight se contrasta con T05 integrada antes del despacho. La división no autoriza trabajo producto paralelo ni omite gates.
