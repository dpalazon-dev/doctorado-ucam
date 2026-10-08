# T01 — corrección ronda 1

Fecha de cierre: 2026-10-02. Autor: `/root/task01_scaffold`. BASE de corrección: `13f985f212b29008e699fbde1ee00d5a556dd408`. Rama/worktree: `agent/task-01-scaffold`, `C:\Users\david\Projects\Research-Workbench\.worktrees\task-01-scaffold`. Estado del worker: **DONE_WITH_CONCERNS**, F1–F8 corregidos, listo para re-review; no implica aprobación ni merge. Commit de entrega comunicado al coordinador tras guardar este informe.

Autoridad: `task-01-fix-1-brief.md`, tres revisiones independientes y `task-01-fix-1-clarification.md` para F6. Se aplicó receiving-code-review de Superpowers verificando cada defecto antes de corregirlo; se mantuvo TDD focalizado, ownership y alcance. Sin subagentes, merges, remotos, configuración global, cambios normativos ni features T02+. UoW y contrato público v1 permanecen intactos.

## Resolución F1–F8

| Hallazgo | Cambio y rutas | Verificación |
|---|---|---|
| F1 | `scripts/check.ps1` propaga inmediatamente LASTEXITCODE del generador. `scripts/tests/check-exitcode.ps1` copia el gate real en un directorio temporal, sustituye únicamente herramientas externas por shims y ejecuta proceso padre PowerShell con generador exit37. | Antes: gate devolvía0 y prueba exit1. Después: padre devuelve37 y prueba exit0, sin ejecutar suite para inyección. |
| F2 | Puerto mínimo y valores internos en `application/settings.rs`: SettingsQuery, LibrarySummary, LibraryStatus, SettingsAppInfo. `adapters/sqlite/settings.rs` implementa puerto sobre actor+mantenimiento. `modules/settings.rs` recibe solo puerto/valores explícitos; transporte convierte a DTO/envelope. Composición concreta suministrada desde `lib.rs` a DesktopState.start. | `tests/settings_service.rs` prueba estado/capabilities/status mediante fake de puerto sin crear DB/actor. MockRuntime existente sigue pasando tras composición. |
| F3 | `transport/dto.rs`: PageDto.total y WorkflowGetPhaseDefinitionArgs.version usan deserialize de valor presente no-null y TS number opcional. Generación corregida. Se inspeccionaron otros opcionales (strings/enums/patch/UUID/duplicado); no se aplicó un cambio global que borrase nullables legítimos. | Rust acepta omisión/número, rechaza null; wire igual; `optional-types.test.ts` comprueba las declaraciones generadas con @ts-expect-error y typecheck. Generador/drift/exportcoverage pasan. |
| F4 | `wire.ts` elimina solo máximo por PDF de BackupDto.sizeBytes; conserva entero no negativo/finito. `backup-aggregate.json` contiene629145600 bytes. ImportPreview conserva500MiB. | Fixture backup>500MiB aceptada, PDF524288001 rechazado; no se implementa backup/importación. |
| F5 | `desktop/lifecycle.rs` añade ExitCoordinator y request_exit; `lib.rs` intercepta CloseRequested/ExitRequested. No autoriza ventana/proceso a cerrar hasta drenar actor; espera250ms por intento en worker, diagnóstico después20 intentos (5s), reintenta Busy conservando estado/lock. Otros errores mantienen salida impedida con diagnóstico y permiten nuevo intento; no fuerza terminación. `windows/startup.rs` muestra texto español de cierre pendiente fuera del hilo UI. | `tests/lifecycle_close.rs` ejecuta DesktopState con primer job bloqueado y segundo admitido: no callback de salida, lock ocupado, admisión mantenimiento cerrada; libera primero, segundo persiste, entonces callback/salida autorizados y lock libre. Smoke nativo vigente abre y cierra normalmente, exit0. No se mostró diagnóstico Busy durante ese smoke. |
| F6 | `adapters/sqlite/schema_probe.rs` clasifica versión sin crear lock fuente. Sin WAL/journal pendiente: immutable URI escapada. Con sidecars: copia DB/WAL/journal necesarios a scratch externo RAII, compara inventario/tamaños/SHA256 fuente antes/después y bytes copiados, deja SQLite resolver copia y consulta versión. SHM fuente se verifica pero se reconstruye en scratch. Futuro conserva conexión de diagnóstico query_only/scratch en hilo DB; no lock ni writes fuente. Soportado adquiere lock normal, revalida conjunto y versión, abre/recover original. Fuente cambiada devuelve Busy; errores de lectura son diagnóstico explícito, nunca se ignora WAL. | Inventario completo de raíz futura sin lock idéntico; futuro99 no checkpointed en WAL y raíz bajo ACL real solo lectura abre y mantiene hashes; WAL soportado dejado por hijo que sale sin Drop se recupera con exclusión de segundo writer. Unit comprueba fuente cambiada Busy y eliminación de scratch. ACL solo fixture creada bajo temp, restaurada por RAII. |
| F7 | `ConceptUpdateArgs.domain` deserialize de campo presente devuelve Some(Option<T>), con default para ausencia. | Round-trip Rust ausencia=None/null=Some(None)/texto=Some(Some), JSON idéntico; TS domain?:string|null conservado. |
| F8 | `client.ts` separa excepción de invoke de validación de respuesta recibida. Respuesta inválida/requestId incorrecto => IntegrityFailure no reintentable. Excepción transporte => StorageUnavailable seguro. Envelopes failure válidos recibidos/lanzados mantienen código/retryable. | Simulación de versión/payload inválidos, ausencia de contenido privado en error, transporte y failure Busy válido. |

## Ownership concreto y firmas

Se editaron únicamente fuentes T01 asignadas: check; Cargo.toml; actor/SQLite mod/settings; adaptador Windows startup; application mod; lifecycle/lib; módulo y comandos Settings; dto y DTO generado; bootstrap/scaffold tests; client/tests y wire/tests; informe versionado y THIRD_PARTY_NOTICES (promoción de tempfile). Nuevos paths de ownership ampliado: `src-tauri/src/application/settings.rs`, `src-tauri/src/adapters/sqlite/schema_probe.rs`, `src-tauri/tests/{settings_service,lifecycle_close,schema_diagnostic}.rs`, `scripts/tests/check-exitcode.ps1`, `src/shared/contracts/optional-types.test.ts`, `contracts/fixtures/backup-aggregate.json`.

`tempfile` existente pasa de dev a runtime, autorizado explícitamente en ruling F6. No cambia versión/resolución ni Cargo.lock, ni añade paquete nuevo; inventario de licencia ya lo incluye. Scratch externo contiene DB/sidecars exclusivamente, no PDFs ni backup público.

Firmas actor/UoW/receipts permanecen congeladas. SettingsQuery: `library_info(&self)->LibrarySummary`; `library_status(&self)->Pin<Box<dyn Future<Output=Result<LibraryStatus,AppError>>+Send+'_>>`. `DesktopState::start` recibe factory `(DbActor,MaintenanceCoordinator)->Arc<dyn SettingsQuery>` suministrada por composición; logger pasa a Arc y state incluye ExitCoordinator. `DesktopState::request_exit(&self,on_ready:impl FnOnce()+Send+'static,on_waiting:impl Fn(AppError)+Send+'static)->bool`; false obliga a impedir evento. ExitCoordinator default250ms/20 intentos. `shutdown()` síncrono existente sigue devolviendo Result para pruebas/callers explícitos; event loop usa request_exit.

## Comandos exactos y resultados

Cwd de producto: worktree indicado. Cargo se ejecutó tras `. ./scripts/development-env.ps1` (entorno solo de proceso y cache `C:\Users\david\Projects\Research-Workbench\work\cargo-target`). Evidencia bajo `work/` de ese worktree, ignorada por Git. Códigos aquí son del hijo real, no del wrapper que continuó para recopilar resultados.

| Comando | Red / green / resultado final | Logs |
|---|---|---|
| `npm test` | red1 (backup/integrity), green0 | fix1-wire-red.log, fix1-wire-green.log |
| `cargo test --manifest-path src-tauri/Cargo.toml --test scaffold_contracts` | red101 (dos comportamientos), green0 | fix1-dto-red.log, fix1-dto-green.log |
| `powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/tests/check-exitcode.ps1` | red1 observado gate0, green0 observado gate37; final0 | fix1-script-red.log, fix1-script-green.log, fix1-script-delivery.log |
| `cargo test --manifest-path src-tauri/Cargo.toml --test settings_service` | red101 (puerto ausente), green0 | fix1-settings-red.log, fix1-settings-green.log |
| `cargo test --manifest-path src-tauri/Cargo.toml --test lifecycle_close` | red101 (coordinador/método ausente), green0 | fix1-close-red.log, fix1-close-green.log |
| `cargo test --manifest-path src-tauri/Cargo.toml --test schema_diagnostic` | red101 (inventario mutado/ACL), green0 inicial3 casos | fix1-schema-red.log, fix1-schema-green.log |
| mismo con `future_wal_version_is_classified_without_readonly_root_writes` | red101 StorageUnavailable bajo readonly; cubierto finalgreen | fix1-schema-readonly-red.log |
| `scripts/generate-contracts.ps1` | 0 | fix1-generate.log |
| `cargo fmt --manifest-path src-tauri/Cargo.toml` | 0 | salida de sesión, formato antes de gate |
| `powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/check.ps1` primer final | 101, clippy collapsible_match en cierre; UI14 verdes | fix1-final-check.log |
| `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` tras corrección guard | 0 | fix1-clippy-green.log |
| `cargo test --manifest-path src-tauri/Cargo.toml` tras corrección | 0 | fix1-rust-final.log |
| `scripts/generate-contracts.ps1 -Check` | 0 | fix1-generation-final.log |
| `powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/check.ps1` gate completo final | **0** | fix1-final-check-green.log |
| `npm run tauri:build -- --debug --no-bundle` después del gate final | **0** | fix1-native-build.log |
| `powershell.exe -NoProfile -ExecutionPolicy Bypass -File work/fix1-native-smoke.ps1` | **0**, ventana vigente capturada | fix1-native-smoke.log |
| cierre normal de PID6524 mediante CloseMainWindow/WaitForExit(10000) | **0**, true/true/exitCode0 | fix1-native-close.log |
| `git diff --check` / staging/estado | 0; hash y árbol limpio en devolución | salida de sesión |

El primer test ACL usaba deny genérico W,D que en Windows impedía lectura por Synchronize; el fixture se corrigió a derechos avanzados de escritura/delete sin negar lectura. Se observó luego el fallo de producto StorageUnavailable antes de corregir F6. Todos los cambios ACL fueron solo sobre fixtures sintéticas, con restauración en Drop. El primer red de inventario imprimió bytes y generó log grande; los asserts actuales comparan hashes compactos. No se interpreta ese fallo de fixture como prueba de producto.

Gate final: npm typecheck/test/build, cargo fmt --check/clippy all-targets -Dwarnings/test y generador Check, todos0. UI **14** en4 archivos. Rust **32 entradas**:31 integración y1unit; una entrada integración `pending_wal_fixture_child` es helper sin acción en ejecución ordinaria, por lo que no se afirma que sean32 verificaciones de dominio. Las30 verificaciones restantes de integración y unit1 son comportamiento; el helper sirve al caso de WAL tras salida sin Drop. Zero unit de binarios/doctests no se contabilizan. PDF worker local empaquetado2,228.48KB y JS386.66KB.

## Smoke nativo vigente y límites

Ejecutable debug construido: `C:\Users\david\Projects\Research-Workbench\work\cargo-target\debug\research-workbench.exe`. Abierto desde work, Start-Process -WindowStyle Hidden para consola, main visible. Biblioteca sintética reutilizada `work/native-smoke/library`; identidad conservada `304dd2bf-4ff9-4ee5-ad5a-77fc61143cbf`. PID6524, título Research Workbench, handle3148074. Captura únicamente esa ventana con DPI correcto: `work/fix1-native-window.png`, inspeccionada por worker. Se observó versión0.0.1, schema1, Disponible, diálogo de copia administrada y navegación futura deshabilitada, alimentados por Settings IPC real. Se solicitó cierre normal: CloseMainWindow true, WaitForExit true, exitCode0 y eventos started/stopped.

UI tests son invoke simulado y paridad wire, no interacción React completa. MockRuntime prueba autoridad real del framework, sin GUI. Rust/ACL usan fixtures locales reales; cierre con jobs bloqueados es integración de DesktopState/controlador, no una sesión gráfica de escritura. La ventana vigente demuestra composición Settings y cierre ordinario; no se mostró el diálogo de cierre con Busy ni se inyectó un job bloqueado en esa GUI. No instalación NSIS/VM limpia/offline/uninstall/reinstall/release sin consola, importador500MiB ni funcionalidad T02+; pendientes de sus tareas. El sondeo copia coherente comprobada de DB/sidecars, no sustituye SnapshotService ni demuestra resistencia a fallos de hardware.

Autorrevisión: diff acotado F1–F8, no grants/comandos nuevos, contratos canónicos corregidos sin cambiar semántica, UoW intacto, fuente futura sin mutación y WAL no ignorado, cierre sin abandono de trabajos admitidos ni wait5s en UI. Inventarios/logs/pantalla solo sintéticos; dependencias congeladas salvo promoción autorizada existente. Re-review e integración quedan al coordinador.
