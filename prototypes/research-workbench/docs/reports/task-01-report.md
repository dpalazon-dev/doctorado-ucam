# T01 — base ejecutable, persistencia y contratos

Fecha: 2026-10-01. Worker: `/root/task01_scaffold` (Sol). Rama: `agent/task-01-scaffold`. Base: `dd31de021d9d1ee60c0dd3d60c6347525d0e5903`. Worktree: `C:\Users\david\Projects\Research-Workbench\.worktrees\task-01-scaffold`.

Estado: **DONE_WITH_CONCERNS**, listo para revisión independiente e integración del coordinador. Sin bloqueo de compilación ni tests. Los pendientes de distribución y futuros módulos se describen abajo; no se han habilitado como funcionalidades reales.

## Resultado y autoridad

T01 entrega una aplicación Tauri 2 con React/TypeScript strict, ventana Home/Settings, confirmación de biblioteca local, SQLite gestionado por actor, migraciones y snapshots, contratos completos v1 y adaptador tipado. Solo `settings_get_app_info`, `settings_get_library_info` y `settings_get_library_status` están implementados. Los diez puertos y los 63 comandos acordados están declarados; las futuras capacidades devuelven `UnsupportedCapability`, también antes de invoke en el cliente. Biblioteca y Conocimiento aparecen deshabilitados en la UI.

Se siguieron AGENTS, INTENT, arquitectura y brief T01. El coordinador autorizó dependencias locales, comprobaciones y commits, así como los archivos adicionales enumerados. No se modificaron documentos normativos, configuración global, biblioteca personal, ramas ajenas, merges ni remotos. Se aplicaron las instrucciones Karpathy y TDD de Superpowers; no se crearon subagentes. La revisión independiente corresponde al coordinador.

Ruling incorporado: `task-01-contract-clarification.md`, central en `.superpowers/sdd/IMPLEMENTATION/`. `PhaseDefinitionDto`/`PhaseRequiredOutputDto` y el cerrado `PhaseRuleKey` formalizan el contrato existente; versiones positivas, arrays de texto, resoluciones acordadas y hash de definición excluyéndose a sí mismo. No se implementan gates futuros. PENDING nunca debe satisfacer answerProcessed cuando el módulo propietario se implemente. El coordinador actualizó la autoridad normativa fuera de este worktree.

## Archivos y ownership

Todos los paths iniciales del brief son responsabilidad T01: configuración npm/Vite/TypeScript; `src/app`; contratos/DTO generado/puertos/wire; adaptador Tauri; fixtures y registry; configuración y manifests Tauri; migración 0001; actor/receipts/UoW; lifecycle/mantenimiento; adaptadores Windows; transporte; tests Rust/UI y scripts de checks/generación.

Archivos adicionales necesarios, también propiedad T01 y autorizados: `.node-version`, `rust-toolchain.toml`, `THIRD_PARTY_NOTICES.md`, `.gitignore` (solo tsbuildinfo y schemas Tauri generados), `scripts/development-env.ps1`, `src/shared/ui/dialog.tsx`, `src/shared/adapters/tauri/pdf-assets.ts`, tests `client.test.ts` y `fixtures.test.ts`, `src-tauri/src/bin/generate_contracts.rs`, `src-tauri/src/adapters/sqlite/settings.rs`, `src-tauri/src/modules/settings.rs` y `src-tauri/src/adapters/windows/startup.rs`. Los directorios vacíos de futuros módulos no simulan servicios. `work/` contiene evidencia y helpers ignorados, nunca datos de producto versionados.

## Decisiones y garantías implementadas

- SQLite bundled: foreign_keys ON, WAL, synchronous FULL, busy_timeout 5000 ms; FTS5 verificado. Migración 0001 y checksums inmutables; foreign_key_check y quick_check tras reapertura. Identidad durable en DB y `library.json` publicado mediante temporal, sync y rename.
- DbActor posee una sola conexión e hilo. Cola64, admisión no bloqueante, Busy determinista. Captura panic de un job y revierte cualquier transacción abierta. Cierre finito conserva el lock hasta que realmente se cierra la conexión; un timeout permite reintentar shutdown.
- UoW usa BEGIN IMMEDIATE. Receipt, resultado, mutación y auditoría genérica comparten transacción; hash de JSON con objetos ordenados; replay persistente y conflicto si cambian comando o payload. Los servicios futuros deben añadir su auditoría de dominio dentro de la misma transacción.
- Un esquema futuro se abre en diagnóstico sin writes. Fallo de migración revierte y conserva backup. Snapshot usa SQLite Backup API, copia recursos fuera de transacción SQL, rechaza symlinks/reparse points, comprueba hashes origen/destino y PDFs referidos, identidad y manifiesto ordenado con tamaños/SHA256. Requiere lease exclusivo del llamador; no habilita todavía una API de backup de usuario.
- `%LOCALAPPDATA%/ResearchWorkbench/library` es la ruta normal. Override `RESEARCH_WORKBENCH_TEST_ROOT` absoluto solo debug, ignorado en release. Backups/logs quedan fuera de binarios. Logs aceptan enum cerrado y rotan; no aceptan cuerpos, citas ni rutas científicas arbitrarias.
- Identidad exacta `local.researchworkbench.desktop`, ventana main/capability main-local. Permisos propios read/write/maintenance derivados del registry, sin filesystem/shell/SQL genéricos ni red/telemetría. Handler cerrado exige autoridad ACL y ventana autorizada; DTOs Serde camelCase rechazan campos desconocidos y nullable omitido. Cliente Zod valida entrada, envelope y payload; requestId inesperado produce IntegrityFailure.
- `run()` conserva Result/AppError. El adaptador Windows de arranque muestra diagnóstico español seguro para Busy y almacenamiento mediante MessageBoxW; ningún comando ni paquete añadido. No queda un expect de arranque invisible en una release sin consola.
- Tailwind y diálogo Radix estilo shadcn incorporados. RHF/Zod disponibles. PDF.js y worker local empaquetados, sin implementar lectura T03 ni red remota.
- NSIS x64 currentUser/WebView2 offlineInstaller/allowDowngrades false configurados; release lleva windows_subsystem windows. Configuración no equivale a distribución instalada probada.

## Firmas internas congeladas y consumo T02

```rust
DbActor::start(root: LibraryRoot) -> Result<DbActor, AppError>
DbActor::info(&self) -> &LibraryInfoDto
DbActor::submit<T: Send + 'static, F: FnOnce(&mut Connection)
    -> Result<T, AppError> + Send + 'static>(&self, job: F)
    -> impl Future<Output = Result<T, AppError>> + Send + use<T, F>
DbActor::shutdown(&self, timeout: Duration) -> Result<(), AppError>
with_transaction<T>(&mut Connection,
    impl FnOnce(&Transaction<'_>) -> Result<T, AppError>) -> Result<T, AppError>
with_receipt(&mut Connection, request_id: &str, command: &str,
    payload: &serde_json::Value,
    impl FnOnce(&Transaction<'_>) -> Result<serde_json::Value, AppError>)
    -> Result<serde_json::Value, AppError>
canonical_hash(&serde_json::Value) -> Result<String, AppError>
LibraryRoot::at(PathBuf) -> LibraryRoot
resolve_data_root() -> Result<LibraryRoot, AppError>
MaintenanceCoordinator::begin_operation/begin_maintenance(&self)
    -> Result<OperationPermit, AppError>
```

El parámetro F explícito y `use<T,F>` notificaron al coordinador el ajuste de captura precisa de Rust2024: el Future posee su respuesta y no toma prestado el actor. Connection no sale hacia UI/transporte. `LibraryRoot` ofrece path/database/backups/logs; el snapshot vive en migrations y requiere exclusividad. `DesktopState` ofrece actor, mantenimiento, logger y shutdown. `handle_invoke<R: Runtime>` es el mismo handler usado por app y MockRuntime. Settings usa módulo y repositorio SQL; transporte solo orquesta DTO/envelope.

T02 debe implementar selector nativo y store/lifetime de importToken, validación de PDF, recuperación de import_operations y servicio Library usando estas interfaces y el esquema acordado. **No hay token store ni importador implementados en T01.** ImportToken ya tiene DTO UUID y tabla de importaciones tiene estados STAGING/PROMOTED/COMMITTED/FAILED. Los jobs de copia/hash/validación largos deben ocurrir fuera de transacciones DB; solo los cambios atómicos/revisions/receipts/auditoría entran en UoW. RecoveryRequired consulta estado durable sin borrar staging desconocido.

Cambios compartidos previstos requieren nueva asignación del coordinador: composición `lib.rs`/handler, modules, registry `implemented`, permisos/build manifest si cambia contrato, DTO/generador/validator, migrator registry y lockfiles si se añade dependencia, y habilitación UI tras capacidad real. T02 no debe editar estos shared files sin transferencia explícita. No es preciso cambiar los puertos acordados para implementar las capacidades ya declaradas.

## Entorno y dependencias

Windows 10.0.26200 x64; Node24.14.1; npm11.11.0; Rust/Cargo1.99.0; MSVC existente y WebView2 observado154.0.4258.48. Lockfiles fijan resolución. React19.3.0, Vite8.3.2, TypeScript7.0.2, Tailwind4.3.3, RadixDialog1.1.23, RHF7.89.0, Zod4.6.5, PDF.js6.3.289, Tauri2.12.1/tauri-build2.7.1, rusqlite0.40.2, ts-rs12.0.1. jsdom29.1.1 fue elegido porque su engine oficial admite Node24.14.1; latest30.1.1 no se usó ni se ignoró su engine.

Al fijar rust-toolchain1.99.0 rustup instaló el alias exacto aunque stable ya resolvía1.99.0. Autorizado por ruling del coordinador, sin cambiar default global ni eliminar toolchains. `development-env.ps1` carga vcvars y PATH únicamente en el proceso y deriva cache común de Git. Target efectivo: `C:\Users\david\Projects\Research-Workbench\work\cargo-target`. Se trasladó cache desde el antiguo target del worktree tras verificar ambos destinos; rutas Tauri antiguas requirieron clean scoped de tauri (831files,479.5MiB) y rebuild. No se limpió ningún dato de producto.

`THIRD_PARTY_NOTICES.md` inventaría241 paquetes npm (incluidos opcionales de plataforma) y260 paquetes Cargo filtrados Windows, con declaraciones SPDX. No asigna licencia a la app ni sustituye revisión final de redistribución/recursos/WebView2 de T10.

## Comandos ejecutados y evidencia

Todos los comandos de producto se ejecutaron en el worktree indicado, salvo ruta de evidencia del diagnóstico independiente. Cargo absoluto: `C:\Users\david\.cargo\bin\cargo.exe`; wrapper de entorno local `. ./scripts/development-env.ps1`. Los logs siguientes son rutas relativas a `work/` de ese worktree y permanecen disponibles, no versionados. Los códigos son del comando hijo real; cuando un helper inicial continuó tras un error, su exit0 no se toma como éxito Cargo.

| Comando / caso | Resultado real | Evidencia |
|---|---|---|
| `npm install` (deps locales compatibles) | 0 | package-lock y mensajes de instalación de la sesión |
| `npm test` inicial, wire aún ausente | 1, red real | salida de sesión; módulo faltante |
| `cargo test --manifest-path src-tauri/Cargo.toml --test db_actor` antes de implementación | 101, módulos ausentes | salida de sesión |
| `cargo test --manifest-path src-tauri/Cargo.toml --test desktop_bootstrap` inicial | 101, módulos/generación ausentes | salida de sesión |
| `cargo test --manifest-path src-tauri/Cargo.toml --test scaffold_contracts` nullable omission | 101, red; posterior green | contracts-red-nullability.log; final-check-1.log |
| `cargo test --manifest-path src-tauri/Cargo.toml --test db_actor` panic/transacción | 101, red; posterior green | actor-panic-red.log; final-check-1.log |
| `cargo test --manifest-path src-tauri/Cargo.toml --test desktop_bootstrap` snapshots | 101, red; posterior0 | snapshot-red.log; snapshot-green.log |
| mismo test, startup diagnostic aún ausente | 101, red; posterior0 | startup-diagnostic-red.log; final-check-1.log |
| `npm test` autor blanco y límites Unicode | 1, red autor; posterior0 | unicode-boundary-red.log; final-check-1.log |
| `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` iteraciones | 101/101, luego0 | clippy-1.log, clippy-2.log, clippy-3.log |
| `scripts/generate-contracts.ps1 -Check` durante refactor | 1, cuatro enums omitidos; corregido0 | salida de sesión; final-check-1.log |
| `cargo metadata --manifest-path src-tauri/Cargo.toml --format-version 1 --offline` | 101, dependencia Android no local | salida de sesión |
| mismo con `--filter-platform x86_64-pc-windows-msvc` | 0 | cargo-metadata.json |
| `cargo test --manifest-path src-tauri/Cargo.toml --test desktop_bootstrap tauri_runtime_authority_allows_main_settings_and_denies_other_window` | inicial loader0xC0000139; después0 | acl-test-red.log; acl-test-green-1.log/2.log |
| `npm run tauri:build -- --debug --no-bundle` inicial | 1, faltaba default-run; corregido0 | tauri-build-debug-1.log; tauri-build-debug-2.log |
| `cargo clean --manifest-path src-tauri/Cargo.toml -p tauri` en cache común verificado | 0 | salida de sesión; corrección de paths cache tras traslado |
| `powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/check.ps1` final | **0** | final-check-1.log |
| `. ./scripts/development-env.ps1; npm run tauri:build -- --debug --no-bundle` vigente | **0** | tauri-build-debug-final.log |
| `cargo test --manifest-path src-tauri/Cargo.toml --test desktop_bootstrap --test scaffold_contracts` tras retirada del permiso residual | **0**, 16 tests | final-permissions-check.log |
| `. ./scripts/development-env.ps1; npm run tauri:build -- --debug --no-bundle` árbol de entrega final | **0** | tauri-build-debug-delivery.log |

El check final ejecutó npm typecheck/test/build, cargo fmt --check, clippy all-targets -Dwarnings, cargo test y generación con drift/coverage check: todos0. **9 tests UI/contratos en3 archivos y23 integración Rust (7actor,12bootstrap,4contratos)**. Zero unit/doctests no se contabilizan como cobertura. Bundle contiene PDF worker local2,228.48KB y JS386.64KB. No se calculó porcentaje de cobertura.

En el review final del staging se retiró un archivo autogenerated `plain_text.toml` residual de una extracción inicial equivocada; no pertenecía al registry63 ni estaba concedido por main-local. Los checks posteriores de permisos/contratos quedan en `final-permissions-check.log` (16 tests, exit0), y un build nativo posterior en `tauri-build-debug-delivery.log` (exit0). Quedan63 archivos autogenerated, exactamente los comandos declarados. El registry y grupos propios ya habían sido corregidos y el test de correspondencia pasa.

El loader Windows de MockRuntime fallaba antes de ejecutar el harness porque System32 COMCTL32 carece de TaskDialogIndirect y la dependencia SxS v6 faltaba en los tests. Se añadió manifest de tests usando la misma identidad v6 de tauri-build, no un DLL privado. El diagnóstico independiente del coordinador validó EXE --list exit0 y WinSxS; evidencia `C:\Users\david\Projects\Research-Workbench\work\reviews\task01-loader-diagnostic.md`. El test de autoridad real del framework ahora pasa. Tauri/Wry es dependencia no opcional: el check default sí compila transporte desktop y el dev feature test habilita MockRuntime; no existe feature desktop que el script omita.

## Qué demuestra cada capa

Rust usa archivos SQLite temporales reales: capacidad64 con barrier determinista, conexión única/reapertura, FTS5/checks integridad, lock de segundo writer, recibos, rollback, futuro schema y snapshot/migración. TS usa invoke simulado para protocolos y validación; no es una prueba de WebView2. MockRuntime usa autoridad y capability reales de Tauri con main y ventana denegada, sin WebView2 visible.

La ventana nativa real se inició desde `work/` con Start-Process -WindowStyle Hidden para la consola y biblioteca **sintética aislada** `work/native-smoke/library`; el main UI se mostró. PID16908, títuloResearchWorkbench, handle1245686. Captura de la ventana con DPI correcto: `work/native-window-dpi.png`, inspeccionada por worker y coordinador. Se observaron versión0.0.1, schema1, Disponible, confirmación de copia administrada y vistas futuras deshabilitadas: respuesta Settings real por IPC. Identidad creada304dd2bf-4ff9-4ee5-ad5a-77fc61143cbf. CloseMainWindow=true, WaitForExit(10000)=true y eventos startup/shutdown; no se afirma exitcode porque Get-Process no lo preservó. Los tests prueban reapertura con identidad durable.

La captura nativa anterior precede al pequeño adaptador de diagnóstico de arranque y al traslado de la query Settings a repositorio. **El build nativo vigente posterior a ambos cambios pasó**, pero no se tomó una nueva captura ni se afirma que se haya mostrado un diálogo de segunda instancia real. La clasificación de diagnóstico está cubierta por test. Ejecutable vigente: `C:\Users\david\Projects\Research-Workbench\work\cargo-target\debug\research-workbench.exe`.

## Autorrevisión y límites

Se corrigieron defectos concretos detectados: required-nullables omitidos, transacción abierta tras panic, snapshot sin manifiesto/hashes de recursos, autor whitespace, extracción errónea plain_text, export incompleto del generador, SQL alojado en transporte, startup expect sin diagnóstico y loader SxS de tests. El generador verifica todas las declaraciones Rust pub struct/enum, además del drift. Staging y diff--check se revisaron para excluir work/cache/schemas/datos privados. No se inició otra ronda abierta de funcionalidades.

Pendiente por alcance: instalador NSIS y VM limpia/offline/uninstall/reinstall (T10), release PE/ventana sin consola y redistribución WebView2, importación real de un PDF próximo500MiB/token store/recovery (T02), lector PDF (T03) y demás servicios futuros. La fixture sizeBytes=524288000 verifica el límite wire, **no** importa un PDF500MiB. El adaptador startup compilado no implica prueba visual del diálogo Busy de segunda instancia. Inventario de licencias no es aprobación final de distribución. Ninguno de esos resultados se presenta como demostrado por T01.

Los checks finales cubrieron el árbol de producto entregado antes de commit; no se modificó producto después salvo retirar el permiso autogenerated residual, cubierto por comprobación pertinente. El hash de entrega y estado limpio se comunican en la devolución al coordinador; este informe no puede referirse circularmente a su propio commit.

---

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

---

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
