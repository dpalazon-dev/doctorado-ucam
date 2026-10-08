# T10-piloto — revisión independiente de preparación

Fecha: 2026-10-03. Revisor: `/root/task10_pilot_review`.

**Resultado: BLOCK.** Dos fallos importantes del script impiden producir el candidato previsto. Deben volver al autor y revisarse antes de ejecutar el build real o integrar esta preparación.

## Corte y alcance

- Worktree: `C:/Users/david/Projects/Research-Workbench/.worktrees/task-10-pilot`.
- BASE: `f2d39a25feb81141b51ca6cf028777b38503818b`.
- HEAD: `fe6af93a23bf1fb54f03e4c24df4cada675ad090`.
- Árbol limpio al iniciar y finalizar. Sin diferencias staged/unstaged.
- Revisados los ocho archivos de BASE..HEAD completos, el helper de entorno, manifiestos/configuración Tauri, package scripts, distribución PDF en Vite y resolución de datos Windows.
- Leídos AGENTS, INTENT, STATUS, despacho/informe central, brief T10, IMPLEMENTATION y apartados pertinentes de SPEC-001, QUALITY y ARCHITECTURE.
- No se ejecutó build, instalador ni aplicación release. No se modificaron producto, configuración global ni bibliotecas; esta revisión solo escribe este informe central.

## Hallazgos

### [HIGH] El preflight rechaza el host x64 correcto por los saltos CRLF

**Archivo:** `scripts/build-release.ps1:120` (origen de la cadena: línea 88).

`Get-ToolOutput` convierte las líneas de `rustc -vV` mediante `Out-String`; Windows PowerShell produce CRLF. En la expresión `(?m)^host: x86_64-pc-windows-msvc$`, `$` coincide antes de LF, pero el CR permanece después de `msvc`. La línea de host está seguida por `release` y `LLVM version`, por lo que el `Trim()` final no quita ese CR. El script aborta antes de invocar Tauri aun con el host exigido.

**Reproducción real, sin build:** cargar las funciones del script y llamar a `Get-ToolOutput -Command 'C:/Users/david/.cargo/bin/rustc.exe' -Arguments @('-vV')`. La salida observada declara `host: x86_64-pc-windows-msvc`; la expresión vigente devuelve `False`, mientras que `(?m)^host: x86_64-pc-windows-msvc\r?$` devuelve `True`. Rust observado: 1.99.0.

**Corrección:** analizar la línea normalizada o admitir CRLF/LF explícitamente. Añadir regresiones con ambas terminaciones y con host incorrecto que ejerciten el preflight. Las pruebas actuales solo cubren selección, PE y versiones y no detectan este fallo.

### [HIGH] La arquitectura x64 se comprueba en el stub NSIS en vez del payload

**Archivo:** `scripts/build-release.ps1:153`; afirmaciones derivadas en líneas 188 y 194, `docs/release-checklist.md:7`, `docs/verification/installed-pilot.md:24` y los informes.

La llamada `Assert-PeX64 -Path $installerPath` exige que el propio setup tenga máquina PE `0x8664`. El bundler Tauri utiliza stubs/plugins NSIS `x86-unicode` incluso cuando el payload y el nombre del instalador son x64. Por tanto, un setup correcto con stub PE x86 será rechazado después del build. Además, esta comprobación no valida la arquitectura del ejecutable Research Workbench contenido en el instalador.

**Evidencia primaria y local:** el [código oficial del bundler NSIS de Tauri](https://raw.githubusercontent.com/tauri-apps/tauri/dev/crates/tauri-bundler/src/bundle/windows/nsis/mod.rs) exige `Stubs/lzma-x86-unicode` y `Stubs/lzma_solid-x86-unicode`, usa plugins `x86-unicode` y calcula por separado el `arch` de la aplicación. La lectura del binario local `node_modules/@tauri-apps/cli-win32-x64-msvc/cli.win32-x64-msvc.node` (CLI 2.12.1) confirma esas mismas referencias y la plantilla incorporada con `!define ARCH "{{arch}}"`. No se ha generado un setup en esta revisión; la evidencia es de las herramientas que lo producirán, no de una instalación.

**Corrección:** comprobar `0x8664` en `research-workbench.exe` dentro del target aislado y registrar por separado nombre/hash/arquitectura del ejecutable y del setup. Validar y registrar el PE real del setup sin imponerle la arquitectura del payload; contrastar después el `.nsi` generado y su ruta de ejecutable. Ajustar documentación/manifiesto y añadir un caso sintético con payload x64 + stub x86 aceptado, además de payload incorrecto rechazado.

## Comprobaciones realizadas

Comandos desde el worktree salvo que se indica una ruta absoluta:

```powershell
git status --short
git diff --staged
git diff
git diff f2d39a25feb81141b51ca6cf028777b38503818b fe6af93a23bf1fb54f03e4c24df4cada675ad090
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/tests/build-release.tests.ps1
git diff --check f2d39a25feb81141b51ca6cf028777b38503818b fe6af93a23bf1fb54f03e4c24df4cada675ad090
. ./scripts/build-release.ps1
$details = Get-ToolOutput -Command 'C:/Users/david/.cargo/bin/rustc.exe' -Arguments @('-vV')
$details -match '(?m)^host: x86_64-pc-windows-msvc$'
$details -match '(?m)^host: x86_64-pc-windows-msvc\r?$'
git status --porcelain=v1
```

- Pruebas focales del autor: exit 0, `build-release behavior checks passed`.
- Diff check: exit 0.
- Reproducción preflight: `False` con regex vigente; `True` admitiendo CRLF.
- Git status final: sin salida.
- Sin cambios Rust/TypeScript en este corte; no requiere reabrir la revisión de T03.

## Resto de la revisión y límites

El target Cargo por GUID evita reutilizar un setup antiguo; la selección exige un único `.exe`. Se comprueban árbol limpio y HEAD/locks antes y después del comando. Un exit no cero o un fallo de validación preceden a la publicación del manifiesto; la copia exige igualdad SHA-256 y el manifiesto se mueve desde un temporal. Un destino de release existente se rechaza, y no hay eliminación de datos en el script. Estas propiedades se comprobaron por lectura; las pruebas presentes no simulan la ejecución completa ni todos los fallos de etapas.

Los comandos de target/NSIS/CI coinciden con los scripts y configuración locales. La documentación mantiene pendientes el payload offline, recursos empaquetados, ejecución release, instalación limpia, migración y conservación de datos. Fuera de las afirmaciones erróneas sobre la arquitectura del setup, no se detectaron nuevos fallos de alta confianza en los otros archivos. El build real y la inspección del `.nsi` siguen reservados al orquestador tras corregir y revisar.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH | 2 | warn |
| MEDIUM | 0 | pass |
| LOW | 0 | pass |

Verdict: **BLOCK para integración/build del piloto** conforme al gate del proyecto: quedan dos hallazgos importantes abiertos. Esta revisión no emite aceptación del instalador ni de QA instalada.
