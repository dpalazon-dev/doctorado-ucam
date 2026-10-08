# T03 fix1 — revisión independiente Rust del delta

Estado: **DONE_WITH_CONCERNS**. Veredicto: **BLOCK**. R1 y R3 cerrados; R2 sigue parcialmente abierto por las pruebas de propiedad durante cancelación/actor/lectura. No se identifica un nuevo defecto de producto en este delta.

- BASE del delta: `870b3321c4e3d46597848ae5d3faf40395229621`.
- HEAD revisado: `b36ae5ae0529ba429edf7a360ade7319e353e8fe`; árbol limpio.
- BASE original de T03: `cb60ee2d375aafc60dd2f8a5d696f1f375321cff`.
- Revisor: `/root/task03_rust_review`.
- Worktree: `.worktrees/task-03-reader`.
- Alcance: los ocho archivos Rust del delta y consecuencias respecto a R1/R2/R3. Se leyeron el brief fix1, informe del autor y revisión Rust original. No se reaudita T02 ni se modifica producto.

## Comprobaciones

Después de cargar `. .\scripts\development-env.ps1`, ejecuté:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo test --manifest-path src-tauri/Cargo.toml --lib --test reader_integration --test document_protocol --test settings_service
```

Todos terminaron con exit 0. La ejecución focal reunió 37 entradas: 16 unitarias, 16 Reader —incluido el helper de proceso hijo—, 4 protocolo y 1 Settings. No repetí la suite completa ni el build nativo. Los `.exit` del gate completo y build fix1 entregados son 0, coherentes con la validación del orquestador. `git diff --check BASE..HEAD` pasó y el árbol permanece limpio. No se lanzó GUI ni se escribió el ejecutable congelado para QA.

## Cierre de hallazgos

### R1 — cerrado

`src-tauri/src/adapters/sqlite/reader_repository.rs:145–149` valida que el paper del receipt sea el solicitado, vuelve a resolver el documento activo del paper solicitado y comprueba documento/asociación. `:169–176` vuelve a contrastar paper/documento durable, paper actual, biblioteca y snapshot del handle antes de devolver el resultado. La ruta replay conserva el receipt original y no ejecuta el helper que produce actividad.

Las pruebas `replay_rejects_document_reassigned_to_another_paper_without_changing_receipt_or_audit` (`reader_integration.rs:570`) y `confirm_open_rechecks_paper_document_association_after_file_verification` (`:656`) reasignan D de A a B en DB y comprueban rechazo y conservación del receipt/auditoría; la primera también conserva actividad. Ambas pasaron. La comparación posterior ocurre en una TX breve separada, dentro del mismo job actor; no hay I/O de PDF en esa TX ni intercalado de otro job.

### R3 — cerrado

El delta elimina `VerifiedFileSnapshot` y el import `PathBuf` de `application/reader_ports.rs`; no introduce una autoridad alternativa basada en rutas ni modifica los puertos públicos aprobados.

### R2 — Important, parcialmente abierto

**R2a: el contador nuevo no prueba retención del handle dentro del actor.** Ubicaciones: `src-tauri/tests/reader_integration.rs:72–79`, `:350–355`, `:371–378`.

`GatedReaderAccess` adquiere el handle real, avisa y espera la liberación; solo **después** construye `ProbeHandle`. El test cancela el caller mientras esa espera sigue cerrada y observa `handle_drops == 0`, cuando el objeto cuyo Drop incrementa el contador aún no existe. Posteriormente libera la espera y comprueba un único Drop después de quedar idle. Eso admite tanto liberar correctamente tras commit como liberar indebidamente antes del commit: no existe una barrera en la cola/job de confirmación que permita observar el intervalo relevante. El texto del assert final atribuye al contador una garantía de orden que no mide.

La fuente del adaptador sigue capturando correctamente `verified` por valor en `DbActor.submit`; no se afirma un fallo actual de retención. Sigue pendiente la regresión exigida expresamente en R2 y en fix1: bloquear el actor después de preparar/verificar, admitir la confirmación, descartar su receptor y comprobar que el handle real sigue protegido hasta que el job termina, tanto éxito como error/rollback. El probe debe estar instalado antes de señalizar su adquisición y la prueba debe sincronizar admisión/cancelación real, no asumirla por llamar a `abort()`. Conservar asimismo la comprobación de mantenimiento y requestId del trabajo propietario de servicio.

**R2b: no se ejercita caller descartado durante lectura del cuerpo.** Ubicaciones: `src-tauri/tests/document_protocol.rs:141–162`; `src-tauri/src/modules/reader/protocol.rs:218–241`.

El test de integración nuevo espera `service.read_document` hasta completarse, comprueba bytes/permiso y luego hace Drop. El test unitario HTTP entrega inmediatamente bytes ya disponibles a un callback síncrono con un DropProbe. Son pruebas útiles y pasan, pero ninguna pausa `read_all` o su trabajo bloqueante ni hace desaparecer el caller/fetch mientras ese trabajo sigue activo. El único `abort()` en los tests Reader está en el caso de apertura anterior. Falta demostrar el punto seguro de cierre durante esa lectura admitida y hasta su entrega terminal. Usar una barrera determinista en la ruta real compartida por el protocolo; comprobar permiso ocupado, lease viva y que el cierre/mantenimiento no se adelanta cuando desaparece el receptor. No se requiere GUI ni cambiar el protocolo/IPC para ello.

**Precisión de la carrera Library–Reader.** La nueva prueba `pending_library_filesystem_saga_owns_shared_request_id_before_reader_access` (`reader_integration.rs:992`) pausa correctamente después de la promoción real, comparte registry y comprueba el receipt final de Library y cero accesos Reader. Conviene sincronizar que la petición competidora ha sido realmente sondeada antes de soltar la saga: el único `yield_now()` no garantiza que esa tarea haya alcanzado `acquire`. Sin esa barrera la ejecución puede convertirse en dos peticiones secuenciales y aun así pasar. Esta observación pertenece a la misma exigencia de pruebas deterministas de R2, no es un defecto adicional de la implementación del registry.

Para cerrar R2 basta completar estas fronteras concretas y ajustar el informe del autor: su afirmación de propiedad comprobada «en el actor pese a cancelar al caller» excede lo que mide el test actual. No se solicita rehacer pruebas ya cubiertas ni fabricar rojos retrospectivos.

## Cobertura de R2 que sí mejora y se acepta

- El caso Library→Reader usa promoción de filesystem real y verifica que Reader termina en Conflict sin acceso al segundo documento ni robo del receipt; queda la precisión de sincronización señalada arriba.
- `open_failure_rolls_back_activity_session_audit_and_receipt_together` inyecta un trigger que falla en la auditoría y comprueba actividad/sesión/auditoría/receipt sin efecto. Junto con la inspección del helper y `with_receipt`, acredita la composición transaccional de esta ruta.
- `position_cas_is_per_document_idempotent_and_rejects_incompatible_receipt_replay` prueba dos documentos, conflicto con currentRevision, replay compatible y payload incompatible. La prueba numérica vacua se sustituyó por llamadas a la validación real que usa el helper, incluidos NaN/infinito y extremos permitidos.
- El nuevo recorrido importa un PDF sintético válido de dos páginas mediante LibraryService y store reales, mueve la fuente, abre/guarda con Reader, reabre en proceso hijo y finalmente archiva/restaura verificando IDs, hash registrado y posición. El selector es controlado; no se atribuye a esta prueba un picker gráfico ni render.
- `read_document` ahora se ejecuta contra store real y devuelve los bytes de la fixture con un permiso retenido. La lectura por el mismo `ManagedFile` se conserva en fuente.
- `full_pdf_response` y `deliver_response` son los helpers usados por `register`; la prueba mide 200, bytes, Content-Length, Content-Type, ausencia de Accept-Ranges y lease/permiso vivos dentro del responder. No hay copia alternativa de la lógica HTTP exclusiva del test. El hueco remanente es la lectura asíncrona pendiente ante caller descartado.

## Límites y condición de integración

Este informe revisa el delta y no acredita WebView2, CSP efectiva, picker, instalador ni equipo limpio. La copia de QA la controla el orquestador. No aparecen conflictos de merge en el corte limpio revisado; se mantiene la exigencia de gate completo verde sobre el merge preparado y cero Critical/Important abiertos. R2 permanece abierto por evidencia de concurrencia todavía insuficiente; R1/R3 no necesitan otra corrección de producto.
