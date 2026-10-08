# T02 — corrección 2/5

Fecha: 2026-10-02. Implementador: `/root/task02_library`. Rama/worktree: `agent/task-02-library` / `C:\Users\david\Projects\Research-Workbench\.worktrees\task-02-library`. BASE: `922a0669d771d6778cd3d264bcc5be91f0e96ec6`. Commit de código verificado: `3cdeae2bfc936e5447ae9910a8f7bc0ea16a730d`. La entrega mantiene los cambios previos de T02/fix1 y aplica seis hallazgos importantes, M5 y la aclaración C1. Sin merges, remotos ni configuración global. El coordinador realiza las revisiones independientes.

**Estado: DONE_WITH_CONCERNS, pendiente de rereview independiente.** El gate final `scripts/check.ps1` y el build nativo debug terminan con exit 0. La verificación no acredita interacción visual del selector, instalación/NSIS ni funcionamiento en un equipo limpio. Para límites funcionales conservados, véase F7 y F10.

## Cambios y evidencia

| Hallazgo | Resolución y evidencia |
|---|---|
| **F3 — frontera de aplicación** | Los helpers de caso de uso reciben `&Transaction<'_>` y `&dyn LibraryRepository`; normalizan/validan metadata, resuelven duplicados, preparan reservas y registran auditoría de entidad dentro de la misma TX. El adapter conserva primitivas SQLite y no toma el commit. `ImportReservation<'_>` agrupa el input de reserva para mantener concisa la frontera. Se conserva separado el evento de receipt/replay. Evidencia previa T02 en `fix2-f3-*`; el gate final revalida todo Rust. |
| **F7 — stage failure recuperable** | El adaptador devuelve `StageFailure { error, cleanup }`; error/código seguro y `PENDING` o `DONE` quedan en la intención versionada `stageFailure`. DONE se guarda solo tras acreditar ausencia. Si cleanup no se prueba, se conserva el staging y recovery issue tras reopen. Invalid PDF, cifrados con contraseña vacía/no vacía, origen ausente, sobrelímite 500 MiB, parcial persistente/reopen y PDF exactamente de 500 MiB se ejercitan en `library_integration`; el gate final terminó 51/51. Caso >500 MiB: `InvalidInput`, intención FAILED/cleanup DONE y sin Paper/Document; recovery no produce issue porque se probó la limpieza. |
| **F9 — exclusión por requestId** | Las mutaciones comparten exclusión asíncrona por requestId antes de adquirir permisos de operación; la tarea admitida conserva ownership hasta el fin. Replay compatible espera al resultado del dueño y replay incompatible no produce segundo efecto. Cancelación mantiene exclusión del token y de maintenance durante FS. Barreras deterministas y cierre se comprueban en `compatible_overlapping_select_replays_same_result_after_first_finishes`, `incompatible_overlapping_cancel_conflicts_without_touching_second_staging`, `same_request_metadata_archive_restore_cannot_race_cancel_cleanup` y `lifecycle_close` (3/3 en gate). |
| **F10 — identidad y protección Windows** | `ManagedRoot` fija una identidad ligada y retiene handles de ancestros. Componentes/descendientes y pines se abren relativamente con no-reparse; los handles de padres impiden reemplazo y el pin persistente evita que un directorio administrado vuelva a ser vacío mientras esté en uso. La ruta de volumen se construye desde el prefijo sin `Path::join` absoluto. Un DB existente se pinnea antes del schema probe; un DB ausente se crea con `FILE_CREATE` después de `LibraryLock`; colisión no abre SQLite. La prueba Rust nativa valida control sin pin, bloqueo por pin, reparse, no-clobber, DB y WAL/checkpoint/reopen; actor 8/8 y schema/future-root 4/4 en gate. NTFS local requerido; UNC/no NTFS se rechazan. El pin `.rw-directory-pin` permanece como archivo interno vacío. No se afirma cobertura de selector GUI, ACL global o instalación limpia. |
| **F11 — cleanup por namespace/referencias** | `validate_cleanup_plan` relee el record e IDs canónicos dentro de TX, compara namespace, intent y plan actuales y rechaza referencias `Document` u otras intenciones. El owner conserva exclusión hasta concluir FS. `destination_without_durable_promotion_authority_is_preserved`, `cleanup_refuses_intent_referencing_an_existing_document_path`, import preparado/cancelado y recovery pending verifican que archivo ajeno/referenciado o sin autoridad se conserve. |
| **F13 — hashing del mismo handle** | Inspección, promoción y cleanup hashean streaming desde el mismo handle autorizado, con buffer fijo de 64 KiB y tamaño/lectura verificados antes/después; cambios/growth son rechazados. Las pruebas instrumentadas y de crecimiento F13 previas se conservan en `fix2-f13-*`; suite Rust y Clippy completos pasan al final. |
| **C1 — duplicado tardío accionable y cancelación del perdedor** | `DuplicateDecisionRequired` incluye `details.candidates` de la misma TX también en late commit; razones DOI/SHA se combinan sin duplicarse. Cancelación del archivo perdedor requiere confirmación durable PROMOTED, ConfirmIntent v1 íntegra/no-reuse, payload/hash/IDs ligados, sin receipt/resultado de confirmación y sin referencias. `promotionConfirmed` queda durable en la intención de cancelación y se revalida en cada retry/recovery; la mera ruta o hash no concede propiedad. Store requiere staging ausente si existe destino, verifica tamaño+hash en el mismo handle, elimina solo el `original.pdf` reservado y confirma DONE/receipt al verificar ausencia de ambas rutas. El test post-promote real pasó de rojo `ImportRecoveryRequired` a verde; replay cancel produce un único receipt, ganador intacto y destino perdedor ausente. Negativo sin promoción conserva bytes y `cleanup=PENDING`; cancellation PENDING post-promote se recupera tras cierre/reapertura. `fix2-c1-*` conserva cada salida. No hay cambio de DTO, wire, IPC ni migración. |
| **M5 — corrección del informe anterior** | El informe fix1 afirmó que con `writable=false` no se construía Store. La implementación real sí construye `LocalDocumentStore`, cuyo constructor no hace I/O; se omite recovery y no hay mutaciones. El reporte fix1 queda como historial y esta versión corrige la afirmación sin cambiar el comportamiento aprobado. `readonly_future_schema_composition_preserves_root_and_rejects_mutation` pasa en la suite. |

Las decisiones cerradas F1/F2/F4/F5/F6/F8/F12 y M1–M4 permanecen como se documentaron en el informe fix1. No se han cambiado contratos de UI ni 0001.

La matriz PDF requerida se conserva: `xref_object_stream_and_incremental_pdf_are_accepted` prueba xref clásico/object stream/incremental; `encrypted_pdfs_with_and_without_user_password_are_rejected` usa PDFs cifrados reales con contraseña de usuario vacía y no vacía; `page_tree_and_first_media_box_must_be_valid` cubre caja válida heredada/ausente/degenerada y ciclo en cadena Parent con descendencia Root→Pages→Page válida y xref correcto; `invalid_xref_root_and_cyclic_page_tree_never_publish_preview` prueba raíz/ciclo Kids; `size_limit_500_mib` y `valid_pdf_at_500_mib_limit_is_staged_and_cancelled_cleanly` cubren sobrelímite y máximo exacto. `growth_during_read_crosses_limit_deterministically` y `growth_after_copy_is_rejected_before_pdf_parser_receives_bytes` controlan crecimiento sin carreras temporales. El límite conservador del parser pdf 0.10 (`PageTree::page_limited(16)`) no se cambió; cadenas de herencia más profundas pueden rechazarse como `InvalidPdf`.

## ABI interno confirmado

El wire público y los ocho comandos de Library no cambian. Firmas internas relevantes:

```rust
prepare_import_in_tx(
    tx: &Transaction<'_>, repository: &dyn LibraryRepository, request_id: &str,
    token: &str, original: &PaperMetadataInput,
    resolution: Option<&DuplicateResolution>,
) -> Result<PreparedImport, AppError>

update_metadata_in_tx(
    tx: &Transaction<'_>, repository: &dyn LibraryRepository, request_id: &str,
    paper_id: &str, expected_revision: i64, metadata: &PaperMetadataInput,
) -> Result<PaperDto, AppError>
archive_paper_in_tx(
    tx: &Transaction<'_>, repository: &dyn LibraryRepository, request_id: &str,
    paper_id: &str, expected_revision: i64,
) -> Result<PaperDto, AppError>
restore_paper_in_tx(/* mismos parámetros que archive_paper_in_tx */)
confirm_in_tx(
    tx: &Transaction<'_>, repository: &dyn LibraryRepository,
    prepared: &PreparedImport, verified: &ResourceInspection,
) -> Result<PaperDto, AppError>

LibraryRepository::reserve_import(
    &self, tx: &Transaction<'_>, reservation: ImportReservation<'_>,
) -> Result<(), AppError>
LibraryPersistence::validate_cleanup_plan(
    &self, plan: CancelPlan,
) -> LibraryFuture<'static, ()>
```

`ImportReservation<'_>` contiene `operation_id`, `request_id`, `metadata_json`, `payload_hash`, `paper_id`, `document_id` y `destination_path`. `has_durable_promoted_confirmation(&ImportRecord) -> bool` valida la evidencia durable sin I/O ni conceder autoridad por existencia/hash de archivo. `LocalDocumentStore::cleanup_owned(CancelPlan) -> LibraryFuture<'static, ResourceInspection>` realiza filesystem en background. Recovery reutiliza el mismo store/plan y conserva exclusiones; las transacciones SQL terminan antes del trabajo de archivos.

## Verificación final

Todos los comandos de gate se ejecutaron en PowerShell con `scripts/development-env.ps1`, toolchain MSVC y `CARGO_TARGET_DIR=work/cargo-target`. Los logs y exit codes están en `work/evidence/` (scratch ignorado por Git).

| Comando | Resultado |
|---|---|
| `powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/check.ps1` | `fix2-final-check2.log`, exit 0. Incluye typecheck; Vitest 14/14; Vite build; `cargo fmt --check`; `cargo clippy --all-targets -- -D warnings`; `cargo test`; contract generation check. |
| Conteo por runner del gate | 11 unit; `db_actor` 8/8; `desktop_bootstrap` 13/13; `library_integration` 51/51; `lifecycle_close` 3/3; `scaffold_contracts` 6/6; `schema_diagnostic` 4/4 (incluye `pending_wal_fixture_child`, fixture hija, no un test de comportamiento adicional); `settings_service` 1/1. 97 entradas de runner equivalen a 96 comprobaciones de comportamiento + 1 proceso hijo fixture. |
| `cargo build --manifest-path src-tauri\Cargo.toml` | `fix2-native-debug-build.log`, exit 0; perfil dev/debug de la aplicación nativa. No implica bundle NSIS ni instalación. |
| `git diff --check` | `work/evidence/fix2-diff-check.log/.exitcode`, exit 0. |

El primer `scripts/check.ps1` de esta ronda falló por Clippy (`join_absolute_paths` y método de reserva con demasiados argumentos). Se corrigió la construcción de ruta y se agrupó la reserva en `ImportReservation`; el segundo gate completo terminó exit 0. Una primera suite de integración detectó dos fallos en `reuseExisting` por exigir payload de cancelación a un cleanup temporal; se corrigió la distinción y la suite final pasó 51/51. Se conservan rojos originales; no se presenta un intento fallido como verde.

## Autorrevisión y límites

- Revisión final debe cubrir `BASE..HEAD` completo, especialmente el contrato `promotionConfirmed`, la validez de ConfirmIntent y que `validate_cleanup_plan`/Store mantengan autoridad desde el chequeo hasta el final FS.
- Los checks prueban el runtime backend Windows y compilación nativa; no prueban una UI visible, instalador NSIS, primer inicio de máquina limpia o upgrade/desinstalación.
- F10 depende de NTFS local y de semantics Windows ejercitadas en esta máquina. No afirma seguridad de ACL de toda la raíz frente a administradores/kernel ni soporta raíces UNC/no NTFS.
- La limitación conservada del parser PDF 0.10 (`PageTree::page_limited(16)`) y matriz de PDFs se mantienen en el informe fix1/PDF validation; no se sustituyó el parser.
- No se generaron ni modificaron bibliotecas personales; fixtures sintéticas y scratch son locales.
