# T03 — revisión independiente Rust

Estado: **DONE_WITH_CONCERNS**. Veredicto: **BLOCK**, por dos hallazgos Important abiertos. No se modificó código de producto ni se hizo merge.

- BASE: `cb60ee2d375aafc60dd2f8a5d696f1f375321cff`.
- HEAD revisado: `870b3321c4e3d46597848ae5d3faf40395229621`, árbol limpio; producto `ccc1f796f1b8af2ecaa8838f9c357c19949ae444`.
- Worktree: `.worktrees/task-03-reader`.
- Revisor: `/root/task03_rust_review`.
- Alcance: los 24 archivos Rust del diff completo BASE..HEAD y las dependencias inmediatas de actor/receipts/guardas/mantenimiento. Se leyeron INTENT, STATUS, briefs T03, TASK03_PORTS y TASK03_BOUNDARIES, además del contrato Reader y esquema 0001. Se aplicó `rust-patterns`. T02 se examinó solo para comprobar las interacciones nuevas.

## Validación ejecutada

Desde el worktree, en una misma shell PowerShell, se cargó `. .\scripts\development-env.ps1` y se ejecutó secuencialmente, deteniéndose ante fallo:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo test --manifest-path src-tauri/Cargo.toml
```

Los cuatro comandos terminaron con código 0. Cargo: 118 entradas de test, ninguna fallida. Dos son helpers de proceso hijo; además, `position_input_rejects_non_finite_or_out_of_range_zoom` solo comprueba propiedades de números y no comportamiento del producto, por lo que no deben presentarse las otras 116 como 116 pruebas de comportamiento del producto.

Se inspeccionó `git diff BASE..HEAD -- '*.rs'` en grupos de archivos y se comprobó `git diff --check BASE..HEAD` sin salida. Los logs ya entregados `work/evidence/task03-gate-verified-final.log/.exit` y `task03-tauri-debug-final.log/.exit` confirman gate y build nativo con exit 0. Esta revisión reejecutó los comandos Rust anteriores, no el build nativo ni pruebas UI. No se lanzó GUI ni se abrió ninguna biblioteca personal.

No existe `.github` en este corte. La aceptación sigue sujeta al gate completo del proyecto sobre el merge preparado, árbol limpio, conflictos resueltos y cierre de hallazgos Important/Critical; los checks locales no equivalen a validar un merge futuro.

## Hallazgos

### R1 — Important: el replay no revalida la asociación con el paper solicitado

Ubicación: `src-tauri/src/adapters/sqlite/reader_repository.rs:145` y `:165–169`.

`prepare_open` recupera el documentId del receipt, pero acepta cualquier registro actual que `registered_document` encuentre para ese documento. El SQL comprueba que el documento está activo para **su paper actual**, sin comprobar que este paper sea `command.paper_id`. En replay `with_receipt` omite `open_paper_in_tx`, precisamente donde sí existe la comparación con el paper solicitado. La comprobación posterior compara `current` únicamente con el snapshot recién obtenido y con la biblioteca; tampoco lo compara con la asociación paper-document del comando/receipt.

Escenario concreto: abrir A/D y conservar su receipt; cambiar la asociación durable de D a B y los punteros activos correspondientes, conservando D como ACTIVE y su archivo administrado. Este estado puede respetar las FKs y el índice de documento activo. Al repetir el requestId original para A, `prepare_open` entrega D/B; el store lo verifica; `confirm_open` considera iguales D/B actual y D/B verificado y devuelve el receipt original A/D con URL válida para D, aunque D ya no sea documento de A. Una apertura nueva de A no seguiría esa ruta. El contrato exige revalidar asociación actual también en replay y no convertir un receipt anterior en autoridad de acceso.

Corrección: validar explícitamente la asociación actual con el paper solicitado y el documento del resultado durable tanto antes de abrir el archivo como al confirmar, conservando intacto el receipt ante rechazo. Añadir una regresión con asociación DB alterada y verificar ausencia de nueva actividad/auditoría y que el receipt original permanece intacto. La evidencia aquí es inspección de las dos rutas y del esquema; no se ejecutó una mutación de DB para reproducirlo, respetando el encargo de revisión sin editar.

### R2 — Important: faltan regresiones de aceptación para las fronteras críticas nuevas

Ubicaciones principales: `src-tauri/tests/reader_integration.rs:91`, `:132`, `:185`, `:270`, `:386`; `src-tauri/tests/document_protocol.rs:50`; `src-tauri/src/application/reader.rs:103`.

Las pruebas pasan, pero no cubren varias condiciones expresamente exigidas por TASK03_PORTS/BOUNDARIES y el brief. Es un hueco de aceptación demostrado por el contenido de la suite, no prueba de que todas esas rutas fallen:

1. La carrera implementada pausa **Reader** en `open_verified` y compite con `Library.archive_paper`. No pausa una saga **Library** con efectos de filesystem para lanzar Reader con el mismo requestId. Falta demostrar que Reader no toca el segundo recurso ni se apropia del receipt mientras la saga está pendiente.
2. El test de caller descartado demuestra que la tarea Reader sobrevive durante `open_verified`; no bloquea el actor en confirmación ni comprueba que el handle permanezca adquirido hasta commit/rollback. Faltan también caller descartado durante lectura de cuerpo y permiso hasta responder.
3. Ningún test llama a `read_document` ni `read_all`; `document_protocol.rs` solo prueba parser, registro desconocido y junction. Por tanto no se ejecuta la nueva lectura por el mismo handle, ni se comprueba el cuerpo 200, longitud exacta y ausencia de Range anunciado. La observación WebView2 puede seguir pendiente; estas comprobaciones backend son aceptación de T03 separada.
4. El test denominado apertura transaccional solo observa éxito, último abierto y ausencia de posición. No inyecta fallo para demostrar rollback conjunto de actividad, sesión, auditoría y receipt.
5. El test de posición usa un único documento y un único save con expectedRevision 0. No prueba conflicto obsoleto/currentRevision, aislamiento entre dos documentos, replay de save ni payload incompatible. El test unitario de zoom no llama al helper: seguiría verde eliminando toda la validación del producto.
6. El replay comprueba igualdad del resultado y no reescritura de `last_opened_at`, luego elimina el archivo y llama únicamente al store. No vuelve a comprobar el receipt/auditoría después del rechazo ni prueba un registro cambiado, que es la frontera de R1.
7. La reapertura sí usa un proceso hijo real, pero parte de Paper/Document insertados por SQL con bytes sintéticos que no constituyen un PDF válido. No enlaza importación T02 → mover original → apertura/guardado Reader → nuevo proceso → archive/restore conservando IDs/hash/posición. Las pruebas separadas existentes de T02 y T03 no acreditan ese recorrido.

Corrección: añadir pruebas de comportamiento focales para estas fronteras usando bibliotecas temporales, barreras deterministas e inyección de fallo, sin GUI ni datos personales. Mantener explícita la separación respecto a QA nativa y corregir el recuento/descripción de evidencia. No se pide un RED retrospectivo ficticio.

### R3 — Minor: tipo de snapshot con ruta física sin consumidor

Ubicación: `src-tauri/src/application/reader_ports.rs:90–94`.

`VerifiedFileSnapshot { path: PathBuf, size_bytes }` es público, no está en los puertos aprobados y no tiene usos en el árbol. Añade un segundo concepto de verificación basado en ruta, mientras el contrato aprobado usa propiedad del handle. No se ha encontrado una exposición IPC de esa ruta ni una vulnerabilidad derivada; conviene eliminar el tipo y su import para conservar una frontera clara y evitar que futuros consumidores lo interpreten como prueba de acceso.

## Aspectos comprobados sin hallazgos adicionales

- Composición de una sola instancia de RequestRegistry compartida por Library/Reader; constructor Library exige inyección. Extracción del algoritmo de weak/pruning sin cambio sustancial y mutex del mapa liberado antes de esperar el mutex async.
- Mutaciones Reader admitidas en tareas propietarias independientes del receptor IPC, con permisos request/mantenimiento. El job de `confirm_open` captura el handle por valor dentro de `DbActor.submit`; no hace I/O de PDF bajo transacción.
- Apertura nueva usa un helper Library bajo la misma transacción que sesión, auditoría y receipt. La actualización SQL solo cambia `last_opened_at`; conserva revisión bibliográfica, updatedAt y lifecycle.
- CAS de posición con incremento comprobado, INSERT inicial y tratamiento explícito de fila existente revision 0. Default determinista sin INSERT. SQL parametrizado.
- Protocolo restringe ventana, origen, método, esquema/authority y UUID canónico antes de acceder a persistencia. El job creado por el callback conserva lease y permiso hasta después de `responder.respond` en éxito. El store lee del mismo `ManagedFile`, con longitud antes/después y límite de 500 MiB.
- Los `unwrap()` nuevos del constructor HTTP operan sobre valores constantes o un origen ya reducido a dos literales válidos; no se ha identificado una entrada aceptada capaz de provocar ese panic. Los envíos oneshot ignorados corresponden al receptor IPC descartable después del resultado terminal. No se tratan mecánicamente como errores críticos sin escenario.

## Límites y autorrevisión

No se ha acreditado render real, CSP efectiva en WebView2, selector, instalador ni equipo limpio. Tampoco se repitió el diagnóstico de cierre nativo T02 que pertenece a otro agente. Los hallazgos distinguen fallo de lógica (R1), falta de evidencia exigida (R2) y limpieza menor (R3). La revisión no reescribe contratos ni autoriza integración: R1 y R2 deben resolverse o recibir una resolución explícita del orquestador antes del merge.
