# T04c — revisión del orquestador, checkpoint

Fecha: 2026-10-03. BASE d4a0c77b691d58493f03f622340005de0e6193ed; producto 198f27b281d97810d16fb2955b620af4f99b1490; entrega c14c97dd8cfa2209e5596ef5ef8c39540973ad99. Estado: NO APTO PARA INTEGRAR.

Sol leyó el diff completo de producto/tests/estilos desde BASE, incluidos useWorkflow, PhaseWorkspace, PhaseAnswerForm, GateIssues y composición App/Library. El commit posterior sólo añade el informe de checkpoint; worktree e integración limpios al consultar git status. Main conserva AGENTS.md externo sin incorporarlo.

## Hallazgos y decisión

Se aceptan S1–S8 del informe independiente task-04c-spec-review.md: reconfirmación NEEDS_REVIEW bloqueada; ausencia de notificación/reconciliación a la instancia remontada; guardas que omiten drafts de otras fases y errores no guardados; contador local reiniciado con colisión durante replay; Guardado sin fila durable; datos P2 ocultos; conflicto sin contexto actualizado ni comparación visible; rechazo confirmado convertido en desconocido por fallo de lectura secundaria. Todos deben corregirse antes del merge. La revisión TypeScript independiente añadirá o precisará escenarios antes del despacho único fix1.

Ruling: la sesión puede notificar a los consumidores actuales y señalar que necesitan releer datos confirmados mediante los puertos existentes. Debe seguir conteniendo sólo drafts y operaciones transitorias, no una segunda biblioteca. No se necesita framework, dependencia, wire ni persistencia frontend. Coste: controlar explícitamente suscripción/limpieza y generaciones; las regresiones de remount deben probarlas.

Ruling: el descarte de edición local no cancela una mutación admitida ni elimina su payload de replay. El contador/identidad de edición debe evitar colisiones al sustituir baseline, descartar o resolver conflicto. La ausencia inicial de una fila no es un cambio pendiente ni un guardado confirmado. Estas reglas se aplican juntas para evitar arreglar una guarda a costa de bloquear formularios nuevos.

Observación adicional de prueba: el caso late_reader_open_cannot_replace_workflow resuelve la promesa con un objeto vacío, no con IpcResult<OpenPaperDto> exitoso. Debe usar un envelope válido para que la prueba pueda detectar una regresión de openIntent. Los tests de labels no acreditan por sí solos teclado nativo ni DPI; la suite que menciona UNKNOWN/NA debe probar ambas resoluciones si se atribuye ambas en el informe.

No se exige refactor por tamaño ni se abren cambios cosméticos generales. El resultado de A no contamina B en la composición con key=paperId observada; el defecto probado por lectura es regresar a A antes de resolver y no notificar al consumidor nuevo. No se atribuye pérdida durable SQLite al contador local.

## Evidencia disponible y límites

Autor informa focales 59/59 y typecheck exit0. Root leyó informe completo, sidecars y colas de logs reales: check.exit.txt=0 y tauri-debug-no-bundle.exit.txt=0 en work/task-04c. La cola del gate muestra 102 pruebas frontend en 11 archivos y comprobaciones Rust/contratos satisfactorias. No se presenta como lectura íntegra de cada línea ni ejecución propia del gate. El build deja advertencia Vite de tamaño de chunk; no es fallo.

Las pruebas verdes no cubren los escenarios anteriores. No se ha ejecutado GUI T04c, instalador, ciclo de cierre/reapertura ni validación en equipo limpio. Tras fix1: focales/regresiones, gate sobre SHA congelado, revisiones SPEC/TypeScript del delta, QA nativa planificada y merge preparado validado. El runner histórico y datos personales permanecen intactos.
