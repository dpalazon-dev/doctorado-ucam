# T03 — informe de corrección 2

Fecha: 2026-10-03 (UTC). Autor: `/root/task03_library_reader`.

## Identidad del corte

- Worktree: `C:\Users\david\Projects\Research-Workbench\.worktrees\task-03-reader`
- Rama: `agent/task-03-reader`
- BASE limpio: `b36ae5ae0529ba429edf7a360ade7319e353e8fe`
- HEAD de producto revisado y construido: `e0a882267d1e5f39bb12f6bf9124d8e33edd4dd8`
- Commit: `fix: resolve reader and import race conditions`
- `git diff --check`: exit 0. Sin cambios pendientes al crear este informe; el informe versionado se añade en un commit documental separado.

## Cambios

- **F2-1, posición de lectura:** un conflicto o resultado incierto detiene las posiciones ya encoladas. El reintento de una respuesta incierta reutiliza exactamente `requestId` y payload. Reader ofrece una resolución accesible: adoptar página/zoom durables o guardar explícitamente la intención local con un `requestId` nuevo y CAS sobre la revisión leída. Un segundo conflicto conserva el estado de conflicto. No se infiere éxito a partir de una revisión durable mayor ni se drenan guardados en cola como consentimiento.
- **F2-2, importación y apertura:** iniciar importación desde cualquiera de los accesos invalida la intención de navegación anterior; la resolución tardía de `Leer` no cierra, cancela ni suplanta el borrador/preview de importación.
- **F2-3, propiedad y ciclo de vida:** el test de Reader crea el `ProbeHandle` antes de la barrera, bloquea al actor, espera confirmación admitida, aborta y espera la terminación del caller, y verifica el propietario de `requestId`, mantenimiento y handle durante commit y rollback. La prueba Library–Reader sondea al competidor antes de liberar la saga. La prueba del protocolo común pausa el `read_all` real dentro de `spawn_blocking`, descarta el receptor y verifica que el permiso de mantenimiento continúa activo hasta que se intenta responder.
- **F2-4, recuperación en Configuración:** refresco del estado tras selección, confirmación y cancelación, incluso en error; consulta al entrar en Configuración; acción para volver a comprobar recuperación; número de secuencia para descartar respuestas antiguas.
- Se incorporan regresiones inline de duplicado tardío tras DOI, selección del segundo candidato durante cancelación pendiente/Escape, borrador conservado ante detalles inválidos y Escape bloqueado durante selección/confirmación.
- **MIME PDF.js:** el servidor Vite local conserva su allowlist y sirve fallbacks `.js` como `text/javascript` y `.wasm` como `application/wasm`. QuickJS continúa excluido.

No cambian contratos IPC, DTO, schema, puertos ni permisos. El único helper de protocolo nuevo es privado; `ReadGate` y su setter están bajo `cfg(test)`. No se añadieron dependencias de producto.

## Evidencia y comandos

Las salidas completas y códigos están en `C:\Users\david\Projects\Research-Workbench\.worktrees\task-03-reader\work\evidence\` (directorio de trabajo ignorado por Git):

- `task03-fix2-conflict-red.log/.exit` (exit 1): dos guardados ya encolados generaban tres escrituras pese al primer `Conflict`.
- `task03-fix2-open-import-red.log/.exit` (exit 1): los dos recorridos con lectura pendiente/preview permitían que la apertura antigua desplazara la importación.
- `task03-fix2-reader-resolution-red.log/.exit` (exit 1): faltaba la resolución explícita en Reader.
- `task03-fix2-recovery-red.log/.exit` (exit 1): Configuración no consultaba el estado actualizado tras un error de selección.
- `task03-fix2-front-green-final.log/.exit`: `npm.cmd test -- src/features/reader/reader.test.tsx src/features/reader/useReadingPosition.test.tsx src/features/library/library.test.tsx src/app/App.test.tsx src/shared/adapters/tauri/pdf-assets.test.ts` — exit 0, 5 archivos y 40 pruebas, 2.24 s. Incluye canvas con posición durable y servidor Vite real para el MIME.
- `task03-fix2-reader-drop-commit-rollback-green.log/.exit`: `cargo test --manifest-path src-tauri/Cargo.toml --test reader_integration dropped_reader_caller_keeps_handle_and_shared_request_owner_until_open -- --nocapture` — exit 0, 2 pruebas, 0.05 s.
- `task03-fix2-protocol-drop-green.log/.exit`: `cargo test --manifest-path src-tauri/Cargo.toml --lib dropped_protocol_receiver_keeps_read_operation_until_blocking_body_read_finishes -- --nocapture` — exit 0, 1 prueba, 0.04 s.
- `task03-fix2-npm-ci.log/.exit`: `npm.cmd ci` — exit 0; 156 paquetes añadidos, auditoría con 0 vulnerabilidades.
- `task03-fix2-gate-clean-ci.log/.exit`: `powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/check.ps1` — exit 0 después de `npm ci`. Typecheck, build frontend, formato Rust, Clippy, tests Rust y contratos sin drift. Frontend: 59/59. Cargo contó 128 entradas en 10 suites/targets de prueba; dos entradas son helpers de proceso hijo, así que no representan 128 comportamientos independientes. `library_integration` 55/55, `reader_integration` 17/17. La integración Reader incluye replay/reapertura en proceso nuevo, revisión bibliográfica intacta, asociación papel-documento, posición/CAS por documento y requestId compartido con Library.
- `task03-fix2-gate-final.log/.exit` conserva el intento anterior fallido por `clippy::manual_noop_waker`; corregido el helper de test a `Waker::noop()`, el gate posterior a la reinstalación pasó completo.
- `task03-fix2-tauri-debug-clean-ci.log/.exit`: `. .\scripts\development-env.ps1; npm.cmd run tauri:build -- --debug --no-bundle` — exit 0; Cargo informó `Finished dev profile` en 11.21 s.
- `git diff --check` — exit 0.

Build final: `C:\Users\david\Projects\Research-Workbench\work\cargo-target\debug\research-workbench.exe`, tamaño 25,866,752 bytes, SHA-256 `32C7DC86458F16D594EDFB5EABC5163EED05C015BC651418E6ACE2701C1D9CD7`. El inventario de `dist\pdfjs` contiene 199 archivos; no contiene QuickJS.

## Autorrevisión y límites

Se comprobó la cola tras conflicto, el uso de revisión CAS en la decisión explícita, el cambio visible de página/zoom al conservar estado durable, la asociación de la misma solicitud entre servicios y la retención de handle/permiso hasta commit o rollback/respuesta. Los casos de reapertura real en nuevo proceso y replay están cubiertos por las pruebas de integración existentes y pasaron el gate.

No se lanzó WebView2 ni se validaron dibujo PDF en una ventana real, selector nativo, instalación NSIS o equipo limpio. Las pruebas visuales de Reader usan mocks; la prueba del MIME sí inicia un servidor Vite local. El build debug sin bundle no acredita instalación ni ejecución interactiva. No se consultó ni escribió memoria OpenViking y no se abrió ninguna biblioteca personal; los documentos usados por los tests son sintéticos.
