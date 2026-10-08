# T03 fix1 — revisión independiente TypeScript

**DONE_WITH_CONCERNS — BLOCK.** Dos hallazgos Important abiertos: TS-01 sigue sin resolver correctamente Conflict y TS-03/S3 conserva una vía de navegación tardía que abandona una importación.

- Reviewer: `/root/task03_typescript_review`.
- Worktree: `C:/Users/david/Projects/Research-Workbench/.worktrees/task-03-reader`.
- BASE fix1: `870b3321c4e3d46597848ae5d3faf40395229621`.
- HEAD: `b36ae5ae0529ba429edf7a360ade7319e353e8fe`; producto `b189371`.
- Baseline original: `cb60ee2d375aafc60dd2f8a5d696f1f375321cff`.
- Alcance: delta y consecuencias frontend, con el brief fix1, informe del autor y revisión TS anterior. No reauditoría T02, cambios de producto, subagentes ni merge. Este archivo es la única escritura de revisión.
- Árbol de producto limpio; revisión local, sin PR/CI remoto verificable. Gate completo/build son evidencia del autor y del orquestador, no reejecutados aquí.

## F1-TS-01 — Important: un Conflict se convierte automáticamente en permiso para sobrescribir

**Ruta:** `src/features/reader/useReadingPosition.ts:65-86`, especialmente `:79-85`; conservación indiferenciada de errores en `:95-99`. **Consumidor:** `src/features/reader/PaperReader.tsx:9,13-15`.

La cola conserva requestId/payload y sabe reproducir una petición incierta, pero trata cualquier fallo como incertidumbre. Si el replay falla, basta que `getReadingPosition` devuelva una revisión mayor que expectedRevision para descartar la intención anterior y ejecutar el siguiente guardado sobre esa revisión. Una revisión mayor puede pertenecer a otro cambio confirmado; no prueba que se haya confirmado la operación propia. Se pierde el código Conflict al convertirlo en Error genérico.

**Reproducción ejecutada con hook real y React/JSDOM:** UI inicial página1/revision0; estado durable página9/revision1 de otro cambio. Encolar save página2 y página3 antes de recibir respuesta alguna. Secuencia observada:

1. save(página2, expectedRevision0, requestId X) → Conflict.
2. Replay idéntico X → Conflict.
3. getReadingPosition → página9/revision1.
4. save(página3, expectedRevision1, requestId Y) → éxito.

Resultado: el estado durable termina en página3/revision2; `status="saved"`, `error=null`. No hubo acción humana entre esos pasos: la segunda intención ya estaba en cola. La posición externa queda reemplazada silenciosamente y el conflicto puede no llegar a mostrarse por la comparación de sequence.

**Impacto:** viola la prohibición expresa de sobrescribir Conflict silenciosamente del brief fix1 y la reconciliación con usuario de CONTRACTS. El caso de pérdida de respuesta mejora, pero TS-01 no se puede cerrar. PaperReader permanece sin control de resolución/reintento: representa «Cambios pendientes» y texto de error; cambiar página/zoom vuelve a activar este flujo automático.

**Corrección esperada:** distinguir resultado incierto de rechazo definitivo/Conflict; detener las intenciones posteriores hasta resolución explícita y preservar su contenido. Una lectura durable con revisión superior no constituye prueba de éxito propio. Añadir la regresión con dos saves preencolados y un Conflict real, además del caso ya cubierto de StorageUnavailable.

## F1-TS-02 — Important: una apertura anterior abandona el diálogo de importación recién iniciado

**Ruta:** `src/features/library/LibraryPage.tsx:51-60,103-106,245-254`; efecto de integración en `src/app/App.tsx:77-80,167-181`.

openIntent se incrementa al iniciar otro Leer y al desmontar LibraryPage, pero abrir Importar PDF no invalida la apertura pendiente. La vista Library sigue montada durante el diálogo, así que su guard acepta la respuesta anterior.

**Reproducción ejecutada con componentes reales, Vite SSR y React/JSDOM:**

1. Pulsar Leer(A) con openPaper diferido.
2. Pulsar Importar PDF y Seleccionar PDF; selectPdf devuelve un preview con token nuevo. La ficha de importación está visible.
3. Resolver la apertura antigua de A.

Resultado medido: `previewVisible=true`, `onOpenPaperCalls=1`, `cancelCalls=0`. El probe mantiene LibraryPage montada para observar el callback. En la integración real, showReader cambia view a PaperWorkspace, desmontando LibraryPage y su ImportPaperDialog; no hay transición cancelImport en ese camino.

**Impacto:** se pierde el borrador visible y se abandona staging/token pese a que X/Escape/exterior ahora usan cancelación confirmada. También funciona si la selección del nuevo PDF aún está pendiente. Es un cierre incompleto de TS-03/S3, no un nuevo requisito de arquitectura.

**Corrección esperada:** invalidar la apertura anterior al iniciar una intención que la sustituye, incluyendo abrir la importación, o impedir iniciar ese flujo hasta resolverla. Probar la secuencia anterior y verificar que el diálogo/borrador no se abandona ni se navega por el resultado viejo.

## Estado de las correcciones anteriores

| Tema | Resultado de revisión |
|---|---|
| TS-01, pérdida de respuesta/reapertura | Parcial: se conserva requestId y se reproduce el payload. F1-TS-01 impide cerrar la corrección completa. |
| TS-02, estado A→B | Cerrado para el defecto observado: identidad/generación en hook, reinicio de estado y key por documento en App. Pruebas de éxito y error tardíos pasan. Al desmontar, la cola puede terminar; las actualizaciones de ese componente desmontado no se trasladan al nuevo. |
| TS-03, Leer A/B y salida de Home | Casos originales corregidos mediante openIntent y guards de montaje. F1-TS-02 conserva otra transición relevante que la misma corrección debe cubrir. |
| TS-04, carga PDF fallida | Cerrado: catch destruye tarea mediante destroyTask idempotente y conserva error original incluso si falla teardown. Pruebas del loader real con seam de tarea verifican carga fallida/abort, destroy y cancel de RenderTask. |
| Metadatos A→B | key de PaperDetails y selectedId en el callback evitan enviar A a B y volver a seleccionar A por un resultado tardío. Casos focales pasan. |
| Duplicados | openFailed se representa antes de !preview; error común con role alert; intent de cancelación ligado al token. El reintento de Reader no vuelve a cancelar/importar. Casos focales de fallo cancel/open pasan. |
| X/Escape/exterior | onOpenChange usa closeThroughTransition; pending no permite salida y el preview se conserva si cancel falla. La salida por navegación anterior de F1-TS-02 aún evita esta transición. |
| Paginación | nextCursor y pila de cursores consumidos; filtros reinician cursor/historial; efectos antiguos se invalidan. La prueba focal verifica cursor siguiente y reset. |
| Recovery/Settings | App consulta estado real, representa recoveryRequired/fallo y ofrece recomprobar errores; Settings reader=true. Ya no deriva Disponible solo de library.writable. |
| Zoom | CSS elimina max-width y flex-shrink del canvas y permite scroll. Corrección de fuente adecuada; layout/zoom real aún pendientes de QA nativa. |
| Assets dev | Middleware usa inventario cerrado de recursos locales, compartido con build y excluye QuickJS. Prueba real de servidor local pasa para fuente, traversal y QuickJS. |

## Cobertura y límites

- `useReadingPosition.test.tsx:40-86` solo simula StorageUnavailable y luego una posición durable coincidente. No contempla Conflict ni una revisión mayor con posición ajena. Su verde no refuta F1-TS-01.
- `library.test.tsx` cubre respuestas Leer A/B invertidas y Home desmontado, pero no Leer pendiente → nueva importación. La prueba de cierre del diálogo comprueba X con cancel fallido; no representa la navegación externa de F1-TS-02.
- Las pruebas de render ahora limpian mocks y verifican AbortSignal/error tardío. El loader dispone de una prueba específica de cancel del RenderTask real a través del seam. Esto mejora la cobertura previa sin convertirla en prueba de WebView2.
- No se añadieron pruebas de producto durante esta revisión. Los dos probes se ejecutaron inline, con APIs simuladas y componentes/hooks reales, sin crear archivos de test.
- Minor documental: el informe del autor afirma que los componentes/hooks tocados fueron formateados; PaperReader.tsx y PaperDetails.tsx siguen comprimidos como en BASE. No es el motivo del bloqueo.
- No hay una nueva afirmación de render nativo, layout, CSP release o instalación. La aceptación visual corresponde al binario congelado SHA-256 `871187877DCA988FFDC0560C0C9B2C30A295F4B721005213626C3462A092C481` y sigue pendiente de QA.

## Validación ejecutada

Desde `.worktrees/task-03-reader`:

1. `git status --short`, `git diff --staged --stat`, `git diff --stat`, `git rev-parse HEAD`: árbol limpio y HEAD indicado.
2. `git diff 870b3321c4e3d46597848ae5d3faf40395229621 b36ae5ae0529ba429edf7a360ade7319e353e8fe --stat` y lectura de diff/consecuencias TS/JS, tests, CSS, Vite y Settings relevante.
3. `npm.cmd run typecheck`: exit 0.
4. ESLint sigue ausente de manifest/bin local; no instalado ni ejecutado. No se declara lint verde.
5. `npm.cmd test -- src/features/library/library.test.tsx src/features/reader/reader.test.tsx src/features/reader/useReadingPosition.test.tsx src/features/reader/pdfLoader.test.ts src/shared/adapters/tauri/pdf-assets.test.ts`: exit 0, 5 archivos/29 pruebas.
6. Probe F1-TS-01: here-string PowerShell enviado a `node --input-type=module`; import del hook real, renderHook/act y JSDOM; exit 0 y resultado descrito arriba.
7. Probe F1-TS-02: mismo mecanismo inline; createServer middlewareMode y ssrLoadModule de LibraryPage, React/Testing Library/JSDOM. Primer intento exit1 por HTMLInputElement ausente en el harness, antes de observar el comportamiento; tras añadir ese global de JSDOM, exit0 y resultado descrito arriba. El primer fallo no es defecto de producto ni rojo de comportamiento.

## Veredicto

**BLOCK** hasta resolver F1-TS-01 y F1-TS-02 y revisar su delta. Los hallazgos son escenarios de código reproducidos; la aceptación visual pendiente no se usa para inventar defectos. Sin cambios de producto ni commits por el revisor.
