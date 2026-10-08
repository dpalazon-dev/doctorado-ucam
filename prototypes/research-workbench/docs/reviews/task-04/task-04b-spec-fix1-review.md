# T04b corrección1 — revisión independiente SPEC y calidad

Fecha: 2026-10-03. Revisor: `/root/task04b_spec_final_review`. **Resultado: PASS / DONE_WITH_CONCERNS**, sin hallazgos críticos/importantes abiertos de SPEC en el producto revisado. La preocupación restante es un límite menor de evidencia de reapertura, descrito abajo; no es un defecto funcional demostrado. Este dictamen no sustituye la revisión Rust, seguridad, TypeScript ni el gate del merge preparado.

## Corte y método

- BASE original: `ce15c95dc6404aae1cb158ca6c4a1fe576b70505`.
- Revisión histórica S1–S9: `5c35067f893aa52ec601cc7a02c18bb95b156145`.
- Producto final: `5fb4e084ed943831e2d8aa2bc3584ae076590836`.
- HEAD documental: `e9a568073c76b8661b8ffd94732e40d118841f6f`; el delta desde producto sólo modifica los dos informes del autor.
- Worktree: `C:/Users/david/Projects/Research-Workbench/.worktrees/task-04b-workflow-backend`; `git diff --staged`, `git diff` y `git status --short` vacíos, log final comprobado.

Leídos AGENTS, INTENT, STATUS, WORKFLOW, briefs original/corrección1, revisión SPEC original, informe del autor y precheck del orquestador. Normas actuales del repo principal: TASK04_PORTS, TASK04_APPLICATION_CASES, WORKFLOW_GATES, SPEC-003, DATA, QUALITY y ADR-020/021. Se aplicaron Karpathy y verification-before-completion. Se inspeccionaron el diff completo de alcance BASE..producto, los casos application, dominio, puertos, persistencia, servicio/composición/dispatcher, integración Library/receipts, y las pruebas pertinentes. La decisión normativa del orquestador sobre el alcance preciso de la comparación tras reopen se recibió durante esta revisión y se registra explícitamente abajo.

**No ejecuté tests, build, Cargo ni modificadores de producto/configuración.** El target compartido quedó reservado al revisor Rust. Los comandos propios fueron de lectura: `git diff`, `git log`, `git status`, `git rev-parse`, `rg` y `Get-Content`; la única escritura es este informe central nuevo. Los resultados de ejecución indicados abajo proceden de logs finales existentes, no de una nueva ejecución mía.

## Cierre de S1–S9

| Hallazgo | Resultado | Evidencia del producto y aceptación |
|---|---|---|
| S1 — PRE no activa P1 | CERRADO | `application/workflow.rs:625-680` obtiene plan de destino, acepta PRE y asigna clock/estado/contexto P1 en la misma TX. `workflow_ipc.rs:300-361` consulta P1 inmediatamente tras advance y comprueba IN_PROGRESS/revision3/contexto P1 antes del touch no-op. Reconfirmación `:1265-1400` preserva P1 NEEDS_REVIEW, snapshot/hash y datos anteriores. |
| S2 — UNKNOWN obsoleto | CERRADO | `domain/workflow.rs:55-78` exige concordancia de reviewType actual también en UNKNOWN. `workflow_ipc.rs:417-478` cubre unknown→tipo conocido obsoleto; journey `workflow_acceptance.rs:285-331` cubre metadato conocido→unknown y confirmación UNKNOWN explicada mediante save real. La respuesta previa se conserva hasta que se cambie explícitamente. |
| S3 — pin persistido | CERRADO | Helper `application/workflow.rs:49-62` selecciona versión exacta y traduce pin no soportado a UnsupportedCapability; evaluate/save/touch usan las filas fijadas, P1 resuelve además el pin PRE. `:625-642` valida destino P1/P2 antes de escribir/aceptar origen. Pruebas `workflow_ipc.rs:480-557` verifican otra definición instalada sin repinar y pin P1 no soportado; `:1462-1612` rechazan PREv1→P1v2 y P1v1 continue→P2v2 con snapshot durable idéntico y cero receipt. No se evalúa gate P2 para iniciarla. |
| S4 — receipt/capability | CERRADO | Save entra en with_receipt antes de ejecutar el caso; prepare_advance consulta receipt antes de capability y referencia. Lookup y wrapper comparten hash/formato y repiten lookup en la TX final. `workflow_ipc.rs:561-680` prueba colisión→Conflict, nuevas operaciones P2→UnsupportedCapability sin receipt. El recorrido de replay existente tras aceptación, pérdida del archivo y cambio posterior de lifecycle conserva resultado original. |
| S5 — historial específico | CERRADO | Application emite answer_changed, context_changed y phase_accepted con entidad Paper y cambios concretos. `application/workflow.rs:696-737` deriva destinationBefore/After de filas iniciales/finales mediante transpose; ausencia no se silencia. Primera aceptación y reconfirmación verifican los destinos en `workflow_ipc.rs:331-356` y `:1354-1387`. No-op/replay no emiten evento efectivo; wrapper conserva auditoría genérica. Triggers de fallo de answer audit, acceptance audit, receipt y paper.archived verifican rollback en `:1035-1230`. |
| S6 — escenarios de aceptación | CERRADO | `workflow_acceptance.rs` compone actor/store/registry/maintenance y Library/Reader/Workflow reales; importa un PDF sintético y llena PRE/P1 mediante save. Aísla gate suficiente sin aceptación PRE→P1 (`:237-364`), P1 IN_PROGRESS→P2 (`:497-533`) y P1 NEEDS_REVIEW suficiente→P2 (`:751-790`) con documento disponible/CAS vigente y sin receipt. Incluye metadata/preview/CAS, authors-only, no-op de JSON, navegación, disponibilidad, continue→P2 fixture→volver/editar→light_read/archive y restore. P2/candidatos indisponibles, COMPLETED reservado, overflow y rollback tienen pruebas explícitas en workflow_ipc. |
| S7 — upgrade/backup/reopen | CERRADO con límite menor | `workflow_migration.rs:253-300` captura columnas exactas de metadatos, autores/venue/enlaces, documento/hash/ruta, lectura/app_session, receipts y audit; matriz NEW/ACTIVE/ARCHIVED/COMPLETED/ARCHIVED-from-COMPLETED. `:395-529` compara antes/después/backup/reopen con conexión original cerrada; `:532-608` compara también rollback/backup/reopen tras fallo. Journey real cierra el actor y reabre, lee payload completo/hash PRE/P1, comprueba hash canónico, estado/pin/decisión esperados y contexto (`workflow_acceptance.rs:1023-1077`). Véase límite abajo. |
| S8 — revisión JSON segura | CERRADO | goBack/touch aplican rango `0..=9007199254740991` antes de CAS de petición nueva (`application/workflow.rs:354-358`, `:489-493`). Tests `workflow_ipc.rs:684-737` cubren negativo/MAX+1→InvalidInput y MAX válido llegando a CAS, sin receipt ni alteración del contexto/fase. |
| S9 — casos application | CERRADO | Las cinco funciones TX y cuatro primitivas adicionales tienen las firmas aceptadas. Application decide CAS/pin/gate/plan/clocks/contexto/lifecycle/audit usando repositorios; el adapter conserva SQL/actor/TX/receipt/revalidación documental/proof/admission. Plan puro en `domain/workflow.rs:194`, probado sin SQLite por `workflow_gates.rs:29-108`. No callbacks genéricos, repositorios paralelos ni cambio LibraryRepository. |

## Escenarios residuales revisados expresamente

**Documento retirado después de PRE aceptada y con P1 completa.** El journey llena todas las respuestas P1 antes del retiro y exige gate completo con PDF disponible (`workflow_acceptance.rs:390-508`). Luego elimina el PDF, exige issue documental, hash distinto y advance GateBlocked; restaura los mismos bytes y exige gate completo/hash original. Compara snapshot aceptado PRE y clock/estado P1, con cero receipt fallido (`:534-635`). El bloqueo ya no puede quedar enmascarado por respuestas P1 incompletas. No atribuyo esto a una aceptación P1 previamente completada ni a prueba ACL nativa.

**Archive guardado y restore Library.** Después del recorrido continue/P2 previo/light_read, el journey guarda readingDecision=archive, acepta P1 y obtiene ARCHIVED, restaura mediante Library y obtiene ACTIVE. El recuento de phase_accepted no aumenta por restore; consulta posterior sigue ACTIVE/P1, conserva la decisión archive y la respuesta P2 fixture con NEEDS_REVIEW (`workflow_acceptance.rs:872-1022`). La fixture P2 es explícitamente un estado futuro inyectado: no demuestra save/evaluate/advance P2 habilitado.

**Clock e inicialización, restos SQL del precheck.** `SqliteWorkflowRepository::set_phase_clock` es sólo UPDATE de valores ya decididos, y max_phase_revision es lectura. Application asigna max+1; invalidación deduplica inputs y aplica D1 en orden PRE/P1/P2, con overflow dentro de la UoW. El INSERT de inicialización fija PRE1/P1-P2 0 únicamente tras comprobar estado virgen. Si initialized=true, `initialize_processing` verifica coherencia y devuelve sin escritura; no reinicializa contexto ni snapshots avanzados (`application/workflow.rs:748-809`). Las pruebas direct-input, inputs solapados e initialized avanzado acreditan las ramas correspondientes (`workflow_migration.rs:118-210`). No queda otro algoritmo de clock o caso de uso de inicialización en SQL además del backfill histórico autorizado.

**Propiedad documental y replay.** Available/Unavailable se revalidan con referencia completa bajo la TX, y los proof/permisos se poseen alrededor de toda la llamada with_receipt, incluido commit/rollback. Evaluate usa DEFERRED sin receipt/escrituras. El delta de concurrency desde el corte original sólo simplifica una condición del helper de señal; permanecen escenarios de cancelación, error tipado, referencia cambiada y Drop tras commit/rollback. No se añade read_all ni hash de PDF bajo actor.

## Alcance, evidencia y límite menor

El delta de producto mantiene ocho comandos, wire/DTO/permisos cerrados, un actor/store y las dependencias existentes. No cambia UI, Reader/T04a, migrations0003/0004, candidatos/resolutores P2, release o harness. El único TypeScript cambiado es el test del cliente; fixture advance ahora complete=true. Settings/manifest anuncian backend Workflow conectado, no formularios T04c ni workflow v0.1 completo.

Verifiqué por `git rev-parse` los blobs finales: 0001 `57e2822dc1e5b2f1acb57930ffbbfd2019d4889d`; 0002 `0394c1db6056b2ebf2697aa3da9c12ba8442a5f0`; runner `68047c068baa1fcdd8567a64bf904c916c9640fa`. Coinciden con el PASS SQL previo registrado por el orquestador. No reabro ese dictamen ni atribuyo a identidad de blobs la corrección de lógica application.

Leídos sidecars finales `work/task-04b-fix1/{check-final,debug-final}.exit`: ambos 0. Los resúmenes de logs confirman 69 pruebas frontend y 181 entradas Rust incluidos helpers, Clippy/contratos y build debug sin bundle. Root comunicó además lectura completa de ambos logs. Esta revisión estática **no ejecutó** esos comandos, no demuestra GUI/instalación/máquina limpia y no autoriza merge por sí sola.

**Límite menor de evidencia, sin hallazgo funcional:** el journey de reopen verifica snapshots completos/hash/estado/pin/contexto, pero no vuelve a consultar por separado phase clocks/completedAt y las filas individuales de respuestas después de reabrir. Sol precisó en esta revisión que la igualdad exacta requerida aplica a filas v1 en upgrade/backup/reopen; para aceptación real se exige payload completo/hash conservados con valores esperados, satisfecho por las consultas finales. No se exige ampliar este corte con comparación byte a byte adicional. QA T04c deberá comprobar persistencia visible entre procesos; cerrar/reabrir DbActor no sustituye esa prueba desktop. Esta observación no reabre S7 ni constituye bug.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH | 0 | pass |
| MEDIUM | 0 | pass |
| LOW | 0 | pass |

Verdict: **APPROVE — PASS de SPEC/calidad para 5fb4e08; S1–S9 cerrados.** Entrega: **DONE_WITH_CONCERNS** por el límite menor de evidencia explicitado. Rust/seguridad/TypeScript e integración permanecen bajo sus revisiones y gate propios; no se declara producto integrado ni instalado.
