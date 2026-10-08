# T03 — revisión independiente TypeScript

**Resultado: DONE_WITH_CONCERNS. Veredicto: BLOCK; hay hallazgos Important abiertos.**

- Reviewer: `/root/task03_typescript_review`; solo revisión, sin cambios de producto, merges, instalaciones ni subagentes.
- Worktree: `C:/Users/david/Projects/Research-Workbench/.worktrees/task-03-reader`.
- BASE: `cb60ee2d375aafc60dd2f8a5d696f1f375321cff`.
- HEAD revisado: `870b3321c4e3d46597848ae5d3faf40395229621`; implementación `ccc1f796f1b8af2ecaa8838f9c357c19949ae444`.
- Árbol limpio antes y después de los comandos. Revisión local BASE..HEAD; staged/unstaged vacíos. Sin remoto configurado: CI y merge readiness de PR no verificables ni aplicables como evidencia de esta revisión local.
- Fuentes: AGENTS, INTENT, STATUS, brief de revisión y autor, brief T03, TASK03_BOUNDARIES/PORTS, CONTRACTS Library/Reader, SPEC-001/002 y QUALITY. Guías consultadas: coding-standards y frontend-patterns. Búsqueda puntual de memoria sin coincidencias relevantes.

## Hallazgos adicionales

### TS-01 — Important: una respuesta perdida deja la cola de posición bloqueada hasta reiniciar la UI

**Ruta:** `src/features/reader/useReadingPosition.ts:11-16`.

Escenario: la primera escritura usa revision 0, el backend confirma revision 1, pero el adaptador devuelve StorageUnavailable por perder la respuesta. La entrada global permanece con revision 0. La siguiente escritura usa un requestId nuevo y revision 0, recibe Conflict y no reconcilia. Incluso cerrar/reabrir con un `OpenPaperDto.readingPosition.revision=1` conserva la entrada antigua mediante `previous ?? ...`, ignorando la revisión recién leída. No hay llamada a getReadingPosition ni recuperación del receipt original.

**Impacto:** todos los guardados posteriores de ese documento fallan durante la sesión; reabrir no recupera la función. CONTRACTS exige volver a leer/reconciliar ante Conflict. Conservar el requestId de la operación cuyo resultado es incierto y reconciliar la cola con el estado durable son necesidades distintas; no basta cambiar silenciosamente expectedRevision y sobrescribir.

**Reproducción ejecutada:** hook real, React `renderHook` y JSDOM; API simulada que incrementa revisión pero devuelve fallo en el primer guardado. Desmontar y montar con revision 1; guardar de nuevo. Resultado: `backendRevision:1, reopenedRevision:1, expectedRevisions:[0,0], status:"error"`. Script inline con `node --input-type=module`, sin crear ni modificar tests.

### TS-02 — Important: el estado de guardado de A se muestra en B

**Ruta:** `src/features/reader/useReadingPosition.ts:8,19`; integración `src/app/App.tsx:16`.

El contador latest solo cambia al guardar. Cambiar initial.documentId no invalida callbacks pendientes ni reinicia status/error. App reutiliza PaperReader sin key cuando cambia opened. Un guardado pendiente de A puede completar después del cambio a B y poner su estado en saved/error. También se hereda el estado previo aunque no haya respuesta pendiente.

**Impacto:** «Guardado» o el error corresponden a otro documento; incumple el descarte de respuestas antiguas explícito de T03. La cola puede sobrevivir al componente para terminar persistencia, pero las notificaciones deben pertenecer a la vista/documento correctos.

**Reproducción ejecutada:** hook real con save(A) diferido; rerender con documentId B; resolver A. Resultado: `statusInB:"saved"`. No se guardó B. El mismo caso con error debe descartarse igualmente.

### TS-03 — Important: aperturas tardías sustituyen la navegación actual

**Ruta:** `src/features/library/LibraryPage.tsx:10,15`; `src/features/library/LibraryHome.tsx:7`; `src/app/App.tsx:13`.

Los botones siguen activos mientras openPaper espera. Sus callbacks invocan onOpenPaper sin comprobar si siguen montados o si otra apertura/navegación los reemplazó. Pulsar Leer(A), Leer(B) y resolver B antes que A acaba mostrando A. Pulsar Leer(A) y después Configuración abre A al completar, aunque LibraryPage ya se desmontó. showReader conserva acceso al estado de App.

**Impacto:** el resultado antiguo cambia de documento o saca al usuario de la vista elegida. El primer caso permite además TS-02 al actualizar opened sobre PaperReader ya montado. Se requiere invalidación por intención de navegación/apertura, conservando el trabajo backend ya admitido.

### TS-04 — Important: las cargas PDF fallidas no liberan su tarea/worker

**Ruta:** `src/features/reader/pdfLoader.ts:14`; consumidor `src/features/reader/PaperReader.tsx:10`.

Si task.promise rechaza por PDF inválido/protegido, catch elimina el listener de abort y relanza sin llamar task.destroy. PaperReader no recibe LoadedPdf, por lo que pdfRef queda null; cerrar/desmontar tampoco destruye esa tarea. La implementación instalada de PDF.js 6.3.289 confirma que DocException rechaza la promesa (`node_modules/pdfjs-dist/build/pdf.mjs:16652`) y que la liberación del worker se hace en PDFDocumentLoadingTask.destroy (`:15581`); el rechazo no hace esa liberación automáticamente.

**Impacto:** intentos fallidos repetidos acumulan workers/recursos; precisamente la ruta recuperable de PDF corrupto/protegido queda fuera de la liberación requerida por T03. Debe destruirse también en la ruta de error, conservando el error original y manejando el posible fallo del teardown.

## Hallazgos del revisor general corroborados

Estos son los mismos defectos comunicados por el orquestador; no contarlos dos veces en el ledger.

| Severidad | Ruta/línea | Caso y efecto confirmado por lectura |
|---|---|---|
| Important | `PaperDetails.tsx:5-7`; `LibraryPage.tsx:16` | Editar A y luego B reutiliza useState de A, mientras paper.id/revision ya son B. Guardar envía los metadatos de A sobre B; es corrupción bibliográfica. Una respuesta antigua de onChanged puede además volver a seleccionar A después de seleccionar B. |
| Important | `useLibrary.ts:5,9-10` | Se solicitan 100 elementos y se descarta nextCursor/total. No hay control que pida la siguiente página: la biblioteca truncada aparenta ser completa. |
| Important | `ImportPaperDialog.tsx:11-14` | cancel confirmado hace setPreview(null); el render evalúa !preview antes de openFailed. Si Reader falla, «Reintentar abrir existente» resulta inalcanzable y solo aparece Seleccionar PDF. |
| Important | `ImportPaperDialog.tsx:11,14` | Un cancel fallido conserva preview y candidates; la rama de candidatos no representa error. El usuario no ve la causa ni una confirmación accesible de que la cancelación falló. Reader correctamente no se abre en ese caso. |
| Important | `ImportPaperDialog.tsx:8,10-11` | cancelRequest persiste tras cancel exitoso y nueva selección en el mismo diálogo. Cancelar el nuevo token reutiliza requestId con payload distinto; el contrato produce Conflict, que queda oculto en la rama de candidatos. |
| Important | `ImportPaperDialog.tsx:14` | X/Escape/clic fuera llaman directamente onClose con preview vigente o trabajo pendiente. Se abandona la intención/borrador sin cancelImport confirmado. Si select/confirm/open resuelve después, puede completar y ejecutar callbacks de navegación con diálogo ya cerrado. |
| Important | `App.tsx:12,16` | No se consulta getLibraryStatus ni se presenta recoveryRequired, obligatorio en TASK03_BOUNDARIES. El estado de Configuración deriva solo de library.writable, insuficiente para reflejar bloqueo de recuperación. |
| Important | `src/app/tokens.css:2` | canvas max-width:100% comprime el tamaño renderizado al contenedor. A partir del ancho disponible, subir zoom aumenta backing pixels pero no el tamaño visual; el control deja de ampliar el PDF. |

La corrección de PaperDetails debe preservar borradores por identidad y manejar respuestas de una operación previa; resetear todo ante cualquier nuevo objeto paper puede borrar edición legítima después de un reload. Las correcciones de duplicados deben conservar requestId para reintentos del mismo token y renovarlo al empezar otra intención.

## Cobertura y observaciones no bloqueantes separadas

- `reader.test.tsx:13-22` comparte mocks sin clear/reset entre tests; afterEach solo hace cleanup. `close_cancels_render` puede aceptar llamadas destroy acumuladas de pruebas anteriores. No comprueba que el AbortSignal de un render pendiente quede aborted. `render_old_result_discarded` comprueba el input de página después de resolver una promesa simulada, pero el mock no dibuja ni intenta un resultado/error tardío y no verifica el abort. No acredita por sí solo el descarte ni la liberación de PDF.js.
- `corrupt_protected_pdf_recoverable` reemplaza loadPdf entero por una promesa rechazada; no ejercita el teardown de TS-04. `scanned_pdf_renders` no abre scanned.pdf; el informe del autor lo reconoce correctamente.
- `conflict_preserves_draft` demuestra que un error no vacía el campo de un único paper, pero no comprueba reconciliación/reintento ni cambio A→B. Los casos obligatorios de cancelación pendiente/fallida y Reader fallido después de cancel no están probados.
- Minor: `vite.config.ts:14` aplica copia de recursos solo en build y no existe public/pdfjs ni middleware dev. `pdf-assets.ts:2-8` pide /pdfjs también bajo tauri:dev. Recursos CMap/font/WASM no quedan servidos por esa configuración en dev. No es evidencia de defecto del bundle instalado; añadir soporte dev o documentar/verificar su limitación.
- Minor: el código de producto se comprime a funciones/JSX de una sola línea de cientos o miles de caracteres. Dificulta aislar cambios, asignar líneas de hallazgos y revisar transiciones. Mantener formato legible en estas máquinas de estado ayudará a revisar las correcciones; no se solicita un refactor ajeno al alcance.

## Comandos y resultados

Ejecutados desde el worktree indicado:

1. `git status --short`, `git diff --staged --stat`, `git diff --stat`: sin cambios.
2. `git diff cb60ee2d375aafc60dd2f8a5d696f1f375321cff 870b3321c4e3d46597848ae5d3faf40395229621 --stat` y diffs/lecturas del código TS/TSX, CSS, Vite, assets y tests modificados. Puertos/clientes y contrato generados inspeccionados como contexto, sin drift en DTO de este cambio.
3. `npm.cmd run typecheck`: exit 0, comando canónico `tsc -b`.
4. ESLint: ausente de package.json, `node_modules/.bin/eslint.cmd` y PATH. No ejecutado ni instalado; no se afirma lint verde.
5. `npm.cmd test -- src/features/library/library.test.tsx src/features/reader/reader.test.tsx src/shared/adapters/tauri/client.test.ts`: exit 0, 3 archivos/21 pruebas.
6. Dos probes inline `node --input-type=module`, importando el hook real `./src/features/reader/useReadingPosition.ts`, `jsdom` y `@testing-library/react`: exit 0; resultados reproducidos en TS-01/02. Son pruebas simuladas del cliente, no IPC nativo.
7. `git rev-parse HEAD`: coincide con el HEAD indicado; `git status --short`: limpio tras revisión.

El gate/build completos previos son evidencia del autor, no reejecutados aquí. No se ejecutaron GUI nativa, nuevas instalaciones ni pruebas sobre bibliotecas personales. No se acredita render WebView2, CSP efectiva, rendimiento ni instalación. PDF render/cancel en ruta exitosa contiene guardas y cancelación explícita; esta revisión no presume que falten solo porque sus mocks sean débiles. La liberación fallida TS-04 sí es un defecto de código identificado.

## Autorrevisión y cierre

No se detectaron nuevos caminos SQL/filesystem/invoke desde componentes ni relajación de strict en el diff. Los resultados del typecheck y tests no cubren los escenarios fallidos anteriores. Corregir en el autor, añadir pruebas que retengan/ordenen promesas por documento y token, y revisar el delta antes de integrar. No se recomienda aprobar T03 con estos Important abiertos.
