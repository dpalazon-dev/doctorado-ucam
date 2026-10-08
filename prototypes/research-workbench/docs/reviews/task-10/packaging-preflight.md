# T10-piloto — preflight de empaquetado

Fecha: 2026-10-02. Autor: `/root/t10_packaging_preflight`. Estado: **DONE_WITH_CONCERNS**.

## Alcance y conclusión

Inspección estática del commit `1f7e4a9a79880a154da8a14ac7eb73464b1ed38d` de `.worktrees/integration`, leído con `git show HEAD:<ruta>`. El árbol de integración estaba limpio. Se consultaron INTENT, STATUS, brief T10, WORKFLOW, ENVIRONMENT y las reglas de distribución de ARCHITECTURE. T03 continúa en corrección y no forma parte de este corte de producto: no se inspeccionó su worktree ni se trató su candidato como final.

La base ya declara NSIS por usuario, WebView2 offline y versión 0.0.1. No aparece una necesidad de cambiar contratos de dominio/IPC para empaquetar el piloto. **Esto permite preparar T10, pero no acredita que el corte actual construya ni entregue el piloto completo:** falta T03 integrada y faltan el script de release, el artefacto y las pruebas instaladas. El target x64 depende actualmente del entorno documentado, sin una guarda explícita en el comando versionado.

No se ejecutaron npm, Cargo, bundle, instaladores ni aplicaciones; no se modificaron cachés, configuración global o datos personales. No se investigó VM/Sandbox: su disponibilidad sigue desconocida. El único archivo escrito por esta tarea es este informe; no hay commits ni merges.

## Mapa de archivos reales y responsabilidad sugerida

| Archivo en el corte observado | Evidencia concreta | Responsabilidad T10 |
| --- | --- | --- |
| `src-tauri/tauri.conf.json` | `productName: Research Workbench`, `identifier: local.researchworkbench.desktop`, `version: 0.0.1`; `bundle.active: true`, `targets: [nsis]`; `installMode: currentUser`; `webviewInstallMode.type: offlineInstaller`, `silent: true`; español sin selector; `startMenuFolder: Research Workbench`; `allowDowngrades: false`; icono ICO | Sol conserva ownership; transferencia expresa antes de cualquier cambio. Verificar la configuración resuelta y el artefacto generado. |
| `src-tauri/icons/icon.ico`, `icon.png` | Ambos versionados; la configuración referencia el ICO | Sol/config; T10 comprueba icono visible en instalación/accesos. La existencia del archivo no prueba su presentación. |
| `package.json`, `package-lock.json` | Nombre `research-workbench`, versión 0.0.1 coherente; CLI/API Tauri 2.12.1, PDF.js 6.3.289 fijados. `tauri:build` ejecuta `tauri build`; `build` ejecuta TypeScript y Vite | Sol manifiestos/lockfiles; T10 consume el corte congelado y registra hashes. |
| `src-tauri/Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml` | Paquete 0.0.1; Tauri 2.12.1 y tauri-build 2.7.1; SQLite bundled; toolchain 1.99.0. Cargo.lock contiene paquete local 0.0.1 | Sol manifiestos/versiones/lockfiles; no hace falta subir versión para este piloto. |
| `scripts/development-env.ps1` | Añade Cargo al PATH del proceso, obtiene entorno `vcvars64.bat` si falta VCINSTALLDIR y usa `work/cargo-target` compartido si CARGO_TARGET_DIR no está definido | Sol/entorno. Release debe registrar el target efectivo y la salida exacta; no confundir la caché compartida con un artefacto congelado. No se ejecutó aquí. |
| `scripts/check.ps1`, `generate-contracts.ps1`, `scripts/tests/check-exitcode.ps1` | Gate existente de UI/Rust/build/contratos; check.ps1 comprueba códigos de salida | T10 consume gate de Sol sobre el corte definitivo. No sustituyen build NSIS ni QA instalada. |
| `src-tauri/src/main.rs` | `windows_subsystem = windows` cuando no hay debug assertions | Ownership producto; T10 comprueba release sin consola en el ejecutable instalado. |
| `src-tauri/capabilities/main-local.json`, `build.rs`, `permissions/` | Ventana `main`, solo `research-read`, `research-write`, `research-maintenance`; sin remote.urls en capability; manifest de comandos explícito | Sol/frontera IPC. No se necesita abrir permisos genéricos para empaquetar. |
| `src-tauri/tauri.conf.json` → `build`, `security.csp` | Frontend de producción `../dist`; beforeBuild `npm run build`. devUrl separado. CSP restrictiva del corte T02, todavía sin los ajustes de recursos PDF autorizados en ADR-015 | Sol con cambios T03 integrados. Releer el corte final y comprobar CSP resuelta; no modificarla desde este preflight. |
| `vite.config.ts`, `src/shared/adapters/tauri/pdf-assets.ts` | Vite básico con target chrome120; adaptador observado solo importa `pdf.worker.mjs?url`. No se observó una estrategia completa de fuentes/cmaps/WASM en esos archivos del corte T02 | T03 posee la implementación de recursos de lector; T10 audita su inclusión y funcionamiento offline tras integrar T03. Observación del corte, no finding contra el trabajo pendiente. |
| `src-tauri/src/adapters/windows/paths.rs` | Release resuelve `%LOCALAPPDATA%/ResearchWorkbench/library`; backups/logs hermanos. `RESEARCH_WORKBENCH_TEST_ROOT` se procesa solo en debug | Producto conserva frontera de datos. QA release necesita entorno/perfil de prueba autorizado y sintético; no lanzar release esperando que el override debug proteja una biblioteca personal. |
| `.gitignore` | Ignora `work/`, `.worktrees/`, `.superpowers/`, `dist/`, targets y otros outputs; **no contiene `dist-release/`** | Sol añade exclusión antes de generar entregables allí o transfiere ese cambio explícitamente. |

## Entregables aún ausentes en HEAD

`git ls-tree` no encuentra estos archivos/rutas asignados a T10:

- `scripts/build-release.ps1`.
- `src-tauri/tests/installed_lifecycle.rs`.
- `docs/INSTALLATION.md`, `docs/release-checklist.md`.
- `docs/releases/0.0.1.md`, `docs/releases/0.1.0.md`.
- `docs/verification/installed-pilot.md`, `docs/verification/installed-v01.md`.
- `docs/reports/task-10-report.md` y artefactos versionados bajo `dist-release/`.
- `src-tauri/installer/research-workbench.nsh`: ausente, **opcional** según brief. No crear un hook solo por completar una lista; revisar primero el NSIS generado y justificar cualquier personalización.

No es una deficiencia que los binarios no estén versionados. El futuro directorio local `dist-release/0.0.1/` debe contener el artefacto congelado y su manifiesto/checksum sin incorporarlos accidentalmente a Git.

## Huecos a resolver antes de dispatch release

1. **Dependencia de producto:** cerrar revisiones, integrar y validar T03. Después fijar BASE/HEAD final para T10; el HEAD de este informe no es el candidato piloto completo. Volver a inspeccionar assets PDF, CSP, lockfiles y capacidades del corte integrado.
2. **Ownership explícito:** Sol mantiene `tauri.conf.json`, manifiestos/lockfiles, capacidades, configuración de build, versiones e integración. Luna T10 recibe script release, pruebas instaladas y documentación enumerada en el brief. Cualquier ajuste a shared files requiere transferencia registrada. `.gitignore` debe quedar expresamente en uno de esos owners.
3. **Target y salida reproducibles:** el comando documentado es `npm.cmd run tauri:build -- --bundles nsis`; no fija target y no hay `.cargo/config*` versionado. ENVIRONMENT registra host x86_64-pc-windows-msvc, pero el script futuro debe comprobar el target efectivo o fijarlo explícitamente y comprobar arquitectura del artefacto. Esto es un control de build, no cambio de contrato. Resolver también el nombre de entrega `Research-Workbench_0.0.1_x64-setup.exe`: el nombre real del bundler todavía no está observado; congelar/copiar a ese nombre y registrar origen si difiere.
4. **Script de release verificable:** falla ante comandos fallidos, comprueba árbol inmóvil/limpio y coherencia 0.0.1, registra sourceSHA/toolchains/comandos/hashes de locks, identifica el instalador producido por esa ejecución y solo entonces genera manifiesto/SHA-256. No reutilizar un setup antiguo de la caché como éxito. No instala SDKs globales. El gate normal y el bundle son operaciones distintas.
5. **QA aislada y evidencia:** concretar entorno autorizado de instalación sintética antes de ejecutar release. La disponibilidad VM/Sandbox no está demostrada aquí; si falta, el resultado debe quedar como candidato construido con instalación pendiente, tal como exige el brief. No empezar con una biblioteca personal ni con un override válido solo para debug.

Los puntos anteriores permiten un brief ejecutable. La instalación limpia/offline, las migraciones y el ciclo de desinstalación son trabajo de T10 y gates de aceptación posteriores, no condiciones ya comprobadas por este preflight.

## Verificaciones que T10 deberá producir

- NSIS generado y payload: runtime WebView2 realmente incluido, arquitectura x64, identidad/versión, icono, Inicio, acceso de escritorio si se ofrece, entrada de aplicaciones y desinstalador. `offlineInstaller` expresa intención de empaquetado; no demuestra bytes incluidos o instalación sin red. No se ha verificado disponibilidad local del payload/herramientas NSIS y no se ha alterado su caché.
- Assets PDF y CSP finales de T03: worker, fuentes y recursos efectivamente requeridos por PDF.js dentro del producto y lectura offline sin servidor/source. La ausencia de `bundle.resources` en el corte observado no prueba por sí sola ausencia de recursos: el frontend puede incluirlos en `dist`.
- Ejecutable instalado release sin consola; datos fuera de binarios; apertura, importación sintética, original movido, página 2, cierre/process ended, reapertura, archivo/restauración y segundo escritor conforme al brief.
- Inspección del script NSIS generado antes del gate destructivo: ninguna eliminación de biblioteca/backups ni rutas ajenas. `allowDowngrades: false` es configuración del instalador; no prueba el rechazo de esquema futuro del producto ni una migración segura.
- Upgrade/reinstall, downgrade de esquema, preservación IDs/hashes/posición y desinstalación/reinstalación sobre fixtures sintéticas. Si una migración real entre versiones aún no existe en el primer piloto, registrar la condición exacta pendiente y acordar el fixture con Sol; no presentar un reinstall de la misma versión como prueba de migración.
- Mantener independientes los estados comprobado/fallido/pendiente. Una build debug previa, WebDriver o un setup generado no acreditan equipo limpio sin herramientas/WebView2 y offline.

## Evidencia de inspección y autorrevisión

Comandos de solo lectura ejecutados en `.worktrees/integration` (todos terminaron con exit0):

```powershell
git rev-parse HEAD
git status --short
git ls-tree -r --name-only HEAD
git show HEAD:src-tauri/tauri.conf.json
git show HEAD:package.json
git show HEAD:package-lock.json
git show HEAD:src-tauri/Cargo.toml
git show HEAD:src-tauri/Cargo.lock
git show HEAD:src-tauri/capabilities/main-local.json
git show HEAD:src-tauri/build.rs
git show HEAD:src-tauri/src/main.rs
git show HEAD:src-tauri/src/adapters/windows/paths.rs
git show HEAD:scripts/development-env.ps1
git show HEAD:scripts/check.ps1
git show HEAD:vite.config.ts
git show HEAD:src/shared/adapters/tauri/pdf-assets.ts
git show HEAD:.gitignore
git show HEAD:rust-toolchain.toml
git show HEAD:src-tauri/tests/scaffold_contracts.rs
git ls-tree -r --name-only HEAD -- .cargo
git ls-tree -r --name-only HEAD -- scripts src-tauri/installer src-tauri/tests/installed_lifecycle.rs docs/INSTALLATION.md docs/release-checklist.md docs/releases docs/verification dist-release
git status --porcelain=v1
```

Los dos lockfiles se filtraron a sus encabezados/entrada del paquete local; no se afirma una auditoría íntegra de dependencias. El inventario del árbol acredita los nombres de archivos; la inspección de config acredita los valores, no sus efectos en una instalación. No se hicieron pruebas nuevas de comportamiento porque estaban fuera del encargo. Se aplicaron Karpathy y verificación antes de completar: cambios acotados al informe y separación explícita entre observación, requisito y resultado pendiente.
