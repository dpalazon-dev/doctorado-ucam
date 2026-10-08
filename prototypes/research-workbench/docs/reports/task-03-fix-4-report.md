# T03 — informe de corrección 4: timestamps Reader UTC

Fecha: 2026-10-03. Autor: `/root/task03_library_reader`.

## Identidad del corte

- Worktree: `C:\Users\david\Projects\Research-Workbench\.worktrees\task-03-reader`; rama `agent/task-03-reader`.
- BASE limpio: `5a512a7bba610e90ecb10d657ed1b36095df4078`.
- HEAD de producto: `55407d12d6b7ca403b2ab2c10390e524f43ffdbd` (`fix: normalize reader UTC timestamps`).
- `git diff --check`: exit 0 antes del commit de producto; el informe central/versionado y la adenda al informe T03 se añaden en un commit documental separado.

## Corrección y ABI

- Los dos escritores Reader (`confirm_open` y `save_reading_position`) ahora generan UTC con milisegundos y sufijo `Z`, igual que los productores canónicos existentes de Library/receipts.
- El decoder de timestamps UTC usa el validador ISO completo y acepta sólo `Z` o `+00:00`. Devuelve el string original, sin normalizarlo. Rechaza fechas locales, offsets distintos de cero, `-00:00`, fechas/horas inválidas y tipos que no sean string.
- No cambian estructura de DTO, tipos TypeScript, comandos, contractVersion, datos SQLite ni receipts. La compatibilidad `+00:00` permite leer datos históricos y no es migración ni reescritura.
- Las regresiones Rust comprueban open/save nuevos en DB, DTO y receipt; replay devuelve el resultado confirmado. Fixtures sintéticos `+00:00` verifican `last_opened_paper`, lectura de posición y replay sin alterar el receipt, la columna timestamp ni el conteo de auditoría. El cliente Tauri real valida envelopes con `lastOpenedAt` y `updatedAt` UTC antiguos y preserva la representación recibida.

## Evidencia

Logs y exit codes: `C:\Users\david\Projects\Research-Workbench\.worktrees\task-03-reader\work\evidence\`.

- `task03-fix4-wire-red.log/.exit`, exit 1: `npm.cmd test -- src/shared/contracts/wire.test.ts src/shared/adapters/tauri/client.test.ts`. Los dos casos nuevos fallaron antes del cambio: el schema rechazó el `+00:00` de `lastOpenedAt` y el cliente clasificó el envelope como `IntegrityFailure`.
- `task03-fix4-open-red.log/.exit`, exit 101: `cargo test --manifest-path src-tauri/Cargo.toml --test reader_integration open_paper_sets_activity_transactionally_without_changing_bibliographic_revision -- --nocapture`. Apertura real del fixture sintético emitía `+00:00`, no `Z`.
- `task03-fix4-position-red.log/.exit`, exit 101: `cargo test --manifest-path src-tauri/Cargo.toml --test reader_integration default_position_is_not_inserted_and_saved_position_is_document_scoped -- --nocapture`. Guardado real emitía `+00:00`, no `Z`.
- `task03-fix4-wire-green.log/.exit`, exit 0: mismo comando wire/client — 2 archivos, 15/15 pruebas, 1.09 s.
- `task03-fix4-reader-green.log/.exit`, exit 0: `. .\scripts\development-env.ps1; cargo test --manifest-path src-tauri/Cargo.toml --test reader_integration -- --nocapture` — 19/19, 0.21 s. Incluye new `Z`, replay y preservación exacta de resultados legacy.
- `task03-fix4-format.log/.exit`, exit 0: `. .\scripts\development-env.ps1; cargo fmt --manifest-path src-tauri/Cargo.toml`.
- `task03-fix4-gate.log/.exit`, exit 1: primer `scripts/check.ps1`; typecheck y Vitest pasaron 69/69, pero `cargo fmt --check` solicitó formato en los tests Rust nuevos. No fue fallo de comportamiento; se conserva el log.
- `task03-fix4-gate-final.log/.exit`, exit 0: `powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/check.ps1`. TypeScript, 69/69 frontend, formato, Clippy, contratos y Cargo completos; Reader 19/19 y Library 55/55.
- `task03-fix4-tauri-debug.log/.exit`, exit 0: `. .\scripts\development-env.ps1; npm.cmd run tauri:build -- --debug --no-bundle`. Vite 0.521 s; Cargo `Finished dev profile` en 24.58 s. Vite conserva su aviso informativo de chunk principal mayor que 500 kB.
- Binario generado: `C:\Users\david\Projects\Research-Workbench\work\cargo-target\debug\research-workbench.exe`; 25,866,752 bytes; SHA-256 `0C4BE713C8D2DD9618B8ACD6AF0A3C6E334D957DDA49B1530A36544A81CC7537`. `dist\pdfjs` contiene 199 archivos; QuickJS: 0.

## Autorrevisión y límites

La validación de fecha calendario y hora la sigue haciendo Zod (`z.iso.datetime`); la condición posterior sólo reduce los sufijos aceptados a las dos formas UTC de ADR-019. Se verifican strings de hasta nueve decimales como el receipt diagnosticado, además de fechas inválidas y valores no textuales. No se cambió ningún lector para reescribir datos existentes, ni se abrió o reparó la base usada por QA: los tests usan fixtures/SQLite temporales sintéticos.

El ejecutable se compiló, pero no se lanzó desde este worktree. Quedan pendientes el nuevo recorrido nativo del orquestador con una copia congelada y la validación real de WebView2/PDF.js. Este arreglo corrige la causa demostrada que impedía llegar a Reader; no afirma todavía que el render o el protocolo funcionen en ventana nativa.
