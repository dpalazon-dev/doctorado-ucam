# T04b — workflow backend

**Estado:** NEEDS_CONTEXT (checkpoint verificado; cierre de aceptación bloqueado por revisión SPEC y brief correctivo consolidado pendiente)  
**Rama:** `agent/task-04b-workflow-backend`  
**Worktree:** `C:\Users\david\Projects\Research-Workbench\.worktrees\task-04b-workflow-backend`  
**BASE:** `ce15c95dc6404aae1cb158ca6c4a1fe576b70505`  
**HEAD de producto:** `a051d88ca4c8127fe4749293b0b03cecc2923a14`

T04b conecta el workflow PRE/P1 en backend: reglas puras en dominio, coordinación por puertos, persistencia SQLite y migración 0002, helper transaccional de import/recibos, ocho comandos IPC cerrados y pruebas focales de protocolo, migración, atomicidad y concurrencia. No cambia DTO ni contrato wire, no implementa GUI/T04c y no modifica archivos de producto de T04a.

## Requisitos y evidencia focal

| Requisito | Evidencia principal |
|---|---|
| Definiciones PRE/P1 fijadas y hash verificable; reglas de gate/completion puras y capacidades futuras cerradas | `workflow_gates`: 6/6; `workflow_ipc::definition_command_uses_dispatcher_and_returns_pinned_definition`, `p3_candidate_dispatch_remains_capability_gated` |
| Guardado con CAS, replay antes de IO, NOT_STARTED bloqueado sin efectos, edición de fase anterior conserva contexto activo | `workflow_ipc::save_answer_persists_with_cas_replay_and_phase_clock`, `save_answer_to_not_started_phase_is_blocked_without_receipt`, `editing_started_prior_phase_keeps_later_active_context_and_invalidates_cas` |
| Evaluación sólo lectura; prueba documental y conflictos/error tipado preservados | `workflow_ipc::evaluate_gate_uses_live_document_proof_and_does_not_write`; `workflow_concurrency`: proof disponible/no disponible con referencia mutada, errores StorageUnavailable/PathNotAllowed/IntegrityFailure y comprobación de Drop tras commit/rollback |
| Avance PRE inicia P1 atómicamente, mueve contexto y reloj; replay devuelve recibo aunque desaparezcan archivo y cambie contexto; P1 distingue continuar/lectura ligera/archivo | `workflow_ipc::evaluate_gate_uses_live_document_proof_and_does_not_write`, `p1_advance_continue_light_read_and_archive_are_distinct_atomic_branches`; prueba de replay dentro del test PRE |
| Gate PRE caduca tanto para ANSWERED como UNKNOWN cuando la metadata de review type deja de coincidir | `workflow_ipc::unknown_review_type_gate_becomes_stale_when_paper_metadata_changes` |
| Reaceptar PRE conserva P1 ya iniciada, sus respuestas y snapshot; actualiza clock y activa contexto sin reiniciar datos | `workflow_ipc::reaccepting_pre_reactivates_existing_p1_without_resetting_its_data` |
| Archivo P1 atómico con ciclo de vida, auditoría y recibo; un fallo revierte también fase, contexto, reloj, snapshot y receipt | `workflow_ipc::p1_archive_failure_rolls_back_phase_lifecycle_audit_and_receipt` |
| Avance forward sólo con cadena aceptada; volver atrás y touch respetan revisión optimista | `workflow_ipc::go_back_preserves_phase_data_and_forward_touch_requires_accepted_chain` |
| Upgrade 0001→0002 poblado, backup/reopen, lifecycle NEW/ARCHIVED/COMPLETED, rollback de DDL/seed/backfill/ledger/user_version, integridad y clock invalidation | `workflow_migration`: 8/8, incluidos `migration_0002_backfills_populated_v1_without_changing_library_records_and_reopens`, `failed_0002_rolls_back_ddl_seed_backfill_ledger_and_user_version_after_backup`, `direct_in_progress_input_keeps_state_but_advances_its_clock`, `overlapping_inputs_deduplicate_each_phase_clock`, `initialized_processing_is_noop_after_phase_context_has_advanced` |
| Import integra inicialización de workflow y confirmación de recibo en la misma transacción | `library_integration::failed_workflow_initialization_rolls_back_import_confirmation`; suite completa Library: 56/56 |
| Cancelación durante proof y tras admisión no libera prematuramente la operación/registry; Library y Reader comparten registry; observación durable de Drop posterior a commit/rollback | `workflow_concurrency`: 5/5 (`cancelling_during_proof_keeps_shared_registry_and_operation_owner`, `cancellation_keeps_request_and_maintenance_until_reference_conflict_rolls_back`, `proof_handle_drop_observes_committed_advance`, `unavailable_proof_with_changed_reference_returns_conflict_without_transition`, `typed_proof_errors_propagate_without_becoming_gate_results`) |
| Los ocho comandos backend están conectados al cliente validado; payloads y respuestas inválidas se rechazan | `src/shared/adapters/tauri/client.test.ts`; incluido en Vitest completo 69/69 y typecheck |

## Commits propios

- `572a012` carga definiciones PRE/P1 fijadas.
- `18fd936` inicialización de workflow, migración y backfill/import.
- `7621da7`, `d959131`, `128027e`, `c7436e6` añaden evidencia de ownership, cancelación, rollback, inicialización, archivo y replay.
- `5c35067` conecta los comandos backend PRE/P1.
- `929c6e4` inicia P1 en el avance PRE y valida freshness de UNKNOWN.
- `a051d88` preserva P1 ya iniciada al reaceptar PRE y mantiene checks de concurrencia limpios.

## Validación

En cada shell se cargó `. .\scripts\development-env.ps1`.

1. Shell session `42423`: `. .\scripts\development-env.ps1; powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/check.ps1` — exit observado **0**. Typecheck, Vitest (10 archivos/69 pruebas), build Vite, rustfmt, Clippy, tests Rust (170 pasaron, 0 fallaron; doc-tests 0) y comprobación de contratos.
2. Shell session `86889`: `. .\scripts\development-env.ps1; npm.cmd run tauri:build -- --debug --no-bundle` — exit observado **0**. Ejecutó el beforeBuild Vite y compiló `C:\Users\david\Projects\Research-Workbench\work\cargo-target\debug\research-workbench.exe`.
3. Regresiones durante corrección: ambas nuevas regresiones fallaron antes del arreglo con la condición defectuosa y luego pasaron; matriz focal final workflow IPC 11/11, gates 6/6, migración 8/8, concurrencia 5/5 y rollback de inicialización Library 1/1.

Los resúmenes y archivos exit observados están en:

- Log: `C:\Users\david\Projects\Research-Workbench\work\task-04b\full-check.summary.log`; exit: `C:\Users\david\Projects\Research-Workbench\work\task-04b\full-check.exit` (`0`).
- Log: `C:\Users\david\Projects\Research-Workbench\work\task-04b\debug-build.summary.log`; exit: `C:\Users\david\Projects\Research-Workbench\work\task-04b\debug-build.exit` (`0`).

No se guardó tee íntegro de stdout; los archivos `*.summary.log` son resúmenes transcritos de la salida observada, y `*.exit` son sidecars con el exit transcrito manualmente (no los generó el proceso). Estos resultados fueron ejecutados y observados por el autor de T04b; no son una verificación independiente. Vite emitió la advertencia existente de chunk >500 kB. La compilación debug no prueba instalación ni ejecución en máquina limpia. QA de instalación limpia y GUI T04c continúan en sus tareas independientes.

## Autorrevisión

`git diff --check` pasó antes de congelar los cambios. HEAD de producto es `a051d88ca4c8127fe4749293b0b03cecc2923a14`; no hay cambios de producto sin commit. La revisión SPEC independiente está en `C:\Users\david\Projects\Research-Workbench\.superpowers\sdd\IMPLEMENTATION\task-04b-spec-review.md`: inspeccionó sólo el corte original `5c35067`, lo bloqueó y dejó S3–S9 abiertos. S1/S2 se corrigieron posteriormente en `929c6e4` y `a051d88` por indicación del orquestador; ese delta aún no tiene rereview SPEC independiente. No empiezo S3–S9 ni doy la aceptación por cerrada antes del brief correctivo consolidado solicitado por Sol, especialmente la resolución de arquitectura S9. No integres este checkpoint como DONE; cualquier continuación usará el mismo autor y deberá añadir revisión y evidencia para cada corrección.

## Corrección consolidada 1

El checkpoint anterior queda sustituido por la corrección consolidada 1, implementada desde BASE `7aa120e46ad180d85c638f39bbcbc9355bab79ca` y congelada en el HEAD de producto `5fb4e084ed943831e2d8aa2bc3584ae076590836`. Se cerraron S3–S9 y TS-01; `scripts/check.ps1` y `tauri:build -- --debug --no-bundle` pasaron con exit 0 y logs completos bajo `C:\Users\david\Projects\Research-Workbench\work\task-04b-fix1`. La última regresión valida que `advance` resuelve el pin de destino antes de aceptar/escribir el origen. El estado de entrega es `DONE_WITH_CONCERNS`: falta cerrar revisión independiente del runtime final, así que esta rama aún no está lista para merge.

Informe y mapa requisito→prueba: [task-04b-fix1-report.md](task-04b-fix1-report.md). Copia central: `C:\Users\david\Projects\Research-Workbench\.superpowers\sdd\IMPLEMENTATION\task-04b-fix1-report.md`.
