# T04c fix2 — revisión independiente TypeScript acotada

Fecha: 2026-10-05. Revisor `/root/task04c_typescript_review`. **DONE — F1/F2 CLOSED; S2/TS9 CLOSED en alcance UI simulada.** Sin hallazgos importantes nuevos demostrados en el delta. Apto desde esta revisión para continuar los gates coordinados; no constituye integración ni aceptación nativa.

## Identidad y scope

- BASE fix2 `1f0fbdf7d4a422bdf14b9aafdcd74dbffdcfe8db`.
- Producto revisado **`5d35f03b13c4e85a0944296bac20e593cf494b5c`**.
- HEAD observado `67446412f9c2f82cab63ccb5bf02d3dff695bb84`: diferencia desde producto sólo `docs/reports/task-04c-fix2-report.md` (41 líneas), no cambio de código.
- Worktree `C:/Users/david/Projects/Research-Workbench/.worktrees/task-04c-workflow-ui`.

Staged/unstaged y status vacíos. Leídos briefs centrales absolutos fix-2/fix2-review, informe autor y cierre fix1 conservado; diff completo de los tres archivos: PhaseWorkspace.tsx, useWorkflow.ts, workflow.test.tsx. Se revisó contexto de reload/listener/montaje, handlers de save/back/touch/advance y guardas de preview. No ampliación a mejoras ajenas al delta. Se mantienen coding-standards/frontend-patterns. No PR/CI remoto en este despacho: revisión local. Sin peer OpenViking confirmado; sólo fuentes locales. Sin cambios de producto/contratos/configuración/commits/merges/subagentes/Cargo/build.

## Cierres

| ID | Estado | Evidencia del producto5d35f03 |
|---|---|---|
| F1 | **CLOSED** | `useWorkflow.ts:226–254`: montaje usa reload(true), con Paper→fase seleccionada/activa pertinente antes de limpiar flag. `:227`/`:233` sólo limpia versión de refresh observada al iniciar. `:566–569` incrementa versión por confirmación. `PhaseWorkspace.tsx:47–50` mantiene Retry sin depender de error visual. Cuatro probes históricos invertidos y seis regresiones canónicas focales PASS. |
| S2/TS2 | **CLOSED, cierre del pendiente F1** | Save y goBack terminan con cero consumidores; montaje posterior rehidrata y desbloquea. Listener/refresco del consumidor ya remontado y unknown/replay/archive fueron verificados en fix1 y no cambiaron semánticamente. Ahora no queda la obligación permanente al volver después. Probes propios verifican listener.size0 antes de confirmar, busy=false y obligación true antes del remount, luego obligación false/controles operativos tras lectura completa. |
| TS9 | **CLOSED, recuperación completa** | Refresh Paper fallido→remount exitoso resuelve flag. Consulta parcial tras fallo no limpia la obligación ni elimina recuperación. Retry completo vuelve a llamar getPaper y después fases, sin reenviar save. Fallo de fase activa también conserva la recuperación; focal `workflow.test.tsx:1045` verifica lifecycle ARCHIVED y fase activa pertinente tras retry. |
| F2 | **CLOSED** | `useWorkflow.ts:407–412` limpia gate en GateBlocked confirmado y conserva pendingAction=null; sin evaluate automático. canAdvance sigue requiriendo gate completo (`PhaseWorkspace.tsx:31–32`). Focal `workflow.test.tsx:639` y probe independiente: preview ausente/avance disabled después del rechazo, click disabled no emite nuevo advance; reevaluación explícita habilita con resultado completo actual. |

Reglas preservadas por inspección: carga parcial no limpia refreshRequired; lectura obsoleta conserva guards mounted/generation; confirmación posterior incrementa refreshVersion; callback reload estable usa selectedPhaseRef para escoger fase vigente sin reinstalar montaje al navegar. Drafts dirty/error/unknown/conflict conservan valores durante hidratación; no se adopta lifecycle/contexto histórico desde un receipt y no se reenvía operación confirmada para refrescar. Nuevas mutaciones siguen bloqueadas mientras faltan lecturas pertinentes. El aviso de recovery puede aparecer durante la carga, pero su acción está disabled por loading/busy y vuelve a estar disponible tras fallo.

No se encontró nueva rotura importante en el delta. No se reabren los otros IDs cerrados en fix1 ni el minor opcional Reader, ajeno a estos tres archivos.

## Evidencia propia

Desde el worktree:

```powershell
git status --short
git rev-parse HEAD
git diff --staged --stat
git diff --stat
git diff --stat 1f0fbdf7d4a422bdf14b9aafdcd74dbffdcfe8db 5d35f03b13c4e85a0944296bac20e593cf494b5c
git diff 1f0fbdf7d4a422bdf14b9aafdcd74dbffdcfe8db 5d35f03b13c4e85a0944296bac20e593cf494b5c -- src/features/workflow/PhaseWorkspace.tsx src/features/workflow/useWorkflow.ts src/features/workflow/workflow.test.tsx
git diff --stat 5d35f03b13c4e85a0944296bac20e593cf494b5c HEAD
. .\scripts\development-env.ps1
npm.cmd run typecheck
$task04cFix2Pattern = '"confirmed_save_without_consumer|confirmed_transition_without_consumer|failed_refresh_survives_unmount|refresh_recovery_remains_available|active_phase_refresh_failure|mount_reconciliation_started|gate_blocked_invalidates_previous"'
npm.cmd test -- src/features/workflow/workflow.test.tsx -t $task04cFix2Pattern
npm.cmd test -- --config work/task-04c-fix2-typescript-probes/vitest.config.ts
git diff --check
```

Resultados propios: Git y typecheck canónico tsc -b exit0. Focal canónica **7 PASS/49 skipped,1 archivo**, exit0 (1.95s): save sin consumidor, transición sin consumidor con token10, refresh fallido/remount, consulta conserva recuperación, fallo de fase activa, hidratación anterior a confirmación y preview GateBlocked. No se repitió la suite total.

Probes independientes: **5 PASS,1 archivo**, exit0 (2.64s). Copia nueva en `C:/Users/david/Projects/Research-Workbench/.worktrees/task-04c-workflow-ui/work/task-04c-fix2-typescript-probes/{review.test.tsx,vitest.config.ts}`. Conserva fixtures y secuencias de los cinco probes demostrativos fix1, invierte expectativas correctas y añade retry/reevaluación explícitos donde corresponden. No modifica históricos fix1 ni probes originales, dependencias o harness canónico. Se verifica ahora: save y goBack sin consumidor recuperan; refresh fallido/remount recupera; consulta parcial mantiene flag+acción y retry completo limpia tras getPaper3 sin save2; GateBlocked elimina preview y no envía avance hasta reevaluar. Los nombres de secuencias heredados describen el antiguo defecto; los asserts de esta copia afirman su ausencia. Datos sintéticos; fuera de src/Git y de typecheck de producto.

ESLint no disponible en el scaffold, como comprobado en las revisiones previas; no se instala ni se atribuye lint PASS. No pruebas Cargo, check.ps1, build Tauri, GUI o drivers.

## Evidencia del autor contrastada

Leídos `work/task-04c-fix2/{red,green,typecheck}.{stdout.log,stderr.log,exit-code.txt}` desde la ruta central. Red exit1 con5 fallos de comportamiento/49 PASS; Green exit0 con56 PASS; typecheck exit0, stderr vacío en Green/typecheck. RED stderr identifica fallos por preview aún presente, evaluate no llamado tras remount y recuperación ausente; no fallos de compilación como sustituto de comportamiento.

Precisión: RED incluye `confirmed_archive_without_consumer_is_observed_on_later_mount`; el corte final reemplaza ese caso por `confirmed_transition_without_consumer_uses_the_current_active_token_on_later_mount` (goBack+token10). Los dos casos adicionales de fase activa/hidratación concurrente se añadieron después de RED. Por tanto los logs sustentan ciclo RED/GREEN general de F1/F2, no un RED exacto individual de cada uno de los siete tests finales. Los resultados verdes propios anteriores sí corresponden al producto congelado. No se atribuyen ejecuciones del autor como propias.

## Límite y entrega

Informe central es la única escritura operativa de revisión; los probes quedan como evidencia ignorada fuera de producto. Árbol del worktree y SHA producto preservados. **Cierres acotados demostrados; gate completo y Tauri debug no-bundle siguen pendientes del orquestador.** No se acredita SQLite entre procesos, QA nativa, protocolo/render PDF, teclado/DPI, instalador ni equipo limpio con estos mocks/typecheck. Esta revisión permite continuar validación, no declara merge ni hito integrado.
