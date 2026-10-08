# T03 fix2 — revisión focal de seguridad

Fecha: 2026-10-03. Estado: **DONE_WITH_CONCERNS**. Veredicto de integración: **BLOCK** por el único residual R2a ya registrado por Rust; no se duplica como hallazgo nuevo. La frontera de lectura/entrega R2b y el MIME SEC-T03-3 quedan cerrados. Sin nuevos Critical/Important/Minor de seguridad en el delta revisado.

- BASE: `b36ae5ae0529ba429edf7a360ade7319e353e8fe`.
- HEAD: `30d532714d053c09d4021bc89bafbb0168b1602a`.
- Worktree: `.worktrees/task-03-reader`; HEAD y árbol limpio verificados.
- Revisor: `/root/task03_security_review`.
- Ownership: solo este informe central. Sin producto, subagentes, merge o GUI.

## Fuentes y alcance

Leídos: task-03-fix-2-review-brief.md, task-03-fix-2-brief.md, informe central del autor, task-03-fix-1-security-review.md, task-03-fix-2-rust-review.md y delta pertinente de store/protocol, harness de Reader, Vite y prueba de recursos. Se conservan INTENT y límites/contratos ya leídos en las revisiones anteriores. No se reaudita T02 ni se reabre R1 cerrado sin cambio en su implementación.

## SEC-T03-3 — cerrado con servidor real

`vite.config.ts:71` sirve los fallbacks .js con `text/javascript; charset=utf-8`; `:73` sirve .wasm con `application/wasm`. La petición sigue resolviéndose exclusivamente por clave en el Map de recursos inventariados; no se transforma en una ruta filesystem libre. El filtro QuickJS permanece antes de construir el inventario de dev/build.

Probe propio sobre la configuración Vite real del checkout, host 127.0.0.1, puerto efímero y cierre en finally:

| Ruta bajo /pdfjs/ | HTTP | Content-Type | Bytes |
|---|---|---|---:|
| wasm/openjpeg_nowasm_fallback.js | 200 | text/javascript; charset=utf-8 | 451590 |
| wasm/jbig2_nowasm_fallback.js | 200 | text/javascript; charset=utf-8 | 145703 |
| wasm/openjpeg.wasm | 200 | application/wasm | 252032 |
| wasm/jbig2.wasm | 200 | application/wasm | 104852 |
| wasm/qcms_bg.wasm | 200 | application/wasm | 96589 |
| standard_fonts/FoxitSerif.pfb | 200 | application/octet-stream | 19469 |
| LICENSE | 200 | text/plain; charset=utf-8 | 10174 |
| wasm/quickjs-eval.js | 404 | ausente | 0 |
| wasm/quickjs-eval.wasm | 404 | ausente | 0 |
| %2e%2e%2Fpackage.json | 404 | ausente | 0 |
| %ZZ | 400 | ausente | 0 |
| wasm/unknown.js | 404 | ausente | 0 |

Esto prueba respuesta HTTP/MIME e inventario autorizado en desarrollo. No acredita ejecución del fallback ni render WebView2.

## SEC-T03-2 / R2 — lectura/entrega cerrada; único residual R2a compartido

**R2b cerrado.** El seam ReadGate de `adapters/documents/store.rs:121` está bajo cfg(test), dentro del spawn_blocking que conserva el ManagedFile real y antes de read_bounded. No sustituye lectura, validación de tamaño o identidad. El helper privado `modules/reader/protocol.rs:47` es utilizado por register y la prueba; consume el resultado del mismo ReaderService y conserva lease/OperationPermit hasta deliver_response. No cambia origen, URI, ventana, método, autoridad, DTO o IPC.

La nueva prueba `protocol.rs:254` espera la entrada en esa lectura real, descarta el receptor y observa mantenimiento ocupado y exclusión de mantenimiento mientras sigue pendiente; después libera la lectura y observa respuesta terminal y permiso liberado. Combinada con la prueba ya aceptada que comprueba lease/permiso dentro del callback de entrega, cubre el hueco anterior. El revisor Rust ejecutó las tres pruebas de protocolo con exit 0; seguridad inspeccionó fuente/consecuencias y no repitió compilación.

**R2a Important residual, mismo hallazgo de Rust.** `reader_integration.rs:113–115` construye el future de confirm_open, envía admitted y lo devuelve sin primer poll. La señal puede observarse antes de que el job llegue al actor, por lo que todavía no prueba la retención en ese intervalo. No se ha identificado liberación incorrecta en producto. Basta la corrección de harness ya acordada en task-03-fix-2-rust-review.md: notificar después del primer Pending del future real, con actor bloqueado, y conservar los dos finales commit/rollback. No se solicita otra prueba ni cambio de producto por seguridad. El cierre Library–Reader informado por Rust se acepta y no se repite.

## Evidencia propia y ajena

Propia: inspección del delta; `git diff --check b36ae5ae0529ba429edf7a360ade7319e353e8fe 30d532714d053c09d4021bc89bafbb0168b1602a` sin errores; HEAD y status limpio; `git diff --name-only` acotado a CSP/capabilities/contracts/manifests/lockfiles sin cambios; probe HTTP anterior, exit 0. No hay ampliación de CSP/host remoto, permisos, dependencias o autoridad filesystem por este delta focal.

El probe se ejecutó inline con `node --input-type=module -e`, importando createServer de vite, con `createServer({server:{host:"127.0.0.1",port:0,strictPort:false},logLevel:"silent"})`; después de listen leyó el puerto real, hizo fetch de las doce rutas de la tabla y registró status, content-type y byteLength. `finally { await server.close(); }` cerró su listener. No creó archivos de producto.

Ajena identificada: revisión Rust fix2 registra check/fmt/Clippy y focales verdes; el orquestador verificó gate de 59 frontend y 128 entradas Rust, además de build exit 0. No se presentan como ejecución propia. Las 128 entradas incluyen helpers de proceso hijo, por lo que no equivalen a 128 comportamientos independientes. No se repitió audit ni gate al no cambiar dependencias ni surgir un riesgo que justificara hacerlo.

## Límites y siguiente paso

Sin bibliotecas personales, interfaz nativa, instalador o cambio de infraestructura. La QA exploratoria de fix1 no acredita fix2. MIME HTTP correcto, pruebas backend y build no prueban CSP efectiva, dibujo, WASM/fallback ejecutado o instalación en equipo limpio. Cerrar R2a con la corrección focal del harness y completar los hallazgos de las otras revisiones antes del gate de integración; SEC-T03-1, R2b y SEC-T03-3 no requieren reabrirse por este delta.
