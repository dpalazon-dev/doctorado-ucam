# Tarea 10 — informe de corte piloto

Estado: preparación del piloto; build NSIS real y QA instalada pendientes.

## Corte

- BASE: `f2d39a25feb81141b51ca6cf028777b38503818b`.
- HEAD de origen del setup: el script registra el commit exacto en `sourceSHA` del manifiesto; el informe central registra los commits de preparación/corrección.
- Versión acordada: `0.0.1`; se valida package.json, tauri.conf.json y Cargo.toml.
- Target fijado: `x86_64-pc-windows-msvc`.

## Cambios

- `scripts/build-release.ps1` comprueba árbol limpio, contrato de versión/configuración, host x64, locks estables y un único setup del directorio de salida aislado. Comprueba PE x64 en `research-workbench.exe`, registra por separado la máquina compatible del stub NSIS y congela setup/manifiesto bajo `dist-release/0.0.1/`.
- `.gitignore` excluye `dist-release/`.
- Guía, release notes 0.0.1, checklist y evidencia instalada separan resultados de build, inspección estática y QA manual.
- No se crea `installed_lifecycle.rs`: una prueba placeholder no ejercitaría instalación real.

## Verificación

Comandos ejecutados el 2026-10-03 en Windows PowerShell 5.1; el build real queda reservado a Sol en el commit limpio revisado.

| Gate | Estado |
| --- | --- |
| `npm.cmd ci --prefer-offline` | Comprobado, exit 0; 156 paquetes instalados, 0 vulnerabilidades reportadas. |
| `npm.cmd run tauri:build -- --help` | Comprobado, exit 0; el CLI local Tauri 2.12.1 expone `--target`, `--bundles` (incluye `nsis`) y `--ci`. |
| `powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/tests/build-release.tests.ps1` | Comprobado, exit 0; `build-release behavior checks passed`. |
| `git diff --check` | Comprobado, exit 0. |
| Bundle NSIS real y checksum | Pendiente: ejecución reservada a Sol. |
| Inspección NSIS/WebView2/recursos PDF | Pendiente. |
| App release ejecutada y QA limpia/offline | Pendiente. |
| Actualización/migración y uninstall/reinstall | Pendiente. |

Corrección 1 de revisión independiente: el preflight acepta host `rustc` LF/CRLF y exige PE x64 del ejecutable de aplicación, registrando por separado hash y máquina PE del stub NSIS. La prueba focal repetida incluye setup stub i386 y probe real del `rustc` local; el hash/arquitectura del artefacto generado siguen pendientes.

## Límites

No se ha ejecutado release, instalador ni aplicación release; no se modificó perfil, biblioteca personal, configuración global ni infraestructura externa. No hay VM limpia/offline demostrada. La guía no certifica conservación de datos; esta requiere la prueba sobre el artefacto exacto.
