# T04b — corrección consolidada 1

**Estado:** DONE_WITH_CONCERNS — implementación y validación local completadas; revisión independiente del runtime final aún pendiente.
**Rama:** `agent/task-04b-workflow-backend`
**Worktree:** `C:\Users\david\Projects\Research-Workbench\.worktrees\task-04b-workflow-backend`
**BASE:** `7aa120e46ad180d85c638f39bbcbc9355bab79ca`
**HEAD de producto congelado:** `5fb4e084ed943831e2d8aa2bc3584ae076590836`

Esta corrección cierra las observaciones S3–S9 y TS-01 del brief sin alterar DTOs, wire, esquema publicado, ocho comandos, UI, T04a ni la frontera ABI aceptada. La última corrección de S3 valida el pin de destino antes de confirmar la fase origen.

## Requisitos y evidencia

| Requisito | Cambio y evidencia observable |
|---|---|
| S3 — los pines guardados controlan validación y proyección | Operaciones usan la definición fijada de cada fase. `advance` resuelve el pin destino antes de aceptar o escribir la fase origen; un destino no soportado retorna `UnsupportedCapability` sin efectos ni receipt. Regresiones `advance_rejects_unsupported_p1_destination_pin_before_accepting_pre` y `advance_rejects_unsupported_p2_destination_pin_before_accepting_p1` en `src-tauri/tests/workflow_ipc.rs`; los casos comprueban origen completo/CAS válido y prueba documental disponible. La consulta explícita de una definición inexistente conserva su contrato. |
| S4 — receipt y colisión preceden a capability | `durable_receipt_conflict_precedes_capability_and_new_p2_is_not_receipted` prueba conflicto de ID antes de capability y que P2 nueva no deja receipt exitoso. El recorrido de aceptación verifica replay tras cambiar la referencia documental sin repetir efectos. |
| S5 — auditoría específica y rollback | Eventos efectivos registran Paper, fase, revisiones y destino previo/posterior; no-op/replay no crean evento específico. `phase_acceptance_and_receipt_failures_rollback_all_workflow_changes`, `answer_change_audit_failure_rolls_back_answer_clock_and_receipt` y pruebas de aceptación inicial/reconfirmación comprueban auditoría y atomicidad. Las acciones tienen nombres cerrados; los cambios de respuestas permanecen en auditoría interna y no en logs. |
| S6 — aceptación, proof, disponibilidad y conservación del recorrido | `src-tauri/tests/workflow_acceptance.rs` recorre servicios reales: import, PRE, P1, volver/editar, light-read/archive y Library archive/restore. Aserta explícitamente PRE sin aceptar→touch P1 bloqueado, continue P1 sin aceptar→touch P2 bloqueado, P1 editada/`NEEDS_REVIEW`→touch P2 bloqueado; además PDF disponible→gate completo, PDF ausente→issue documental y advance bloqueado sin receipt, restauración→gate completo con hash original. El archivo/restore Library conserva la decisión `archive` previamente guardada y el contexto P2 fixture `NEEDS_REVIEW`; el test no archiva de nuevo tras restore. Prueba reopen confirma snapshot aceptado/hash. Los mutadores reservados `COMPLETED` rechazan sin efectos en `workflow_ipc.rs`. |
| S7 — migración 0001→0002 con datos existentes | `migration_0002_backfills_populated_v1_without_changing_library_records_and_reopens` compara snapshots exactos de tablas/datos v1 antes/después, backup y reopen con conexión original cerrada. Fixtures incluyen `NEW`, `ACTIVE`, `ARCHIVED`, `COMPLETED` y `ARCHIVED` desde `COMPLETED`; excluye de comparación sólo campos de procesamiento que 0002 actualiza. `failed_0002_rolls_back_ddl_seed_backfill_ledger_and_user_version_after_backup` verifica rollback y conservación exacta de datos/reopen tras fallo. |
| S8 — rango seguro de revisiones | `go_back_and_touch_enforce_safe_revision_range_without_receipts` rechaza fuera de `0..9007199254740991` como input inválido antes de CAS, sin cambios persistidos ni receipt; el orden replay existente se conserva. |
| S9 — frontera aplicación/repositorio | Application coordina los casos de uso y reglas puras mediante las primitivas ABI aceptadas; SQLite adapter conserva SQL/UoW, actor, recibos, prueba documental y permisos hasta commit/rollback. Sin SQL añadido a dominio/application, callbacks genéricos ni cambios de `LibraryRepository`. `workflow_acceptance`, `workflow_ipc`, `workflow_concurrency` y `workflow_migration` ejercitan los recorridos transaccionales y fallos. |
| TS-01 — fixture de avance exitoso | `src/shared/adapters/tauri/client.test.ts` usa `complete: true` para el resultado exitoso; Vitest y typecheck pasan. No cambian el cliente ni el wire. |

## Commits de producto

- `034fcc885b2705cff35d024774074e31ac2c8c00` — completa frontera application y pruebas de aceptación.
- `1941f45392e9221e4b26e1918cf57cf0d1a8000b` — auditoría con destinos before/after y capacidades de candidatos cerradas.
- `19ac3156edf9ef5e03dbdcc161d0cced53d8c6b6` — journey de aceptación y matriz de migración.
- `5fb4e084ed943831e2d8aa2bc3584ae076590836` — valida pin destino antes de aceptar fase origen.

## Validación observada sobre HEAD de producto

En cada shell se cargó `. .\scripts\development-env.ps1`. Ambos comandos se ejecutaron desde el worktree indicado:

1. `powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/check.ps1` — exit **0**. Incluyó typecheck, Vitest (10 archivos, 69 pruebas), build Vite, rustfmt, Clippy con warnings como errores, suite Cargo completa y validación de contratos. Log completo: `C:\Users\david\Projects\Research-Workbench\work\task-04b-fix1\check-final.log`; sidecar del exit observado: `C:\Users\david\Projects\Research-Workbench\work\task-04b-fix1\check-final.exit` (`0`).
2. `npm.cmd run tauri:build -- --debug --no-bundle` — exit **0**. Generó `C:\Users\david\Projects\Research-Workbench\work\cargo-target\debug\research-workbench.exe`. Log completo: `C:\Users\david\Projects\Research-Workbench\work\task-04b-fix1\debug-final.log`; sidecar del exit observado: `C:\Users\david\Projects\Research-Workbench\work\task-04b-fix1\debug-final.exit` (`0`).

Los logs son capturas completas de stdout/stderr; los sidecars registran el exit observado inmediatamente tras cada comando. La aplicación web emite la advertencia ya existente de chunk JavaScript mayor de 500 kB. No se afirma validación de GUI, instalador ni máquina limpia; QA de instalación y T04c siguen separados.

## Revisión y límites

El autor verificó `git diff --check` para el delta de producto y mantuvo el worktree limpio al congelar `5fb4e08`. Root leyó el delta S3 final y verificó los dos logs completos y sus exits `0`. La lectura estática de seguridad anterior cubrió `1941f45`; no se presenta como revisión del delta final. Las revisiones independientes del runtime final (incluidas SPEC y Rust) aún no constan cerradas en este informe. Por eso el estado es `DONE_WITH_CONCERNS`, no autorización de merge. No se integró ni publicó esta rama.

Informe histórico T04b: [task-04b-report.md](task-04b-report.md). Este documento sustituye su estado de checkpoint para la corrección consolidada 1.
