# T04c — revisión independiente SPEC/calidad

Fecha: 2026-10-03. Revisor: `/root/task04c_spec_review`. Resultado: **BLOCKED para integración del corte revisado**, por cinco hallazgos HIGH. No se ha modificado producto, configuración, ledger ni STATUS.

BASE: `d4a0c77b691d58493f03f622340005de0e6193ed`.
HEAD producto congelado: `198f27b281d97810d16fb2955b620af4f99b1490`.
Worktree leído: `C:/Users/david/Projects/Research-Workbench/.worktrees/task-04c-workflow-ui`.
Commits del rango: `6e8ffcb` y `198f27b`. Once archivos UI/estilos/tests; 1873 inserciones y 8 eliminaciones. Sin Rust, DTO, wire, dependencias, capacidades ni configuración en el rango.

## Método y alcance de evidencia

Lectura estática completa del diff BASE..HEAD y de los archivos finales cambiados, con `git show HEAD:ruta` para mantener la identidad del corte. Se contrastaron AGENTS, INTENT, STATUS vigente, briefs centrales T04c implementación/revisión, brief T04, TASK04_PORTS, SPEC-003, DOMAIN fases/invalidation, CONTRACTS Workflow/ADR-020, WORKFLOW_GATES, definiciones PRE/P1/P2 v1 y QUALITY. Karpathy aplicado para limitar cada hallazgo a un escenario y una corrección acotada. T06/ADR-023 no amplía este corte.

No se ejecutaron npm, Cargo, tests, check.ps1, build, aplicación, GUI ni instalador: el autor tenía el gate/build en ejecución. Los resultados focales/typecheck informados al despacho son información recibida, **no ejecución observada por este revisor**. Los escenarios siguientes se deducen directamente de las ramas del código; requieren regresiones deterministas para su cierre. No se afirma pérdida durable SQLite, fallo nativo ni instalación probada.

## Hallazgos HIGH

### S1 — NEEDS_REVIEW no puede reconfirmarse

**Archivo:** `src/features/workflow/PhaseWorkspace.tsx:31`.

**Problema:** `canAdvance` sólo admite `phase.state === "IN_PROGRESS"`. Una PRE/P1 completada y luego editada queda NEEDS_REVIEW según el backend. La UI permite guardar y evaluar esa fase, pero incluso con gate completo, activa y sin drafts pendientes, el botón de completar permanece deshabilitado. El recorrido de corrección y reconfirmación queda cerrado.

**Regla:** DOMAIN invalidación determinista, SPEC REQ-003-06 y brief T04c: NEEDS_REVIEW exige `advancePhase` para volver a completar, aunque el contenido vuelva a pasar el gate.

**Corrección mínima:** permitir la acción explícita de reconfirmar en NEEDS_REVIEW, manteniendo fase activa, revisión actual, resultado backend y demás guardas. No convertir el estado localmente ni completar al evaluar.

**Prueba de cierre:** fixture PRE/P1 activa NEEDS_REVIEW; evaluate completo; click en acción explícita; assert llamada `advancePhase` con revisión observada, preservación de datos y estado confirmado. La prueba existente `needs_review_preserves_later_data` sólo consulta los datos, no prueba reconfirmación.

### S2 — Remontar el mismo Paper durante una operación deja una vista sin reconciliar

**Archivo:** `src/features/workflow/useWorkflow.ts:112` (también 42–61, 274–301 y 327–390); composición con remount en `src/app/App.tsx:201`.

**Escenario:** guardar A con promesa diferida; salir a Biblioteca; volver a A antes de resolver el guardado; terminar la hidratación nueva; resolver el envío antiguo. El `PaperSession` compartido cambia sus drafts/pending/busy, pero `bump`, `setPaper`, `setPhase` y el refresh pertenecen al hook desmontado. La sesión no dispone de suscripción o notificación para el hook nuevo. El workspace visible puede seguir mostrando Guardando/botones deshabilitados hasta otro render; un render posterior puede desbloquearlo con tokens/contexto/lifecycle leídos antes del resultado. El mismo problema ocurre con advance/touch/goBack, donde el resultado no guarda una reconciliación que llegue al workspace nuevo.

**Regla:** brief T04c, serialización y sesión: A→B→A y remount conservan operaciones y reconcilian el Paper original; las revisiones/contexto se releen tras mutación, sin depender de un hook que ya salió.

**Corrección mínima:** notificar a los consumidores actuales del Paper cuando cambie su operación, y reconciliar sus lecturas tras un resultado confirmado. Mantener drafts/operaciones transitorios, sin crear biblioteca canónica ni persistencia frontend. Guardar separado el resultado de sesión y su aplicación visual.

**Prueba de cierre:** A→Biblioteca→A con save/action aún pendiente; completar hidratación nueva antes de resolver; comprobar actualización visible sin otra interacción, limpieza de busy y refresh de tokens/contexto. Incluir fallo desconocido/retry y éxito de advance archivando. `late_mutation_updates_only_original_paper` resuelve sobre B; no regresa a A mientras sigue pendiente.

### S3 — Gate y transiciones ignoran drafts de otras fases y guardados rechazados

**Archivo:** `src/features/workflow/useWorkflow.ts:492` (consumidores 306, 395, 403 y 485).

**Problema consolidado:** `hasDirtyDrafts` filtra por la fase consultada e ignora status `error`. Dos recorridos dejan evaluate/advance o cambio de contexto habilitados con intención local no resuelta:

1. Editar una fase iniciada, consultar otra fase sin guardar/descartar y evaluar o activar allí. El draft anterior sigue en sesión, pero desaparece del bloqueo.
2. Editar una respuesta que tenía un baseline válido, obtener InvalidInput/GateBlocked no retryable al guardar, y pulsar Evaluar. El draft queda `error`, `pending=null`; evaluate lee el baseline antiguo y puede devolver completo. Advance puede actuar sobre una decisión confirmada anterior mientras la nueva edición rechazada sigue visible.

**Regla:** brief T04c: evaluar/avanzar sólo con drafts **de ese Paper** guardados o descartados explícitamente; los errores conservan intención local y no convierten el rechazo en resolución.

**Corrección mínima:** distinguir ausencia inicial de cambio local pendiente, y computar la condición para el Paper completo incluyendo errores con cambios no confirmados. Ofrecer resolución explícita para los drafts bloqueantes y aplicar la misma condición en controles y handlers.

**Prueba de cierre:** draft PRE mientras se consulta P1; draft P1 mientras se consulta PRE; InvalidInput no retryable sobre una respuesta previamente válida. Afirmar ausencia de evaluate/advance/transiciones hasta guardar o descartar deliberadamente. Las pruebas actuales de InvalidInput sólo comprueban la asociación del error.

### S4 — Descartar durante un retry incierto reutiliza el contador y puede pisar una edición nueva

**Archivo:** `src/features/workflow/useWorkflow.ts:415` (reset en 76, comparación de confirmación en 277); botón permitido en `src/features/workflow/PhaseAnswerForm.tsx:199`.

**Secuencia permitida por la UI:** editar (editVersion=1), guardar y perder la respuesta de transporte; editar de nuevo para tener status dirty; pulsar Descartar cambios (busy=false, aunque `pending` sigue existente); `fromAnswer` reinicia editVersion=0; escribir texto nuevo (editVersion=1); reintentar el payload original congelado. El éxito viejo compara `latest.editVersion === pending.editVersion` y acepta la igualdad 1===1; sustituye el texto nuevo por `fromAnswer(result.data)` aunque la edición sea posterior al envío.

**Impacto:** pérdida de edición local y falso estado Guardado para ese valor posterior. No se infiere pérdida durable backend: el payload viejo es el que el backend debía confirmar.

**Regla:** brief T04c: contador local de edición y éxito de una operación anterior no reemplazan valores posteriores; descartar sólo cancela edición local, no revierte ni cancela el save admitido.

**Corrección mínima:** mantener una identidad/contador de edición monotónico por draft incluso al descartar o reemplazar baseline, o impedir un reset que pueda colisionar con operaciones vivas. Conservar requestId/payload exactos del retry.

**Prueba de cierre:** transporte desconocido→edit→discard→edit→retry diferido; igualdad completa de payload entre envíos; valor final nuevo permanece dirty, revisión/baseline confirmados actualizados.

### S5 — Una respuesta inexistente se presenta como Guardado

**Archivo:** `src/features/workflow/useWorkflow.ts:78`; presentación en `src/features/workflow/PhaseAnswerForm.tsx:196`.

**Escenario:** abrir Procesamiento de un Paper recién importado con `getPhaseAnswers=[]`. Cada salida ausente pasa por `fromAnswer(undefined)`: baseline=null/revision=0/PENDING, pero status=saved. El formulario anuncia Guardado, aunque no hay respuesta persistida ni save exitoso. También ocurre al descartar una edición sobre una salida originalmente ausente.

**Regla:** brief: una fila ausente es draft PENDING con expectedRevision=0; no fabricar fila persistida. QUALITY y entrega requieren Guardado sólo para un cambio confirmado.

**Corrección mínima:** representar ausencia/estado inicial sin llamarlo guardado, distinguiéndolo de edición pendiente para que por sí solo no bloquee la evaluación. Una respuesta PENDING realmente persistida puede anunciarse guardada sin presentar gate completo.

**Prueba de cierre:** fila ausente, PENDING persistido y discard de draft sin baseline; comprobar estado anunciado y cero saves al cargar. El fixture actual introduce filas ausentes pero no afirma la ausencia del anuncio Guardado.

## Hallazgos MEDIUM

### S6 — La consulta P2 oculta las respuestas que acaba de cargar

**Archivo:** `src/features/workflow/PhaseWorkspace.tsx:104`.

**Problema:** P2 carga definición/respuestas en el hook, pero la rama sólo renderiza un aviso de capacidad pendiente; excluye todas las salidas y sus datos. Un `getPhaseAnswers(P2)` exitoso con datos existentes no permite leerlos. No se pide implementar formularios editables, artefactos ni candidatos T06.

**Regla:** brief T04c y TASK04_PORTS: P2 muestra definición/datos en consulta y conserva la prohibición de mutaciones/evaluate/advance/candidatos.

**Corrección mínima:** mostrar salidas/datos existentes con una proyección sólo lectura y estado vacío honesto, sin controles de escritura. La prueba actual de consulta P2 comprueba el aviso, no la visibilidad de una respuesta.

### S7 — Revisar el conflicto no presenta la versión observada ni refresca el contexto

**Archivo:** `src/features/workflow/useWorkflow.ts:424` y `src/features/workflow/PhaseAnswerForm.tsx:188`.

**Problema:** se lee sólo `getPhaseAnswers`, sin Paper/fase/contexto. La versión observada se conserva en `draft.observed`, pero el formulario continúa mostrando sólo los valores del draft local y anuncia únicamente el número de revisión externa. El usuario elige usar la versión guardada —descartando el draft— sin ver su texto, estructura, resolución o explicación. Además no se rehidratan metadato/contexto/clocks potencialmente cambiados que causaron o acompañaron el conflicto.

**Regla:** brief T04c: Revisar versión guardada relee baseline/Paper/contexto sin pisar draft y ofrece elección explícita contra esa revisión observada. La revisión actual conserva correctamente el draft y no adopta currentRevision de error automáticamente; falta la revisión visible del baseline y contexto.

**Corrección mínima:** presentar la respuesta observada como texto separado junto al draft y releer el contexto mediante los puertos existentes, con protección de generación. Conservar/reaplicar sigue siendo una operación nueva tras elección humana.

**Prueba de cierre:** valores locales y externos diferentes en los cuatro campos; ambas versiones visibles antes de elegir; Paper/fase activos refrescados y draft intacto; identidad nueva sólo al guardar la reaplicación.

### S8 — Fallo de una reevaluación convierte un avance rechazado en resultado desconocido

**Archivo:** `src/features/workflow/useWorkflow.ts:364` (catch 385–387).

**Escenario:** advancePhase responde `ok=false/GateBlocked`, por tanto el rechazo de la transición es confirmado. El handler llama evaluateGate automáticamente; esa lectura rechaza transporte. El catch compartido almacena `pendingAction=advance` y anuncia que no se confirmó la transición. Se bloquean nuevas mutaciones y se ofrece replay de una transición que ya había sido rechazada, para resolver una incertidumbre que sólo pertenece a la lectura secundaria.

**Regla:** brief: GateBlocked ofrece reevaluación explícita y conserva estado; errores distinguen resultado durable confirmado de lectura/refresh fallidos. Un fallo secundario no debe modificar la clasificación de la mutación principal.

**Corrección mínima:** reevaluación explícita o manejo separado de su error; conservar el rechazo confirmado sin crear pendingAction por una lectura. Prueba de rechazo GateBlocked seguido de evaluate rechazado, sin transición incierta/replay ni cambios de fase.

## Aspectos conformes observados

- Entrada por paperId sin Reader/touch/restore; invalidación openIntent antes de callback; App compone Workflow sin opened y mantiene Reader condicionado al Paper actual.
- Pin de definición desde PhaseDto; guards de generación en hidratación; lectura de fase separada de activación explícita y tokens de fase activa para contexto.
- PRE/P1 usan salidas/labels de definición; explicación separada; confirmación PRE explícita; P1 review_type textual y decisión por estructura completa, sin interpretar answerText. Save no invoca archivePaper.
- Payload/requestId de retry se retiene; serialización inmediata por Paper impide doble submit; éxito save conserva una edición posterior si no colisionó su contador. Resultado de A no modifica la instancia B en composición key=paperId.
- ARCHIVED/COMPLETED reservados se presentan readonly; P2 no expone saves/evaluate/advance/candidatos; P3/P4 deshabilitados. Writability/capability se consumen en App. PDF no se consulta antes de entrar/guardar.
- React presenta texto sin HTML ejecutable; campos tienen labels/fieldset/legend, estados y errores accesibles y CSS de foco visible para textarea. La navegación por fase se deshabilita mientras busy: no se atribuye una carrera de save→consulta directa inexistente.
- Tests de promesas diferidas comprueban efectos/payload y resultados de UI; no hay timers arbitrarios añadidos. Los casos de cierre enumerados no están cubiertos por el rango actual.

## Comandos y limitaciones

Comandos propios de lectura, todos sin efectos de producto:

```powershell
git diff --staged
git diff
git diff --stat d4a0c77b691d58493f03f622340005de0e6193ed 198f27b281d97810d16fb2955b620af4f99b1490
git diff --name-only d4a0c77b691d58493f03f622340005de0e6193ed 198f27b281d97810d16fb2955b620af4f99b1490
git diff d4a0c77b691d58493f03f622340005de0e6193ed 198f27b281d97810d16fb2955b620af4f99b1490 -- src/app/App.tsx src/app/ShellView.ts src/features/library/LibraryPage.tsx src/app/tokens.css
git diff d4a0c77b691d58493f03f622340005de0e6193ed 198f27b281d97810d16fb2955b620af4f99b1490 -- src/app/App.test.tsx src/features/library/library.test.tsx
git log --oneline d4a0c77b691d58493f03f622340005de0e6193ed..198f27b281d97810d16fb2955b620af4f99b1490
git status --porcelain
```

Los comandos Git anteriores devolvieron exit0; staged/unstaged y status del worktree vacíos. También se usaron `git show SHA:ruta`, `Get-Content`, `rg` y `Select-Object` para leer archivos y localizar líneas. Un `rg` inicial sobre Rust desde el directorio por defecto no encontró las rutas; se repitió con workdir explícito y se leyó el backend congelado con git show. No se interpreta ese fallo como aprobación. Algunas salidas agrupadas se truncaron; se releyeron los segmentos relevantes por separado.

Este informe es la única escritura del revisor, en la ruta central autorizada. No tiene commits propios. Falta evidencia nativa de los recorridos PRE/P1, reapertura persistente, ramas, PDF indisponible, teclado/DPI y de dos procesos; su ausencia no es un defecto descubierto por lectura estática ni queda suplida por mocks. Gate/build y QA posteriores corresponden al autor/orquestador; no se atribuye su resultado aquí.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH | 5 | warn |
| MEDIUM | 3 | info |
| LOW | 0 | pass |

Verdict: **WARNING — 5 HIGH deben resolverse antes de integrar conforme AGENTS. Estado operativo BLOCKED para el corte198f27b.** Sin hallazgos críticos de seguridad demostrados. El cierre requiere correcciones por el autor, revisión del nuevo SHA y evidencia de verificación coordinada; este informe no aprueba merge ni instalación.
