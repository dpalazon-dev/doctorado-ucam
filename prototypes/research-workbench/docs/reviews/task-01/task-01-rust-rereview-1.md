# F5 — ADDRESSED; F6 — ADDRESSED; F7 — ADDRESSED

Re-review Rust acotada de T01 fix ronda 1. BASE `13f985f212b29008e699fbde1ee00d5a556dd408`, HEAD `892e4e63f4c1b8fef3de054e8f510a1e1926d95a`. Checkout `C:/Users/david/Projects/Research-Workbench/.worktrees/task-01-scaffold`. Autoridad: brief de fix, clarificación F6 vinculante, informe del autor y paquete `review-13f985f..892e4e6.diff`, leído por secciones. Se aplicó la plantilla instalada Superpowers `re-review-prompt.md`. No se reabrió la revisión general de código intacto, no se despacharon agentes ni se editó producto/Git.

## Finding Verdicts

- **F5 — salida prematura tras Busy del actor: ADDRESSED.** `src-tauri/src/lib.rs:23` previene ExitRequested y `:27` previene CloseRequested mientras `request_exit` no autoriza cierre. `src-tauri/src/desktop/lifecycle.rs:133` evita drenadores simultáneos; `:136` cierra admisión de mantenimiento, conserva clones de actor/logger/controlador y `:140` drena fuera del hilo de ventana. `:144` usa intentos acotados; Busy reintenta sin forzar terminación y `:153` inicia diagnóstico una sola vez. `:147` publica ready únicamente después de shutdown exitoso y `:148` solicita salida. `src-tauri/tests/lifecycle_close.rs:16` verifica integración DesktopState/controlador con un job bloqueado y otro admitido: `:59` no entrega salida, `:60` conserva lock, `:65` completa el segundo job y `:71` comprueba su cambio persistido antes de autorizar cierre. Esto corrige el defecto específico de timeout de la revisión anterior. N1 abajo es otro camino de fallo introducido por esta corrección.
- **F6 — esquema futuro sin lock ni escritura fuente, incluyendo WAL: ADDRESSED.** `src-tauri/src/adapters/sqlite/actor.rs:48` consulta existencia de **database**, por lo que un directorio existente vacío sigue inicializándose. `:53` sondea antes del lock; `:61` conserva la conexión/scratch de diagnóstico con lock=None y `:78` impone query_only. En esquema soportado `:63` adquiere exclusión de escritor, `:65` revalida conjunto fuente y `:69` vuelve a comprobar versión al abrir/recuperar original. `src-tauri/src/adapters/sqlite/schema_probe.rs:55` obtiene inventario/tamaños/SHA256 de DB y sidecars; `:56` detecta WAL/journal pendientes. En ese caso copia DB/WAL/journal a scratch externo RAII (`:62`, `:68`), compara bytes copiados y fuente (`:72`, `:77`) y reconstruye SHM allí. Sin pendientes abre URI escapada immutable (`:100`). `:108` consulta versión resuelta y `:115` comprueba estabilidad otra vez; un cambio produce Busy en `:48`. Pruebas reales: `src-tauri/tests/schema_diagnostic.rs:37` mantiene inventario completo sin lock; `:82` clasifica versión futura presente en WAL bajo ACL de solo lectura manteniendo todos los hashes (`:100`); `:105` recupera WAL soportado dejado por hijo sin Drop y conserva exclusión de segundo escritor. `src-tauri/src/adapters/sqlite/schema_probe.rs:122` verifica Busy tras cambio y eliminación del scratch. ACL limitada a fixture temporal y restaurada en Drop (`src-tauri/tests/schema_diagnostic.rs:52`, `:67`). No se ignora WAL ni se fuerza diagnóstico permanente para esquemas soportados.
- **F7 — diferencia entre domain omitido/null/string: ADDRESSED.** `src-tauri/src/transport/dto.rs:7` deserializa un campo nullable presente como `Some(Option<T>)`; `:1965` mantiene default para ausencia y `:1967` aplica el helper a domain. `src-tauri/tests/scaffold_contracts.rs:28` ejercita ausencia/null/texto; `:40` distingue None / Some(None) / Some(Some(...)) y `:41` comprueba JSON round-trip idéntico. Se conserva `domain?: string|null`; no se implementa updateConcept ni se modifica semántica v1.

## New Breakage in the Fix Diff

**N1 — Important: crear el hilo de drenaje puede provocar panic en el callback de ventana.** `src-tauri/src/desktop/lifecycle.rs:133` fija running=true y `:140` usa `std::thread::spawn`. Esa API provoca panic cuando el SO rechaza crear un hilo, un fallo recuperable; la documentación oficial de Rust **1.99.0** lo confirma y señala `Builder::spawn` como alternativa fallible. [Rust std::thread::spawn, Panics](https://doc.rust-lang.org/std/thread/fn.spawn.html#panics).

La llamada ocurre dentro del guard del evento (`src-tauri/src/lib.rs:23` / `:29`), **antes** de ejecutar prevent_exit/prevent_close. Si falla la creación, no llega al manejo AppError ni a la política de diagnóstico/reintento del worker; queda running=true sin drenador y el callback unwindea en vez de garantizar cierre impedido. No se afirma que este fallo haya ocurrido en las pruebas ni que los commits realizados se corrompan. Es un camino de error concreto nuevo en el cierre, cuya finalidad era conservar estado/jobs también ante fallos.

**Remedio focalizado:** usar creación de hilo fallible, manejar el error sin panic, mantener ready=false/salida impedida y restaurar running para que un nuevo intento pueda funcionar, conservando actor/estado/lock. Informar de modo seguro por el mecanismo existente. No hace falta un comando IPC ni contenedor nuevo. **Verificación pertinente:** fallo inyectado de creación de drenador seguido de intento exitoso; comprobar que el primer intento no panica ni autoriza salida ni libera el lock y que el segundo puede drenar. No se provocó agotamiento real de recursos del equipo.

Sin Critical ni otros nuevos Important/Minor confirmados en esta pasada acotada.

## Out-of-Scope Observations

Ninguna observación nueva de código intacto. F1–F4/F8 y la revisión general/TypeScript corresponden a sus revisores; la lectura de sus hunks de fix no amplía este dictamen a aprobación de esos ámbitos.

## Comprobaciones y límites

El mandato superior del rol fue «Run cargo check, cargo clippy -- -D warnings, cargo fmt --check, and cargo test — if any fail, stop and report», seguido de «Run git diff HEAD~1 -- '*.rs'». El ruling del coordinador permitió ese mínimo obligatorio pese a la prohibición ordinaria de repetir suites. Sobre este HEAD se ejecutó **una vez cada uno**, secuencialmente; no se repitieron comprobaciones correctas.

Desde el checkout revisado, tras `. ./scripts/development-env.ps1`, se llamó al ejecutable `C:/Users/david/.cargo/bin/cargo.exe`:

| Argumentos exactos Cargo | Resultado |
| --- | --- |
| `check --manifest-path src-tauri/Cargo.toml` | exit 0 |
| `clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` | exit 0, sin warnings |
| `fmt --manifest-path src-tauri/Cargo.toml --check` | exit 0 |
| `test --manifest-path src-tauri/Cargo.toml` | exit 0; 31 entradas integración + 1 unit |

De las 31 entradas de integración, `pending_wal_fixture_child` es helper sin acción en la ejecución ordinaria; no equivale a una verificación independiente de dominio. Su ejecución hija sí sirve a la prueba de recuperación WAL. Los tests de cierre, readonly/ACL/WAL, fuente modificada y tres estados de domain nombrados arriba pasaron.

`git diff HEAD~1 -- '*.rs'` terminó exit 0 por mandato del rol; la revisión y números de línea usan el paquete asignado BASE..HEAD. No hubo otro rastreo Git ni builds nativos adicionales.

Comprobación enfocada adicional de N1: se intentó consultar rustdoc local en la ruta del toolchain fijado, que no estaba instalada; se consultó exclusivamente la página oficial std::thread::spawn, que reporta Rust 1.99.0 y confirma el comportamiento de panic/error recuperable. No se ejecutaron nuevos tests/suites para generar este hallazgo.

**Cannot Verify:** no se observó en esta re-review el diálogo nativo Busy de cierre, ni se produjo fallo real de creación de hilo; el smoke normal declarado por el autor no equivale a esos escenarios. La prueba de WAL pendiente/ACL es real y pasó; no hay prueba de corte hardware ni de rollback journal pendiente en proceso hijo. El código incluye ese journal en el mismo sondeo, pero no se afirma evidencia empírica específica de ese escenario. NSIS, equipo limpio y release sin consola continúan fuera de esta ronda.

## Verdict

**Fix round: Findings remain open — N1 (Important).** F5, F6 y F7 originales están ADDRESSED. **Spec Compliance de esos tres fixes: compliant; Task Quality del fix Rust: Needs fixes**, por el nuevo fallo de creación del drenador. Una corrección limitada de N1 y su prueba de error/reintento basta para volver a revisar este punto; no se solicita una nueva revisión general.
