# T04a — revisión Rust independiente final

Fecha: 2026-10-03. Revisor: `/root/task04a_rust_review`.

**Resultado: PASS, con una observación media de documentación.** No se han encontrado defectos críticos o importantes que bloqueen este corte. La integración requiere el gate sobre el merge preparado y las restantes revisiones del orquestador.

## Corte y alcance

- BASE: `9188ac5dc4f1cc5ad22347ef1422a2c51d795b1b`.
- HEAD: `20c7da9f9c9cb1757bd9755aceb00906d6f56239`.
- Commit de producto: `883b38584027bc6fedb160ceb85d580cb2eb47f2`.
- Worktree: `C:\Users\david\Projects\Research-Workbench\.worktrees\task-04a-document-proof`; HEAD verificado y árbol limpio antes y después de las comprobaciones. Sin entradas Git sin resolver.
- Revisado el delta completo BASE...HEAD: `reader_ports.rs`, `store.rs`, `managed_files.rs`, `reader_integration.rs` y `docs/reports/task-04a-report.md` (cinco archivos).
- Leídos AGENTS, INTENT, STATUS vigente, brief, informe de autor, revisión de seguridad del diseño, ABI documental de TASK04_PORTS y ADR-018; consultados ENVIRONMENT y scripts de validación. Skill aplicada: `C:/Users/david/.agents/skills/rust-patterns/SKILL.md`.

La consulta ligera de memoria sólo orientó a comprobar el checkout y sus fuentes actuales; ninguna conclusión depende del estado histórico de T01/T02. No se consultó OpenViking ni se asumió un peer efectivo.

## Hallazgos priorizados

### MEDIUM — documentar las invariantes del nuevo puerto público

`src-tauri/src/application/reader_ports.rs:31-44` añade enums y un trait públicos sin comentarios `///`. Su distinción esencial no se deduce sólo de los nombres: `Unavailable` procede exclusivamente de tres causas OS de apertura, no acredita las guardas posteriores y no incluye errores de identidad, tamaño o join. `Available` posee un handle que debe conservarse durante la operación consumidora. El contrato está explicado en TASK04_PORTS, pero no acompaña la API que implementará/consumirá T04b.

Recomendación no bloqueante: añadir rustdoc breve al resultado y a `prove_access`, con las causas admisibles, propagación de errores fatales y propiedad del handle. No requiere cambiar firmas ni comportamiento. Los tipos auxiliares públicos nuevos de `managed_files.rs:13-24` también se beneficiarían de indicar que `From<AppError>` siempre conserva un fallo fatal.

No se han encontrado otros hallazgos críticos, importantes o medios. Las limitaciones de prueba de abajo no se presentan como fallos de producto demostrados.

## Comprobaciones de código

1. **Una apertura compartida.** `store.rs:152-209`: Reader llama al mismo `prove_access`; sólo adapta el resultado tipado. `managed_file_open` y `managed_directory_with_cause` recorren el árbol administrado; `open_child_inner`/`open_file_inner` concentran adquisición y guardas para ambos wrappers. No hay otra apertura de PDF ni consulta `exists()` añadida para inferir disponibilidad.
2. **Clasificación limitada al origen OS.** `managed_files.rs:44-61,648-649,765-766` compara constantes de windows-sys únicamente al fallar `NtCreateFile`. Sólo nombre/ruta ausentes, acceso denegado y sharing producen `Os`. Colisión sigue siendo `Fatal(Conflict)`; el resto, `Fatal(StorageUnavailable)`. `From<AppError>` siempre da `Fatal`, sin inspeccionar códigos/mensajes. `map_document_open_failure` conserva exactamente el AppError fatal.
3. **Root y guardas no se degradan.** `store.rs:179-196,259-278` mantiene validación de biblioteca/UUID/status/ruta/tamaño y root binding. Las guardas de componente, metadata, pin, regularidad, padre y descendencia usan `?` hacia la conversión fatal. Tamaño registrado fuera de rango conserva `PathNotAllowed`; divergencia real conserva `IntegrityFailure`. `flatten_blocking_result` convierte el error externo de join en `Err(StorageUnavailable)`, nunca en indisponibilidad documental.
4. **FFI y permisos conservados.** Comparado contra BASE: mismos `RootDirectory`, componente validado, `OBJ_DONT_REPARSE`, `FILE_OPEN_REPARSE_POINT`, máscaras de acceso, share flags y disposiciones. Los archivos Managed siguen solicitando lectura/escritura/DELETE y compartiendo sólo lectura. Pins mantienen `ReadPin/OpenOrCreate`. Los bloques unsafe conservan justificación y propiedad del handle adquirido. No se añade unsafe, unwrap/expect o panic en producción.
5. **Lifetimes conservados.** La closure bloqueante posee root/registro; `Available` mueve `ManagedDocumentReadHandle` con su `ManagedFile`, que retiene padre/root/ancestros/pin. No se convierte en una ruta ni se reabre al devolverlo. El wrapper Reader mueve el mismo handle. Las regresiones existentes de cancelación observan su retención hasta commit y rollback. La futura retención Workflow hasta commit pertenece a T04b.
6. **Compatibilidad de otros consumidores.** Las conversiones añadidas en creación de DB, bind y wrappers preservan códigos legacy. En fallback, `create_dir` sigue transformando todos sus errores en StorageUnavailable antes de la conversión fatal; no utiliza el mapper de apertura. `File::open` y `OpenOptions::open` conservan sus mappings anteriores al volver a AppError. No se ha ejecutado este fallback en Windows.
7. **Alcance acotado.** Ningún cambio de SQL, migraciones, DTO/wire, configuración, capacidades, dependencias, UI o composición. Sin lectura/hash completo del PDF en proof. La nueva caja del future Reader añade una asignación pequeña al wrapper; no hay evidencia de un problema de rendimiento en esta apertura ocasional.

La implementación usa `ManagedOpenCause` más `ManagedOpenError::{Os,Fatal}` y sufijos `*_for_document`, frente a los nombres `ManagedOpenFailure`/`*_for_proof` del diseño. La partición de estados y la propagación son equivalentes; el puerto de aplicación acordado conserva sus firmas. Se deja explícita esta diferencia nominal para la comprobación de conformidad de Sol.

## Verificación ejecutada por este revisor

Desde el worktree indicado, dot-sourcing de `scripts/development-env.ps1`, con target compartido `work/cargo-target` y sin modificar configuración global:

```powershell
. .\scripts\development-env.ps1
cargo check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo test --manifest-path src-tauri/Cargo.toml
```

Los cuatro comandos terminaron correctamente; el encadenamiento comprobó `$LASTEXITCODE` después de cada paso y finalizó con **exit 0**. Cargo test: **137 entradas aprobadas**, incluidos helpers; ninguna fallida/ignorada. Incluye 22 entradas de Reader y 4 de document_protocol. No se cuenta este total como 137 comportamientos distintos. Evidencia independiente observada en la sesión del revisor (exec `1476`).

Inspección Git: `git diff BASE...HEAD -- '*.rs'`, delta completo del informe, `git status --short`, `git ls-files -u` y `git rev-parse HEAD`. `git diff --check BASE...HEAD -- '*.rs'` pasa. El mismo check sobre todo el delta avisa de una línea en blanco final en `docs/reports/task-04a-report.md:47`; observación cosmética fuera de Rust, sin cambios de código realizados por el revisor.

Se contrastaron los logs del autor `work/task-04a/check-final2.log` y `build-debug-final2.log`, junto con sus `.exit = 0`: 69 pruebas UI, 137 entradas Rust, Clippy con `--all-targets`, comprobación de contratos y build debug sin bundle. Son evidencias del autor leídas por el revisor; no se atribuyen como una nueva ejecución UI/build independiente.

## Alcance real de las pruebas

- Ausencia de archivo y directorio intermedio: filesystem sintético real a través de `prove_access`; archivo ausente también comprueba `NotFound` legacy.
- Sharing violation: handle real retenido sobre NTFS, probado en managed files y en el store. El control positivo obtiene `Available`; la segunda apertura da `Unavailable(SharingViolation)` y Reader conserva StorageUnavailable. Acredita que el resultado mantiene el handle.
- Tamaño real divergente y tamaño registrado inválido: pruebas a través del store con `IntegrityFailure` y `PathNotAllowed`, respectivamente.
- Denegación y NTSTATUS genérico: pruebas deterministas directas de `classify_ntstatus`; la traducción de AccessDenied a outcome/legacy y la preservación de guardas NotFound/StorageUnavailable se comprueban directamente en `map_document_open_failure`. No inyectan esos fallos a través de toda la llamada pública `prove_access`, ni ejercitan una ACL real.
- Panic: un trabajo `spawn_blocking` real provoca panic y se entrega al mismo `flatten_blocking_result` usado por producción. No es un panic introducido dentro del recorrido completo de apertura.
- Cancelación/propiedad y reparse: regresiones Reader/protocolo existentes ejecutadas con el nuevo núcleo; las pruebas commit/rollback envuelven el store real, y la prueba junction verifica rechazo y contenido exterior intacto.

La revisión del cableado completa la evidencia unitaria de los mappers; no se exagera como inyección integral. No se ha ejecutado app, instalador, ACL real o fallback no Windows. No se afirma PRE/P1 ni Workflow implementados.

## Integración y entrega

No hay configuración CI versionada en `.github` en este corte. AGENTS exige revisiones y gate sobre el merge preparado; este PASS presupone que ese gate sigue verde y que no hay conflictos de integración pendientes. El árbol revisado está limpio y sin conflictos, pero aquí no se preparó ni aprobó ningún merge.

Único archivo escrito: este informe central. Sin cambios de producto, commits, subagentes, merges, push ni configuración global. Autorrevisión: hallazgos limitados al delta congelado, causas OS y guardas diferenciadas, evidencia real e inyectada identificadas y límites de instalación conservados.

**Entrega: DONE_WITH_CONCERNS — PASS Rust; observación media de rustdoc, sin bloqueos críticos/importantes.**
