# T03 fix1 — revisión de conformidad

Estado: DONE_WITH_CONCERNS. Revisor: /root/task03_spec_review. Fecha: 2026-10-02.

BASE: `870b3321c4e3d46597848ae5d3faf40395229621`.
HEAD limpio verificado: `b36ae5ae0529ba429edf7a360ade7319e353e8fe`.
Worktree: `.worktrees/task-03-reader`.

Leídos brief/report fix1, revisión SPEC anterior y delta pertinente con consecuencias sobre S1–S7. Solo se crea este informe central; sin edición de producto, tests o configuración, subagentes, merges, GUI ni biblioteca personal. El orquestador ya revisó el diff completo y verificó gate/build; no se repitió la suite completa.

## Resultado S1–S7

| Hallazgo original | Resultado de fix1 |
|---|---|
| S1, borrador A aplicado a B | Cerrado para el defecto identificado: PaperDetails tiene key por paper.id; selectedId filtra la respuesta tardía y hay regresiones de cambio A→B y guardado tardío. |
| S2, recuperación de duplicados/requestId | Cerrados los defectos identificados: openFailed tiene prioridad visual; error común accesible; cancelIntent vinculado al token, reseteado al seleccionar; cancelar sin abrir cierra el componente. Véase cobertura adicional abajo. |
| S3, salidas locales del diálogo | Corregidas ×/Escape/exterior mediante closeThroughTransition, bloqueo pending y cancelación confirmada. El abandono causado por una apertura previa ajena al diálogo sigue cubierto por TS-03 residual ya registrado; no se declara cierre global de ese riesgo ni se cuenta de nuevo. |
| S4, paginación | Cerrado: nextCursor/history, navegación y reinicio de filtros implementados; descarte por vida del efecto; reload conserva el cursor. La prueba persistente ejercita cursor y filtro; no demuestra un corpus real de 101 papers solo porque total sea 101. |
| S5, recuperación | Parcial; queda el hallazgo importante F1-S5 siguiente. |
| S6, recorrido integrado backend | Cerrado en este alcance: `reader_integration.rs:838` usa selector simulado, LibraryService y ReaderService/adaptadores reales, PDF sintético válido de dos páginas, mueve fuente, guarda, consulta desde hijo y archiva/restaura verificando IDs/hash/posición. No demuestra render o instalación. |
| S7, capability Reader | Cerrado: modules/settings.rs anuncia reader=true; settings_service verifica Reader disponible y Workflow futuro deshabilitado. |

## [HIGH / Important] F1-S5 — Recuperación queda obsoleta después de una importación fallida o cancelada

Rutas: `src/features/library/LibraryPage.tsx:250`, `src/features/library/ImportPaperDialog.tsx:99`, `:131`, `:208`; efecto visible en `src/app/App.tsx:149` y `:172`.

El único callback de importación que llega a refreshLibraryStatus es onImported, llamado en confirmImport exitoso. Selección, confirmación fallida y cancelación (tanto éxito como fallo) no notifican su resultado terminal. Estas operaciones pueden crear, dejar o limpiar import_operations pendientes, que Settings usa para recoveryRequired. Navegar a Configuración tampoco actualiza la consulta. Además, «Comprobar de nuevo» solo aparece cuando falla la consulta de estado, no cuando esta ha devuelto recoveryRequired=true.

Dos escenarios reproducidos con App real en Vite SSR/JSDOM y APIs simuladas:

1. Inicio devuelve recoveryRequired=false. selectPdf devuelve ImportRecoveryRequired y el backend simulado pasa a recoveryRequired=true. Cerrar y entrar en Configuración conserva «Disponible» sin aviso ni forma manual de consultar. Salida: `RECOVERY_AFTER_FAILED_SELECT {"statusCalls":1,"backendRecovery":true,"visibleAvailable":true,"recoveryNotice":false,"retryStatus":false}`.
2. Inicio devuelve recoveryRequired=true durante recuperación. El backend pasa a false; entrar en Configuración conserva «Recuperación necesaria», sin nueva consulta ni botón. Salida: `RECOVERY_CLEARED_AFTER_START {"statusCalls":1,"backendRecovery":false,"visibleRecovery":true,"retryStatus":false}`.

Esto incumple el punto 7 de fix1: refrescar después de operaciones relevantes y distinguir recuperación de disponibilidad. Es el mismo S5 original, no un requisito arquitectónico nuevo.

Corrección acotada: notificar finalización de selección/confirmación/cancelación incluso ante error, volver a comprobar al entrar en Configuración y ofrecer comprobación manual mientras haya aviso. Si concurren consultas de estado, descartar respuestas obsoletas. No hacen falta nuevos IPC ni una API para reanudar drafts. Añadir regresiones de los dos escenarios y de cancelación que elimina la última operación pendiente.

## Cobertura y límites de cierre

Las pruebas Library persistentes añadidas sí cubren candidatos en preview, cancelación fallida visible, reintento solo Reader, × con cancelación fallida, cambios de ficha y aperturas fuera de orden. No contienen regresiones de DuplicateDecisionRequired devuelto por confirmImport tras introducir DOI, elección entre candidatos distintos, details inválidos ni salida durante una promesa de selección/confirmación pendiente. Tampoco hay pruebas App del nuevo aviso de recuperación. Son ramas de recuperación exigidas expresamente en fix1, no una petición de tests cosméticos.

Para distinguir falta de regresión de defecto funcional, esta revisión comprobó además los componentes reales con mocks, por scripts inline sin archivos:

- Preview sin candidatos → título y DOI → confirm devuelve dos candidatos → escoger B con cancelación pendiente. No abrió Reader antes de resolver cancelación y ocultó ×: `DUPLICATE_LATE_PENDING {"choices":2,"opens":0,"closeButton":false}`. Tras enviar Escape durante la espera y confirmar cancelación, abrió solo B: `DUPLICATE_LATE_SELECTED {"opens":["00000000-0000-4000-8000-000000000002"],"closed":1}`.
- Candidato inválido en confirmImport: conserva título, muestra error y no ofrece abrir UUID inválido; cancelar sigue funcionando. Salida: `INVALID_CANDIDATE {"draft":"Preserved draft","error":"Se detectó un duplicado, pero no se pudo validar su ficha. Puedes cancelar la importación.","openChoice":false}`.

Estos resultados permiten cerrar el defecto de código S2 en los escenarios inspeccionados, pero no sustituyen regresiones persistentes. Deben incorporarse al arreglo de los flujos afectados; no se cuenta un segundo HIGH solo por su ausencia. No se pide recrear rojos retrospectivos.

Los residuales TS-01 (conflicto de cola), TS-03 (apertura previa tardía desmonta importación) y R2 (determinismo de barreras actor/caller) ya están consolidados por sus revisiones especializadas. No se duplican en el conteo de esta revisión ni se asume su cierre. S3 queda limitado por TS-03 como se indica en la tabla.

## Comprobaciones ejecutadas

- `git status --short`, `git rev-parse HEAD`: limpio, HEAD indicado.
- `git diff --stat 870b3321c4e3d46597848ae5d3faf40395229621 b36ae5ae0529ba429edf7a360ade7319e353e8fe`: 24 archivos; 3866 inserciones/210 eliminaciones, en buena parte formato.
- Lectura dirigida de App, LibraryPage, ImportPaperDialog, useLibrary, dialog, pruebas Library/Settings y recorrido backend integrado; diff Settings y prueba de capabilities.
- Dos ejecuciones `node --input-type=module` con scripts inline, ambas exit0. Vite SSR + React Testing Library + JSDOM; primera carga App con APIs Settings/Library/Reader simuladas y un seam de PaperReader para no cargar PDF.js; segunda carga ImportPaperDialog real con promesas controladas. Scripts completos y salidas constan en las herramientas de esta revisión. No se modificaron archivos para ejecutar estas comprobaciones.
- Gate44 frontend/126 entradas Rust y build nativo exit0 son evidencia verificada por el orquestador para este corte, no ejecuciones nuevas de este revisor. No se equiparan entradas/helper con comportamientos independientes.
- Sin afirmación sobre WebView2, selector del OS, layout real de zoom, PDF protegido/escaneado real, instalador, rendimiento o equipo limpio. El reporte fix1 conserva explícitamente estos límites.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH | 1 | warn |
| MEDIUM | 0 | pass |
| LOW | 0 | pass |

Verdict: WARNING — S5 permanece importante. No integrar hasta corregirlo y cerrar también los residuales especializados ya registrados. Esta revisión queda terminada con DONE_WITH_CONCERNS.
