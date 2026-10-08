# T03 — revisión independiente de conformidad y calidad

Estado: DONE_WITH_CONCERNS. Revisor: /root/task03_spec_review. Fecha: 2026-10-02.

BASE: `cb60ee2d375aafc60dd2f8a5d696f1f375321cff`.
HEAD: `870b3321c4e3d46597848ae5d3faf40395229621`.
Producto: `ccc1f796f1b8af2ecaa8838f9c357c19949ae444`.
Worktree: `.worktrees/task-03-reader`; limpio antes y después de comprobar. Solo se escribe este informe central; ningún archivo de producto, test o configuración modificado.

## Alcance y método

Leídos AGENTS, INTENT, STATUS, briefs central/de tarea/de revisión, TASK03_BOUNDARIES, TASK03_PORTS, SPEC-001/002, QUALITY y los contratos/datos pertinentes. Aplicadas pautas Karpathy. Consultado el diff BASE..HEAD de 46 archivos, componentes y sus dependencias, implementación Reader/Library nueva y pruebas. La revisión especializada Rust, TypeScript y seguridad es independiente: este informe no la sustituye ni presume aprobación.

Los defectos de interfaz indicados abajo se derivan de código y, donde se indica, de componentes reales cargados en Vite SSR/JSDOM con APIs simuladas. No se abrió GUI, WebView2 ni biblioteca personal. La comprobación de cierre T02 queda fuera.

## Hallazgos importantes (HIGH)

### S1 — Cambiar de ficha permite guardar el borrador de otro paper

Ruta: `src/features/library/LibraryPage.tsx:16`; causa complementaria `src/features/library/PaperDetails.tsx:5`.

Escenario: Editar A, cambiar su título, pulsar Editar B en la lista que sigue visible y Guardar cambios. `PaperDetails` conserva su instancia y sus estados inicializados con A; las props ahora contienen el ID/revisión de B. El payload mezcla el borrador de A con la identidad válida de B. Puede sobrescribir los metadatos del segundo paper sin un conflicto de revisión.

Reproducido en JSDOM: `METADATA_CROSS_PAPER {"id":"00000000-0000-4000-8000-000000000002","title":"Draft A"}`.

Corrección: asociar inequívocamente el borrador a paper.id y controlar el cambio de selección; no basta cambiar las props. Añadir regresión A→B que compruebe ID y contenido, y un cambio de selección durante un guardado pendiente. REQ-002-02/04 y conservación de borradores.

### S2 — El flujo de duplicados pierde recuperación y reutiliza una identidad incompatible

Ruta: `src/features/library/ImportPaperDialog.tsx:11`; render `:14`.

Tres fallos de una misma máquina de estados:

- Después de cancelImport confirmado se ejecuta `setPreview(null)`. Si Reader falla, se fija openFailed, pero el render evalúa `!preview` primero: el botón «Reintentar abrir existente» es inalcanzable. Aparece de nuevo Seleccionar PDF. Reproducido: `DUPLICATE_RETRY {"cancelled":1,"opened":1,"retry":false,"select":true}`.
- Si cancelImport falla cuando hay candidatos, se fija error, pero la rama de candidatos no lo muestra. El diálogo queda aparentemente igual, sin explicación de que la cancelación falló; error solo se representa en el formulario o cuando no hay preview.
- Cancelar una primera importación vuelve al selector sin cerrar el componente; seleccionar un segundo token conserva cancelRequest.current. La siguiente cancelación envía el mismo requestId con otro importToken: el backend debe devolver Conflict por el contrato de receipts. Reproducido: `CANCEL_REUSE {"sameRequestId":true,"differentToken":true}`.

Corrección: representar por separado selección, preview, cancelación y apertura posterior; mostrar el error en todas las ramas; conservar requestId únicamente para reintentos del mismo token. Probar cancel pendiente/fallido, apertura fallida después de cancelar, segunda selección y candidatos de confirmación tardía. Son regresiones expresamente exigidas por TASK03_BOUNDARIES, actualmente ausentes en las seis pruebas Library.

### S3 — Cerrar el diálogo abandona el token y permite perder el borrador durante una operación

Ruta: `src/features/library/ImportPaperDialog.tsx:14`; interacción con `src/shared/ui/dialog.tsx:7`.

Escenario: seleccionar PDF y cerrar mediante ×, Escape o clic exterior; `onOpenChange` llama directamente a onClose. No usa cancelImport, no conserva el borrador y no comprueba pending. La × siempre está habilitada por el componente compartido. La copia staging/intención ya existe y queda sin una vía UI para recuperar o cancelar ese token. Si se cierra mientras la selección o confirmación sigue pendiente, la respuesta llega a un componente desmontado; puede desaparecer el borrador/error o producirse una importación tras haber cerrado el diálogo.

Reproducción con × después de seleccionar: `DISMISS_WITH_STAGE {"closed":1,"cancelled":0}`.

Corrección: encauzar todas las salidas por una transición explícita, impedir el descarte mientras una operación requiere respuesta y cerrar el preview solo después de cancelación confirmada o éxito confirmado. Un fallo de cancelación debe conservar diálogo y borrador. No introducir borrado directo de staging desde UI.

### S4 — La biblioteca solo permite recorrer los primeros 100 resultados

Ruta: `src/features/library/useLibrary.ts:9`; límite en `:5` y vista en `src/features/library/LibraryPage.tsx:15`.

El hook fija limit=100 y conserva únicamente result.data.items; descarta nextCursor. La vista no tiene paginación ni carga adicional. Con 101 papers o más, los posteriores quedan fuera de la lista de activos/archivados y no hay forma de recorrerlos. Buscar un título concreto puede reducir el conjunto, pero no reemplaza la navegación completa de la biblioteca. El backend sí implementa cursor y devuelve el siguiente.

Corrección: conservar y consumir el cursor con navegación/carga explícita, reiniciándolo al cambiar filtros. Probar al menos 101 resultados y archivo/restauración en una página posterior. REQ-002-04 y PageDto/LibraryApi.

### S5 — La UI omite el aviso obligatorio de recuperación

Ruta: `src/app/App.tsx:12` y `:16`.

App solo obtiene getAppInfo y getLibraryInfo; no consulta getLibraryStatus/recoveryRequired. Configuración muestra «Disponible» a partir de writable aunque existan importaciones pendientes o una recuperación fallida. Una biblioteca escribible con issues conserva precisamente recoveryRequired=true; ese hecho no se deriva de writable. Tras una interrupción el usuario no recibe el aviso exigido por TASK03_BOUNDARIES, y el problema solo puede aparecer indirectamente al intentar alguna acción.

Corrección: consultar y representar el estado de recuperación, con actualización tras operaciones relevantes y tratamiento de errores. No prometer reanudación de drafts no implementada. Probar biblioteca writable con recoveryRequired=true y recuperación que falla. El dispatch transfirió App y errores/recovery explícitamente a T03.

### S6 — Falta el recorrido backend integrado exigido entre Library y Reader

Ruta: `src-tauri/tests/reader_integration.rs:270`; fixture `:54`.

El caso de reapertura en otro proceso es real, pero la fixture inserta Paper/Document directamente con SQL y escribe el PDF directamente. No atraviesa select/confirmImport ni mueve el original; tampoco continúa con archive/restore en el mismo recorrido. Las pruebas Library previas cubren otras partes por separado, y su diff aquí solo adapta constructores. No hay prueba del recorrido requerido importación → mover fuente → Reader.openPaper → guardar posición → otro proceso → reanudar → archive/restore conservando IDs/hash/posición.

Es una falta de evidencia de integración requerida, no una afirmación de corrupción demostrada ni una exigencia de instalación nativa. Corrección: añadir ese caso con servicios/adaptadores reales y biblioteca sintética, pudiendo simular solo el selector. Mantener separadas las pruebas del render WebView2/instalador pendientes para T10.

## Hallazgo menor (MEDIUM)

### S7 — Capabilities públicas discrepan sobre Reader

Ruta del cambio: `contracts/manifest.json:44`; dependencia sin actualizar: `src-tauri/src/modules/settings.rs:13`.

El manifiesto habilita Reader y el dispatcher ejecuta sus comandos, pero settings.getAppInfo sigue anunciando `capabilities.reader=false`. T03 introduce la incoherencia al activar el módulo; el dispatch incluyó explícitamente modules/settings.rs para su integración. La App actual ignora esta bandera, por lo que no bloquea el lector visible, pero el contrato de disponibilidad publicado es incorrecto.

Corrección: sincronizar la capability real con el módulo implementado y probar getAppInfo más manifest/dispatcher. No marcar capacidades futuras como disponibles.

## Evidencia comprobada y límites

- `git diff --staged`, `git diff`, `git status --short`: árbol limpio. `git rev-parse HEAD`: HEAD indicado arriba.
- `git diff --stat cb60ee2d375aafc60dd2f8a5d696f1f375321cff 870b3321c4e3d46597848ae5d3faf40395229621`: 46 archivos, 1922 inserciones/53 eliminaciones.
- Ejecutado `npm.cmd test -- src/features/library/library.test.tsx src/features/reader/reader.test.tsx`: exit 0, 2 archivos, 13 pruebas (6 Library + 7 Reader). Las 12 focales citadas en el reporte pueden corresponder al corte previo a la prueba fraccionaria; el corte final tiene 13.
- Dos scripts inline por `node --input-type=module`, sin crear archivos, cargaron los componentes vía `vite.createServer({configFile:false,plugins:[react()],server:{middlewareMode:true},appType:'custom'})` y `ssrLoadModule` en JSDOM. Usaron Testing Library para las secuencias S1–S3. Ambos exit 0; salidas reproducidas arriba. APIs simuladas, sin pruebas de IPC/backend/render PDF nativo. Scripts completos registrados en las llamadas de esta revisión.
- Leídos `work/evidence/task03-gate-verified-final.log/.exit` y `task03-tauri-debug-final.exit` dentro del worktree: ambos exit 0. El log del gate muestra UI28 y 118 entradas Rust. No se volvió a ejecutar el gate completo ni el build nativo durante esta revisión. Las 118 entradas incluyen dos helpers según el inventario del coordinador.
- El informe del autor declara con claridad que scanned_pdf_renders usa loader simulado, que protocolo se prueba como parser/adaptador y que el build no demuestra instalación. La desviación TDD backend está documentada; no se pide crear un rojo retrospectivo.
- No se afirma render real, CSP efectiva release, picker nativo, funcionamiento instalado, rendimiento de PDF grande ni equipo limpio. Eso sigue pendiente de evidencia propia de T10; no se ha convertido en hallazgo de defecto nativo.
- No hay modificación de contrato público/schema0001 ni acceso filesystem/SQL desde los componentes revisados. No se eleva el formato compacto a un hallazgo por sí solo.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH | 6 | warn |
| MEDIUM | 1 | info |
| LOW | 0 | pass |

Verdict: WARNING — 6 hallazgos importantes pendientes. Conforme al gate del proyecto, **no integrar T03** hasta cerrar los importantes, volver a revisar las correcciones y ejecutar el gate del merge preparado. DONE_WITH_CONCERNS describe la revisión terminada, no aprobación del producto.
