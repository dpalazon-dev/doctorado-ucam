# T04c fix1 — revisión independiente TypeScript

Fecha: 2026-10-04. Revisor `/root/task04c_typescript_review`. **BLOCKED para integración** por F1 HIGH abierto; F2 MEDIUM también confirmado. No hay otros defectos CRITICAL/HIGH nuevos demostrados en el delta. Typecheck propio PASS. No producto/configuración/commits/merges/subagentes/Cargo/build.

## Corte y alcance

BASE total T04c `d4a0c77b691d58493f03f622340005de0e6193ed`; BASE fix1 `c14c97dd8cfa2209e5596ef5ef8c39540973ad99`; producto congelado **`37169112abd10d7639265af589b76904e734b290`**; HEAD observado `1f0fbdf7d4a422bdf14b9aafdcd74dbffdcfe8db` añade sólo informe. Todas las líneas siguientes corresponden a producto37169112.

Worktree `C:/Users/david/Projects/Research-Workbench/.worktrees/task-04c-workflow-ui`. Scope local establecido con staged/unstaged vacíos. Lectura completa del delta de cinco archivos (App.test, PhaseWorkspace, PhaseAnswerForm, useWorkflow y workflow.test), contexto final completo del hook y composición App. Brief original/revisiones anteriores conservados como contexto; leídos briefs fix-1 y fix1-review, informe autor, revisión estática del orquestador y STATUS vigente. Se mantienen coding-standards/frontend-patterns. No metadata PR/CI remoto disponible para esta revisión local. No peer OpenViking confirmado; sólo fuentes locales.

## Matriz consolidada

CLOSED significa cierre del defecto indicado por lectura del delta y focales identificadas, limitado a UI simulada; no aceptación nativa. Las filas OPEN están detalladas después.

| ID | Estado | Evidencia en el corte |
|---|---|---|
| S1/TS1 | **CLOSED** | `PhaseWorkspace.tsx:31` admite IN_PROGRESS o NEEDS_REVIEW con el resto de guardas. `workflow.test.tsx:591` PRE/P1 reconfirmadas con token11; focales propias2 PASS. Evaluate sigue sin completar localmente. |
| S2/TS2 | **OPEN — F1** | Listener por Paper, limpieza y refresh del consumidor actual funcionan cuando éste ya está montado (`useWorkflow.ts:234–241`). `workflow.test.tsx:755`, `:892` y probes positivos propios confirman save, unknown/retry y archive. Confirmación sin consumidor seguida de remount aún bloquea indefinidamente. |
| S3/TS3 | **CLOSED** | `useWorkflow.ts:559–562` examina todo el Paper y statuses dirty/saving/error/unknown/conflict. Guardas de handlers `:353`, `:434`, `:442`; UI usa hasDirty. Descarte visible para error/unknown (`PhaseAnswerForm.tsx:209`). `workflow.test.tsx:665` ambas direcciones y InvalidInput no retryable2 focales PASS. Ausencia inicial excluida. |
| S4/TS4 | **CLOSED** | editSequences por key (`useWorkflow.ts:272–273`), discard conserva monotonicidad `:459`, elecciones de conflicto aumentan `:511–513`/`:526–528`. `workflow.test.tsx:736` y probe propio con DOM actual confirman payload idéntico, edición nueva dirty, revision5 y baseline del payload antiguo. |
| S5/TS5 | **CLOSED** | fromAnswer usa empty si no hay fila (`useWorkflow.ts:81`); formulario distingue Sin respuesta/Guardado pendiente (`PhaseAnswerForm.tsx:206`). Focales `workflow.test.tsx:613` y `:656` PASS: ausencia, discard y PENDING durable. Carga no llama save. |
| S6/TS6 | **CLOSED** | `PhaseWorkspace.tsx:109–116` presenta datos/resolución/estructura/explicación P2 sin formularios; salidas vacías honestas. Focales `workflow.test.tsx:699`/`:723` PASS, cero save/evaluate/advance. |
| S7/TS7 | **CLOSED** | `useWorkflow.ts:466–492` relee Paper, fase consultada, respuestas, definición pin y fase activa; conserva draft y generation guards. `PhaseAnswerForm.tsx:189–200` compara cuatro campos locales/observados. Focales `workflow.test.tsx:387`/`:422` PASS, revisión5 para reaplicar con requestId nuevo y lectura tardía ignorada. |
| S8/TS8 | **CLOSED, defecto original** | Rechazo GateBlocked confirmado no crea pendingAction ni lectura automática (`useWorkflow.ts:409–412`). Reevaluación separada `PhaseWorkspace.tsx:143–144`. Focales `workflow.test.tsx:624`/`:639` PASS: lectura secundaria fallida no crea replay de transición. Coherencia de preview queda **OPEN F2** como regresión separada. |
| TS9 | **OPEN — F1 variante** | Botón ahora llama reload completo (`PhaseWorkspace.tsx:44`, `useWorkflow.ts:223–231`); retry directo correcto, focal `workflow.test.tsx:870` PASS con getPaper3 y sin segundo save. Pero consultar fase borra error/botón y no resuelve refreshRequired ni Paper; recovery total no está cerrado. |
| TS10 | **CLOSED** | `useWorkflow.ts:287–292` conserva writability, selecciona pending antes de lifecycle de operación nueva; retry ARCHIVED manda payload anterior. Focales `workflow.test.tsx:823`/`:853` PASS. Resultado histórico no aplica lifecycle a UI; `:892` y probe archive actual reciben relectura vigente. Sin bypass recovery ni nuevas escrituras readonly. |
| R1 | **CLOSED** | `App.test.tsx:140–153` envelope IpcResult<OpenPaperDto> tipado con ok=true llega al guard que debe descartarlo. Ambos outcomes focales PASS. pageIndex0 y file:///synthetic.pdf no son fixture nativa válida; mejora menor ya señalada por Root, no bloquea la guarda bajo prueba. |
| Gaps acotados | **CLOSED en evidencia simulada** | Hidratación Paper y fase tardía A→B (`workflow.test.tsx:783`/`:801`), capability=false (`App.test.tsx:164`), NOT_APPLICABLE explicado separado (`workflow.test.tsx:927`) focales PASS. Las pruebas de hidratación cambian Paper y desmontan como App; no prueban navegación concurrente de fase en un mismo montaje. La UI bloquea esos botones durante loading/busy. No se afirma teclado/foco nativo por atributos. |

## F1 HIGH — hidratación completa al montar no resuelve refreshRequired; consulta parcial oculta recuperación

**OPEN; corrobora el orquestador, sin duplicar el ID.** `src/features/workflow/useWorkflow.ts:223–250`, `:448–452`; `src/features/workflow/PhaseWorkspace.tsx:30`, `:41–44`, `:65`.

Sólo reload limpia refreshRequired en línea229. El efecto de montaje usa loadPaper y loadPhase directamente y no limpia ese flag. Si una mutación se confirma sin listeners, el callback antiguo fija refreshRequired=true y su reload retorna false por mounted=false. La sesión libera busy, pero al volver después la nueva hidratación exitosa no desbloquea. Error=null y no se ofrece Reintentar carga. Editar puede seguir, pero guardar/evaluar/transiciones permanecen bloqueados sin salida desde el workspace.

Secuencias independientes **reproducidas en este corte**:

1. Save diferido A→unmount (listeners.size0)→confirmación sin consumidor→remount después. Paper/respuestas actuales se muestran, draft confirmado, busy=false, refreshRequired=true; no Retry y Evaluar/Guardar deshabilitados.
2. goBack diferido→unmount→confirmación→remount después. Contexto PRE actualizado llega desde lectura; refreshRequired permanece true y no se puede evaluar ni recuperar.
3. Save confirmado con getPaper fallido→Retry visible→salir→remount con Paper/fase exitosos. Tercera getPaper observada, pero refreshRequired=true, error desaparece y Retry deja de existir.
4. Save confirmado con getPaper fallido→consultar P1. loadPhase borra error sin releer Paper ni limpiar flag. getPaper.calls sigue2, phase P1 visible, refreshRequired=true y se pierde el único Retry. Consultar/volver/remontar no recupera el bloqueo.

**Regla:** brief fix1 ruling refreshRequired: retry/relectura vigente completa debe desbloquear incluso tras remount; una consulta parcial no debe eliminar la recuperación de Paper pendiente. La operación confirmada no se reenvía.

**Corrección mínima:** hacer que toda hidratación completa vigente del consumidor resuelva la obligación de refresh, incluido montaje, y mantener acción de recuperar mientras siga requerida aunque cambie la consulta o se borre el error visual. No limpiar con una carga parcial de fase ni respuesta obsoleta. Considerar generación/identidad de confirmación si una carga completa ya en curso podría ser anterior a la mutación que acaba de confirmar. Mantener datos canónicos en backend y no añadir otra biblioteca/store.

**Regresiones de cierre:** invertir los asserts demostrativos de los cuatro probes; después de carga completa actual, refreshRequired=false y controles apropiados habilitados sin reenviar save/action. Si Paper no se pudo actualizar, recuperación completa sigue disponible al consultar otra fase. Repetir caso de save y acción confirmada sin listeners y variante refresh fallido→remount. Conservar drafts posteriores y tokens/lifecycle de la lectura actual, no del receipt histórico.

## F2 MEDIUM — rechazo GateBlocked conserva preview completa y permite otro avance sin reevaluar

**OPEN; corrobora el orquestador.** `src/features/workflow/useWorkflow.ts:409–412`; `src/features/workflow/PhaseWorkspace.tsx:31–32`, `:143–153`.

advancePhase devuelve GateBlocked confirmado, pendingAction se limpia y gateRejected se marca; gate completo previo no se invalida y canAdvance ignora gateRejected. El usuario ve Revisión preliminar completa y Volver a evaluar al mismo tiempo, con Completar PRE habilitado. Un segundo click emite otro advance sin segunda evaluate.

**Demostración propia:** evaluate.calls1, advance.calls2 y preview completa aún visible después del rechazo. El backend vuelve a evaluar: no se atribuye corrupción durable ni mutación rechazada que haya sucedido.

**Corrección mínima:** invalidar la preview que contradice el rechazo y exigir reevaluación explícita vigente antes de otro avance; no fabricar GateEvaluationDto ni volver a clasificar el rechazo como desconocido. Mantener resolución independiente de la lectura y draft/contexto conservados.

**Regresión:** complete preview→GateBlocked→preview completa ausente/acción avance deshabilitada, sin evaluate automático ni pendingAction; reevaluación explícita exitosa vuelve a habilitar sólo si demás guardas vigentes.

## Evidencia propia y límites

Desde el worktree congelado:

```powershell
git status --short
git rev-parse HEAD
git diff --staged --stat
git diff --stat
git diff --stat c14c97dd8cfa2209e5596ef5ef8c39540973ad99 37169112abd10d7639265af589b76904e734b290
git diff c14c97dd8cfa2209e5596ef5ef8c39540973ad99 37169112abd10d7639265af589b76904e734b290 -- src/features/workflow/useWorkflow.ts src/features/workflow/PhaseWorkspace.tsx src/features/workflow/PhaseAnswerForm.tsx src/app/App.test.tsx
git diff c14c97dd8cfa2209e5596ef5ef8c39540973ad99 37169112abd10d7639265af589b76904e734b290 -- src/features/workflow/workflow.test.tsx
. .\scripts\development-env.ps1
npm.cmd run typecheck
npm.cmd test -- --config work/task-04c-fix1-typescript-probes/vitest.config.ts
$task04cReviewPattern = '"needs_review_.*can_be_reconfirmed|missing_answer_is_initial|durable_pending_answer|unresolved_.*draft_blocks|p2_displays_existing|p2_empty_outputs|unknown_save_replay_after_discard|remounted_consumer_reconciles|conflict_preserves_draft|late_conflict_read_error|gate_blocked_requires_explicit|gate_reevaluation_read_failure|archived_remount_can_resolve|archived_replay_remains|retry_after_refresh_failure|remounted_archived_consumer|late_paper_hydration|late_phase_hydration|not_applicable_requires|late_reader_open|workflow_is_unavailable"'
npm.cmd test -- src/features/workflow/workflow.test.tsx src/app/App.test.tsx -t $task04cReviewPattern
npm.cmd test -- --config work/task-04c-fix1-typescript-probes/vitest.config.ts
git diff --check
```

- Git scope y typecheck `tsc -b`: exit0; no diffs locales de producto observados. ESLint sigue sin script/dependencia en el scaffold; no lint PASS atribuido.
- Primer run probes: exit0, 5 tests PASS/1 archivo; **asserts verifican los cinco comportamientos defectuosos observados** (cuatro F1, uno F2), no aprobación.
- Focales canónicas seleccionadas para verificar cierres concretos del delta: exit0, **24 PASS/32 skipped en2 archivos**, duración2.49s. No suite total repetida ni Cargo.
- Segundo run probes tras añadir tres escenarios positivos de cierre parcial: exit0, **8 PASS/1 archivo**, duración1.74s. Cinco reproducciones de defecto y tres correctos: consumidor remontado recibe unknown/retry idéntico y limpia listener; consumidor remontado recibe archive confirmado y permanece readonly; discard/retry mantiene input DOM actual nuevo dirty y baseline/revisión confirmados. La nueva ejecución se justifica por esos tres escenarios añadidos.

Probes conservados fuera de producto/Git en `C:/Users/david/Projects/Research-Workbench/.worktrees/task-04c-workflow-ui/work/task-04c-fix1-typescript-probes/{review.test.tsx,vitest.config.ts}`. Se reutilizó sólo el prefijo fixtures/helpers del archivo anterior en una **copia nueva**, sin modificar ni borrar `work/task-04c-typescript-probes`. Sin dependencias, scripts o harness canónico nuevos. Estos artefactos no entran en el typecheck de src. Todos los datos son sintéticos.

Un intento de focal con pipes de regex sin quotes transportados a npm.cmd no inició Vitest: cmd interpretó el pipe y reportó missing_answer_is_initial como comando no reconocido. Se corrigió pasando quotes dobles literales en la variable de patrón y guardando exit inmediato. No se usa el exit final de una lectura Git posterior para afirmar éxito del primer intento. También se intentó leer task-04c-fix1-brief inexistente; rg encontró task-04c-fix-1-brief y se leyó correctamente. Ninguno de esos fallos es fallo de checks de producto ni aprobación.

Autor informa check.ps1 exit0 con122 tests UI/181 entradas Rust: evidencia del autor identificada en task-04c-fix1-report, **no ejecución propia de esta revisión**. Tauri build está diferido por F1; no se ejecutó ni se afirma PASS. No GUI, PDF render, SQLite entre procesos, instalación, machine limpia, foco/teclado/DPI nativos. Tampoco se atribuye pérdida durable a F1/F2.

## Resultado

Revisión terminada: **integración BLOCKED** en37169112. S1/S3–S8/TS10/R1 cerrados dentro del alcance indicado; S2 y recuperación total TS9 siguen OPEN por F1. F2 es la regresión restante de preview. Las correcciones vuelven al mismo autor en fix2; se requieren nuevo SHA, revisión de delta y gates coordinados. Producto y HEAD documental conservados, sin merge.
