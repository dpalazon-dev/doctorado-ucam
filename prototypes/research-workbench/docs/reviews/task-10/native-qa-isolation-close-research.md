# Investigación: aislamiento y cierre de QA nativo

Estado: **DONE_WITH_CONCERNS**. Fecha: 2026-10-02. Agente: `/root/windows_handle_ruling`.

## Alcance y decisión propuesta

Investigación de solo lectura sobre el smoke T02 y fuentes fijadas. No se ejecutó la aplicación, no se modificó producto, no se inspeccionó el producto T03 mutable y no se crearon subagentes. Baseline de producto: `git show c4436bd:<ruta>`. La propuesta requiere una ejecución QA posterior; este informe no acredita sus resultados.

El siguiente intento necesita dos cambios mínimos en el script existente: solicitar el perfil mediante `tauri:options.webviewOptions.userDataFolder`, y solicitar el cierre nativo mediante `System.Diagnostics.Process.CloseMainWindow()` sobre el proceso propio antes de borrar ventanas o sesiones WebDriver. No requiere un nuevo harness, IPC de producto, permiso de ventana ni override de perfil en release.

## Evidencia observada

Artefactos: `work/qa/t02-native-smoke/reviewed-attempt-20261002-057cfd3016ea4a41b5343cfda4877436/`. Script revisado: SHA256 `7DE976F3B00CCA80F14721EE9F97DAA27073004D4CDB756A3DE4F0943E8A2B6A`. Ejecutable congelado T02: SHA256 `C3FB989E17B20640FB4C530CC08959CDC839A72E5DF54E5442827F20837924D8`.

`smoke-result.json` registra sesión real, PID de aplicación 41808 y Settings real. La solicitud de sesión contiene application/args, pero no `webviewOptions`; la respuesta indica `msedge.userDataDir=""`. El directorio sintético `webview2-user-data` no contiene entradas. Estos hechos prueban que el aislamiento solicitado solo mediante entorno no quedó acreditado; no identifican el perfil realmente usado ni prueban ausencia de escrituras fuera del run.

`DELETE /window` devolvió 200, pero la espera de 15 segundos no produjo salida observable con código 0: `normalClose=false`, `appExitCode=null`. Después, `DELETE /session` devolvió 200 y no se encontró el proceso de aplicación. El log sintético `logs/desktop.log` contiene únicamente `2026-10-02T13:22:33.034Z started`, sin `stopped`. La liberación del writer lock está explícitamente sin verificar. La UI pasa parcialmente; el cierre normal y el aislamiento efectivo siguen pendientes.

## Perfil: causa acotada y procedimiento

`tauri-driver 2.1.0/src/server.rs:31–68` deserializa `webviewOptions` y lo transfiere a `ms:edgeOptions.webviewOptions`. Solo adapta `POST /session` (`:79`); no convierte los otros comandos en acciones nativas de Tauri. La [documentación oficial de capabilities Edge](https://learn.microsoft.com/en-us/microsoft-edge/webdriver/capabilities-edge-options) define `webviewOptions.userDataFolder` como la selección del perfil WebView2; si falta, el driver puede usar un perfil temporal. Por tanto, la capability es el punto mínimo previsto para el perfil de este lanzamiento.

Tauri `2.12.1/src/manager/webview.rs:559–575` selecciona `app_local_data_dir()` si no hay `data_directory` y crea ese directorio. `src/path/desktop.rs` resuelve normalmente LocalAppData más el identificador. En `c4436bd:src-tauri/tauri.conf.json` el identificador es `local.researchworkbench.desktop` y no hay `dataDirectory`. Wry `0.57.0/src/webview2/mod.rs:293–299,349–357` pasa el directorio de contexto a `CreateCoreWebView2EnvironmentWithOptions`.

No se concluye que Wry o Tauri ignoren siempre `WEBVIEW2_USER_DATA_FOLDER`: [WebView2 documenta que el entorno puede sobrescribir el argumento](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/webview2-idl?view=webview2-1.0.3967.48). El driver administra el lanzamiento, y su binario no aporta aquí evidencia de la precedencia exacta aplicada. La ausencia de capability, el directorio vacío y la respuesta vacía son suficientes para corregir el procedimiento, pero no para atribuir con certeza el resultado a una única precedencia interna.

Procedimiento mínimo para el autor QA:

1. Conservar la copia congelada del ejecutable, el run único y la biblioteca sintética. El override `RESEARCH_WORKBENCH_TEST_ROOT` de `c4436bd:src-tauri/src/adapters/windows/paths.rs` está restringido a debug y solo aísla la biblioteca.
2. Dentro de `tauri:options`, solicitar `webviewOptions.userDataFolder` con la ruta absoluta del perfil sintético del run, además de application y args. El entorno puede conservar la misma ruta por coherencia, pero no cuenta como verificación.
3. Registrar la solicitud exacta y las capabilities recibidas. Comprobar el directorio sintético después de crear la sesión y correlacionar, cuando esté disponible, `--user-data-dir` en los procesos WebView2 descendientes del PID propio. Limitar lecturas a procesos propios y metadatos del run; no explorar perfiles personales.
4. Un `msedge.userDataDir` vacío no acredita ni invalida por sí solo el perfil. Si no se consigue evidencia positiva de la ruta efectiva, conservar resultado parcial. El accessor [ICoreWebView2Environment7.UserDataFolder](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2environment7?view=webview2-1.0.3595.46) permite consultar la ruta real dentro del proceso; añadir tal diagnóstico necesitaría una decisión separada, no está implícito en esta propuesta.

Límite: la capability aísla el perfil solicitado al driver, pero Tauri puede crear su directorio predeterminado antes de la creación del entorno. No se promete aislamiento de toda escritura del proceso. Evitar incluso esa creación exigiría suministrar explícitamente `WebviewBuilder::data_directory` en una composición debug revisada: no se propone introducir ahora ese cambio de producto ni una variable release que admita rutas libres. Tampoco se inspeccionó el directorio personal para averiguar qué ocurrió en el smoke anterior.

## Cierre: ruta nativa y procedimiento

Wry `0.57.0/src/webview2/mod.rs:230,257–266` crea un HWND contenedor con `WS_CHILD`, dentro de la ventana padre. Su `WindowCloseRequested` destruye ese contenedor (`:683–686`), con comentario explícito sobre `window.close` de JavaScript. Tao `0.37.1/src/platform_impl/windows/event_loop.rs:999–1006` transforma `WM_CLOSE` de la ventana nativa en `WindowEvent::CloseRequested`. Tauri-runtime-wry `2.12.1/src/lib.rs:4253,4350–4380` envía ese evento a los handlers y al callback de aplicación, respetando `prevent_close`.

En `c4436bd:src-tauri/src/lib.rs`, el callback CloseRequested llama a `allow_desktop_exit` y previene el cierre mientras se drena el actor. `c4436bd:src-tauri/src/desktop/lifecycle.rs` espera las operaciones, ejecuta shutdown, escribe `Stopped` y solo entonces llama al callback `app.exit(0)`. Esta es la ruta que el QA debe ejercitar. El permiso IPC de cierre no figura en `c4436bd:src-tauri/capabilities/main-local.json`; no hace falta añadirlo para un mensaje nativo.

La separación entre HWND hijo y ventana padre explica de forma consistente el resultado de `DELETE /window`, pero no se inspeccionó el código interno de EdgeDriver: la traducción exacta del comando permanece una inferencia. La desaparición posterior a DELETE session tampoco prueba CloseRequested. Además, `tauri-driver 2.1.0/src/main.rs:31–40` usa un Job Windows con kill-on-job-close; el cleanup del driver puede finalizar descendientes.

Procedimiento mínimo para el autor QA:

1. Completar la observación UI con la WebView y la ventana nativa presentes. No enviar antes `DELETE /window`, `window.close`, `app.exit`, destroy ni kill.
2. Obtener y refrescar el `Process` del PID propio, comprobar la ruta del ejecutable congelado y registrar `MainWindowHandle` no nulo y título. Usar `CloseMainWindow()` sobre ese proceso. La [API .NET](https://learn.microsoft.com/en-us/dotnet/api/system.diagnostics.process.closemainwindow?view=net-9.0) solicita el cierre a su ventana principal; devuelve si envió el mensaje, no si terminó el proceso. Puede devolver false cuando no hay ventana principal o está deshabilitada por un modal.
3. Esperar de forma acotada y registrar salida real/código 0 **antes** de DELETE session o de detener el driver. Correlacionar `Started` → `Stopped` en el log sintético. Si devuelve false o vence el plazo, declarar cierre no acreditado y preservar la evidencia.
4. Solo después ejecutar cleanup de sesión/driver/procesos propios, registrándolo por separado. Una desaparición provocada o posterior al cleanup no cambia `normalClose` a true.

Si `MainWindowHandle` no identifica correctamente la ventana principal, el fallback sería enumerar ventanas top-level y comprobar su PID con `GetWindowThreadProcessId` antes de enviar `WM_CLOSE`; no usar búsqueda solo por título ni broadcasts. No se necesita ese fallback para el primer intento mínimo. [WM_CLOSE](https://learn.microsoft.com/en-us/windows/win32/winmsg/wm-close) es una solicitud interceptable por la aplicación, adecuada para el comportamiento CloseRequested; no equivale a forzar la salida.

El lock requiere una comprobación independiente sobre la biblioteca sintética si forma parte del criterio QA. Ni código 0 ni ausencia del proceso sustituyen la evidencia explícita de reapertura/exclusión del lock. Tampoco este procedimiento prueba por sí mismo el diálogo de cierre ocupado.

## Fuentes locales fijadas y límites

Fuentes de crates: `C:/Users/david/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/{tauri-2.12.1,tauri-runtime-wry-2.12.1,wry-0.57.0,tao-0.37.1}`. Driver: `work/tools/cargo-home/registry/src/index.crates.io-1949cf8c6b5b557f/tauri-driver-2.1.0/src/{server.rs,webdriver.rs,main.rs}`. Producto únicamente mediante `git show c4436bd` de las rutas citadas.

No se ejecutaron experimentos nuevos. La propuesta concreta es revisable por el autor QA antes de su siguiente ejecución. No modifica contratos T02/T03 ni acredita selector nativo, lector PDF, instalación, equipo limpio, perfil personal intacto durante el intento anterior o cierre ocupado. Quedan por demostrar el perfil efectivo del siguiente run y el cierre normal anterior al cleanup.
