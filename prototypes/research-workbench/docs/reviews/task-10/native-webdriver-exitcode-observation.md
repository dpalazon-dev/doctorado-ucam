# Observación de salida de la app con WebDriver nativo

Fecha: 2026-10-02. Intento único autorizado después de revisión independiente PASS (`585C6FE7`). Alcance: confirmar comportamiento de `CloseMainWindow` y lectura del código de salida sobre la copia congelada T02; no es prueba de instalación.

## Resultado observado

La sesión WebDriver real se creó correctamente (`tauri-driver` + EdgeDriver/WebView2), mostró `Research Workbench` y abrió Configuración. El contenido observado incluyó `CONFIGURACIÓN`, `Estado` y `Disponible`; se guardó captura. Se mantuvo el mismo objeto `Process` y su `SafeHandle` válido. `CloseMainWindow()` devolvió `True`; `WaitForExit(15000)` devolvió `True` tras 17.2078 ms; el getter de `ExitCode` vía reflexión devolvió valor `0` sin excepción, antes del `DELETE session`. Por tanto, este intento sí demuestra cierre normal observado con código 0. El log sintético también contiene `started` y `stopped`.

El resultado JSON informa `sessionCreated=true`, `sessionDeleted=true`, `normalClose=true`, sin fallo. Tras cleanup, el driver propio y sus hijos EdgeDriver ya no estaban; el proceso de app de ruta exacta tampoco estaba. Las consultas de cleanup usaron errores visibles (`ErrorAction Stop`); no se encontraron listeners de los puertos exclusivos 7463/7464.

## Reproducibilidad y artefactos

Run: `work/qa/t02-native-smoke/exitcode-observer-20261002-99ebc6abae74423a8de779b75c5fb5cb/`

Comando ejecutado una vez:

```powershell
pwsh -NoProfile -File .\work\qa\t02-native-smoke\exitcode-observer-20261002-99ebc6abae74423a8de779b75c5fb5cb\native-smoke.ps1
```

El comando terminó con exit code 0. Versiones registradas: PowerShell 7.6.5, .NET 10.0.11, WebView2/EdgeDriver 154.0.4258.48. SHA-256:

- Script revisado: `585C6FE7873D2839AE4FFF36746627A2CC3192A34E5FF28844BDB61532890A10` (`native-smoke.ps1`).
- Binario congelado: `C3FB989E17B20640FB4C530CC08959CDC839A72E5DF54E5442827F20837924D8` (`research-workbench.exe`).
- tauri-driver: `2684A7B9E1E667D6689BEA8EAFF2AEFE093C5B1F5CA329B3156B384A9CAB9286`.
- EdgeDriver: `4032E9D74DA42B30805EC5125EEAF9CBD4F678C2A41104EE90471BE16CAAD997`.
- Captura: `settings-smoke.png`, `9E0E6BB3F87B4B6EE385A681DFD3FF76DCF135C1DA4BC42D868C65447C879506`.

Evidencia conservada en `smoke-result.json`, `logs/desktop.log`, `console.stdout.log`, `console.stderr.log`, `driver.stdout.log`, `driver.stderr.log` y `settings-smoke.png`. `driver.stderr.log` contiene el diagnóstico Chromium `Failed to unregister class Chrome_WidgetWin_0. Error = 1412`; pese a ello, sesión y proceso terminaron según lo descrito y cleanup verificó ausencia de procesos/listeners propios.

## Límites

La capability devuelta `msedge.userDataDir` fue vacía (`webviewProfileCapabilityMatches=false`), así que no se afirma aislamiento demostrado por la capability. Sí se observaron 129 archivos bajo el directorio de perfil de este run y seis procesos WebView2 correlacionados con el PID de la app y el `--user-data-dir` de ese run. Esto acredita actividad del perfil solicitado, pero no que todas las escrituras de Tauri estén aisladas; no se inspeccionó ni modificó AppData personal.

No se comprobó liberación del lock de biblioteca, lector/PDF, picker nativo, instalador ni ejecución en equipo limpio. No se inspeccionó ninguna biblioteca personal. El intento anterior ambiguo permanece descrito en `work/native-webdriver-isolation-close.md`; este nuevo resultado no reinterpreta su evidencia.
