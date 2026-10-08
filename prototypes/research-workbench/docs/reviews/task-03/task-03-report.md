# Informe de implementación T03 — Biblioteca y lector

**Estado:** DONE_WITH_CONCERNS
**Rama:** `agent/task-03-reader`
**BASE:** `cb60ee2d375aafc60dd2f8a5d696f1f375321cff`
**HEAD de implementación:** `ccc1f796f1b8af2ecaa8838f9c357c19949ae444`
**Commits propios:** `af668c1` (backend/protocolo), `ccc1f79` (interfaz/assets/fixtures).
**Biblioteca de pruebas:** siempre sintética y temporal.

## Cambios

Se conectó la Biblioteca española a LibraryApi T02 y ReaderApi v1 existentes. Inicio recupera el último paper y permite reanudar; Biblioteca lista, busca, filtra por estado, edita metadatos con control optimista, archiva/restaura, importa PDF, maneja duplicados y ofrece errores recuperables. El lector muestra PDF.js con navegación de página/zoom, restauración y guardado serializado de posición, cancelación de render anterior y cierre/liberación del documento.

Reader reutiliza el actor SQLite, LibraryRepository, DocumentStore, RecoveryStatus y MaintenanceCoordinator de T02. El `RequestRegistry` compartido se inyecta a Library y Reader. `openPaper` verifica primero el acceso actual, conserva la revisión/fecha bibliográfica, guarda actividad, último abierto y receipt en una transacción; el replay devuelve el receipt original sin repetir actividad. El guardado de posición mantiene revisiones optimistas y filas por documento. La interfaz no invoca Tauri directamente.

Se conectaron los cuatro comandos Reader al dispatcher/capability existente y se registró `research://` en el builder. El handler acepta exclusivamente la URI interna Windows canónica, `main`, GET y orígenes permitidos; resuelve un documento activo registrado, lee desde el handle protegido, devuelve PDF completo con longitud exacta y no anuncia Range. Una consulta/petición de lectura conserva su permiso de mantenimiento durante su trabajo. CSP permite WASM con `wasm-unsafe-eval`, sin `unsafe-eval` de JavaScript. PDF.js 6.3.289, worker, cmaps, fuentes, ICC, WASM/fallbacks y licencias se empaquetan localmente; no se copia QuickJS.

**ABI producida:** implementación de `ReaderPersistence`, `ReaderRepository`, `DocumentReadAccess`, `DocumentReadHandle`, comandos internos y `RequestRegistry` según `TASK03_PORTS.md`; se consumen DTO y ReaderApi v1 ya existentes. No se modificó `CONTRACTS.md`, `DATA.md`, schema 0001 ni el wire público. El manifiesto/capability marca los comandos Reader ya contratados como implementados.

## Pruebas y evidencia

- `npm ci` — exit 0; dependencias instaladas según lockfile.
- Focal inicial UI — RED por módulos/componentes todavía ausentes; luego GREEN al conectar Biblioteca y Lector. El caso adicional de página fraccionaria falló primero con `2.5` visible, y pasó tras limitar entradas a páginas enteras.
- `npm.cmd test -- src/features/library/library.test.tsx src/features/reader/reader.test.tsx` — 12 pruebas focales, exit 0. En el gate final, `npm test` — 6 archivos, 28 pruebas, exit 0. Las pruebas del lector simulan el loader PDF.js; `scanned_pdf_renders` verifica el flujo de vista React y **no** demuestra render del PDF escaneado real.
- `cargo test --manifest-path src-tauri/Cargo.toml --test reader_integration --test document_protocol -- --nocapture` — inicialmente 9 pruebas de comportamiento, todas verdes. Después se añadieron/revalidaron propiedad del trabajo, conflicto Library–Reader con `requestId` compartido, acceso tras caller cancelado, lectura durante mantenimiento y reapertura desde otro proceso. La salida final muestra 10 entradas `reader_integration` (9 pruebas de comportamiento; el décimo nombre es `reader_reopen_child_process`, helper que no-op en el proceso principal y que el caso padre lanza como proceso nuevo) y 3 `document_protocol`, exit 0.
- `protocol_junction_escape_rejected` crea una junction NTFS temporal hacia un PDF sintético externo; `open_verified` la rechaza y el archivo externo permanece intacto. Traversal y UUID desconocido también pasan. Esto prueba el parser/persistencia/guardas del adaptador, **no** una petición del protocolo observada en WebView2.
- `reader_last_opened_and_position_survive_new_process_reopen` cierra el actor SQLite y lanza un proceso hijo que vuelve a abrir la biblioteca sintética; verifica último abierto y posición 2 / zoom 1,25 / revisión 1.
- `scripts/check.ps1` — exit 0: typecheck, Vitest 28/28, build Vite, `cargo fmt --check`, Clippy con `-D warnings`, todas las pruebas Cargo y generación/verificación de contratos. Evidencia completa en `work/evidence/task03-gate-verified-final.log`; exit en `work/evidence/task03-gate-verified-final.exit`.
- `npm.cmd run tauri:build -- --debug --no-bundle` con `scripts/development-env.ps1` cargado en esa shell — exit 0. Evidencia en `work/evidence/task03-tauri-debug-final.log` y `.exit`. Compila, pero no prueba instalación ni ejecución del instalador.
- Inventario de `dist/pdfjs`: 199 archivos (169 cmaps, 16 fuentes, 2 ICC, 11 WASM/fallbacks, cero nombres QuickJS); build además incluye worker local y licencia PDF.js. Vite advierte que el JS principal (839,27 kB) y worker (2.228,48 kB) superan el umbral informativo de 500 kB.
- `git diff --check` — exit 0. El gate intermedio `task03-gate-final.log` salió 1 por formato Rust después de añadir reapertura; se formateó y el gate completo final salió 0. El primer intento también reveló `clippy::let_and_return`; quedó corregido. Logs intermedios se conservan en `work/evidence` sin presentar sus fallos como pruebas de comportamiento.

## Desviación TDD, autorrevisión y límites

El primer código backend se escribió antes de las pruebas de comportamiento. Se registra como desviación del orden TDD solicitado; no se inventó un RED retroactivo ni se revirtió para escenificarlo. A continuación se implementaron las pruebas desde SPEC/contratos y se corrigieron errores de compilación del harness, el caso de fila de revisión cero y la carrera compartida Library–Reader. Las pruebas focales de UI sí mostraron RED por componentes ausentes antes de implementarlos.

La autorrevisión comprobó conexión de App/IPC/capability, ausencia de invoke en componentes, uso del actor único, propiedad del handle dentro del trabajo admitido, validación de autoridad/path del protocolo, replay y fecha/revisión bibliográfica, carga de recursos PDF locales y fixture sintéticas. La revisión independiente del diff completo queda a cargo del orquestador según el dispatch; no se ejecutó revisión propia con agentes.

**Pendiente de evidencia nativa:** no se ejecutó esta compilación en una WebView2/ventana Tauri. Por ello quedan sin demostrar el flujo real picker→importación→apertura, el render PDF.js de `scanned.pdf`/PDF protegido, la respuesta HTTP del protocolo en WebView2 y CSP efectiva de release. Las pruebas UI usan mocks y las pruebas de protocolo son de parser/adaptador/guardas. No se afirma que el instalador haya sido probado.

**Ejecutable candidato congelado para QA coordinada:** `C:\Users\david\Projects\Research-Workbench\work\qa\t03-native\candidate-6600b994\research-workbench.exe`; SHA-256 `6600B9941F5ACF188BC0687DBF5CE78C3B8C13A7E2B46F3B4DC64638216F24F1`; tamaño 25.859.584 bytes. La prueba UI nativa queda a cargo de QA sobre copia aislada del mismo hash.

**BASE–HEAD:** `cb60ee2d375aafc60dd2f8a5d696f1f375321cff..ccc1f796f1b8af2ecaa8838f9c357c19949ae444`. El commit de informe se añade después del HEAD de implementación y no cambia producto.
