# T04c — revisión independiente TypeScript

Fecha: 2026-10-03. Revisor `/root/task04c_typescript_review`. **BLOCKED para integración**: 6 HIGH y 4 MEDIUM, sin CRITICAL demostrados. Informe único central; no cambios de producto, contratos, configuración global, commits ni merges. No subagentes.

## Corte y método

- BASE `d4a0c77b691d58493f03f622340005de0e6193ed`.
- Producto congelado `198f27b281d97810d16fb2955b620af4f99b1490`.
- HEAD observado `c14c97dd8cfa2209e5596ef5ef8c39540973ad99`; el commit posterior sólo añade `docs/reports/task-04c-report.md`. Las líneas de producto siguientes corresponden a ambos SHA.
- Worktree `C:/Users/david/Projects/Research-Workbench/.worktrees/task-04c-workflow-ui`.

Se estableció scope con staged/unstaged vacíos y diff completo BASE..HEAD. Se leyeron AGENTS/INTENT/STATUS, briefs centrales, contratos Workflow/ADR-020 y puertos TASK04; skills coding-standards y frontend-patterns. Después de la primera inspección se cruzó el informe SPEC. TS1–TS8 corroboran S1–S8 y deben consolidarse una sola vez; TS9–TS10 son adicionales. No hay PR asociado a este despacho ni metadata remota de merge readiness/CI; la revisión es local.

Se inspeccionaron todos los archivos modificados: App/ShellView/tests, Biblioteca/tests, CSS, cuatro archivos Workflow y workflow.test.tsx. No cambios Rust, DTOs, SQL, wire, manifests, dependencias ni Reader en el rango. No se ejecutó Cargo, check.ps1, build, GUI o instalador, conforme coordinación del target compartido. Lectura acotada de Rust se usó únicamente para verificar la semántica del replay archivado.

## HIGH

### TS1 — NEEDS_REVIEW no puede reconfirmarse [duplica S1]

`src/features/workflow/PhaseWorkspace.tsx:31` exige state IN_PROGRESS para canAdvance. PRE/P1 activa NEEDS_REVIEW con evaluación completa, revisión actual y sin drafts pendientes mantiene deshabilitado el botón de completar.

Impacto: el recorrido de reconfirmación después de invalidación queda cerrado. Regla: brief T04c y contrato Workflow requieren aceptación explícita mediante advancePhase para volver a completar. Corrección mínima: admitir NEEDS_REVIEW, conservando fase activa, token, gate y todas las otras guardas; nunca completar localmente.

Regresión: PRE y P1 NEEDS_REVIEW→evaluate completo→avance explícito con expectedPhaseRevision observado; confirmar estado desde backend. `workflow.test.tsx:562` sólo verifica consulta/conservación de datos.

### TS2 — Remount del Paper durante una operación queda sin notificación/reconciliación [duplica S2]

`src/features/workflow/useWorkflow.ts:42`, `:112`, `:274`, `:327`; `src/app/App.tsx:201` monta por key paperId. PaperSession es compartida, pero bump/setState/loadPaper pertenecen al hook original y no notifican al nuevo consumidor.

Secuencia: guardar A con promesa diferida→salir→volver a A→terminar su nueva hidratación→resolver save antiguo. La sesión queda busy=false/status=saved, pero el workspace nuevo sigue Guardando/Evaluar deshabilitado. Un render ulterior puede usar tokens/lifecycle anteriores al resultado. También aplica a advance/touch/goBack: el resultado y su refresh se ejecutan sobre la instancia desmontada.

Corrección mínima: notificar cambios de la operación a consumidores actuales del Paper y reconciliar las lecturas de su instancia actual después del resultado confirmado. Mantener sólo drafts/operaciones transitorios; no añadir canonstore frontend. Regresión: remount durante save y acciones, éxito/fallo desconocido/retry y archive; comprobar DOM, busy, tokens/contexto/lifecycle sin otra interacción.

**Demostración propia:** probe_same_paper_remount_has_no_notification_after_old_save PASS como reproducción del defecto. Afirma sesión busy=false/draft saved simultáneos a DOM Guardando y Evaluar disabled. No acredita SQLite ni ventana nativa.

### TS3 — Gate/transiciones ignoran drafts de otras fases y errores de guardado [duplica S3]

`src/features/workflow/useWorkflow.ts:492`; llamadas `:306`, `:395`, `:403`, `:485`. hasDirtyDrafts sólo mira fase consultada y statuses dirty/saving/unknown/conflict; excluye error.

Secuencias: editar PRE y consultar P1 permite evaluar/activar allí con draft PRE pendiente; editar una respuesta válida, recibir InvalidInput/GateBlocked no retryable y evaluar permite usar baseline anterior aunque la nueva intención esté en error. En P1 podría avanzar con la decisión anterior. El botón Descartar sólo aparece en dirty (`PhaseAnswerForm.tsx:199`), por lo que un error tampoco ofrece descarte directo.

Regla: evaluar/avanzar sólo tras guardar o descartar explícitamente todos los drafts pendientes de ese Paper. Corrección mínima: condición por Paper, distinguiendo ausencia inicial de cambio pendiente e incluyendo errores con intención no confirmada; ofrecer resolución explícita y aplicar guardas en handlers/controles. Regresión: PRE dirty mientras P1 consultada, caso inverso y rechazo sobre baseline válido; cero evaluate/advance/transiciones hasta resolver.

### TS4 — Reset del contador al descartar permite que un retry pise una edición nueva [duplica S4]

`src/features/workflow/useWorkflow.ts:415`, reset en `:76`, comparación `:277`; descarte permitido con pending desconocido en `PhaseAnswerForm.tsx:199`.

Secuencia reproducida: editVersion1→save pierde transporte→editar→descartar (reinicia a0)→editar nuevo (vuelve a1)→retry original exitoso. Igualdad de contador acepta el resultado antiguo como si no hubiese edición posterior; reemplaza el valor nuevo y anuncia Guardado. El payload durable original es correcto; el defecto es pérdida de intención local.

Corrección mínima: identidad/contador monotónico por draft incluso al descartar/reemplazar baseline, conservando payload/requestId congelados. Regresión: unknown→edit→discard→edit→retry diferido; la edición nueva debe quedar dirty, baseline/revision del éxito actualizarse y ambos envíos ser idénticos.

**Demostración propia:** probe_discard_reset_aliases_original_save_edit_version PASS como reproducción: envíos idénticos, input final Payload inicial y ausencia de Borrador sin guardar, pese a nueva edición posterior. Se lee el input actual tras refresh, no una referencia DOM desmontada.

### TS5 — Respuesta inexistente anunciada como Guardado [duplica S5]

`src/features/workflow/useWorkflow.ts:68–81`, presentación `PhaseAnswerForm.tsx:196`. fromAnswer(undefined) produce baseline=null/revision0/PENDING y status=saved. Un Paper nuevo con answers=[] anuncia Guardado para todas las respuestas inexistentes; también después de descartar un draft sin baseline.

Regla: fila ausente es draft PENDING/expected0 y no prueba persistencia. Corrección mínima: representar ausencia/estado inicial sin Guardado y sin bloquear automáticamente evaluación; distinguir PENDING realmente persistido. Regresión: ausencia, PENDING persistido y discard sin baseline, con cero saves al cargar.

### TS10 — Retry pendiente visible en ARCHIVED no envía la identidad original [nuevo]

`src/features/workflow/useWorkflow.ts:243–246`; botón en `PhaseWorkspace.tsx:125–127`.

Secuencia: save A pudo confirmarse durablemente, pero se perdió transporte→volver a Biblioteca→archivar A explícitamente→entrar a Procesamiento A archivado con biblioteca writable. La sesión conserva pending y muestra Reintentar el mismo guardado habilitado. El click llama save(retry=true), pero retorna por lifecycle ARCHIVED antes de seleccionar pending; ninguna segunda llamada llega al backend y no se puede resolver ese resultado original desde esta vista.

Contrato: retry conserva requestId/payload; replay durable se resuelve antes de lifecycle/CAS. Confirmado por lectura de `src-tauri/src/adapters/sqlite/workflow_repository.rs:270` (with_receipt envuelve application) y `src-tauri/src/adapters/sqlite/receipts.rs:47` (lookup antes de ejecutar), frente a lifecycle de `src-tauri/src/application/workflow.rs:238–240`. No se propone permitir una escritura nueva en ARCHIVED.

Corrección mínima: separar resolución de la operación pendiente congelada de guardas de creación de una operación nueva; permitir solicitar esa identidad original cuando el backend está disponible, y consumir su resultado sin activar formularios ni cambiar lifecycle desde un receipt histórico. Mantener backend como autoridad. No saltarse recovery/writability: `WorkflowService::ensure_mutable` se comprueba antes del adapter y puede rechazar durante recovery; no hay evidencia de replay disponible en ese estado.

Regresión: éxito durable/respuesta perdida→archivo explícito→remount ARCHIVED writable→retry con args idénticos→baseline/revision/confirmación reconciliados y readonly/contexto archivado conservados; cero nuevas mutaciones. Incluir rechazo backend sin ocultarlo.

**Demostración propia:** probe_archived_remount_exposes_but_never_sends_pending_save_retry PASS como reproducción: botón habilitado, pending retenido y save.calls sigue1 después del click. El probe no simula un receipt SQLite real: la semántica del receipt procede de la lectura backend anterior.

## MEDIUM

### TS6 — P2 no muestra sus respuestas cargadas [duplica S6]

`src/features/workflow/PhaseWorkspace.tsx:104–109`. P2 carga definición/respuestas pero renderiza sólo aviso de capacidad; no muestra outputs/datos existentes. Regla: consulta P2 incluye definición/datos sin writes/evaluate/advance/candidatos. Corrección: proyección readonly y vacío honesto. Regresión: P2 con respuesta existente visible, cero mutaciones. La prueba actual sólo verifica aviso.

### TS7 — Conflicto no permite revisar la versión observada ni refresca Paper/contexto [duplica S7]

`src/features/workflow/useWorkflow.ts:424`, `PhaseAnswerForm.tsx:188`. Revisar versión guardada sólo relee respuestas y muestra revisión externa; texto/estructura/resolución/explicación externa no son visibles antes de elegir. Tampoco refresca Paper/fases/clocks.

Corrección: presentar ambos valores como texto y refrescar contexto conservando draft, con guards de generación; reaplicación mantiene nueva identidad sólo después de elección humana. Regresión: cuatro campos externos/locales diferentes visibles antes de elegir, contexto actualizado y draft intacto.

### TS8 — Lectura secundaria fallida convierte GateBlocked confirmado en transición incierta [duplica S8]

`src/features/workflow/useWorkflow.ts:364–387`. advance devuelve GateBlocked confirmado; evaluateGate automático rechaza transporte; catch compartido crea pendingAction de advance y ofrece retry de transición. Clasifica mal la incertidumbre y bloquea nuevas operaciones. También falta la reevaluación explícita exigida por brief.

Corrección: reevaluación explícita o catch independiente de esa lectura; conservar rechazo confirmado sin pendingAction por fallo secundario. Regresión: GateBlocked→evaluate transporte fallido, sin replay/estado desconocido de advance.

**Demostración propia:** probe_secondary_gate_read_failure_misclassifies_rejected_transition PASS como reproducción: advance.calls1 y botón Reintentar la misma transición tras rechazo confirmado más lectura fallida.

### TS9 — Reintentar carga después de un fallo getPaper no reintenta Paper [nuevo]

`src/features/workflow/PhaseWorkspace.tsx:44`; `useWorkflow.ts:113–143`, `:286–291`, `:158`, `:201`.

Secuencia reproducida: save confirmado→refresh getPaper falla→se conserva faseCode PRE y se muestra éxito confirmado con vista sin actualizar/error de Paper→pulsar Reintentar carga. Al existir phaseCode, llama sólo loadPhase. Ésta limpia error y recarga fase/respuestas, pero no vuelve a leer Paper. El aviso que implicaba fallo de Paper desaparece con Paper/metadata/lifecycle/contexto sin actualizar; no existe retry pertinente dentro de esa vista.

Corrección mínima: distinguir error de carga Paper de fase y reintentar la fuente fallida, o acción de refresh completo Paper/contexto/fase con protección de generación y drafts. Nunca reenviar save/advance para refrescar. Regresión: getPaper falla tras éxito, siguiente getPaper disponible con estado distinto; retry debe consumirlo, preservar confirmación/drafts y no repetir mutación. No se afirma que el save por sí solo cambie metadatos/lifecycle.

**Demostración propia:** probe_reloading_phase_after_failed_paper_refresh_never_reloads_paper PASS como reproducción: getPaper.calls sigue2 después del retry; error desaparece y Paper ACTIVE antiguo sigue visible aunque hay una tercera respuesta mock ARCHIVED disponible. Es inyección sintética de cambio de estado, no concurrencia nativa demostrada.

## Cobertura y aspectos conformes

Entrada por paperId/openIntent correcta por inspección; sin Reader/touch/restore al consultar. Definición fijada por PhaseDto; DTO generados; resoluciones/texto separados; decisiones P1 estructuradas sin inferencia de texto ni archive al save. Serialización inmediata por Paper para doble submit, retry idéntico y preservación de edición posterior al save simple sí están implementados. No eval/HTML dinámico, secretos, SQL ni filesystem UI añadidos. CSS se acota a Workflow, foco textarea y breakpoint local; no se atribuye QA visual/DPI a la lectura.

Gaps de evidencia: `src/app/App.test.tsx:148` resuelve Reader con `{}`, no IpcResult<OpenPaperDto> exitoso válido. Esa rama no llega a onOpenPaper aunque faltase su guard de éxito, y puede dar un falso negativo a la regresión. Root ya consolidó fixture correcta en fix1 (R1); no es hallazgo adicional de producto. La prueba llamada workflow_keyboard_labels_and_error_associations usa fireEvent.change/click, no navegación real por teclado/foco; sólo acredita atributos/error/texto. También faltan carreras de hidratación por fase/paper con respuestas diferidas y una prueba explícita de capabilities.workflow=false; son huecos de evidencia a cubrir conforme brief, no defectos nuevos demostrados.

## Comandos/resultados propios

Desde el worktree congelado:

```powershell
git status --short
git rev-parse HEAD
git diff --staged --stat
git diff --stat
git diff --stat d4a0c77b691d58493f03f622340005de0e6193ed c14c97dd8cfa2209e5596ef5ef8c39540973ad99
git diff d4a0c77b691d58493f03f622340005de0e6193ed c14c97dd8cfa2209e5596ef5ef8c39540973ad99 -- src/app/App.tsx src/app/ShellView.ts src/features/library/LibraryPage.tsx src/app/tokens.css src/app/App.test.tsx src/features/library/library.test.tsx
. .\scripts\development-env.ps1
npm.cmd run typecheck
npm.cmd test -- src/features/workflow/workflow.test.tsx src/features/library/library.test.tsx src/app/App.test.tsx
npm.cmd test -- --config work/task-04c-typescript-probes/vitest.config.ts
npm.cmd test -- --config work/task-04c-typescript-probes/vitest.config.ts -t probe_archived
```

- Git: exit0; scope establecido, árbol producto limpio. HEAD c14c97d.
- Typecheck canónico `tsc -b`: exit0; no drift ni relajación tsconfig por este corte.
- ESLint: no script/dependencia ni `node_modules/.bin/eslint.cmd`; no se instaló ni se afirmó lint PASS.
- Tres focales existentes: exit0, 3 archivos/59 tests PASS, duración3.00s. Se ejecutaron una sola vez para conocer evidencia propia del rango; no se amplió a gate/build.
- Probes: primera ejecución exit0, 4 tests/1 archivo PASS; segunda ejecución focal exit0, 1 test PASS y4 skipped. PASS significa **reproducción del comportamiento defectuoso**, no aprobación del producto. Los asserts verifican el estado incorrecto observado; el autor debe invertirlos como regresiones del comportamiento requerido.

Probes temporales conservados fuera de producto y Git, en `C:/Users/david/Projects/Research-Workbench/.worktrees/task-04c-workflow-ui/work/task-04c-typescript-probes/{review.test.tsx,vitest.config.ts}`. Se reutilizaron las fixtures/helpers de las primeras175 líneas del test actual ajustando únicamente imports a src; luego se añadieron los cinco escenarios anteriores. Config específica jsdom/include work, sin nuevas dependencias ni cambios al harness canónico. No están incluidos en typecheck de producto, ni son pruebas de SQLite/instalación/desktop. No contienen datos personales.

Algunas lecturas agrupadas se truncaron y se releyeron los segmentos necesarios. Un intento de lectura de archivos UI desde checkout central y una búsqueda con glob literal PowerShell fallaron por ruta; se corrigió workdir/ruta y no se interpretó ese exit1 como resultado de checks de producto.

## Cierre

Revisión terminada, **integración BLOCKED** hasta resolver HIGH y revalidar corte nuevo. TS1–TS8 se consolidan con SPEC sin duplicar tareas; TS9–TS10 vuelven al mismo autor con regresiones. La evidencia verde de tests/typecheck no cubre los defectos reproducidos. No se confirma persistencia entre procesos ni QA nativa, teclado/DPI, instalación limpia o bundle; permanecen gates del orquestador. Esta revisión no autoriza merge.
