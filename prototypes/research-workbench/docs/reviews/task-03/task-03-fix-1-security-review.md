# T03 fix1 — revisión independiente de seguridad

Estado: **DONE_WITH_CONCERNS**. Veredicto: **BLOCK** por SEC-T03-2/R2 parcialmente abierto. SEC-T03-1/R1 cerrado. Un nuevo Minor de recursos de desarrollo; ningún nuevo Critical/Important de producto demostrado.

- BASE del delta: `870b3321c4e3d46597848ae5d3faf40395229621`.
- HEAD: `b36ae5ae0529ba429edf7a360ade7319e353e8fe`.
- Worktree: `.worktrees/task-03-reader`, limpio al inicio y final.
- Revisor: `/root/task03_security_review`.
- Fuentes: brief e informe central fix1, revisión de seguridad original, revisión Rust fix1, delta y pruebas pertinentes. Mismos límites/contratos de TASK03_PORTS y TASK03_BOUNDARIES. Sin producto, subagentes ni merge.

## SEC-T03-1 / R1 cerrado

`src-tauri/src/adapters/sqlite/reader_repository.rs:145` comprueba identidad del Paper del receipt, resuelve el documento activo del Paper solicitado y exige el mismo documento durable/asociación. `:169` vuelve a comparar Paper/documento del resultado durable, Paper actual, biblioteca y snapshot de la lease antes de devolver la URL. El replay no ejecuta el helper de actividad ni sustituye su receipt.

Se inspeccionaron las regresiones `reader_integration.rs:570` y `:656`: reasociación D de A a B antes de prepare y entre verificación/confirmación, rechazo y conservación de receipt/auditoría; la primera además comprueba actividad intacta. La revisión Rust fix1 ejecutó ambas con éxito. No se repitió su ejecución ni se atribuye a seguridad un rojo propio. La fuente corregida y esa evidencia cierran el hallazgo original.

## SEC-T03-2 / R2 — Important de validación, parcialmente abierto

Este es el mismo R2 registrado por Rust; **no son hallazgos acumulativos**.

Se acepta la mejora: `document_protocol.rs:143` ya recorre ReaderService.read_document con persistencia/store reales y comprueba bytes y permiso retenido. `protocol.rs:219` prueba los helpers utilizados por register: 200, Content-Length exacto, application/pdf, cuerpo completo, ausencia de Accept-Ranges y lease/permiso vivos dentro del callback de entrega. La extracción conserva el orden de propiedad de producción y no duplica una implementación exclusiva de test.

Permanece pendiente el escenario de una lectura admitida todavía activa al desaparecer el receptor. La prueba de integración espera que read_document termine; la unitaria empieza con bytes ya disponibles. Ninguna detiene read_all o su trabajo bloqueante mientras cancela el caller/fetch y observa que mantenimiento/cierre siguen esperando hasta la entrega terminal. Es una ausencia de la regresión exigida, no evidencia de una fuga o liberación prematura actual.

También se corrobora el límite del probe de apertura señalado por Rust: `reader_integration.rs:72` señaliza antes de construir ProbeHandle y lo construye después de liberar la barrera. El contador cero durante esa barrera no demuestra la vida del handle dentro del job del actor; el único Drop al final tampoco fija el orden respecto al commit/rollback. La implementación captura correctamente la lease por valor, pero falta la observación determinista del intervalo requerido. El detalle y la corrección mínima están en `task-03-fix-1-rust-review.md`; no se exige rehacer las categorías de R2 ya aceptadas.

Para cerrar: barreras sobre lectura activa y job actor admitido, cancelación realmente observada, lease/permiso vivos mientras sigue el trabajo y liberación terminal en éxito/error. Ajustar el informe del autor, cuya afirmación de retención ya comprobada dentro del actor supera el alcance del probe actual.

## SEC-T03-3 — Minor: MIME incorrecto de fallbacks JavaScript en desarrollo

Ubicación: `vite.config.ts:67`.

El nuevo middleware devuelve application/octet-stream para todo recurso salvo LICENSE. Esto incluye `wasm/openjpeg_nowasm_fallback.js` y `wasm/jbig2_nowasm_fallback.js`. PDF.js instalado los carga mediante import() en `node_modules/pdfjs-dist/build/pdf.worker.mjs:9705`; un módulo JavaScript de navegador necesita MIME JavaScript. Si la decodificación cae al fallback en desarrollo, su importación será rechazada.

Comprobación focal ejecutada con createServer de Vite y la configuración real del checkout, host 127.0.0.1 y puerto efímero, seguido de server.close():

| Recurso | HTTP | Content-Type | Bytes |
|---|---|---|---:|
| openjpeg_nowasm_fallback.js | 200 | application/octet-stream | 451590 |
| jbig2_nowasm_fallback.js | 200 | application/octet-stream | 145703 |

El tipo y contenido se midieron por fetch; el rechazo del módulo es una consecuencia de su MIME/import(), no una observación de render WebView2. Alcance limitado al middleware dev y al uso del fallback; no demuestra un fallo del camino WASM normal ni del bundle release. Mantener el allowlist de archivos y servir .js con MIME JavaScript, incluyendo una expectativa de MIME en la prueba de recursos. No hace falta ampliar CSP ni habilitar QuickJS.

## Consecuencias restantes revisadas

- Allowlist de recursos construido a partir de directorios PDF.js locales; la petición solo busca una clave del Map y no se convierte en ruta filesystem. Traversal, escapes inválidos y nombres ajenos no conceden acceso a archivos libres. QuickJS continúa excluido.
- CSP, URLs del worker/recursos, manifests y lockfiles no cambian en este delta. No se añade origen remoto ni permiso de scripting PDF.
- El loader conserva enableXfa=false y ahora destruye la tarea en fallo/abort, conservando el error original. No se incorpora viewer/sandbox ni HTML no escapado.
- full_pdf_response se llama desde register después de validar el origen; el helper extraído no abre una nueva entrada IPC/HTTP. deliver_response conserva lease y OperationPermit hasta finalizar el callback.

## Evidencia y límites

Ejecutado por seguridad: git diff del delta y diff --check (exit 0), lectura focal de código/pruebas, git rev-parse HEAD y status limpio, y probe HTTP Vite anterior (exit 0, servidor cerrado en finally). El probe usó createServer({server:{host:"127.0.0.1",port:0,strictPort:false},logLevel:"silent"}), fetch de las dos rutas /pdfjs/wasm indicadas y lectura de status/content-type/arrayBuffer.

No se repitió el gate: revisión Rust fix1 registra sus 37 entradas focales verdes, y el orquestador confirmó gate completo (44 frontend/126 entradas Rust) y build exit 0. Es evidencia ajena identificada, no ejecución adicional propia. No se repitió npm audit porque no hay delta de dependencias.

Sin GUI, bibliotecas personales o modificación del ejecutable congelado. Render real, CSP efectiva, decodificadores y QA WebView2 siguen pendientes sobre el binario SHA-256 `871187877DCA988FFDC0560C0C9B2C30A295F4B721005213626C3462A092C481`. No se afirma instalación ni funcionamiento nativo por los tests backend.
