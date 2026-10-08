# T04c fix1 — revisión independiente SPEC/calidad

Fecha: 2026-10-04. Revisor: `/root/task04c_spec_review`. **BLOCKED para integrar producto `37169112abd10d7639265af589b76904e734b290`**: F1 HIGH permanece abierto y F2 MEDIUM debe corregirse en el mismo corte acotado. Lista de defectos de esta revisión: **F1 y F2**, sin hallazgos importantes adicionales sobre el delta completo.

## Identidad y alcance

- BASE total T04c: `d4a0c77b691d58493f03f622340005de0e6193ed`.
- BASE fix1: `c14c97dd8cfa2209e5596ef5ef8c39540973ad99`.
- Producto fix1: `37169112abd10d7639265af589b76904e734b290`.
- HEAD observado: `1f0fbdf`, sólo `docs/reports/task-04c-fix1-report.md` después del producto, sin cambios UI adicionales.
- Worktree: `C:/Users/david/Projects/Research-Workbench/.worktrees/task-04c-workflow-ui`.
- Delta: cinco archivos (`App.test.tsx`, `PhaseAnswerForm.tsx`, `PhaseWorkspace.tsx`, `useWorkflow.ts`, `workflow.test.tsx`), 580 inserciones y 106 eliminaciones. Leídos todos los cambios y su conexión con composición/contratos originales. Staged/unstaged y status del worktree vacíos en la inspección.

Se leyeron los briefs centrales original/fix1/revisión, revisiones SPEC/TypeScript originales, checkpoint del autor, informe del orquestador y fuentes actuales INTENT/STATUS/DOMAIN/CONTRACTS relevantes. Las reglas T04c/ADR-020 gobiernan el corte; T06 no lo amplía. Se mantiene Karpathy: escenarios concretos y corrección mínima, sin rediseño o umbrales cosméticos.

**Evidencia de esta revisión: lectura estática**, incluido diff completo inmutable por SHA y pruebas canónicas. No se ejecutaron probes, npm, Cargo, tests, gate, build, GUI o instalador. Root comunicó gate observado exit0, 122 UI y 181 entradas Rust; el informe del autor identifica sus logs en `work/task-04c-fix1`. Esa ejecución se atribuye al autor/orquestador, no a este revisor. Build Tauri diferido. La revisión TypeScript de fix1 no estaba publicada al comenzar; se leyó completa al publicarse antes de cerrar este informe. Sus cinco reproducciones (cuatro F1 y una F2), tres probes positivos y 24 focales PASS se atribuyen a ese revisor; no se duplicaron sus pruebas. Confirma la misma lista F1/F2 y no agrega otro defecto importante demostrado.

## Defectos abiertos para fix2

### F1 [HIGH] — Un refresh pendiente puede quedar bloqueado aunque el Paper vuelva a cargarse correctamente

**Archivo/línea del corte:** `src/features/workflow/useWorkflow.ts:243`. Relacionadas: `:223–230`, `:234–240`, `:330–332`, `:448–452`; acción de recuperación condicionada por `error` en `src/features/workflow/PhaseWorkspace.tsx:41–44`.

**Confirmación independiente:** sólo `reload` limpia `state.refreshRequired` después de cargar Paper y fase. La hidratación de montaje llama directamente `loadPaper` y `loadPhase`, y no limpia ese estado. El listener sólo reacciona a confirmaciones mientras está registrado; no recibe la confirmación que ocurrió antes del montaje nuevo. Consultar fase llama sólo `loadPhase`, que limpia el error visual y conserva `refreshRequired`.

**Tres secuencias concretas del mismo defecto:**

1. A guarda con resultado diferido; sale a Biblioteca; el save se confirma mientras A no tiene consumidor. El hook antiguo fija `refreshRequired=true`; su refresh sale por `mounted=false`, y libera busy. Volver después a A hidrata Paper/fase correctamente, pero el flag sigue true. Guardar/evaluar/advance/contexto permanecen bloqueados, sin error ni botón para reconciliar.
2. Una mutación se confirma estando A visible, pero falla `getPaper` o la carga posterior; A conserva el flag y muestra recuperación. Salir y volver después provoca el mismo bloqueo tras una hidratación correcta sin limpieza del flag.
3. Tras confirmación+fallo de `getPaper`, consultar otra fase elimina el error y el botón Reintentar carga mediante `selectPhase/loadPhase`. No se ha releído Paper, el flag sigue true y la sesión pierde la única recuperación ofrecida. No se debe limpiar el flag basándose sólo en esa lectura parcial.

**Impacto:** operación durable correctamente confirmada, pero sesión UI atrapada sin camino de relectura; puede editar nuevos drafts que no podrá guardar en esa sesión. No se demuestra pérdida SQLite ni debe reenviarse la mutación para salir del bloqueo.

**Regla:** S2 y ruling de fix1 requieren reconciliación vigente tras confirmación aun a través de remontajes, con tokens/lifecycle desde lecturas actuales; TS9 requiere recuperación pertinente de Paper y no sólo respuestas. El flag transitorio debe impedir usar tokens viejos y poder resolverse por una relectura completa.

**Corrección mínima:** toda hidratación completa y vigente de Paper + fase consultada + fase activa relevante debe completar la reconciliación que corresponda a esa sesión; si sigue requerida, presentar recuperación explícita aunque no exista `error` o se consulte otra fase. Preservar guards de generación y drafts; no limpiar por lecturas parciales, no crear otra incertidumbre ni reenviar la mutación confirmada. No añade contrato ni canonstore.

**Regresiones de cierre:** reproducir las tres secuencias; afirmar busy/refreshRequired resueltos sólo tras lecturas vigentes, controles/tokens/lifecycle correctos y una sola mutación. Para la tercera, el botón de recuperación debe sobrevivir a consultar fase y volver a obtener Paper. Incluir resultado de advance archivando y fallo de carga de fase activa, además de save. Los probes ya ejecutados por TypeScript se reutilizan como evidencia/referencias; no se requiere un harness nuevo.

El test `remounted_consumer_reconciles_confirmed_save_without_extra_input` (`workflow.test.tsx:755`) vuelve a montar A **antes** de resolver el save: ejercita la notificación del listener, no la ventana con cero consumidores. El test `retry_after_refresh_failure_reloads_paper_before_phase_and_does_not_resend_save` (`:870`) pulsa directamente el botón; no consulta otra fase ni desmonta antes del retry. No cierran F1.

### F2 [MEDIUM] — GateBlocked conserva una preview completa y permite repetir advance sin reevaluación

**Archivo/línea del corte:** `src/features/workflow/useWorkflow.ts:409–412`; consumidor `src/features/workflow/PhaseWorkspace.tsx:31–32` y render `:157`.

**Confirmación independiente:** advance rechazado `GateBlocked` limpia pendingAction y pone `gateRejected=true`, pero no borra `gate`. La condición canAdvance comprueba gate.complete y no `gateRejected`. Tras finally, busy=false; si no hay otros bloqueos, el botón de completar sigue habilitado y continúa apareciendo “Revisión preliminar completa”, junto a un botón de reevaluación.

**Escenario:** preview completa → advance reevalúa y devuelve GateBlocked → se mantiene fase/datos, pero UI conserva la preview como vigente y ofrece de nuevo completar. Puede repetir nuevas operaciones sin una nueva lectura del gate. Backend conserva su autoridad y vuelve a rechazar, por lo que no se atribuye pérdida durable; la UI presenta información contradictoria justo después del rechazo.

**Regla:** el brief original invalida preview obsoleta, exige autoridad del backend y reevaluación explícita tras GateBlocked sin inventar GateEvaluationDto. S8 se corrigió respecto a clasificación de incertidumbre, pero esta coherencia de preview quedó abierta.

**Corrección mínima:** invalidar la preview al rechazo confirmado (sin fabricar issues desde error.details) y mantener la acción explícita de reevaluación. No habilitar nuevo advance hasta una evaluación vigente suficiente. Mantener los drafts y contexto; no hacer reevaluación automática ni repetir el avance.

**Regresión de cierre:** después de GateBlocked assert ausencia de preview completa y botón de completar deshabilitado; cero evaluate automáticos y cero pendingAction. Una reevaluación explícita completa puede habilitar de nuevo el avance. El test `gate_blocked_requires_explicit_reevaluation_and_never_becomes_pending` (`workflow.test.tsx:624`) verifica ausencia de lectura automática/replay, pero no la invalidación del gate anterior; `:1025` reevalúa enseguida y tampoco comprueba esa ventana.

## Matriz de cierre de los IDs originales

`CLOSED` significa que el defecto original queda resuelto por los mecanismos y pruebas indicados en este corte; no significa gate nativo ni aceptación instalada. `OPEN` mantiene el residual señalado, sin duplicarlo como tareas independientes.

| ID | Estado | Evidencia exacta y límites |
|---|---|---|
| S1 / TS1 | CLOSED | `PhaseWorkspace.tsx:31` admite IN_PROGRESS y NEEDS_REVIEW; sigue requiriendo fase activa, gate y guards. `workflow.test.tsx:591` parametriza PRE/P1 con revisión11 y afirma advance explícito con ese token. Evaluar no asigna COMPLETED. |
| S2 / TS2 | OPEN — F1 | PaperSession agrega listeners/editSequences/refreshRequired; cleanup de listener `useWorkflow.ts:240`, guard de generación al salir y notificación en bump(true). `workflow.test.tsx:755` prueba consumidor nuevo ya montado antes de confirmación y `:892` preserva ARCHIVED frente a receipt histórico ACTIVE. Falta reconciliar confirmación sin consumidor/refresh fallido al volver o consultar (F1). El caso unknown/replay entre instancias no tiene test canónico diferido específico nuevo; TypeScript informó un probe positivo que confirma notificación, retry exacto y limpieza de pending/listener. |
| S3 / TS3 | CLOSED | `hasUnresolvedDrafts` al final del hook recorre todos los drafts del Paper e incluye error; lo usan evaluate/advance/contexto y UI. Empty no bloquea. `workflow.test.tsx:665` parametriza ambas direcciones PRE/P1 con rechazo InvalidInput y desbloqueo por discard; `PhaseAnswerForm.tsx:209` ofrece descarte en error/unknown/dirty. |
| S4 / TS4 | CLOSED | `editSequences` conserva máximo por draft; edit/reapply/useSaved crecen y discard conserva la identidad máxima, sin volver a cero. Confirmación sólo reemplaza valores con identidad igual. `workflow.test.tsx:736` prueba unknown→discard→edit→retry con payload idéntico y última edición dirty. Baseline/revision de la rama dirty se actualizan desde result.data en `useWorkflow.ts:324`; TypeScript informó probe positivo sobre DOM actual que afirma baseline y revisión5. No se atribuye ejecución de ese probe a SPEC. |
| S5 / TS5 | CLOSED | `fromAnswer` distingue empty y saved; Form anuncia Sin respuesta guardada frente a Guardado · pendiente. `workflow.test.tsx:613` ausencia/discard y `:656` PENDING durable. No se generan saves al hidratar. |
| S6 / TS6 | CLOSED | `PhaseWorkspace.tsx:108` proyecta baseline P2: texto/resolución/estructura/explicación y vacío. `workflow.test.tsx:699` datos legibles sin Save/Evaluate/Advance y `:723` vacío. No se implementa edición P2/candidatos ni P3/P4. |
| S7 / TS7 | CLOSED | ReviewConflict relee Paper/fase/respuestas, definición pinned y fase activa distinta con generación; no reemplaza draft. Form `:188–198` presenta cuatro campos locales/observados como texto. `workflow.test.tsx:387` usa datos externos distintos, ve valores/contexto/slr y conserva draft; reaplicación envía identidad nueva con revisión5. `:422` conserva protección del error tardío al consultar P2. |
| S8 / TS8 | CLOSED; F2 separado | `runAction:409–412` no hace evaluate automático ni crea pendingAction para GateBlocked. Reevaluación es acción explícita. `workflow.test.tsx:639` falla esa lectura y conserva ausencia de replay/transición incierta; `:1025` vuelve a obtener issues backend. La preview aún obsoleta se recoge exclusivamente en F2. |
| TS9 | OPEN limitado a F1 | El botón `PhaseWorkspace.tsx:44` llama reload completo; `workflow.test.tsx:870` vuelve a getPaper y obtiene ARCHIVED/P1/slr, readonly y save.calls1. El recorrido directo original está corregido. La misma recuperación deja de existir al consultar/desmontar tras el fallo: F1, no un tercer defecto. |
| TS10 | CLOSED | `save:286–292` diferencia replay existente de nuevo save, mantiene writable y lifecycle guard para nuevas operaciones; args del pending se reutilizan sin reconstruir. `workflow.test.tsx:823` replay idéntico tras ARCHIVED y cero save nuevo; `:853` writable=false bloquea ese replay. `:892` receipt histórico ACTIVE no reabre una vista ARCHIVED porque manda relectura actual. F1 conserva el bloqueo ante refresh pendiente; no se adopta lifecycle histórico como reparación. |
| R1 / gaps concretos | CLOSED en alcance UI | App.test.tsx:137–156 usa IpcResult<OpenPaperDto> de éxito tipado real, y :164 capability=false sin llamadas Workflow. `workflow.test.tsx:783/:801` prueban Paper/fase A tardíos después de montar B; :927 prueba NOT_APPLICABLE con explicación separada. Guards de generación para selección de fase en mismo Paper permanecen por lectura. Estos mocks acreditan UI/payload, no renderer, SQLite ni teclado nativo. |

## Calidad, alcance y límites restantes

El delta conserva ownership de UI y contratos/DTOs generados, sin Rust, SQL, filesystem, wire, deps, configuración o Reader de producto. Las decisiones P1 siguen estructuradas, sin archivo al guardar ni parsing de texto; lifecycle actual procede de getPaper tras confirmación. Lecturas no inician fases. Pending se conserva con payload exacto y backend decide replay. No hay nuevo store canónico, localStorage, timeout durable, cancelWorkflow ni recuperación por restauración automática. Formularios/lecturas siguen presentando texto React, labels/fieldset/errores y estados; no se introducen HTML ejecutable o logs de payload.

Las pruebas añadidas son de comportamiento y diferidos, sin timers frágiles. Los asserts de bloqueo global/payload/read-only son pertinentes. No se exige dividir archivos por tamaño para aprobar este fix. R1 contiene `pageIndex:0` y `file:///synthetic.pdf`, valores poco representativos del contrato real, pero esa respuesta debe descartarse antes de Reader; el envelope ahora permite detectar la retirada de la guarda. Ajustar esa fixture a posición/protocolo reales es mejora menor si se toca, no un tercer bloqueo ni prueba de renderer.

Persistencia entre dos procesos, QA nativa PRE/P1/ramas/PDF indisponible, teclado/foco/DPI, Tauri debug e instalación siguen evidencias separadas. No se afirma su fallo ni su éxito con mocks. El check verde comunicado no cierra F1/F2: sus ventanas no están en los asserts actuales.

## Comandos propios y resultados

Desde el worktree congelado se ejecutaron sólo lecturas:

```powershell
git diff --staged
git diff
git diff --stat c14c97d 37169112abd10d7639265af589b76904e734b290
git diff --name-only c14c97d 37169112abd10d7639265af589b76904e734b290
git log --oneline -5
git diff c14c97d 37169112abd10d7639265af589b76904e734b290 -- src/features/workflow/useWorkflow.ts
git diff c14c97d 37169112abd10d7639265af589b76904e734b290 -- src/features/workflow/PhaseWorkspace.tsx src/features/workflow/PhaseAnswerForm.tsx src/app/App.test.tsx
git diff c14c97d 37169112abd10d7639265af589b76904e734b290 -- src/features/workflow/workflow.test.tsx
git diff --stat 37169112abd10d7639265af589b76904e734b290 1f0fbdf
git status --porcelain
```

Git exit0; staged/unstaged/status vacíos. `git diff --check c14c97d 37169112abd10d7639265af589b76904e734b290` desde checkout central también exit0. Se usaron `git show SHA:ruta`, Get-Content, Select-Object y Select-String para completar lectura inmutable y localizadores. Una ruta inicial `task-04c-fix1-brief.md` no existía: se localizó y leyó la correcta `task-04c-fix-1-brief.md`. El primer intento de leer TypeScript fix1 falló porque aún no existía; se leyó completo después desde la ruta central al publicarse. Salidas agrupadas truncadas se releyeron por segmentos. Ninguno de esos fallos acredita aprobación.

La única escritura de este revisor es este nuevo informe central absoluto. Sin cambios de producto, commits, merges, subagentes, memorias de proyecto o configuración. Peer OpenViking no verificado: fuentes locales.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH | 1 | warn |
| MEDIUM | 1 | info |
| LOW | 0 | pass |

Verdict: **WARNING — F1 HIGH debe cerrarse antes de integrar; estado operativo BLOCKED.** F2 forma parte de la lista acotada para fix2. No hay nuevos contratos o ampliaciones de alcance solicitados. Corregir con el mismo autor, identificar nuevo SHA y revisar cierres antes de ejecutar el build Tauri pendiente y el gate de integración coordinado.
