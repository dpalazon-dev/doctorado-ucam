# T03 — informe de corrección 3

Fecha: 2026-10-03 (UTC). Autor: `/root/task03_library_reader`.

## Identidad del corte

- Worktree: `C:\Users\david\Projects\Research-Workbench\.worktrees\task-03-reader`
- Rama: `agent/task-03-reader`
- BASE limpio: `30d532714d053c09d4021bc89bafbb0168b1602a`
- HEAD de producto: `e268862f61f7896f2efe6baab65b25f01c4c9f18` (`fix: preserve reader recovery across views`)
- `git diff --check`: exit 0 antes del commit de producto. El informe y la corrección editorial del informe fix2 van en un commit documental separado.

## Cambios y ABI

- La cola Reader es la única autoridad de recuperación por documento. Las vistas montadas se suscriben y reciben conflictos/resultados inciertos aunque la operación se iniciara desde una vista ya desmontada; cada vista vuelve a hidratarse desde el mismo estado del documento.
- Resolver guardando la posición local limpia solo el conflicto que resolvió, una vez confirmado el CAS. Si llegó otra intención durante esa operación, se conserva y se procesa sobre la revisión confirmada. Un replay incierto que confirma una posición anterior deja visible la posición más nueva como pendiente, sin mostrar «Guardado».
- Las pruebas de Reader cubren resolución→siguiente guardado, intención más reciente durante la resolución, replay de posición antigua, y cierre/reapertura del mismo documento ante respuesta tardía de conflicto e incertidumbre. La prueba de importación cubre detalles de duplicado malformados: error accesible, borrador conservado, cancelación disponible y ninguna apertura. El camino «conservar posición guardada» existente sigue cubierto como decisión sin escritura.
- El wrapper de prueba Rust observa `confirm_open` tras el primer `Poll::Pending`. Las variantes commit/rollback comprueban cancelación del caller, handle vivo, mantenimiento, propiedad del mismo `requestId` frente a Library y liberación tras terminar el actor.
- ABI externa producida: ninguna. No cambian contratos IPC/DTO, esquema, puertos ni protocolo PDF. La suscripción por documento es un detalle interno de `features/reader`; el cambio Rust afecta solo al harness de integración.

## Evidencia y comandos

Los logs y exit codes están en `C:\Users\david\Projects\Research-Workbench\.worktrees\task-03-reader\work\evidence\` (directorio ignorado por Git):

- `task03-fix3-hook-red.log/.exit`, exit 1: focal previo al cambio. Las cuatro regresiones nuevas fallaron; las pruebas existentes restantes pasaron. Reveló limpieza de conflicto ausente, replay que marcaba como guardada una posición vieja, y pérdida de notificación a la vista reabierta.
- `task03-fix3-reader-red.log/.exit`, exit 1: focal previo al cambio. Fallaron el estado visible pendiente después del replay y los controles tardíos de recuperación tras reabrir el documento.
- `task03-fix3-focal-green.log/.exit`, exit 0: `npm.cmd test -- src/features/reader/useReadingPosition.test.tsx src/features/reader/reader.test.tsx` — 2 archivos, 21/21 pruebas, 1.84 s.
- `task03-fix3-m1-green.log/.exit`, exit 0: `npm.cmd test -- src/features/library/library.test.tsx -t malformed_duplicate_candidate_keeps_draft` — 1 prueba pasó, 21 omitidas. El caso ya estaba corregido; no se simuló un rojo ni se modificó producto para esta revisión.
- `task03-fix3-r2a-green.log/.exit`, exit 0: `. .\scripts\development-env.ps1; cargo test --manifest-path src-tauri/Cargo.toml --test reader_integration dropped_reader_caller_keeps_handle_and_shared_request_owner_until_open -- --nocapture` — 2/2 pruebas, 0.05 s; incluye compilación de test de 22.24 s.
- `task03-fix3-gate.log/.exit`, exit 0: `powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/check.ps1` — TypeScript estricto, frontend 67/67, `cargo fmt`, Clippy, contratos sin drift y pruebas Rust completas. Rust: 128 entradas en 10 targets con pruebas; dos entradas son helpers de proceso hijo y no 128 comportamientos independientes. `library_integration` 55/55; `reader_integration` 17/17. Incluye replay/reapertura en proceso nuevo y persistencia de revisión bibliográfica.
- `task03-fix3-tauri-debug.log/.exit`, exit 0: `. .\scripts\development-env.ps1; npm.cmd run tauri:build -- --debug --no-bundle` — Vite completó en 0.418 s y Cargo en 12.56 s. Vite informó que un chunk JavaScript supera 500 kB minificado.
- Build: `C:\Users\david\Projects\Research-Workbench\work\cargo-target\debug\research-workbench.exe`, 25,866,752 bytes; SHA-256 `60130E7090F12D5C145D1D4E966C9A7FE3688278CE6A398080D7F14DAAD168BE`. `dist\pdfjs`: 199 archivos; QuickJS: 0.

## Autorrevisión y límites

La limpieza usa identidad del conflicto/posición capturados para no borrar un conflicto distinto ni una intención posterior, incluso si coincide en página y zoom. El estado compartido publica cambios a las vistas activas y evita depender del callback de una vista desmontada. La barrera Rust señala solo tras sondear el future real y observar `Pending`.

No se lanzó el binario. WebView2 real, render PDF en ventana nativa, selector PDF del sistema, instalador NSIS y equipo limpio no se verificaron en este corte; las pruebas visuales usan PDF sintético y mocks locales. El aviso de chunk grande es informativo; gate y build terminaron con exit 0. No se abrió ninguna biblioteca personal ni se consultó/escribió memoria OpenViking.
