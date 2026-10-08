# T04b — precheck preliminar de persistencia SQLite

Estado: DONE_WITH_CONCERNS. Revisión estática de un corte parcial; **no constituye aprobación final ni autoriza integración**.

BASE: `ce15c95dc6404aae1cb158ca6c4a1fe576b70505`. Corte exclusivo: `18fd936a8372f05f4b1a710496b2ea62d73f89a9`. Producto leído únicamente mediante `git show`/`git diff` de esos objetos. No se leyó producto WIP, no se ejecutó SQL, tests, builds o app, ni se modificó producto, commits o ramas. Normativas: INTENT, STATUS, DATA, DOMAIN, WORKFLOW_GATES, TASK04_PORTS y brief central T04b.

## Hallazgos reales del corte

1. **Importante — clock omitido al modificar inputs directos de fase iniciada.** `src-tauri/src/application/workflow.rs:46–54`: `affected` sólo incluye dependientes o inputs directos COMPLETED. Un cambio efectivo de título/reviewType con PRE IN_PROGRESS deja su revisión intacta; lo mismo ocurre para un input directo NEEDS_REVIEW. WORKFLOW_GATES:71/73 exige valor fresco al cambiar inputs reales, aunque el estado de una input directa IN_PROGRESS se conserve. Ejemplo: PRE revision1, P1/P2 NOT_STARTED; metadata cambia y PRE sigue revision1, permitiendo que un expectedPhaseRevision anterior pase CAS. Separar selección de fases cuyo clock cambia de selección del nuevo estado: toda input directa iniciada recibe clock; COMPLETED cambia a NEEDS_REVIEW, IN_PROGRESS conserva estado si no es dependiente; posteriores iniciadas reciben NEEDS_REVIEW. Coordinar con el llamador para un único clock final por UoW. Verificar PRE directo IN_PROGRESS, input directo NEEDS_REVIEW, P1 directo con P2 iniciada y unión `[PRE,P1]`, rollback/no-op/overflow.

2. **Importante — no-op de inicialización restringido al estado inicial exacto.** `src-tauri/src/application/workflow.rs:13–20` exige active PRE, PRE IN_PROGRESS/revision1 y P1/P2 NOT_STARTED/revision0. Un Paper coherente que ya guardó, avanzó o navegó devuelve IntegrityFailure al volver a llamar initialize_processing. TASK04_PORTS exige no-op para filas/pins coherentes, sin reset; coherencia no implica permanecer recién inicializado. Validar conjunto completo, pins existentes y contexto/estados consistentes preservando progreso. Añadir fixture ya avanzada/con respuestas y fixture con una versión posterior disponible que no altera pins. Además, la comprobación actual admite la configuración inicial con completedAt/snapshots artificiales: decidir explícitamente las invariantes de coherencia y comprobarlas, sin reparar silenciosamente.

## Lectura favorable y límites

- 0001 permanece intacta; 0002 se registra con SCHEMA_VERSION2. El runner existente ejecuta DDL, seed, backfill, ledger y user_version bajo la misma TX IMMEDIATE y verifica integridad antes del commit. Backup se crea antes para upgrade; errores devuelven MigrationFailed con rollback por propiedad de Transaction. Esto es inspección de código, sin evidencia nueva de ejecución.
- Seed SQL contiene los tres payloads/versiones/hashes. La prueba de biblioteca fresca compara JSON completo con las definiciones embebidas; `definitions::get` recalcula el hash sin definitionHash. No existe seed tardío ni INSERT OR REPLACE.
- Backfill modifica únicamente processing_initialized/current_phase; conserva lifecycle/archivedFrom/UUID/documento/revision/updatedAt. PRE inicia1 y P1/P2 quedan0, sin respuestas/completions. La asignación sentinela2 viola deliberadamente el CHECK antiguo ante contexto inesperado y aborta, en vez de ignorar incoherencia. Es válida como mecanismo SQLite, aunque poco autoexplicativa.
- FKs compuestas preservan el pin code/version y pertenencia answer→phase. PK paper_phases comienza paper_id; PK phase_answers comienza paper_id,phase_code, de modo que el índice phase_answers_paper_phase es redundante para ese acceso. Observación menor, sin necesidad de ampliar diseño.
- Import inicializa en confirm_in_tx antes de construir resultado durable; mismas TX/llamadas para recovery. Metadata compara título/reviewType normalizados y llama invalidación dentro de su TX. D1 marca correctamente posteriores IN_PROGRESS NEEDS_REVIEW y preserva completedAt/snapshot; sólo falla la renovación del clock directo descrita arriba.
- Constraints comprueban enums, revisiones, JSON y pareja snapshot/hash. No prueban por sí solos hash real, keys de definición, resoluciones permitidas ni contexto consistente; esas reglas corresponden al adaptador/aplicación final, aún parcial.

## Cobertura pendiente del corte parcial, no defectos SQL demostrados

workflow_migration sólo inspecciona strings SQL, biblioteca fresca, seed/paridad y una inicialización idempotente recién creada seguida de invalidación posterior. No demuestra upgrade0001→0002 poblado con NEW/ARCHIVED/COMPLETED, preservación integral, reopen, fallo al final de0002 que revierta tablas/seed/backfill/ledger/user_version y retenga backup, import/backfill equivalentes ni fallo de inicialización que revierta confirmación. desktop_bootstrap adapta la versión futura pero su SQL artificial no verifica rollback del backfill real0002. library_integration cambia firmas sin añadir aserciones Workflow en este commit. Deben añadirse las pruebas del brief antes de revisión final; su ausencia no se atribuye a una implementación final inexistente.

No se evalúan servicio/comandos/proof/replay final o integración WIP: están fuera del corte revisado. El precheck tampoco verifica SQL por ejecución ni instalación nativa. Será necesaria revisión completa BASE→HEAD final y gates pertinentes después de corregir y completar.

## Registro y autorrevisión

Comandos de inspección: `git diff --stat BASE HEAD`, `git diff --name-only BASE HEAD`, `git diff BASE HEAD -- <rutas indicadas>`, `git show HEAD:<ruta>`; lecturas normativas con Get-Content/rg. Todos concluyeron exit0. Una salida inicial truncada se completó con lecturas específicas de migration/helper/tests/runner y normativa. Sin afirmación de PASS de tests.

Se comunicaron ambos hallazgos importantes al orquestador durante la revisión. Informe único escrito en repo central; cambios ajenos preservados. Consulta auxiliar de memoria sólo identificó la pauta histórica de leer fuentes locales; no se utilizó evidencia histórica para evaluar producto. La skill using-superpowers contiene SUBAGENT-STOP y no aplica su workflow al revisor delegado.
