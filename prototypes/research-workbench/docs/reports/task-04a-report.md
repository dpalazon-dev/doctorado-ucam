# T04a — prueba documental tipada

Fecha: 2026-10-03. Worker: `/root/task04a_document_proof`. Rama: `agent/task-04a-document-proof`. Worktree: `C:\Users\david\Projects\Research-Workbench\.worktrees\task-04a-document-proof`. BASE: `9188ac5dc4f1cc5ad22347ef1422a2c51d795b1b`. Commits de implementación: `883b38584027bc6fedb160ceb85d580cb2eb47f2` (puerto y apertura compartida), `ec527ef9757b700d063ffe6a93ee666ef9282275` (cobertura a través del store y rustdoc). Estado: **DONE_WITH_CONCERNS**, revisión focal del fix1 e integración pendientes de Sol. Sin push ni merge.

## Resultado

`reader_ports.rs` expone `DocumentUnavailableReason`, `DocumentAccessOutcome` y `DocumentProofAccess`. `LocalDocumentStore` ejecuta `prove_access` sobre el mismo recorrido de apertura relativa y guardas que conserva Reader. Reader adapta `Available` al handle existente y las causas conocidas a sus errores legacy: `Missing → NotFound`; `AccessDenied` y `SharingViolation → StorageUnavailable`.

`managed_files.rs` conserva la causa OS al abrir directorios/archivos relativos. Sólo ausencia, acceso denegado y sharing violation se convierten en disponibilidad no probada; los estados genéricos siguen como `StorageUnavailable`. Root, validación del registro, regularidad, reparse, metadata/padre/descendencia, tamaño y verificaciones del pin siguen siendo errores fatales. El pin `OpenOrCreate`, accesos solicitados y share flags se conservan. La creación fallback no se traduce con el mapper de apertura. No se añadieron rutas, DTOs, capacidades ni cambios wire.

La corrección1 añade un seam privado `cfg(test)` por store. El trabajo bloqueante real consume el fallo configurado justo en el límite de apertura; builds normales conservan la llamada directa a filesystem. Las pruebas del store público cubren ambos puertos, fallos fatales, panic dentro de la closure y una apertura positiva posterior que comprueba que la inyección se consumió.

## Archivos y commits

Ownership de producto limitado a:

- `src-tauri/src/application/reader_ports.rs`
- `src-tauri/src/adapters/documents/store.rs`
- `src-tauri/src/adapters/windows/managed_files.rs`
- `src-tauri/tests/reader_integration.rs`

El commit `883b38584027bc6fedb160ceb85d580cb2eb47f2` implementa el acceso tipado; `ec527ef9757b700d063ffe6a93ee666ef9282275` cierra el hallazgo HIGH de cobertura. El commit final del informe se comunicará con el HEAD final.

## Evidencia y comandos

Target Cargo compartido: `C:\Users\david\Projects\Research-Workbench\work\cargo-target`. Logs y exit codes fuera de Git: `C:\Users\david\Projects\Research-Workbench\work\task-04a\`.

**Pruebas del store a través de los puertos públicos**

| Comando / prueba | Resultado | Evidencia |
|---|---:|---|
| `. .\scripts\development-env.ps1; cargo test --manifest-path src-tauri\Cargo.toml public_proof_and_reader_ports_preserve_injected_open_failures` | 1/1 PASS | test incluido en `fix1-check.log` |
| Recorrido `prove_access` y `open_verified` con AccessDenied inyectado | Proof: `Ok(Unavailable(AccessDenied))`; Reader: `Err(StorageUnavailable)` | misma prueba anterior |
| Recorrido de ambos puertos con `Fatal(NotFound)` y `Fatal(StorageUnavailable)` | los dos errores originales permanecen `Err` | misma prueba anterior |
| Panic inyectado dentro del `spawn_blocking` real por ambos puertos | `Err(StorageUnavailable)` | misma prueba anterior |
| Apertura real positiva tras consumir todas las inyecciones | `Available(handle)` | misma prueba anterior |

**Mappers/helpers, separados de los recorridos por el store**

| Comando / prueba | Resultado | Evidencia |
|---|---:|---|
| `cargo test --manifest-path src-tauri\Cargo.toml ntstatus_open_mapping_classifies_only_the_three_expected_os_causes` | PASS: Missing/AccessDenied/SharingViolation frente a NTSTATUS genérico | `fix1-check.log` |
| `cargo test --manifest-path src-tauri\Cargo.toml mapping_helper_classifies_denial_and_preserves_guard_errors` | PASS: helper conserva `NotFound`/`StorageUnavailable` fatales | `fix1-check.log` |
| `cargo test --manifest-path src-tauri\Cargo.toml join_helper_maps_blocking_panic_to_storage_error` | PASS: helper de join | `fix1-check.log` |

**Recorridos filesystem y gates**

| Comando / prueba | Resultado | Evidencia |
|---|---:|---|
| `cargo test --manifest-path src-tauri\Cargo.toml --test reader_integration` | 22/22 PASS; incluye missing, directory intermedio, tamaños, retención y sharing NTFS real | `fix1-check.log` |
| `cargo test --manifest-path src-tauri\Cargo.toml --test document_protocol` | 4/4 PASS; incluye rechazo de junction y lectura por handle | `fix1-check.log` |
| `pwsh -NoProfile -ExecutionPolicy Bypass -File .\scripts\check.ps1` | **exit 0**; 69 pruebas UI, 138 entradas Rust, fmt/Clippy y contratos sin drift | `fix1-check.log`, `fix1-check.exit` = `0` |
| `. .\scripts\development-env.ps1; npm.cmd run tauri:build -- --debug --no-bundle` | **exit 0**, binario debug construido | `fix1-build-debug.log`, `fix1-build-debug.exit` = `0` |
| `git diff --check 9188ac5dc4f1cc5ad22347ef1422a2c51d795b1b..HEAD` | comprobación del diff completo registrada antes de entregar | se ejecutó tras finalizar commits |

Durante TDD se observó primero el fallo esperado porque el seam aún no existía. El primer recorrido con panic mostró que el mutex del seam quedaba envenenado; se corrigió extrayendo la inyección y soltando el lock antes de ejecutar el panic. La nueva prueba focal y el gate final pasaron después de esa corrección. No quedan fallos conocidos. El build Vite informa que un chunk supera 500 kB; termina correctamente y el aviso no está relacionado con este cambio.

## Autorrevisión y límites

Revisé el diff completo desde BASE y mantuve el alcance de producto en los cuatro archivos asignados. La prueba pública de `LocalDocumentStore` usa una raíz y registro válidos y un archivo sintético para acceso/tamaño; no acredita parser PDF; la denegación se inyecta en el límite real del trabajo bloqueante y **no** acredita ACL real. El mapper NTSTATUS se comprueba aparte. La sharing violation se provocó con handles sobre NTFS sintético. Este entorno Windows no ejecuta el fallback no Windows.

El rustdoc del puerto aclara que Available posee handle/pins, Unavailable expresa sólo causas de apertura conocidas y no garantiza identidad/pertenencia, y Err conserva errores fatales. Se mantuvieron ownership de handle/pins y regresiones existentes de cancelación Reader. No cambié DTO/wire, configuración, schema ni composición. No se abrió la aplicación ni el instalador; instalación limpia sigue pendiente. T04a sólo deja listo el puerto: no declara PRE/P1, Workflow ni el gate T04 completo. Sol realizará la revisión focal Rust/seguridad y el merge preparado.
