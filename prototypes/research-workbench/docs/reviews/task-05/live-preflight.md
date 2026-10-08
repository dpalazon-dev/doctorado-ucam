# T05a — preflight de integración, sólo lectura

**DONE_WITH_CONCERNS — preparado, no activado.** Referencia: `702d08b02f2d2655584a932455ac394fb026f8d2`; fecha 2026-10-05. Código/norma leídos mediante Git desde ese SHA. Brief central: `C:/Users/david/Projects/Research-Workbench/.superpowers/sdd/IMPLEMENTATION/task-05a-brief.md`. No se leyó producto WIP del merge UI. Peer OpenViking no verificado: sólo fuentes locales.

T05a espera QA/aceptación e integración T04c y BASE/rama/worktree/autor que fije Sol; este SHA no es BASE de implementación.

## Ownership existente exacto

Además de los nuevos archivos del brief, transferir estos **cinco archivos existentes**:

| Ruta desde repo | Cambio permitido | Prueba relevante |
|---|---|---|
| `src-tauri/src/domain/mod.rs` | Sólo exports knowledge/concepts/ontology/provenance. | Nuevos tests de dominio/captura. |
| `src-tauri/src/application/mod.rs` | Sólo exports knowledge/knowledge_ports. | capture_item_in_tx/create_concept_in_tx. |
| `src-tauri/src/adapters/sqlite/mod.rs` | Sólo exports knowledge_repository/concept_repository. | Roundtrip SQLite/corrupción/reopen. |
| `src-tauri/src/adapters/sqlite/migrations.rs` | Sólo SCHEMA_VERSION2→3 y entrada0003 al final de MIGRATIONS. | Upgrade0002→0003/fallo tardío; no editar runner/snapshot/ledger. |
| `src-tauri/tests/workflow_migration.rs` | Expectativas de esquema actual y aislamiento del upgrade histórico0002; sin cambios de comportamiento Workflow. | Tres ajustes concretos debajo. |

Ajustes exactos de `workflow_migration.rs`:

- `schema_v2_contains_one_atomic_workflow_migration` (línea19) fija SCHEMA_VERSION2 y lista completa[1,2]. Conservar comprobación del prefijo histórico0001/0002 y SQL0002; llevar comprobación de catálogo actual[1,2,3] al nuevo test0003, o actualizar esa parte explícitamente.
- `fresh_library_applies_schema_v2_and_seeds_canonical_definitions` (línea38) usa DbActor y espera2. Actualizar nombre/expectativa a esquema actual; conservar las tres definiciones canónicas.
- `migration_0002_backfills_populated_v1_without_changing_library_records_and_reopens` (línea396) usa MIGRATIONS completo en línea398. Pasar **`&MIGRATIONS[..2]`**, conservar assertions schema2, backup1 y preservación/reopen. No convertir silenciosamente esa prueba en upgrade a3.
- `failed_0002_rolls_back_ddl_seed_backfill_ledger_and_user_version_after_backup` (línea532) ya usa pasos locales1/2. Conservar íntegra.

## Tests/fixtures sin edición necesaria por0003

| Ruta | Evidencia |
|---|---|
| `src-tauri/tests/desktop_bootstrap.rs` | `migration_failure_keeps_backup` usa SCHEMA_VERSION+1 y SCHEMA_VERSION. Futuro usa99. `snapshot_fixture` crea esquema actual/PDF sintético. |
| `src-tauri/tests/schema_diagnostic.rs` | Fixtures actuales por actor, futuro99, inventario completo/ACL/WAL; conservar. |
| `src-tauri/tests/db_actor.rs` | Actor actual, sin versión fija; una conexión/cola64, rollback, replay, colisión, panic, shutdown y reopen. |
| `src-tauri/tests/scaffold_contracts.rs` | Roundtrip wire, no comparación de schemaVersion con runtime; patch Concept domain ya probado. |
| `contracts/fixtures/app-info.json`, `backup-aggregate.json` | schemaVersion1 es muestra wire válida, no expectativa runtime; no bump ni capabilities nuevos. |
| `src-tauri/tests/library_integration.rs`, `settings_service.rs` | Diagnóstico99, sin dependencia de versión2 hallada en búsqueda acotada. |
| `src-tauri/tests/fixtures/task02/*.pdf` | Únicos fixtures persistidos bajo tests; no necesarios para captura0003 sin I/O PDF. |

No existe módulo de fixture compartida. `populated_v1_library`, `library_user_data_snapshot`, `snapshot_table` son privados de workflow_migration.rs (líneas303/253/223). Los nuevos tests necesitan fixture **schema2 poblada**, incluidos pins/answers/snapshots/revisiones; no extraer/refactorizar esos helpers por inferencia. Si se necesita helper nuevo compartido, Sol asignará ruta concreta, no tests/** genérico.

Ubicar pruebas nuevas0002→0003/catálogo/constraints en `src-tauri/tests/ontology.rs`, ya asignado; captura/Concept en `knowledge_capture.rs`/`concepts.rs`. No hace falta otro test target por la evidencia disponible.

## Puertos reales y composición reutilizable

| Ruta/símbolo | Uso y límite |
|---|---|
| `adapters/sqlite/actor.rs`: DbActor::start/submit/shutdown | Una conexión, hilo research-db, sync_channel64. El símbolo es **DbActor**, no DbActor64. submit presta &mut Connection y revierte TX abandonada/panic. Tests nuevos pueden usarlo sin servicio IPC nuevo. |
| `application/unit_of_work.rs`: with_transaction | TX IMMEDIATE, closure con TX prestada, commit sólo en éxito. with_receipt ya lo usa; no anidar TX/receipts. |
| `adapters/sqlite/receipts.rs`: with_receipt/lookup_receipt | with_receipt es público; lookup_receipt pub(crate). UUID canónico, lookup durable requestId/command/hash antes del closure; replay retorna resultado, colisión Conflict. Acción+receipt+audit genérico atómicos. No cambiar visibilidad para integration tests. |
| `application/ports.rs`: Clock/IdGenerator/SystemClock/RandomIds | Existen. Clock e IDs de captura se usan **dentro del closure nuevo** de with_receipt; así replay no los consume. Pasar now/ids al caso TX según norma. |
| `adapters/sqlite/library_repository.rs`: SqliteLibraryRepository.audit_change (línea142) | Primitiva reutilizable por el nuevo adapter vía trait LibraryRepository. Orden Library: request_id,action,entity_id; orden Knowledge normativo: request_id,item_id,action. Adaptar explícitamente; no otro repositorio Library. Evento efectivo debe contener entidad/before/after, separado del audit genérico del receipt. |
| `application/workflow.rs`: invalidate_effective_change (línea811), `application/workflow_ports.rs`: WorkflowRepository | Misma TX; una llamada/Paper deduplicado con [P2]. Exige tres fases; NOT_STARTED no cambia. Usa máximo revision; COMPLETED→NEEDS_REVIEW, IN_PROGRESS conserva estado. No consulta lifecycle ni reescribe PhaseAnswer/snapshot/completed_at. |
| `adapters/sqlite/workflow_repository.rs`: set_phase_clock/max_phase_revision (líneas115/139) | Actualizaciones durables dentro de TX, rollback las revierte. Fixture debe incluir tres fases. Captura no llama initialize_processing para fabricar contexto/iniciar P2. |
| `modules/reader/service.rs`, `application/request_registry.rs`, `desktop/maintenance.rs`, `application/settings.rs` | ReaderService retiene RequestPermit/OperationPermit mediante owned spawn+oneshot y controla writable/RecoveryStatus. Referencia T05b; T05a no edita ni registra estos servicios. Test TX no acredita cancelación IPC/permisos futuros. |

El wrapper receipt/audit existente genera timestamp/UUID propios con chrono/uuid; no tiene Clock/IdGenerator inyectados. Esto no exige cambiarlo: replay evita esos inserts. «Rollback sin consumir clocks» acredita revisiones durables de fases, no deshacer llamadas externas a generadores durante una acción fallida.

## Conservación0001/0002 y diagnóstico

migrate ya verifica ledger, crea snapshot previo si current>0, aplica pasos/ledger/user_version en una TX y verifica quick_check/FKs. Sus pasos locales permiten inyectar fallo tardío0003 sin editar runner. Fixture0002 debe usar checksums reales; no abrirla con actor actual antes de observar estado previo0003.

Blobs Git protegidos (no confundir con SHA256 SQL del ledger):

- 0001_library.sql: `57e2822dc1e5b2f1acb57930ffbbfd2019d4889d`.
- 0002_workflow.sql: `0394c1db6056b2ebf2697aa3da9c12ba8442a5f0`.

actor::prepare detecta versión>SCHEMA_VERSION antes de rama escritora, usa schema_probe/query_only, omite migración/WAL/synchronous/creación de manifest/directorios y no toma writer lock. Basta cambiar constante; actor/schema_probe/Windows siguen fuera de ownership. Tests futuro99 continúan aplicando.

## Discrepancia que debe resolver Sol

**Ownership omitido:** TASK05_PORTS§6 enumera shared tests para T05b pero omite workflow_migration.rs, que rompe inmediatamente con schema3. El brief deja su confirmación pendiente. Transferir ahora su ajuste acotado a T05a, manteniendo histórico0001→0002 y nuevas pruebas0002→0003 separadas.

No se halló otra necesidad de cambiar contratos o producto compartido. Firmas de edición/servicios completos son T05b; no implementar stubs para satisfacerlas en T05a. La precondición de tres fases se resuelve con fixture coherente, no cambiando helper ni iniciando fase desde captura.

Lecturas: Get-Content de instrucciones/brief/skill Karpathy; git show/grep/ls-tree/rev-parse del SHA indicado. Única escritura: este informe. **Sin tests/Cargo/npm, GUI, commits/merges, config, memorias ni subagentes.** Nuevas migraciones y pruebas siguen pendientes; ninguna afirmación PASS runtime. Sol debe comprobar estos cinco archivos al fijar BASE final.
