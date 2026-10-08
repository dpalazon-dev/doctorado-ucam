# T04c fix2 — revisión independiente SPEC/calidad acotada

Fecha: 2026-10-05. Revisor: `/root/task04c_spec_review`. **PASS en el alcance revisado:** F1 y F2 CLOSED; sus residuales S2/TS9 CLOSED. Ningún nuevo defecto CRITICAL/HIGH/MEDIUM demostrado en el delta completo. Este resultado habilita continuar los gates coordinados; no acredita merge, build Tauri, funcionamiento nativo ni instalación.

## Corte y método

- BASE fix2: `1f0fbdf7d4a422bdf14b9aafdcd74dbffdcfe8db`.
- Producto congelado: `5d35f03b13c4e85a0944296bac20e593cf494b5c`.
- HEAD observado: `67446412f9c2f82cab63ccb5bf02d3dff695bb84`; añade únicamente `docs/reports/task-04c-fix2-report.md` después del producto.
- BASE total T04c conservada: `d4a0c77b691d58493f03f622340005de0e6193ed`.
- Worktree leído: `C:/Users/david/Projects/Research-Workbench/.worktrees/task-04c-workflow-ui`.

Staged/unstaged/status vacíos. Leído **todo** el delta de tres archivos: `PhaseWorkspace.tsx`, `useWorkflow.ts`, `workflow.test.tsx` (250 inserciones/18 eliminaciones), más el contexto final de las lecturas, listeners, mutaciones y controles. Se usó git show/diff sobre SHA; no WIP. Leídos briefs centrales fix-2/revisión, reporte del autor, cierre SPEC fix1 y las fuentes actuales INTENT/STATUS; contratos y reglas T04c ya contrastados en revisiones anteriores permanecen aplicables. Karpathy conserva alcance y escenarios; no se reabrieron mejoras fuera de fix2 ni se solicitaron contratos nuevos.

Evidencia propia: **lectura estática y consulta de logs existentes**, sin ejecutar npm/tests/Cargo/check/build/probes/runner/GUI. Al publicarse se leyó completo `task-04c-fix2-typescript-review.md`: coincide con F1/F2/S2/TS9 CLOSED y ningún importante nuevo; informa typecheck0, siete focales canónicas PASS y cinco probes históricos invertidos PASS. Esa ejecución se atribuye a TypeScript, no a SPEC, y no se duplicó. Peer OpenViking no verificado, fuentes locales. Única escritura propia: este informe central absoluto.

## Cierres

### F1 CLOSED — Reconciliación completa al montar y recuperación persistente

`useWorkflow.ts:249` monta mediante `reload(true)` en lugar de separar loadPaper/loadPhase sin limpiar obligación. `reload:226–236` captura la versión de refresh, relee Paper y selecciona la fase actual o consultada; loadPhase sigue obteniendo PhaseDto/respuestas/definición pin y fase activa si es distinta. Sólo una relectura completa exitosa puede limpiar refreshRequired; errores o generaciones obsoletas conservan la obligación. Cada confirmación save/back/touch/advance llama `requireRefresh` (`:566`) e incrementa refreshVersion: una lectura que comenzó antes de otra confirmación no borra una obligación más nueva.

La selección consultada se conserva en `selectedPhaseRef`, evitando que cambiarla recree reload y repita el efecto de montaje. El listener mantiene limpieza al desmontar. Las generaciones y guards mounted de las lecturas siguen descartando resultados antiguos. La solución mantiene exclusivamente estado de sesión transitorio, no una biblioteca canónica frontend.

`PhaseWorkspace.tsx:47` ofrece Reintentar carga mientras refreshRequired siga true aunque no haya error. Consultar otra fase puede limpiar un mensaje, pero no elimina la recuperación ni limpia el flag por esa lectura parcial. Los botones de recuperación se deshabilitan durante loading/busy para no iniciar duplicados por doble click. Guardas de mutación conservan bloqueo mientras falta reconciliar. Recuperar sólo relee, sin reconstruir/repetir la mutación ya confirmada.

Regresiones nuevas leídas en el corte:

| Localizador workflow.test.tsx | Recorrido y efecto afirmado |
|---|---|
| 918 | Save se confirma con A desmontado; montar después muestra valor confirmado, permite evaluar y advance con token10; save sólo una vez. |
| 953 | goBack se confirma sin consumidor; volver lee contexto PRE y token activo10 para siguiente avance; goBack sólo una vez. |
| 988 | Save confirmado + refresh getPaper fallido → remount con lectura actual; recuperación desaparece sólo al reconciliar, se usa token10 y no se reenvía save. |
| 1020 | Fallo getPaper → consultar P1 → Reintentar carga sigue disponible; relectura obtiene ARCHIVED actual y save.calls1. |
| 1045 | Falla lectura de fase activa distinta de la consultada; leer parcialmente P2 mantiene recuperación; reload vuelve a obtener Paper y fase activa P1, lifecycle ARCHIVED, sin otro save. |
| 1076 | Lectura iniciada antes de una confirmación y refresh concurrente fallido no adopta el Paper antiguo ni oculta recuperación; nueva relectura obtiene estado actual y save.calls1. Es una prueba simulada con dos consumidores, no dos procesos nativos. |

El test de generaciones concurrentes ejercita además la guarda que descarta el resultado viejo; el contador de confirmaciones complementa esa protección sin relajar las lecturas actuales. Mantiene drafts dirty/conflict/unknown existentes: loadPhase sólo reemplaza drafts saved/empty, y la rama de éxito save conserva una edición posterior mediante identidad local monotónica. No se adopta lifecycle/contexto histórico del receipt: las acciones siguen esperando relectura canónica y los controles siguen readonly en ARCHIVED.

### F2 CLOSED — Preview invalidada al rechazo confirmado

`useWorkflow.ts:411` pone gate=null cuando advance responde GateBlocked. Conserva pendingAction=null para ese rechazo confirmado y gateRejected para ofrecer reevaluación explícita. No llama evaluate automáticamente ni convierte el rechazo en resultado desconocido. canAdvance sigue requiriendo gate.complete; al eliminar la preview queda deshabilitado sin fabricar issues desde error.details.

`workflow.test.tsx:639` comprueba preview completa → GateBlocked → preview ausente y avance deshabilitado, evaluate.calls1/advance.calls1; sólo al pulsar Volver a evaluar aparece una nueva preview suficiente y vuelve a habilitarse el botón, evaluate.calls2 sin otro avance. Las demás guardas de drafts/phase/lifecycle siguen vigentes. El cambio no completa ni cambia contexto localmente.

## Matriz acotada

| ID | Estado | Alcance del cierre |
|---|---|---|
| F1 | CLOSED | Confirmación sin consumidor, fallo→remount, recuperación tras consulta parcial, fase activa pertinente y lectura anterior a otra confirmación. |
| S2 / TS2 | CLOSED | El residual que impedía reconciliar al volver queda resuelto por reload completo al montar. Notificación al consumidor ya montado, retry incierto y limpieza de listeners conservan cierre fix1. |
| TS9 | CLOSED | Recovery vuelve a Paper/contexto/fases y ya no desaparece al consultar o remontar; cero reenvíos de mutación confirmada. |
| F2 | CLOSED | GateBlocked invalida preview; advance espera reevaluación explícita y vigente. Clasificación confirmada de S8 permanece cerrada. |
| Nuevo importante introducido por fix2 | Ninguno demostrado | Delta completo revisado, sin cambios de wire, backend, deps, configuración o producto ajeno. |

Los cierres restantes S1/S3–S8/TS10/R1 conservan su adjudicación fix1 dentro del alcance UI. No se afirma una repetición independiente de sus suites completas. El fixture opcional App/Reader no cambió y no forma parte de fix2.

## Evidencia de ejecución ajena contrastada

Se leyeron los archivos del autor en `C:/Users/david/Projects/Research-Workbench/work/task-04c-fix2/`; Root ya informó haber contrastado los mismos resultados:

- `red.exit-code.txt=1`; red.stdout registra 5 fallos y 49 PASS. red.stderr contiene fallos de comportamiento: preview aún presente, llamadas de evaluación/avance ausentes por bloqueo y botón de recuperación desaparecido; no son errores de compilación usados como RED.
- `green.exit-code.txt=0`; green.stdout registra **56/56 PASS** en un archivo Workflow, duración4.05s.
- `typecheck.exit-code.txt=0`; typecheck.stdout identifica `tsc -b`.

Los comandos del autor son `npm.cmd test -- --run --reporter=dot src/features/workflow/workflow.test.tsx` y `npm.cmd run typecheck`, con development-env previamente cargado conforme su informe. **Son ejecuciones del autor, no de este revisor.**

Precisión de RED/GREEN: el log RED conserva el nombre `confirmed_archive_without_consumer_is_observed_on_later_mount`; el corte verde usa `confirmed_transition_without_consumer_uses_the_current_active_token_on_later_mount`, con goBack y comprobación token10. No se atribuye una identidad literal entre esos tests ni un nuevo caso de archive sin consumidor. GREEN contiene los siete casos añadidos actuales, incluidos los dos finales de fase activa/concurrencia que no estaban en RED. El cierre no se basa sólo en nombres o conteos: se contrastaron secuencias/asserts del código final y sus mecanismos. La preservación ARCHIVED ya cubierta en fix1 se conserva y los casos de recuperación nuevos leen lifecycle ARCHIVED.

## Comandos propios y límites

Desde el worktree congelado, sólo lectura:

```powershell
git diff --staged
git diff
git diff --stat 1f0fbdf7d4a422bdf14b9aafdcd74dbffdcfe8db 5d35f03b13c4e85a0944296bac20e593cf494b5c
git diff --name-only 1f0fbdf7d4a422bdf14b9aafdcd74dbffdcfe8db 5d35f03b13c4e85a0944296bac20e593cf494b5c
git diff 1f0fbdf7d4a422bdf14b9aafdcd74dbffdcfe8db 5d35f03b13c4e85a0944296bac20e593cf494b5c -- src/features/workflow/PhaseWorkspace.tsx src/features/workflow/useWorkflow.ts
git diff 1f0fbdf7d4a422bdf14b9aafdcd74dbffdcfe8db 5d35f03b13c4e85a0944296bac20e593cf494b5c -- src/features/workflow/workflow.test.tsx
git diff --check 1f0fbdf7d4a422bdf14b9aafdcd74dbffdcfe8db 5d35f03b13c4e85a0944296bac20e593cf494b5c
git diff --stat 5d35f03b13c4e85a0944296bac20e593cf494b5c 67446412f9c2f82cab63ccb5bf02d3dff695bb84
git log --oneline -4
git status --porcelain
```

Git exit0, diff-check sin hallazgos; staged/unstaged/status vacíos. Se usaron git show SHA:ruta y Get-Content/Select-String/Select-Object con paths absolutos o workdir explícito para leer código, briefs y logs. Ningún nuevo probe o ejecución de producto. No modificaciones de producto/config, subagentes, commits, merges o memoria de proyecto.

Gate completo de fix2, build debug Tauri y QA nativa/runner/procesos permanecen fuera de esta revisión y se coordinan después. UI simulada/typecheck no prueban SQLite entre procesos, PDF real, teclado/DPI, installer ni máquina limpia. La lectura estática no los afirma aprobados ni fallidos.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH | 0 | pass |
| MEDIUM | 0 | pass |
| LOW | 0 | pass |

Verdict: **APPROVE / PASS acotado para fix2 `5d35f03b`**. F1/F2/S2/TS9 cerrados, coincidente con revisión TypeScript; continuar los gates coordinados antes de integrar. Sin nueva ampliación de alcance solicitada.
