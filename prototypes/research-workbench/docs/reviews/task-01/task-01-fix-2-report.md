# T01 — corrección ronda 2 / N1

Fecha: 2026-10-02. Worker `/root/task01_scaffold`. BASE `892e4e63f4c1b8fef3de054e8f510a1e1926d95a`. Rama `agent/task-01-scaffold`, worktree `C:\Users\david\Projects\Research-Workbench\.worktrees\task-01-scaffold`. Estado: **DONE**, corrección N1 lista para re-review; no equivale a aprobación o merge. Hash de entrega en devolución al coordinador.

Autoridad: `task-01-fix-2-brief.md` y `task-01-rust-rereview-1.md`. F1–F8 ya están ADDRESSED; no se reabren. Se continuó recepción de review/TDD de Superpowers. Sin subagentes, paquetes/configuración global, contratos nuevos, merges ni funcionalidades futuras.

## Cambio focalizado

Único archivo de producto cambiado: `src-tauri/src/desktop/lifecycle.rs`. `DesktopState::request_exit` crea el drenador mediante `std::thread::Builder::spawn`, que devuelve Result. Si falla: retorna false sin unwind, mantiene ready=false/estado/actor/lock, restaura running=false para permitir reintento, registra evento cerrado shutdown_busy y entrega AppError StorageUnavailable al callback de diagnóstico existente. No serializa el error del SO ni rutas/contenido privado.

Helper **privado** `request_exit_with_spawn` permite inyectar el resultado de creación en unit tests. El camino de producción siempre usa Builder; no hay flags, variables de entorno o configuración de fallo. El callback de diagnóstico queda compartido mediante Arc<Mutex<F>> para disponer de él tanto en el hilo creado como ante rechazo de creación, preservando el bound Send público sin exigir Sync. No cambia firma pública, política250ms/20 intentos, `lib.rs`, IPC, admisión ni comportamiento observable de cierre exitoso.

La prueba unitaria local `failed_drainer_creation_preserves_exit_lock_and_allows_retry` crea biblioteca sintética, bloquea un trabajo y admite otro. El primer intento inyecta error sin agotar recursos reales del SO; catch_unwind confirma ausencia de panic, devuelve false, no anuncia salida, conserva lock, ready=false/running=false y entrega diagnóstico seguro. El segundo intento utiliza el Builder real: no autoriza salida ni libera lock mientras el primer trabajo está bloqueado; al liberarlo, drena el segundo, comprueba su cambio persistido y autoriza salida/libera lock. El log no contiene el texto sintético del error OS. La prueba de integración existente de DesktopState/cierre sigue pasando.

Ownership: lifecycle.rs y su unit module privado; `docs/reports/task-01-report.md` para esta sección de evidencia. `src-tauri/tests/lifecycle_close.rs` se ejecutó sin modificarlo. Ningún otro archivo de producto cambiado; firmas actor/UoW/Settings/request_exit permanecen congeladas.

## Evidencia y comandos

Cwd: worktree indicado. Antes de Cargo: `. ./scripts/development-env.ps1`, entorno de proceso/cache común existentes. Logs bajo `work/` del worktree, no versionados; no bibliotecas personales.

| Comando | Resultado real | Evidencia |
|---|---|---|
| `cargo test --manifest-path src-tauri/Cargo.toml --lib failed_drainer_creation_preserves_exit_lock_and_allows_retry` antes de corrección | 101, red: panic en creación inyectada | fix2-spawn-red.log |
| mismo tras manejo fallible | 0, 1test | fix2-spawn-green.log |
| `cargo fmt --manifest-path src-tauri/Cargo.toml` | 0 | salida de sesión |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | 0 | fix2-format.log |
| `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` inicial | 101: io_other_error en constructor del error de fixture | fix2-clippy.log |
| mismo tras usar Error::other en fixture | **0** | fix2-clippy-final.log |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --check` final | **0** | fix2-format-final.log |
| `cargo test --manifest-path src-tauri/Cargo.toml --lib desktop::lifecycle::tests` final | **0**, 1test, otra unit filtrada | fix2-unit-final.log |
| `cargo test --manifest-path src-tauri/Cargo.toml --test lifecycle_close` final | **0**, 1test | fix2-lifecycle-final.log |
| `git diff --check` y staging | 0 | salida de sesión |

Para el red se introdujo primero el seam privado conservando la semántica previa de panic ante creación fallida; la inyección produjo ese panic y la aserción de no-unwind falló. No se afirma que el SO haya rechazado un hilo real ni se provocó agotamiento. La corrección posterior usa el mismo seam con manejo Result sin panic; el segundo intento de la prueba sí crea/drena un hilo real sobre SQLite sintético.

Autorrevisión: solo N1 y prueba local; salida impedida ante error, reintento funcional, lock/estado retenidos, sin nueva firma compartida ni grants/comandos. No se repitieron tests UI, sondeo SQLite ni smoke gráfico normal, conforme al brief. El gate completo y build nativo del nuevo HEAD corresponden a integración; el smoke nativo previo demuestra892e4e6, no se presenta como observado en este HEAD. No se observó el diálogo Win32 bajo fallo real de recursos ni se probó NSIS/VM limpia. Esos límites no sustituyen la prueba determinista de N1 ni implican trabajo de instalación realizado.
