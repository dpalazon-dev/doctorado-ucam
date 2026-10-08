# T03 QA nativa — preparación para revisión

**Estado:** PRIMERA EJECUCIÓN BLOQUEADA EN PICKER; segundo intento autorizado abortado por deriva de runtime en preflight
**Scope:** Windows WebView2 real, importación por selector del sistema, protocolo `research://`, render PDF.js y persistencia de página/zoom tras cerrar y reabrir. Esta etapa no da aprobación de integración ni de merge.
**Candidato congelado:** `work/qa/t03-native/candidate-87118787/research-workbench.exe`
**SHA-256 candidato:** `871187877DCA988FFDC0560C0C9B2C30A295F4B721005213626C3462A092C481`
**Tamaño:** 25.864.192 bytes
**Origen declarado:** producto T03 `b18937127563e19a684e348ea1cdda9e269142dd`, HEAD `b36ae5ae0529ba429edf7a360ade7319e353e8fe` de `.worktrees/task-03-reader`; el binario congelado coincide con el SHA anterior.

## Artefactos preparados

- Runner actual: `work/qa/t03-native/native-qa.ps1`, SHA-256 `AB334368DD6DF34B8A981622E4F4E5F18D5B869F08DB3864DBFD23D9C9C6DF6C`.
- Snapshot previo al diagnóstico del picker: `work/qa/t03-native/evidence/native-qa-3006-before-picker-filter.ps1`, SHA-256 `3006ECAA112C717F5D3FF8A6BB81B5DA43F2DA4945D2610EEC955DFA08C9EC52`; preservado.
- Snapshot de evidencia del runner original revisado: `work/qa/t03-native/evidence/native-qa-371111f0-review-blocked.ps1`, SHA-256 `371111F01A057FBA1B6CEDB58738961D61C3C24195E6A57DF72E7D35EA6ED9FD`; preservado sin cambios.
- Snapshot del delta bloqueado por la revisión focal: `work/qa/t03-native/evidence/native-qa-dcf3-blocked.ps1`, SHA-256 `DCF3A5FD534A014089363A8A953D3BA1E1161A851F6AAD28AE3CF4E02CE428D2`; preservado sin cambios.
- Generador determinista del PDF: `work/qa/t03-native/make-synthetic-pdf.ps1`.
- PDF local: `work/qa/t03-native/fixtures/synthetic-two-pages.pdf`, 968 bytes, SHA-256 `A65C300B22EBE0A8CDCD7253C47BE5B8752680470B9D3B89F6BD4A5D57703E95`.
- El PDF contiene dos páginas con texto/vector simple, objetos y tabla `xref`; el generador comprobó número de páginas, encabezado, EOF y que `startxref` apunte a `xref`. No contiene fuentes ni documentos de terceros.

El runner no inicia la app sin el parámetro explícito `-ExecuteAfterReview`. Sin ese parámetro valida únicamente hashes, la versión de Edge y el entorno de PowerShell; esta ejecución preparatoria terminó con `prepared; app launch intentionally gated`. La salida registró Edge `154.0.4258.48` y PowerShell `7.6.5`. Hashes que verifica: ejecutable, fixture, `tauri-driver` (`2684A7B9E1E667D6689BEA8EAFF2AEFE093C5B1F5CA329B3156B384A9CAB9286`) y EdgeDriver (`4032E9D74DA42B30805EC5125EEAF9CBD4F678C2A41104EE90471BE16CAAD997`). No se cambió PATH ni configuración global.

## Recorrido propuesto por el runner

1. Crear un directorio nuevo `native-run-<GUID>` con biblioteca sintética vacía y perfil WebView2 propio. Pasar `RESEARCH_WORKBENCH_TEST_ROOT` absoluto y `WEBVIEW2_USER_DATA_FOLDER` solo al proceso de `tauri-driver`; comprobar Edge/WebView2 coincidentes y reservar dos puertos loopback libres.
2. Crear sesión con `tauri-driver`/EdgeDriver contra el ejecutable congelado. El preflight exige que no haya ya una instancia del ejecutable exacto; la instancia nueva se correlaciona por intervalo de creación y ascendencia hasta el driver, y se conserva su `SafeHandle`. El perfil WebView2 requiere archivos y al menos un proceso con `--user-data-dir` correlacionado con ese PID. La capability devuelta se registra: si viene vacía, se documenta, sin afirmar aislamiento de todas las escrituras.
3. Observar Biblioteca vacía, abrir Importar PDF y elegir **la ruta exacta** de la fixture mediante Windows UI Automation. El helper enumera solo ventanas superiores del PID de la app o ventanas con `GW_OWNER` directamente asociado a ese PID; exige un control de nombre de archivo y un botón Aceptar. Usa `ValuePattern.SetValue` e `InvokePattern.Invoke`, sin portapapeles, teclas globales ni ruta enviada a IPC.
4. Confirmar desde la UI un paper sintético con título `T03 PDF sintético nativo — dos páginas`; observar el nombre de archivo del preview y la ficha resultante en Biblioteca. Abrirlo con `Leer`, usando el backend real.
5. En la WebView real, comprobar el fetch del URL de documento `research.localhost`: estado 200, `application/pdf`, longitud declarada igual a bytes recibidos, cabecera PDF y marcador EOF. Confirmar que PDF.js informa dos páginas y que los canvas de página 1 y página 2 tienen dimensiones y píxeles oscuros muestreados; esto distingue páginas dibujadas de un elemento vacío. Guardar captura.
6. Confirmar en página 1 una regla raster exclusiva de esa página; avanzar a página 2 y confirmar su regla exclusiva en canvas antes de medir. Registrar por separado dimensiones CSS y raster, subir zoom a 110% y medir ambos cambios. El texto `110%` no se toma como aprobación visual; una escala inesperada queda como hallazgo y no corta las observaciones de persistencia. Esperar `Guardado`, cerrar y reabrir desde Biblioteca usando identidad exclusiva de página 2; exigir el marcador raster de página 2 además del estado de página/zoom y guardar capturas.
7. Recoger browser logs cuando WebDriver los soporte, recursos cargados, respuesta del protocolo, eventos de ciclo de vida y capturas. Cerrar la misma instancia mediante `CloseMainWindow`; sobre el mismo `Process`/`SafeHandle`, esperar y leer `ExitCode` antes de borrar la sesión. El cleanup registra resultados visibles, comprueba sesión, app, procesos WebView correlacionados por PID/creación/perfil, driver, EdgeDriver y listeners; cualquier comprobación fallida degrada PASS. Solo termina procesos propios identificados; conserva run y evidencias.

No se usa un endpoint de seed ni se invoca `selectPdf` con una ruta simulada. La UI conduce toda la importación; PDF recibido, Paper y documento deben venir del backend vivo.

## Selector nativo: límite actual

La carga de `UIAutomationClient` y `UIAutomationTypes` desde el PowerShell disponible sí funcionó. Los documentos de Microsoft describen la obtención de elementos UIA desde un HWND y la búsqueda acotada dentro del árbol de esa ventana mediante `FindFirst`/`FindAll` ([Obtaining UI Automation Elements](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-obtainingelements), [AutomationElement.FromHandle](https://learn.microsoft.com/en-us/dotnet/api/system.windows.automation.automationelement.fromhandle?view=windowsdesktop-10.0)). Esto acredita disponibilidad de las API locales, **no** que el diálogo concreto del plugin Tauri exponga los controles esperados ni que su HWND/ownership sea observable por el proceso.

Por instrucción, todavía no se lanzó el candidato; por eso la automatización del selector sigue sin estar demostrada. Si al ejecutar tras la revisión no aparece una ventana UIA asociada por PID/owner, el runner termina marcado `blocked` antes de cargar/confirmar y registra la limitación. Se propone entonces una fixture seed etiquetada solo para comprobar lector/protocolo/persistencia, y un paso manual separado para verificar selector→preview→confirmación real. Ese seed no contará como prueba de importación ni sustituirá el picker; no se preparó ni ejecutó una ruta bypass.

## Comandos y resultados de preparación

Comandos ejecutados:

```powershell
pwsh -NoProfile -File .\work\qa\t03-native\make-synthetic-pdf.ps1
pwsh -NoProfile -File .\work\qa\t03-native\native-qa.ps1
```

Resultados observados: generador exit 0 con PDF de 968 bytes y SHA-256 indicado arriba; runner exit 0 con gate de lanzamiento activo y hashes/versiones esperadas. El análisis estático del parser PowerShell no reportó errores. No se creó sesión WebDriver, no se inició la app, no se abrió selector, no hubo importación/renders ni se inspeccionó AppData personal. No se ejecutaron pruebas de producto.

Ejecución posterior a revisión, si el orquestador la autoriza:

```powershell
pwsh -NoProfile -File .\work\qa\t03-native\native-qa.ps1 -ExecuteAfterReview
```

El run escribe JSON, logs, capturas y datos exclusivamente bajo una nueva carpeta `work/qa/t03-native/native-run-<GUID>`. No elimina esa carpeta. El cleanup se limita a la sesión WebDriver y a procesos cuyo ejecutable/PID/ruta fueron identificados como propios.

## Autorrevisión y limitaciones

- Mantiene la separación entre prueba UI simulada del worker y evidencia de ejecución nativa. El navegador/WebView real y la respuesta del protocolo son condiciones explícitas para afirmar importación/render.
- La fixture es sintética y propia. Las comprobaciones visibles del reader exigen página 2, zoom 110%, estado Guardado, reapertura y canvas no vacío; no se aceptará HTML del PDF como render.
- El runner fija hashes y bloquea el arranque por defecto. La validación de sintaxis/preflight no demuestra que el runner automatice correctamente el diálogo, la WebView o la app.
- Estado actual: pendiente de revisión focal independiente del hash actual del runner y validación del orquestador antes de su única ejecución exploratoria; ningún resultado QA T03 nativo se afirma todavía.

## Delta de revisión focal (seis hallazgos)

1. Se eliminó la colisión de mayúsculas de PowerShell con la variable automática `$PID`: los PIDs de ventanas usan `$windowProcessId`. Las excepciones UIA se capturan como errores de automatización; solo un timeout de búsqueda del selector queda clasificado como no automatizable.
2. La navegación coincide con el DOM de HEAD `b36ae5a`: espera «Esta es tu biblioteca local» y «Entendido», navega desde Home con «Abrir biblioteca», espera el estado vacío de Biblioteca y tras guardar exige el artículo exacto `.paper-card`, botón «Leer» habilitado y acción/modal cerrado. La reanudación tolera nodos aún no montados, sin ocultar fallos persistentes.
3. La fixture aporta líneas distintas por página; las mediciones se toman después de avanzar y tras reabrir con la línea exclusiva de página 2. El informe registra por separado geometría CSS y raster. Un desacuerdo de escala es hallazgo visual; la etiqueta «110%» no basta para aprobar apariencia.
4. El estado final depende de limpieza observada de sesión, aplicación, WebView2 correlacionado, driver, EdgeDriver y listeners. Se vuelven a consultar PIDs/puertos; cualquier error o incertidumbre degrada PASS antes de JSON/código de salida.
5. El perfil requiere archivos y al menos un proceso WebView con argumento de perfil y ascendencia a la app retenida. Una capability vacía se registra como limitación y se permite solo con evidencia positiva; capability distinta o falta de correlación degrada el resultado.
6. Antes de crear sesión se exige ausencia de la ruta exacta del ejecutable. La app propia debe nacer dentro del intervalo de sesión y descender del driver. Si la creación de sesión falla, la limpieza recupera solo una instancia única demostrada por esos mismos criterios; los casos ambiguos se dejan intactos.

El ejecutable original `candidate-6600b994`, los snapshots `371111F0` y `DCF3A5FD`, y la fixture se conservan como evidencia. Runner preparado para el candidato nuevo: `work/qa/t03-native/native-qa.ps1`, SHA-256 actual `67898C9C201032B64F3B8EC86CF439541FCE6909D5585BBD297A7150B9231D07`. No ejecutar el candidato hasta completar su revisión focal.

## Delta de revisión focal 2

La revisión `task-03-native-qa-fix-1-review.md` detectó que `Get-NetTCPConnection` genera `CmdletizationQuery_NotFound` cuando un puerto no tiene listeners. El helper ahora convierte únicamente ese ID en colección vacía y relanza los demás errores. La espera tras cerrar el onboarding ahora exige que el botón habilitado «Abrir biblioteca» aparezca, esperando a que Home termine de resolver el último documento antes de navegar.

Pruebas offline del helper extraído del AST del runner, sin iniciar app, driver o sesión: puerto efímero cerrado devolvió `Count=0`; listener loopback creado por la prueba devolvió `Count=1`, PID propio y puerto coincidente; tras detenerlo devolvió `Count=0`. Una excepción simulada ajena a `CmdletizationQuery_NotFound` se propagó correctamente. El parser PowerShell sigue sin errores.

El preflight sin `-ExecuteAfterReview` se intentó contra el nuevo SHA, pero ahora termina en `Edge version drift`: Edge instalado es `154.0.4258.53` y WebView2 registrado `154.0.4258.48`, mientras el runner mantiene la pareja fijada `.48`. No se relajó ese pin. Esto es estado cambiante del entorno y queda notificado a root para su delta review; no se inició aplicación, driver ni selector.

## Delta de revisión focal 3

Se corrigió la comprobación mal dirigida de versión: Edge navegador se registra (`154.0.4258.53`) pero no condiciona preflight, ya que no es el navegador WebDriver de esta prueba. Antes del gate de preparación, el runner exige hashes fijados de app/driver, EdgeDriver `154.0.4258.48`, versión registrada del runtime WebView2 `154.0.4258.48`, existencia de `msedgewebview2.exe` en la carpeta registrada y coincidencia de su `ProductVersion`. Estos controles pasaron sin instalar ni modificar software.

Tras crear sesión, cada proceso WebView2 correlacionado registra ruta ejecutable y `ProductVersion`; evidencia de perfil solo cuenta completa cuando coincide con la ruta/runtime registrado y la versión `154.0.4258.48`, además de los controles de perfil anteriores.

Resultado de preparación previo a la autorización de ejecución: parser PowerShell sin errores; preflight exit 0, `prepared; app launch intentionally gated`, con Edge `.53`, EdgeDriver `.48`, registro WebView2 `.48` y ejecutable WebView2 `.48`. Hash candidato `871187877DCA988FFDC0560C0C9B2C30A295F4B721005213626C3462A092C481`; SHA runner `3006ECAA112C717F5D3FF8A6BB81B5DA43F2DA4945D2610EEC955DFA08C9EC52`.

## Ejecución exploratoria autorizada

**Fecha:** 2026-10-02. **Única ejecución:** `pwsh 7.6.5 -NoProfile -File work/qa/t03-native/native-qa.ps1 -ExecuteAfterReview`. Se verificaron antes de lanzar runner `3006ECAA112C717F5D3FF8A6BB81B5DA43F2DA4945D2610EEC955DFA08C9EC52` y candidato `871187877DCA988FFDC0560C0C9B2C30A295F4B721005213626C3462A092C481`. No se cambió el runner durante el recorrido ni se reintentó.

- Exit code: `1`; duración 27,4 s (15:03:30.316Z–15:03:57.722Z). Resultado: `blocked: native OS selector automation unavailable; UIA-scoped exact-path method failed` en `choose exact synthetic PDF through app-owned Windows picker`.
- Artefactos de captura de proceso: `work/qa/t03-native/evidence/exploratory-510bcea2886d4c29836bbfd9608293e7/stdout.log` (154.111 bytes), `stderr.log` (0 bytes), `exitcode.txt` (1 y marcas de tiempo). El stdout se conserva localmente y contiene el JSON/WebDriver que incluye respuesta de screenshot codificada; no se volcó a la conversación.
- JSON de resultado: `work/qa/t03-native/native-run-0c31da2bcfed42f0bcd918ae06ebef29/native-result.json`. Captura anterior al picker, mostrando Biblioteca vacía: `01-library-empty.png`, SHA-256 `DFC3D5EE5E373955056416E44F1FEE5A47A075ACFAC2298800E91CFC4F316064`. Logs de `tauri-driver` en el mismo run: `driver.stdout.log` y `driver.stderr.log`, ambos 0 bytes; `logs/desktop.log` registra `started`, sin evento `stopped` visible.
- Evidencia positiva previa al bloqueo: sesión WebDriver creada; Home resuelto, onboarding «Entendido» cerrado, navegación a Biblioteca vacía completada y captura guardada; clicks de UI para «Importar PDF» y «Seleccionar PDF» respondieron HTTP 200. Luego la búsqueda UIA acotada no encontró una ventana con controles Edit/Button para PID `47344` o HWND con `GW_OWNER` directo; `pickerWindow=null`, `uiaErrors=[]`. La espera expiró. Por ello el resultado identifica exactamente la limitación del runner/UIA, pero no distingue si el picker no apareció o si Windows lo presentó fuera del ownership/UIA observado. No se afirma defecto de producto ni importación.
- Perfil/runtime observados: capability `userDataDir` vacía; 137 archivos de perfil y seis procesos WebView correlacionados por perfil/ascendencia, todos ejecutando `C:\Program Files (x86)\Microsoft\EdgeWebView\Application\154.0.4258.48\msedgewebview2.exe`, `ProductVersion 154.0.4258.48`. `profileEvidenceComplete=true` y `webviewRuntimeEvidenceComplete=true`.
- No se llegó a fetch de documento, importación/confirmación, render PDF.js, cambio de página/zoom, reapertura o cierre normal. `CloseMainWindow`, espera y `ExitCode` están `null`/`not-read`; la sesión se borró con HTTP 200 después del bloqueo. No se registra cierre normal como aprobado.
- Cleanup del runner: sesión eliminada; app ausente; seis procesos WebView, EdgeDriver hijo PID `38448` y driver ausentes; `cleanup.complete=true`, `errors=[]`, `portsClear=true`, `portListenersAfter=[]`. Verificación independiente inmediata encontró sin procesos la ruta exacta del candidato, sin PIDs del run y sin WebView que portase el perfil de este run. El JSON no conserva los números de los puertos escogidos, por lo que la evidencia de puertos se limita a la consulta de listeners vacía del cleanup.

Siguiente opción acotada si se decide continuar: un paso manual del selector de Windows para demostrar importación UI real; alternativamente una fixture seed claramente marcada puede explorar lector/protocolo/persistencia, pero no contará como importación ni como bypass del picker. Esta ejecución no aprueba producto T03, instalación ni integración.

## Corrección focal del picker — pendiente de revisión antes de segundo intento

El diagnóstico `task-03-picker-diagnosis.md` detectó que el helper inicial filtraba primero por PID de app o `GW_OWNER` de app, pero luego excluía las ventanas del mismo PID sin owner directo. El timeout es compatible con ese defecto del runner, aunque no es una causa observada en la primera ejecución. Se preservan sin cambios el snapshot `3006ECAA…` y el resultado original `native-run-0c31da2bcfed42f0bcd918ae06ebef29/native-result.json` (SHA-256 `05992D846834B05F8C42127BA8DCDEB5D0D0363F922F33FDC2F45625C9E58A7F`).

Cambios acotados en el runner actual `AB334368DD6DF34B8A981622E4F4E5F18D5B869F08DB3864DBFD23D9C9C6DF6C`:

- Antes de clicar «Seleccionar PDF», valida y conserva el `MainWindowHandle` del `Process` retenido y comprueba que su PID sea el mismo. El picker solo considera HWND cuyo PID sea la app o cuyo owner directo sea la app; excluye expresamente ese HWND principal y ya acepta otro HWND del PID propio aunque no tenga owner.
- Para esos HWND propios, registra handles/owners, PID, clase/título, clasificación principal/candidato y controles Edit/Button por `AutomationId`, `Name`, tipo y disponibilidad de Value/InvokePattern. No lee ni guarda valores de campos. Deduplica por estado serializado para no repetir inventario en cada poll. Errores UIA reales siguen siendo errores; candidatos ambiguos detienen el flujo.
- Solo acepta un diálogo con un único control de nombre de archivo (ValuePattern) y un único botón de aceptación (InvokePattern); mantiene selección de la ruta sintética exacta.
- Añade los dos puertos escogidos (`webDriver`, `native`) al JSON de resultado para auditoría posterior.

Validación posterior al cambio, **sin iniciar candidato, driver ni sesión**: parser PowerShell con 0 errores; 23 literales JavaScript compilan con `new Function`; preflight exit 0 en `2026-10-03T02:35:48Z`, runner hash actual, candidato `.exe`/fixture/tauri-driver/EdgeDriver intactos, Edge `154.0.4258.53` registrado, EdgeDriver y WebView2 registro/ejecutable `154.0.4258.48`; salida `prepared; app launch intentionally gated`. El nuevo SHA requiere revisión focal independiente antes de considerar otro intento. No se preparó seed, paso manual ni cambio de producto.

## Intento autorizado 2 — abortado por preflight, aplicación no iniciada

Root autorizó un segundo intento único con runner `AB334368DD6DF34B8A981622E4F4E5F18D5B869F08DB3864DBFD23D9C9C6DF6C`, ejecutable `871187877DCA988FFDC0560C0C9B2C30A295F4B721005213626C3462A092C481` y fixture `A65C300B22EBE0A8CDCD7253C47BE5B8752680470B9D3B89F6BD4A5D57703E95`. Las tres huellas seguían exactas. El preflight de `pwsh 7.6.5 -NoProfile` terminó con exit `1` el 2026-10-03 a las 03:04 UTC antes de producir JSON; mensaje exacto: `WebView2 registry runtime version drift: 154.0.4258.53`. Conforme a la guarda, no se ejecutó `-ExecuteAfterReview`; no se creó run nativo, sesión, driver ni proceso de app.

La lectura independiente a las 03:05:08 UTC confirmó el nuevo registro WebView2 `154.0.4258.53`, ubicación `C:\Program Files (x86)\Microsoft\EdgeWebView\Application`, y ejecutable presente `...\154.0.4258.53\msedgewebview2.exe`, `ProductVersion 154.0.4258.53`. EdgeDriver conserva `154.0.4258.48`; Edge navegador está en `.53`. La ruta exacta del candidato no tenía procesos. Evidencia del aborto (stdout preflight vacío; stderr con excepción y archivo de exit code): `work/qa/t03-native/evidence/exploratory-fc84863a726640c79a22844daf2ccfcf/`. No se relajó pin, ni se instaló o modificó sistema, ni se reintentó.

## Delta de preflight focal 4 — EdgeDriver/WebView2 `.53` (sin ejecución de app)

El preflight previo abortó porque el runtime registrado cambió a `154.0.4258.53`. Se releyó el registro WebView2 y el ejecutable versionado antes del ajuste: ambos son `154.0.4258.53` en `C:\Program Files (x86)\Microsoft\EdgeWebView\Application\154.0.4258.53\msedgewebview2.exe`. El directorio `.48` permanece intacto. Microsoft documenta que EdgeDriver debe corresponder al runtime WebView2 usado ([WebView2 WebDriver](https://learn.microsoft.com/en-us/microsoft-edge/webview2/how-to/webdriver)); el paquete se obtuvo del host oficial enlazado por el sitio de descargas de Microsoft ([Edge WebDriver downloads](https://developer.microsoft.com/en-us/microsoft-edge/tools/webdriver/), [paquete x64 `.53`](https://msedgedriver.microsoft.com/154.0.4258.53/edgedriver_win64.zip)).

Se preservó antes del cambio el runner AB334 en `work/qa/t03-native/evidence/native-qa-ab334-before-edgedriver53.ps1`, SHA-256 `AB334368DD6DF34B8A981622E4F4E5F18D5B869F08DB3864DBFD23D9C9C6DF6C`. El ZIP oficial local `work/tools/msedgedriver/154.0.4258.53/edgedriver_win64.zip` tiene SHA-256 `16ACCCCA3DD71DA28348A5A6852B9FA681D1F177F758339EABACFBD3CAC8D72B`. Se extrajo solo `msedgedriver.exe` a la carpeta de versión; SHA-256 `008115B68B38437B1C0138F3CCED648F61F333CF4623A9895296E7C1C01ABCB8`, `ProductVersion`/`FileVersion` `154.0.4258.53`, Authenticode `Valid`, firmante `CN=Microsoft Corporation, O=Microsoft Corporation, L=Redmond, S=Washington, C=US`. Driver `.48` queda conservado en su ruta original. No se instaló ni actualizó el runtime, no se cambió PATH/configuración global ni se modificó producto.

Cambio mínimo al runner: cuatro constantes únicamente — ruta del EdgeDriver, hash fijado del ejecutable, pin de versión EdgeDriver y pin de versión WebView2— ahora apuntan a `.53`. Candidate, fixture, tauri-driver, lógica del picker, selector, recorrido y cleanup no cambiaron. Nuevo SHA-256 del runner `874F2552F3D11A61531C922A50DBD191E9A38612BF6566FEB7CF1530D82BFB19`.

Validación el `2026-10-03T05:05:35Z`: parser PowerShell con cero errores; los 23 literales JavaScript pasan `new Function`; `pwsh 7.6.5 -NoProfile -File work/qa/t03-native/native-qa.ps1` (sin `-ExecuteAfterReview`) exit `0`, `prepared; app launch intentionally gated`. El preflight verificó hashes congelados del candidato `871187877DCA988FFDC0560C0C9B2C30A295F4B721005213626C3462A092C481`, fixture `A65C300B22EBE0A8CDCD7253C47BE5B8752680470B9D3B89F6BD4A5D57703E95`, tauri-driver `2684A7B9E1E667D6689BEA8EAFF2AEFE093C5B1F5CA329B3156B384A9CAB9286` y EdgeDriver `.53`; registró Edge, EdgeDriver, WebView2 de registro y ejecutable todos `.53`. Salida y procedencia capturadas en `work/qa/t03-native/evidence/preflight-20261003-0505-edgedriver53/`: `stdout.json` SHA-256 `B2145D0A4A40CFF3A603D690A0360CB3C082D038CFE9AB53214D1EFC84292AEA`, `stderr.log` vacío (SHA-256 `E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855`), `exitcode.txt`, `provenance.json` SHA-256 `A1EF4AEF2B46A6C145E1326B163309F06AE67A33489C4F8F2AC9CA63EBE13E5B`. Comprobación de solo lectura al final: cero procesos del ejecutable congelado.

Estado: preparado y pendiente de revisión focal del cambio de pin/ruta/hash. No se ejecutó aplicación, WebDriver, selector ni recorrido, y esta evidencia de preparación no aprueba producto ni autoriza por sí sola la siguiente ejecución.

## Segundo intento autorizado — picker visible, helper no identifica controles

**Fecha UTC:** `2026-10-03`, 05:09:02.678–05:09:29.056. Se revalidaron inmediatamente antes del intento los SHA revisados del runner (`874F2552F3D11A61531C922A50DBD191E9A38612BF6566FEB7CF1530D82BFB19`), candidato fix1 (`871187877DCA988FFDC0560C0C9B2C30A295F4B721005213626C3462A092C481`), fixture (`A65C300B22EBE0A8CDCD7253C47BE5B8752680470B9D3B89F6BD4A5D57703E95`) y EdgeDriver `.53` (`008115B68B38437B1C0138F3CCED648F61F333CF4623A9895296E7C1C01ABCB8`), y registro/ejecutable WebView2 `.53`. Preflight exit `0`; se ejecutó una sola vez el comando autorizado `pwsh 7.6.5 -NoProfile -File work/qa/t03-native/native-qa.ps1 -ExecuteAfterReview`, sin modificar el runner.

El resultado termina exit `1`, `blocked: native OS selector automation unavailable; UIA-scoped exact-path method failed`, en `choose exact synthetic PDF through app-owned Windows picker`. Mensaje: `nativePickerNotAutomatable: Timed out waiting for: native PDF picker owned by the app`. Hay evidencia de que el selector sí apareció: UIA enumeró 26 ventanas y encontró HWND `0x230F50`, clase `#32770`, título «Abrir», PID `16704` (el mismo PID de la app), owner PID `0`. Por tanto, no es evidencia de que el picker no apareciera ni de que Windows lo hubiera presentado fuera del PID de la app.

El inventario de ese HWND contiene 63 elementos `ControlType.Edit`: `SearchEditBox`/«Cuadro de búsqueda», `System.ItemFolderPathDisplay`/«Ubicación del archivo», celdas repetidas de `System.ItemNameDisplay`/«Nombre» (16), `System.DateModified`/«Fecha de modificación» (15), `System.ItemTypeText`/«Tipo» (15) y `System.Size`/«Tamaño» (15). No hay un control único coincidente con AutomationId `1148` ni con los nombres configurados para el nombre de archivo; las celdas «Nombre» son metadatos de los elementos listados, no evidencia del campo de nombre de archivo. El inventario de `ControlType.Button` contiene diez acciones de ayuda, vista, organizar, filtro y búsqueda; no muestra botón «Abrir»/`Open`/«Seleccionar». Así el helper no seleccionó una ruta ni invocó aceptar.

**Diagnóstico del helper y límite causal:** `Get-DialogForApp` obtiene `AutomationElement.FromHandle(hwnd)`, usa `FindAll(TreeScope.Descendants, PropertyCondition(ControlType.Edit/Button))`, y prueba patrones con `$candidate.TryGetCurrentPattern(ValuePattern.Pattern, [ref]$valuePattern)` y su equivalente Invoke. La firma y el contrato de `TryGetCurrentPattern` son `bool` + objeto `out`: Microsoft dice que devuelve `true` cuando el patrón está soportado y el objeto `null` si no ([documentación API](https://learn.microsoft.com/en-us/dotnet/api/system.windows.automation.automationelement.trygetcurrentpattern?view=windowsdesktop-10.0)). La invocación no muestra un error sintáctico obvio de PowerShell; una comprobación aislada de `TryParse(..., [ref]$value)` también conserva el retorno `System.Boolean`, por lo que no atribuyo los `true` a un cast general de PowerShell. Sin embargo, el JSON registra `valuePattern=true` y `invokePattern=true` para los diez botones (también `valuePattern=true` en los 63 Edit) y no conserva tipo del objeto out ni valores `Is*PatternAvailable`. Esto es incoherente con la expectativa de que una barra de botones de Shell necesite ValuePattern; no permite determinar si el proveedor UIA lo anuncia, si el objeto devuelto es realmente un `ValuePattern`, o si hubo una anomalía de marshaling. No se leyó ningún valor de campo.

La evidencia no incluye el árbol UIA completo (solo descendientes filtrados por Edit/Button) ni una captura del diálogo. Hipótesis acotada para una futura inspección: controles filename/accept podrían estar expuestos con otro `ControlType`/fragmento UIA o fuera del subárbol de control consultado; el resultado presente no discrimina estas opciones ni acredita un defecto de producto. Delta sugerido solo para el runner, pendiente de instrucciones: en el mismo HWND/PID, inventariar de forma acotada y redactada todos los nodos ControlView/RawView con tipo, AutomationId, Name, patrón disponible y HWND/PID; registrar por separado `IsValuePatternAvailable`/`IsInvokePatternAvailable`, el tipo real del objeto `out` y comprobar cast a `ValuePattern`/`InvokePattern`; identificar filename y aceptar por controles reales, normalizar mnemonic `&` si aparece, mantener unicidad estricta y no guardar `Value`. No usar clipboard, teclas globales, ruta IPC, selector manual ni seed. No se editó el script ni se abrió otra app/ventana.

Progreso observable antes del bloqueo: WebDriver `/status`, creación de sesión, llamadas UI/element/click, lectura de Biblioteca y screenshot devolvieron HTTP 200; captura de Biblioteca vacía `01-library-empty.png`, SHA-256 `E807EC7ABAADBCA8E1464E149B31E366DEF28A590F3B7314B0DE7214036E4F60`. Perfil: seis procesos WebView correlacionados y `profileEvidenceComplete=true`, `webviewRuntimeEvidenceComplete=true`; capability `userDataDir` vacía. Se observaron Edge/WebView2 `.53`. No hubo confirmación de importación ni fetch del PDF, observación de protocolo PDF, canvas, página/zoom, reapertura, ni cierre normal/ExitCode app; esos campos son null/no leídos. WebDriver registra DELETE session HTTP 200.

Resultado JSON `work/qa/t03-native/native-run-4a2c62d2ae2d471388eef9a1905381c4/native-result.json` (179603 bytes, SHA-256 `CBA6870E4EEC5D37FF68FAC8F26DE401A0EC903D3BA0AE718AF2FE161A231CC1`). La captura stdout coincide byte por byte con el JSON y puede contener cuerpos WebDriver codificados; se conserva en `work/qa/t03-native/evidence/exploratory-20261003-second/stdout.log` (179603 bytes, SHA-256 `CBA6870E4EEC5D37FF68FAC8F26DE401A0EC903D3BA0AE718AF2FE161A231CC1`); stderr vacío (SHA-256 `E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855`), `exitcode.txt=1`. Puertos asignados WebDriver/native `35501`/`35502`; cleanup informa sesión eliminada, app ausente, seis WebView ausentes, EdgeDriver PID `3268` finalizado, `errors=[]`, `portsClear=true`, `complete=true`. Verificación externa posterior confirma cero listeners en ambos puertos y cero procesos con las rutas exactas del candidato, tauri-driver o EdgeDriver. Sin reintentos ni cambios posteriores al runner.

Conclusión: el intento ejercitó inicio nativo, navegación a Biblioteca vacía y abrió el selector de Windows; terminó en una insuficiencia observable del descubrimiento/identificación UIA antes de seleccionar el PDF. No determina si el helper de controles, la superficie UIA del diálogo o ambos explican la falta de controles aceptables. No aprueba importación, render PDF.js, persistencia ni producto T03.
