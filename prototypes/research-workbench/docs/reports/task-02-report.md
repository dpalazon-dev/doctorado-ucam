# T02 — biblioteca e importación recuperable

Fecha de cierre: 2026-10-02. Worker: `/root/task02_library`. Rama: `agent/task-02-library`. Worktree: `C:\Users\david\Projects\Research-Workbench\.worktrees\task-02-library`. BASE: `94f1796513383ae75486f0b429ab25bae0fb63c1`. Commit de implementación: `6a7a1fd608bc0fcf76ff185747af90ae398789b8`. El hash final que incluye este informe se comunica en la devolución; el informe no se referencia circularmente a sí mismo.

Estado: **DONE_WITH_CONCERNS**, implementación y gates completos, pendiente de revisión independiente del coordinador. Árbol limpio tras los commits. No se integró ni publicó.

## Resultado

Los ocho métodos `LibraryApi` llegan ahora desde el registro Tauri al `LibraryService` y los adaptadores SQLite/filesystem reales. `selectPdf` cancela con `null`, copia el PDF seleccionado a almacenamiento gestionado y emite preview solo tras hash y validación estructural. Confirmar registra intención, promueve en el mismo volumen y confirma Paper/authors/Document/receipt/audit en una transacción; duplicados requieren resolución y `reuseExisting` no altera el Paper ni el documento existente. Cancelación elimina únicamente staging que coincide con la intención. Recovery reconcilia operaciones al inicio y deja archivos ambiguos intactos con issue explícito.

El almacenamiento usa copia/hash en streaming, buffer de 64 KiB y tope de 500 MiB. Una validación PDF activa por biblioteca comprueba xref/trailer/catálogo, existencia y resolución de la primera página, MediaBox finita y positiva, y rechaza cifrado. No modifica los bytes. Errores de formato se convierten en `InvalidPdf` seguro; una lectura fallida de la fuente da `SourceUnreadable`.

Las mutaciones de Library usan receipts/revisiones, y el conflicto incluye `details.currentRevision` en el envelope. En arranque, se obtiene exclusión de mantenimiento antes de publicar Library y se ejecuta `reconcile_imports` fuera del hilo de ventana. Los comandos devuelven `Busy` durante recuperación. El cierre drena trabajos filesystem admitidos y recuperación antes de parar el actor/liberar el lock. Con esquema futuro no writable, Library queda disponible en diagnóstico pero rechaza mutaciones con `SchemaTooNew`; no se crean directorios del DocumentStore ni se ejecuta recovery/escritura.

## Ownership y archivos

Se implementó el ownership T02 del brief: `domain/library.rs`; `application/library_ports.rs`; `modules/library/{service,repository,recovery}.rs`; `adapters/sqlite/library_repository.rs`; `adapters/documents/{store,imports,pdf_probe}.rs`; `adapters/windows/pdf_picker.rs`; `transport/commands/library.rs`; `tests/library_integration.rs`; fixtures sintéticas cifradas y dependencias `pdf=0.10.0` sin features por defecto y `tauri-plugin-dialog=2.8.1`, lockfile y avisos.

Con transferencias explícitas se extendieron `lib.rs`, exports de adapters/application/domain/modules, settings y commands, lifecycle/maintenance, `transport/error.rs` (solo transporte de `currentRevision` normativo), y pruebas de composición/cierre/settings. Se preservaron actor, UoW, receipts, migración 0001, manifest/registry de 63 comandos, DTOs y permisos; no se habilitaron capacidades genéricas frontend `fs`/`dialog`.

## ABI implementado para T03/T04

La autoridad externa es el `LibraryApi` v1 de `CONTRACTS.md`, sin cambios de wire:

```text
selectPdf(requestId: UUID) -> IpcResult<ImportPreviewDto|null>
confirmImport({requestId, importToken, metadata, duplicateResolution?}) -> IpcResult<PaperDto>
cancelImport({requestId, importToken}) -> IpcResult<null>
listPapers({requestId, filter}) -> IpcResult<PageDto<PaperDto>>
getPaper({requestId, paperId}) -> IpcResult<PaperDto>
updateMetadata({requestId, paperId, expectedRevision, metadata}) -> IpcResult<PaperDto>
archivePaper({requestId, paperId, expectedRevision}) -> IpcResult<PaperDto>
restorePaper({requestId, paperId, expectedRevision}) -> IpcResult<PaperDto>
```

ABI Rust conectado: `DesktopState::library() -> Result<Arc<LibraryService>, AppError>`; `LibraryService::{select_pdf,confirm_import,cancel_import,list_papers,get_paper,update_metadata,archive_paper,restore_paper}` son async, reciben UUID/DTOs propios y devuelven DTOs/resultados de aplicación; `reconcile_imports() -> Result<RecoveryReport,AppError>` es ruta de arranque/mantenimiento, no IPC. Los comandos en `transport/commands/library.rs` son adaptadores delgados al servicio.

Puertos internos async (futures `Send`, sin conexión SQLite fuera del actor): `NativePdfSelection::select_pdf() -> Option<SelectedPdf>`; `DocumentStore::{stage,inspect_owned,promote,cleanup_owned}`; `LibraryPersistence` cubre intención/receipt, preview, preparación y commit de confirm/cancel, consultas/mutaciones Paper y consultas/actualizaciones de recovery. `modules/library/repository.rs::require_current_revision(tx, paper_id, expected_revision)` debe llamarse dentro de la transacción del llamador y nunca confirma por sí mismo. T04 puede consumir `get_paper`, `update_metadata`, `archive_paper` y `restore_paper` mediante `LibraryService`; sus commits de repositorio y cambios Paper permanecen en un único UoW. T03 puede consultar Paper por `get_paper` y obtener DTOs estables. T02 no añade lectura/stream de bytes PDF para el Reader. Según T03_BOUNDARIES/ADR-015, T03 extiende el mismo store con `DocumentReadAccess` y un handle autorizado, sin duplicar reglas de paths. T02 no expone rutas libres ni adelanta esa API.

## Recovery, concurrencia y seguridad de datos

El token de selección expira a 24 h y se vincula a sesión y biblioteca; concurrencia de mutaciones por token se serializa. Retry de requestId/payload idéntico devuelve receipt existente; payload distinto produce `Conflict`. DOI se normaliza localmente (trim/prefijo/host/lowercase y validación `10.<registrante>/<sufijo>`), sin red.

Las operaciones filesystem viven fuera del hilo DB, hilo UI y transacciones SQL. Las etapas guardan hashes/tamaños e intención durable; `reconcile_imports` verifica propiedad/hash e idempotencia, reanuda promociones confirmadas, cancela únicamente casos seguros y conserva destinos ambiguos. La prueba de fallo SQL demuestra rollback de Paper sin éxito prematuro y recovery tras reinicio.

La composición para schema futuro conserva el inventario íntegro de la raíz y prueba que las mutaciones contestan `SchemaTooNew`. La prueba de cierre bloquea determinísticamente trabajo filesystem admitido y recuperación; la salida no cierra actor ni suelta lock hasta liberar ambas clases de permit.

## PDF_VALIDATION / ADR-014

| Requisito de la matriz | Evidencia T02 |
|---|---|
| xref clásico / xref stream y object stream / actualización incremental | `xref_object_stream_and_incremental_pdf_are_accepted` |
| texto renombrado, cabecera/EOF o xref inválidos, Root y Pages inválidos | `invalid_pdf_no_success`, `invalid_xref_root_and_cyclic_page_tree_never_publish_preview` |
| catálogo sin páginas, primera página no resoluble, árbol vacío/cíclico | `page_tree_and_first_media_box_must_be_valid`, `invalid_xref_root_and_cyclic_page_tree_never_publish_preview` |
| MediaBox ausente o dimensiones degeneradas | `page_tree_and_first_media_box_must_be_valid` |
| cifrado real con user password y user vacío/owner | `encrypted_pdfs_with_and_without_user_password_are_rejected`; fixtures generadas localmente con pypdf 6.10.0, `work/evidence/pdf-fixtures.md`, SHA256 registrados allí |
| `/Encrypt` como texto de metadato no cifrado | `encrypt_token_inside_document_metadata_is_not_encryption` |
| >500 MiB, válido exactamente 524,288,000 bytes | `size_limit_500_mib`, `valid_pdf_at_500_mib_limit_is_staged_and_cancelled_cleanly` |
| crecimiento mientras lee, sin sleeps ni carrera probabilística | `growth_during_read_crosses_limit_deterministically` |
| mover original tras staging; inaccesible | `import_survives_original_move`, `source_unreadable` |

El PDF al límite se genera temporalmente y se elimina; no hay fixture gigante en Git. La prueba tarda 33.35 s dentro del integration runner completo. El parser valida una estructura mínima y una página; no certifica ISO ni renderiza/decodifica todas las páginas. El buffer está acotado, pero expansión de objetos comprimidos puede requerir memoria adicional; serializar limita concurrencia, no establece cuota total RAM.

## Evidencia y comandos

Cwd de producto: worktree del encabezado. Target/cache Cargo: `C:\Users\david\Projects\Research-Workbench\work\cargo-target`. MSVC objetivo verificado: `x86_64-pc-windows-msvc`, Rust 1.99.0 (`work/evidence/rustc-target.log`). Las rutas `work/evidence` son locales ignoradas, no datos de producto.

| Comando / ejecución | Resultado | Evidencia |
|---|---|---|
| `powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/check.ps1` (typecheck, UI tests/build, fmt, Clippy all targets `-D warnings`, tests Rust, check contratos) | **exit 0** | `work/evidence/check-final.log`, `.exitcode` |
| Gate UI dentro del script | 4 archivos, **14/14** | `check-final.log` |
| Unit Rust | **4/4** | `check-final.log` |
| Integración Rust: db_actor / desktop_bootstrap / library_integration / lifecycle_close / scaffold_contracts / schema_diagnostic / settings_service | **7/7 / 12/12 / 27/27 / 3/3 / 6/6 / 4 entradas / 1/1** | `check-final.log` |
| Conteo interpretado Rust | 64 entradas observadas: 4 unit + 60 integration-runner; `pending_wal_fixture_child` es helper de proceso, así que 63 comprobaciones de comportamiento + 1 helper, sin contar de nuevo el subproceso | `check-final.log`; conteo coordinador |
| `npm run tauri:build -- --debug --no-bundle` | **exit 0**, aplicación Windows debug compilada | `work/evidence/tauri-build-debug.log`, `.exitcode` |
| `cargo test --manifest-path src-tauri/Cargo.toml --test library_integration` focalizado previo | **20/20** de la versión de ese corte | `work/evidence/library-integration.log`; el gate final posterior ejecutó 27/27 |
| `git diff --check` | **exit 0** | comprobación previa a commit, sesión del implementador |

Los tests de contrato de error prueban `Conflict.details.currentRevision` en el envelope. `check.ps1` además verifica generación/drift de contratos. El build Tauri y rustc objetivo demuestran compilación Windows MSVC, no instalación NSIS.

Los tests se escribieron primero y los casos focalizados fallaron antes del cambio correspondiente durante las iteraciones TDD. No se conservaron logs rojos individuales en `work/evidence`; sus salidas se observaron en las herramientas de esta sesión. Los logs versionados localmente preservan los verdes finales, que son la evidencia primaria de entrega. No se repitió ninguna suite para reconstruir salidas después del gate.

## Autorrevisión y límites

Revisé el diff completo desde BASE: ownership acotado a brief/transferencias, sin cambio de contrato, DTO, permisos ni migración. El callback de picker puentea la respuesta/cancelación con oneshot y test determinista; el `Future` sí se ejecuta. Selección, copia/hash, promoción y recovery no bloquean hilo de ventana. No hay `block_on`, `mmap`, `unsafe` propio, rutas de filesystem desde IPC ni escritura/recovery al construir Library si esquema no writable. `git diff --check` pasó. Clippy completo no reportó warnings.

No se observó el selector GUI real; picker se compiló/enlazó y el puente de callback se probó, así que no afirmo diálogo nativo visible. Tampoco se instaló NSIS ni se probó equipo limpio/offline/reinstall. No hay garantía de render de todas las páginas, certificación ISO ni cota total de RAM PDF. La lectura PDF por Reader queda en la extensión `DocumentReadAccess` asignada a T03 (ver ABI); no es parte del alcance de T02. La revisión independiente de código/spec/seguridad y la integración quedan a cargo del coordinador.
