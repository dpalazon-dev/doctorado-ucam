# T03 fix2 — revisión Rust focal de R2

Estado: **DONE_WITH_CONCERNS**. Veredicto: **BLOCK**, únicamente por una sincronización incorrecta del harness R2a. R2b y la carrera Library–Reader quedan cerrados. No se identifica un defecto nuevo de producto.

- BASE: `b36ae5ae0529ba429edf7a360ade7319e353e8fe`.
- HEAD: `30d532714d053c09d4021bc89bafbb0168b1602a`, árbol limpio.
- Worktree: `.worktrees/task-03-reader`.
- Revisor: `/root/task03_rust_review`.
- Fuentes: brief fix2, brief de revisión fix2, informe del autor, revisión Rust fix1 y delta completo de los tres archivos Rust modificados. No se reabren R1/R3, CAS, atomicidad ni recorrido integrado ya aceptados.

## Evidencia propia

Después de `. .\scripts\development-env.ps1` ejecuté:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo test --manifest-path src-tauri/Cargo.toml --test reader_integration dropped_reader_caller_keeps_handle_and_shared_request_owner_until_open
cargo test --manifest-path src-tauri/Cargo.toml --test reader_integration pending_library_filesystem_saga_owns_shared_request_id_before_reader_access
cargo test --manifest-path src-tauri/Cargo.toml --lib modules::reader::protocol::tests
```

Todos terminaron con exit 0: 2 pruebas commit/rollback, 1 carrera Library–Reader y 3 pruebas del protocolo. No se repitió el gate completo ni el build nativo; sus resultados 59 UI/128 entradas Rust/build 0 pertenecen a la evidencia del autor y comprobación del orquestador. `git diff --check BASE..HEAD` pasó y el árbol siguió limpio. Sin GUI, bibliotecas personales, cambios de producto o cambios en el ejecutable congelado.

## R2a — Important residual: «admitted» se señaliza antes de sondear el future real

Ubicación: `src-tauri/tests/reader_integration.rs:108–115`, función `ConfirmObservedPersistence::confirm_open(&self, command: OpenPaperCommand, verified: VerifiedDocument) -> ReaderFuture<'static, OpenPaperDto>`. El consumo de la señal está en `:445–451`.

El wrapper ejecuta `let result = self.inner.confirm_open(command, verified)`, envía `admitted` y devuelve `result`. Sin embargo, `SqliteReaderPersistence::confirm_open` crea un `Box::pin(async move { actor.submit(...).await })`: llamar al método no ejecuta `submit`; ocurre durante el primer poll del future devuelto.

Por ello el hilo del test puede recibir `admitted` y comprobar `handle_drops == 0` antes de que la confirmación haya llegado a la cola del actor. Es una señal de construcción del future, no de admisión. Una regresión que liberase el handle al comenzar ese future, antes del submit, todavía podría pasar: el assert intermedio ocurre antes de la liberación y el assert final solo exige un Drop tras idle, sin demostrar cuándo ocurrió. El actor sí está bloqueado y el caller sí se aborta y espera; el fallo restante es exclusivamente el punto en que se emite la señal.

**Corrección acotada solicitada:** conservar el wrapper de test, pero sondear el future real antes de notificar. Por ejemplo, devolver un future construido con `std::future::poll_fn` que delegue a `result.as_mut().poll(cx)` y emita la señal una sola vez después de observar el primer `Poll::Pending`. Con el actor bloqueado y este adaptador concreto, ese Pending sucede después de `DbActor.submit` y mientras espera su resultado. Comprobar entonces handle adquirido/no soltado, mantenimiento ocupado y competidor requestId pendiente; finalmente liberar el actor y conservar las comprobaciones de commit/rollback. No hacen falta cambios de producto, puertos ni una nueva API del actor.

El orquestador confirmó esta resolución en la revisión. La modificación debe volver al mismo autor. La próxima revisión puede limitarse al wrapper y sus dos tests; no es necesario repetir R2b ni los hallazgos ya cerrados.

## Cierres y consecuencias del delta

**Probe y cancelación efectiva, corregidos.** `GatedReaderAccess` ahora crea `ProbeHandle` antes de señalizar la adquisición y esperar (`reader_integration.rs:81–90`). El test verifica una adquisición, bloquea realmente el actor con un job y espera la terminación del caller abortado (`:424–443`). Los dos finales ejercitan commit y rollback real con trigger, liberación y resultado posterior del requestId. Esto resuelve la prueba vacua de fix1 salvo el aviso prematuro descrito en R2a.

**R2b, cerrado.** `ReadGate` y su setter existen únicamente bajo `cfg(test)` en `adapters/documents/store.rs`. La pausa está dentro del `spawn_blocking` real, con el `ManagedFile` adquirido y antes de `read_bounded`; no sustituye la lectura. `dropped_protocol_receiver_keeps_read_operation_until_blocking_body_read_finishes` (`modules/reader/protocol.rs:254`) espera esa entrada, descarta el receptor, comprueba mantenimiento ocupado y rechazo de mantenimiento exclusivo, libera la lectura y observa el intento terminal del responder y permiso liberado.

El helper privado `serve_pdf_read` (`protocol.rs:47`) se usa tanto en la ruta de `register` como en la prueba. Conserva lease/OperationPermit hasta `deliver_response`, cuya prueba ya aceptada verifica ambos vivos dentro del responder. Los bytes siguen leyéndose del mismo handle en el store; no se introduce un ejecutor alternativo, una nueva regla de autorización ni un puerto. La prueba representa receptor desaparecido, no abortar arbitrariamente la tarea propietaria interna, coherente con el callback de protocolo que permanece admitido aunque desaparezca fetch/UI.

**Library–Reader, cerrado.** El competidor ahora se sondea directamente con `poll_once` antes de liberar la saga (`reader_integration.rs:1162–1167`), en vez de depender de `yield_now`. La primera espera del Reader real es la adquisición del requestId compartido; el test comprueba Pending, cero acceso al segundo documento, Conflict final y único receipt de Library. No hay cambio del algoritmo de RequestRegistry.

**Seguridad y alcance preservados.** Los cambios del store de este delta son seams bajo `cfg(test)`; el protocolo solo extrae la lógica existente de respuesta. Se mantienen parser, origen/ventana/método, allowlist, longitud/límite y propiedad de la respuesta. No se ha observado modificación de DTO, esquema, capability ni autoridad filesystem por este delta Rust.

## Precisión de evidencia y condición de merge

El informe del autor llama «128 pruebas de comportamiento» al total Cargo; son 128 entradas, entre ellas los dos helpers de proceso hijo ya identificados. Es una corrección editorial de conteo, no un nuevo bloqueo de producto.

La revisión focal no acredita WebView2, picker, CSP efectiva, instalador ni equipo limpio. La integración requiere cerrar R2a, conservar las demás revisiones sin Important/Critical y ejecutar el gate pertinente sobre el merge preparado sin conflictos. No se hizo merge ni se emitió aprobación de integración en este corte.
