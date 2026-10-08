# T03 — corrección 1

Estado: `DONE_WITH_CONCERNS` — las correcciones de código y sus comprobaciones están completas; la revisión independiente y la comprobación visual/WebView2 siguen a cargo del orquestador/QA.

## Identidad del corte

- Rama/worktree: `agent/task-03-reader` / `.worktrees/task-03-reader`.
- BASE limpia verificada: `870b3321c4e3d46597848ae5d3faf40395229621`.
- HEAD de producto: `b189371` (`fix: close T03 reader review regressions`). Este informe se agrega en un commit documental inmediatamente posterior.
- No se modificaron contratos públicos, DTO, schema0001, registro de IPC, dependencias del producto ni configuración global.
- Datos de prueba: fixtures sintéticos y temporales; ninguna biblioteca personal.

## Correcciones

- **R1 — asociación durante apertura/replay:** `prepare_open` vincula el receipt durable con el paper solicitado y su documento activo actual. `confirm_open` vuelve a comprobar la relación dentro de la transacción después de verificar el handle. Reasignar D de A a B causa rechazo sin sustituir receipt, actividad o auditoría.
- **R2 — fronteras backend:** se añadieron pruebas con servicios y adaptadores reales para la carrera de una saga de Library frente a Reader con el mismo `requestId`; la propiedad del handle y del request permanece en el actor pese a cancelar al caller; bytes PDF reales y lease/permiso de mantenimiento hasta responder; rollback conjunto de actividad/sesión/audit/receipt; CAS por documento, replay compatible/incompatible, posiciones inválidas y asociación durable; y el recorrido importación sintética de dos páginas → mover el original → abrir/guardar en Reader → consultar desde proceso hijo → archive/restore conservando ID, hash y posición. El caso `reader_reopen_child_process` es un helper/entrada de test que ejecuta el recorrido entre procesos, no una conducta independiente adicional.
- **S1 — metadatos:** `PaperDetails` se remonta por ID, un guard impide aplicar una respuesta tardía sobre otra ficha y la página descarta respuestas al cambiar de selección. El borrador de B no recibe el guardado de A; un guardado tardío no vuelve a seleccionar A.
- **S2/S3 — duplicados y salida:** cancelación conserva su `requestId` para el mismo token y cambia con otro token. Fallo de cancelación conserva diálogo, preview y borrador; X/Escape/exterior usan la misma transición confirmada; no se descarta con operaciones pendientes. Si cancelar tuvo éxito pero abrir el candidato falla, el error es accesible y el reintento invoca solo Reader.
- **S4 — paginación:** la UI consume `nextCursor`, ofrece anterior/siguiente, descarta respuestas obsoletas y resetea cursor/historial al cambiar filtros. Archivar/restaurar actualiza el listado y mantiene el contexto de página.
- **S5/S7 — recuperación y capacidad:** la app consulta el estado real de Settings, muestra recovery, permite volver a comprobar y diferencia fallo de consulta/recuperación de disponibilidad. `reader` queda habilitado en el servicio de Settings y en su prueba; módulos futuros siguen deshabilitados.
- **TS-01/02 — posición:** la cola por documento conserva comando/`requestId` inciertos, los reproduce de forma idempotente y consulta el estado durable antes de avanzar la revisión. Al reabrir con revisión nueva no descarta la intención anterior. Secuencias/identidad de vista evitan que éxito o error tardío de A cambie el estado visible de B; el trabajo ya admitido sigue persistiendo.
- **TS-03 — apertura tardía:** Library/Home invalidan efectos de navegación al iniciar una intención posterior o desmontar la vista. `App` monta Reader con clave del documento para reiniciar estado visual al cambiar documento.
- **TS-04 — PDF.js:** el loader real destruye la tarea si `task.promise` falla o se aborta, conserva el error de carga original aun con fallo de teardown, destruye recursos y cancela/limpia renders. Los tests usan el loader real con una tarea PDF.js controlable, no sustituyen la función completa.
- **Zoom:** se eliminó el máximo de ancho que reducía el zoom a resolución de raster; el canvas puede crecer dentro del contenedor desplazable. Esto es un cambio de CSS, no una medida de layout demostrada por JSDOM.
- **Limpieza/recursos:** se retiraron `VerifiedFileSnapshot` y `PathBuf` sin consumidor. El plugin Vite sirve recursos PDF.js permitidos en desarrollo y build, sin CDN ni rutas arbitrarias; QuickJS queda excluido. El inventario construido contiene 199 archivos PDF.js, cero coincidencias QuickJS y 11 archivos de licencia. Prueba local verifica bytes de una fuente, rechazo de traversal y exclusión de QuickJS.
- **Formato:** Rust se pasó por `cargo fmt`; los archivos TypeScript/CSS tocados se formatearon con Prettier efímero (no añadido a manifests/lockfiles) para que la revisión sea legible.

## Evidencia y comandos

Instalación limpia de dependencias: `npm.cmd ci` — exit 0; 156 paquetes, auditoría sin vulnerabilidades.

Pruebas focales frontend después del formato: `npm.cmd test -- src/features/library/library.test.tsx src/features/reader/reader.test.tsx src/features/reader/useReadingPosition.test.tsx src/features/reader/pdfLoader.test.ts src/shared/adapters/tauri/pdf-assets.test.ts` — exit 0, 29 tests en 5 archivos.

El foco de `confirm_open_rechecks_paper_document_association_after_file_verification` compiló inicialmente con error E0373 en el cierre de la prueba (captura prestada del request ID). Tras capturarlo por valor, el mismo test focal pasó, exit 0. Este fue un error de compilación del test, no un rojo de comportamiento.

Rojos de comportamiento previamente observados y no guardados como archivo —salieron únicamente en la herramienta—:

- `npm.cmd test -- src/features/library/library.test.tsx` — exit 1 antes de las correcciones; cuatro de diez expectativas de comportamiento fallaron: borrador A aplicado a ficha B, cursor siguiente no consumido, reintento de Reader no accesible después del fallo de cancelación y error oculto en rama con candidatos.
- `. .\scripts\development-env.ps1; cargo test --manifest-path src-tauri/Cargo.toml --test reader_integration replay_rejects_document_reassigned_to_another_paper_without_changing_receipt_or_audit -- --nocapture` — exit 1 antes de R1; la prueba falló con “replay must reject a document now owned by a different paper”.

Gate completo final: `& .\scripts\check.ps1` — exit 0. Salida: [task03-fix1-gate-final.log](../../work/evidence/task03-fix1-gate-final.log), exit [task03-fix1-gate-final.exit](../../work/evidence/task03-fix1-gate-final.exit). Resultados: typecheck; 44 pruebas frontend (9 archivos); build Vite; fmt; Clippy con `-D warnings`; Cargo completo (126 entradas de test Rust, incluyendo el helper de proceso hijo en `reader_integration`); contrato generado sin deriva. El primer gate se conservó como [task03-fix1-gate-failed-1.log](../../work/evidence/task03-fix1-gate-failed-1.log) / [.exit](../../work/evidence/task03-fix1-gate-failed-1.exit), exit 101 por `clippy::needless_question_mark`; se eliminó el `Ok(...?)` redundante y el gate final pasó. El build frontend avisa que el bundle JS de PDF.js supera 500 kB; no es fallo.

Build nativo: `. .\scripts\development-env.ps1; npm.cmd run tauri:build -- --debug --no-bundle` — exit 0; binario [research-workbench.exe](../../work/cargo-target/debug/research-workbench.exe), 25,864,192 bytes, SHA-256 `871187877DCA988FFDC0560C0C9B2C30A295F4B721005213626C3462A092C481`. Log: [task03-fix1-tauri-debug-final.log](../../work/evidence/task03-fix1-tauri-debug-final.log); exit [task03-fix1-tauri-debug-final.exit](../../work/evidence/task03-fix1-tauri-debug-final.exit).

## Límites y autorrevisión

No se lanzó WebDriver ni se abrió la aplicación de este corte. La prueba de recursos Vite confirma la respuesta local en servidor de desarrollo/build, no la respuesta bajo WebView2. El zoom se verificó en fuente/CSS, no mediante render visual real. La QA nativa debe usar la copia exacta de este binario y una biblioteca sintética; esta versión aún no tiene una copia congelada confirmada por el orquestador.

Revisé `git diff --check`, formato Rust, locks/manifests y el diff de los archivos propios. El primer gate falló por Clippy y está preservado; el gate final y build acabaron en cero. No queda una discrepancia de contrato identificada. El árbol del worktree se limpiará tras añadir este informe; las revisiones independientes del delta y la aceptación nativa quedan pendientes del orquestador.
