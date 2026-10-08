# T10-piloto — revisión estática del artefacto NSIS

Fecha: 2026-10-03. Revisor: `/root/task10_pilot_review`.

**Resultado: PASS como candidato inspeccionado estáticamente.** Sin hallazgos CRITICAL/HIGH. La trazabilidad debe distinguir el ejecutable del directorio de build del payload parcheado por Tauri; los hashes medidos aquí permiten completar la evidencia anexa sin reconstruir ni alterar el setup. Instalación limpia/offline y conservación efectiva de datos siguen pendientes.

## Artefacto y evidencia

- Source SHA: `510edb1aba4234e74812e668f0c9ee736a9c68b7`, confirmado contra HEAD limpio del worktree.
- Setup congelado: `.worktrees/task-10-pilot/dist-release/0.0.1/Research-Workbench_0.0.1_x64-setup.exe`.
- Setup SHA-256 verificado: `03682f9d0a63b70d525c61ff2f21eb67c2cca1981bf79a9648f8bd099dc50bb9`; 222023458 bytes; manifiesto declara stub i386.
- NSIS revisado: `work/release-build/0.0.1-31c466269bed4801b8d498e73ebf13b3/x86_64-pc-windows-msvc/release/nsis/x64/installer.nsi`, con `utils.nsh` y `Spanish.nsh`.
- Evidencia local conservada bajo `work/evidence/task-10-pilot-artifact-03682f9d/`: copias de esos tres scripts, listado `7zip-list.txt`, archivos extraídos y `hashes.json`.
- SHA del `installer.nsi`: `ce92e5879e1a164252c026f45fa1868e9303f7f0dfacee343f95340063724ce8`.
- SHA del `utils.nsh`: `72cae7532ea22bd39b5433dfd8ac47975844defa916fc113880479dedfee54f6`.

## Resultados de la inspección

**Instalación por usuario.** `INSTALLMODE=currentUser` y `RequestExecutionLevel user` en líneas 40 y 104–106. `utils.nsh:SetContext` usa el contexto del usuario y la vista de registro de 64 bits. La ruta predeterminada es `$LOCALAPPDATA\Research Workbench` (con espacio), distinta de la raíz de datos `$LOCALAPPDATA\ResearchWorkbench`. Puede recuperar la ruta previa desde la clave propia del producto. Hay entrada de desinstalación por usuario, menú Inicio y opción de acceso al escritorio. La página final ofrece iniciar la app: no se utilizó.

**Payload de aplicación.** `MAINBINARYSRCPATH` apunta exactamente al ejecutable del target aislado registrado por el manifiesto (línea 48); `File` lo incorpora en línea 639. El ejecutable extraído es PE x64, tiene 18127872 bytes y el hash detallado abajo. El source de Tauri y el registro local de build explican la diferencia con el hash del ejecutable restaurado tras empaquetar.

**WebView2 offline incorporado.** `INSTALLWEBVIEW2MODE=offlineInstaller`, argumentos `/silent`, origen `MicrosoftEdgeWebView2RuntimeInstallerX64.exe` (líneas 56–59). La rama activa incorpora el archivo mediante `File`, lo deposita en TEMP y lo invoca con `/install` (576–595). Las ramas del downloader y bootstrapper corresponden a otros valores de la constante y no se activan en esta configuración; `MINIMUMWEBVIEW2VERSION` está vacío. 7-Zip extrajo 212272848 bytes del runtime, cuyo SHA coincide con el archivo de caché indicado por el NSIS. `Get-AuthenticodeSignature` del runtime extraído devuelve `Valid`, firmante Microsoft Corporation. Esto prueba inclusión e identidad de bytes; no demuestra que la instalación del runtime complete sin red en Windows limpio.

**Desinstalación y datos.** Las eliminaciones ordinarias se limitan a los ejecutables instalados, desinstalador, accesos comprobados contra el destino de esta aplicación y claves del producto (748–819). `RMDir "$INSTDIR"` no es recursivo. La casilla opcional «Eliminar los datos de aplicación», si se selecciona y no es actualización, activa dos borrados recursivos (835–836): `$APPDATA\local.researchworkbench.desktop` y `$LOCALAPPDATA\local.researchworkbench.desktop`. Son las rutas de datos/cache Tauri, no `ResearchWorkbench\library`, `ResearchWorkbench\backups` o `ResearchWorkbench\logs`. No se observó un borrado de esas tres rutas de investigación. Por tanto, no se debe resumir el NSIS como «sin ningún borrado recursivo»: tiene los dos indicados bajo condición. La conservación real de la biblioteca requiere QA instalada.

**Aplicación abierta.** `utils.nsh:CheckIfAppIsRunning` registra el ejecutable en Restart Manager; en modo gráfico puede pedir confirmación para forzar su cierre, y en modos silencioso/pasivo puede intentar cerrarlo. La guía exige cerrar la aplicación antes de actualizar/desinstalar. No se atribuye cierre controlado o recuperación correcta a esta lectura estática.

## Observaciones

### [MEDIUM] El hash `applicationBinary` identifica el output restaurado, no el payload instalado

Archivo: `scripts/build-release.ps1:234`, reflejado en el `manifest.json` congelado.

| Archivo observado | Bytes | SHA-256 |
|---|---:|---|
| App del target, después del bundler | 18127872 | `844cd831241ca8d5e57c9c72a61c3bf877f738d810e183e96ba43ecca73945d7` |
| App extraída del setup | 18127872 | `e4934df2037cf912aeda08ef55e9035fb2c4a61c6a83110ecd72c4e7cc5d943f` |
| Runtime offline extraído | 212272848 | `f6df8e4bc857786ff641cd01da1449169eaf8236c936ced485ea61685ba4da40` |

La comparación de todos los bytes encontró **exactamente tres diferencias**, en offsets `0xDCAB12`, `0xDCAB13` y `0xDCAB14`: `UNK` en el output y `NSS` en el payload dentro del marcador `__TAURI_BUNDLE_TYPE_VAR_`. El resto es idéntico. El log `work/evidence/task10-pilot-release-retry.log:511` registra el parche NSIS. El [bundler oficial de Tauri](https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-bundler/src/bundle.rs) sustituye ese marcador antes de empaquetar y restaura después el binario original. La diferencia está explicada y no es evidencia de corrupción.

**Acción:** conservar el manifiesto original y acompañarlo con una inspección que identifique explícitamente `buildOutput` y `packagedApplication` con sus hashes; usar el segundo para contrastar la futura instalación. El orquestador ha indicado que añadirá `inspection.json`. Esta revisión proporciona las mediciones; no afirma que ese anexo ya exista. En futuros scripts debe evitarse interpretar el hash del output como hash del ejecutable instalado.

### [LOW] El paquete incluye el helper de desarrollo `generate_contracts.exe`

Archivo generado: `installer.nsi:644` (retirado en línea 763).

El bundler incorpora además del producto el generador de contratos de `src-tauri/src/bin/generate_contracts.rs`. La lectura de su source muestra generación de declaraciones DTO y salida por stdout; no tiene autoridad de aplicación ni escritura de biblioteca. Es contenido de desarrollo innecesario para el usuario instalado. El orquestador ha decidido excluirlo en T10-final; no bloquea este candidato piloto. No se ejecutó el helper.

## Comandos principales y resultados

Se utilizó 7-Zip ya instalado en `C:/Program Files/7-Zip/7z.exe`, versión 24.08. No se instaló ninguna herramienta.

```powershell
& 'C:/Program Files/7-Zip/7z.exe' l -slt '.worktrees/task-10-pilot/dist-release/0.0.1/Research-Workbench_0.0.1_x64-setup.exe'
& 'C:/Program Files/7-Zip/7z.exe' e '.worktrees/task-10-pilot/dist-release/0.0.1/Research-Workbench_0.0.1_x64-setup.exe' '-oC:/Users/david/Projects/Research-Workbench/work/evidence/task-10-pilot-artifact-03682f9d' 'research-workbench.exe' '$TEMP\MicrosoftEdgeWebView2RuntimeInstaller.exe' -y
Get-AuthenticodeSignature -LiteralPath 'work/evidence/task-10-pilot-artifact-03682f9d/MicrosoftEdgeWebView2RuntimeInstaller.exe'
git -C .worktrees/task-10-pilot status --porcelain=v1
```

Listado y extracción exit 0; extracción `Everything is Ok`, dos archivos. 7-Zip informa `NSIS-3 Unicode BadCmd=13`: no se considera su listado prueba completa del flujo del instalador, que se revisó leyendo el `.nsi` y sus macros. Los archivos seleccionados se extrajeron con nombres aplanados exclusivamente en el directorio nuevo de evidencia; los hashes y la comparación completa de bytes fundamentan las conclusiones sobre ellos. `Get-FileHash -Algorithm SHA256` verificó setup, output, payload, runtime y scripts; `Get-PeMachine` del script revisado confirmó x64 en ambos ejecutables de aplicación. Status final del worktree vacío.

No hubo ejecución de setup, app, runtime ni desinstalador. No hubo instalación, acceso a biblioteca personal, cambio global o exploración de VMs. La revisión de recursos PDF pertenece al orquestador y no se duplica aquí.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH | 0 | pass |
| MEDIUM | 1 | info |
| LOW | 1 | note |

Verdict: **PASS estático con observaciones documentadas**. El paquete puede conservarse como candidato identificado; el gate de instalación continúa pendiente.
