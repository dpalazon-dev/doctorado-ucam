# T10-piloto — revisión estática del artefacto corregido

Fecha: 2026-10-03. Revisor: `/root/task10_pilot_review`.

**Resultado: PASS limitado a inspección estática.** El setup nuevo, su aplicación x64, runtime offline y recursos PDF quedan identificados y comprobados mediante extracción y lectura de bytes. No se ejecutaron instalador, aplicación, runtime ni desinstalador. La aceptación instalada continúa pendiente.

## Identidad del corte

- Build source SHA: `6ad3aa475f72f30798907692f25445361a99ad8b`.
- Worktree: `.worktrees/integration`.
- Setup: `.worktrees/integration/dist-release/0.0.1/Research-Workbench_0.0.1_x64-setup.exe`.
- Target: `work/release-build/0.0.1-3dfd07050450459d91552be323f212ee/x86_64-pc-windows-msvc/release`.
- El manifiesto original se leyó y conservó intacto. Su SHA figura en el anexo de inspección.
- HEAD se observó inicialmente en el SHA de build y después en `9188ac5dc4f1cc5ad22347ef1422a2c51d795b1b`. Se verificó que el delta hasta ese HEAD no modifica `src`, `src-tauri`, `contracts`, `package.json`, `package-lock.json`, `scripts`, `vite.config.ts` ni `rust-toolchain.toml`; status final limpio. La identidad del artefacto sigue siendo el SHA de build, no el posterior commit documental.
- El artefacto histórico `03682f9d...` y su evidencia permanecen separados y sin modificaciones.

## Hashes y arquitecturas medidos

| Objeto | Bytes | PE | SHA-256 |
|---|---:|---|---|
| Setup congelado | 222023265 | i386 `0x014c` | `4d8dbfc13bc7f3a4478f691399f8dfd51c0e7551f5b0791c4866498909eae579` |
| App del target tras build | 18128384 | x64 `0x8664` | `c845c9bf69f84e1ff00f1819f6d72813deb33378b2c9b85da8127b269cf09c8b` |
| App extraída del setup | 18128384 | x64 `0x8664` | `59c1530e5f5ae13cdad56a0c31cc39dc1c6c0032bd9389ca34aa978a85cb05e7` |
| Runtime offline extraído | 212272848 | No utilizado como gate de app | `f6df8e4bc857786ff641cd01da1449169eaf8236c936ced485ea61685ba4da40` |

Setup y app del target coinciden con los hashes/tamaños del manifiesto original. La comparación byte a byte entre ambas apps confirma **solo tres diferencias**, offsets `0xDCAB52`, `0xDCAB53`, `0xDCAB54`: el marcador `__TAURI_BUNDLE_TYPE_VAR_UNK` del output es `__TAURI_BUNDLE_TYPE_VAR_NSS` en el payload. Todos los demás bytes son idénticos. El log del build corregido registra el parche NSIS; no se presupuso el resultado de la revisión anterior.

El anexo `work/evidence/task-10-pilot-artifact-4d8dbfc1/inspection.json` distingue expresamente `buildOutputApplication` de `packagedApplication`, conserva los offsets/valores de los tres bytes e identifica el manifiesto original por hash. Esto resuelve la ambigüedad de trazabilidad para este candidato sin editar aquel manifiesto. El hash de la app instalada deberá compararse con **`59c1530e...`**.

## Runtime offline y NSIS

7-Zip extrajo del propio setup el runtime de 212272848 bytes. Su SHA coincide con el archivo fuente que `WEBVIEW2INSTALLERPATH` identifica en el NSIS. `Get-AuthenticodeSignature` del archivo extraído devolvió `Valid`, firmante `CN=Microsoft Corporation, O=Microsoft Corporation, L=Redmond, S=Washington, C=US`.

Se verificaron en el NSIS actual `INSTALLMODE=currentUser`, `RequestExecutionLevel user`, `INSTALLWEBVIEW2MODE=offlineInstaller`, `MAINBINARYSRCPATH` apuntando al target nuevo y la incorporación del runtime mediante `File` en la rama offline. El `.nsi` completo coincide con el anteriormente revisado después de normalizar únicamente el GUID del directorio de build; `utils.nsh` es idéntico por SHA. Se leyeron además las rutas y condiciones relevantes del archivo nuevo.

La instalación predeterminada usa `$LOCALAPPDATA\Research Workbench`, distinta de `$LOCALAPPDATA\ResearchWorkbench`. La desinstalación borra los ejecutables conocidos y accesos del producto, y usa `RMDir "$INSTDIR"` sin recursión. Los dos borrados recursivos condicionales, al seleccionar «Eliminar los datos de aplicación» y no estar actualizando, son `$APPDATA\local.researchworkbench.desktop` y `$LOCALAPPDATA\local.researchworkbench.desktop`. No apuntan a la biblioteca, backups o logs bajo `ResearchWorkbench`. No se detectó una nueva ruta destructiva dirigida a esos datos.

La inclusión offline y esta lectura de rutas **no prueban** instalación sin red, cierre seguro, desinstalación efectiva o conservación real de biblioteca. Los callbacks que ofrecen abrir la app y el cierre por Restart Manager no se ejercitaron.

## Recursos PDF: comprobación nueva sobre el payload extraído

Un probe Node de una sola ejecución leyó `dist` de integración, los archivos `tauri-codegen-assets` del target nuevo y el ejecutable **extraído** del nuevo setup. Para cada recurso:

1. Descomprimió Brotli el asset producido por Tauri.
2. Comparó SHA-256 del contenido descomprimido con el archivo correspondiente de `dist`.
3. Localizó la secuencia completa de bytes comprimidos dentro de la app extraída.
4. Registró ruta, SHA del contenido, SHA/tamaño del comprimido y offset dentro del ejecutable en `inspection.json`.

**Resultado: 200/200 recursos comprobados**, incluyendo:

| Grupo | Archivos |
|---|---:|
| Worker `assets/pdf.worker-TGcf_-kp.mjs` | 1 |
| CMaps | 169 |
| Fuentes estándar | 16 |
| Directorio WASM | 11 |
| ICC | 2 |
| LICENSE | 1 |

No hay archivos `quickjs` entre los recursos PDF seleccionados. La comprobación acredita bytes presentes; no acredita renderizado, carga del worker en WebView2, rutas atendidas en ejecución ni ausencia universal de tráfico.

## Ejecución y evidencia preservada

Directorio exclusivo de esta revisión: `work/evidence/task-10-pilot-artifact-4d8dbfc1/`.

Contiene `inspection.json`, `7zip-list.txt`, `runtime-authenticode.json`, los dos ejecutables extraídos y copias del `installer.nsi`, `utils.nsh` y `Spanish.nsh` inspeccionados. No se ejecutaron esos binarios. El directorio se creó nuevo y la extracción usó nombres aplanados y una lista explícita de dos archivos.

```powershell
& 'C:/Program Files/7-Zip/7z.exe' l -slt '.worktrees/integration/dist-release/0.0.1/Research-Workbench_0.0.1_x64-setup.exe'
& 'C:/Program Files/7-Zip/7z.exe' e '.worktrees/integration/dist-release/0.0.1/Research-Workbench_0.0.1_x64-setup.exe' '-oC:/Users/david/Projects/Research-Workbench/work/evidence/task-10-pilot-artifact-4d8dbfc1' 'research-workbench.exe' '$TEMP\MicrosoftEdgeWebView2RuntimeInstaller.exe' -y
Get-AuthenticodeSignature -LiteralPath 'work/evidence/task-10-pilot-artifact-4d8dbfc1/MicrosoftEdgeWebView2RuntimeInstaller.exe'
git -C .worktrees/integration diff --name-only 6ad3aa475f72f30798907692f25445361a99ad8b HEAD -- src src-tauri contracts package.json package-lock.json scripts vite.config.ts rust-toolchain.toml
git -C .worktrees/integration status --porcelain=v1
```

7-Zip 24.08 ya instalado: listado/extracción exit 0 y `Everything is Ok`, dos archivos. Continúa indicando `BadCmd=13` al describir el formato NSIS; no se usa el listado como prueba completa de flujo del instalador. El probe Node terminó exit 0 tras todas las aserciones descritas y escribió solo la evidencia asignada; no cambió producto ni configuración. Diff de rutas de producto vacío y status limpio. No hubo scans de VM, instalaciones de herramientas, merges o cambios globales.

## Observaciones y límites

El helper `generate_contracts.exe` sigue en el NSIS (línea 644), como en el candidato anterior. Su función de desarrollo imprime declaraciones DTO y no aporta autoridad de aplicación. Permanece la observación LOW conocida, aceptada por el orquestador para este piloto y con retiro previsto en T10-final. No se vuelve a elevar como hallazgo nuevo.

El anexo resuelve para este artefacto la separación de hashes de output/payload. El manifiesto original conserva estados pendientes y no se reescribe. App ejecutada, runtime instalado, instalación limpia/offline, migración y preservación de datos permanecen pendientes.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH | 0 | pass |
| MEDIUM | 0 abiertos en esta inspección | pass |
| LOW | 1 conocido | note |

Verdict: **PASS estático** del setup `4d8dbfc1...`, sin aceptación de QA instalada.
