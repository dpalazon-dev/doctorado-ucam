# Diagnóstico del cierre nativo T02

Estado: **DONE_WITH_CONCERNS**. Fecha: 2026-10-02. Agente: `/root/native_close_diagnosis`.

## Resultado

La hipótesis prioritaria es un **falso negativo al leer ExitCode en el script QA**. La evidencia disponible no demuestra que `WaitForExit(15000)` devolviera false, ni que el proceso permaneciera vivo hasta DELETE session. Hay una ruta concreta en .NET/PowerShell que produce `appExitCode=null`, `normalClose=false` y `failure=null` aunque la espera haya terminado correctamente. Antes de cambiar producto debe medirse explícitamente el resultado booleano y conservarse un handle del proceso durante cierre y lectura del código.

No se encontró un join pendiente del actor después de `stopped`, ni workers/oplocks en las guardas F10. El teardown de Tauri/Wry contiene llamadas nativas potencialmente relevantes, pero no hay evidencia que las identifique como bloqueo de este intento.

## Alcance y fuentes

Producto leído exclusivamente mediante `git show c4436bd:<ruta>` y búsquedas `git grep ... c4436bd -- src-tauri/src`. No se inspeccionó T03 mutable, no se ejecutó producto, prueba nueva ni subagente. Única escritura: este informe, autorizada por el orquestador. Leídos INTENT, STATUS, brief T02, investigación previa y las reglas de cierre/evidencia de SPECS y QUALITY. La búsqueda ligera en MEMORY.md no produjo resultados relevantes; no se utilizó memoria histórica para conclusiones.

Evidencia del intento: `work/qa/t02-native-smoke/profile-close-attempt-20261002-4ecddb6dca5f483b8402a09728e19059/`. Ejecutable SHA256 `C3FB989E17B20640FB4C530CC08959CDC839A72E5DF54E5442827F20837924D8`; script registrado SHA256 `C168CE9FCEB317F04702FDD2F6C5B5B1B93D434B2B572400EEA32DE5AE840684`.

## 1. Observación incompleta de salida: hipótesis de mayor prioridad

`native-smoke.ps1:41–45` crea el observador con Get-Process. En `:75` lo sustituye por otro Get-Process inmediatamente antes del cierre. `:76` obtiene MainWindowHandle, que es un HWND de ventana, no el handle retenido del proceso. `:78` guarda ExitCode solo dentro de un if sobre WaitForExit; no guarda el booleano ni el tiempo transcurrido. Por ello los campos null/false no identifican de forma unívoca la rama tomada.

La fuente [.NET 10.0.11 Process.Windows.cs](https://github.com/dotnet/runtime/blob/v10.0.11/src/libraries/System.Diagnostics.Process/src/System/Diagnostics/Process.Windows.cs#L149-L171) muestra que WaitForExitCore usa un handle temporal y lo desecha. [Process.cs](https://github.com/dotnet/runtime/blob/v10.0.11/src/libraries/System.Diagnostics.Process/src/System/Diagnostics/Process.cs#L922-L933) exige `_haveProcessHandle` para ExitCode; SafeHandle/Handle abren uno duradero mediante GetOrOpenProcessHandle. Obtener un proceso por PID no lo abre automáticamente. La sesión de inspección tiene PowerShell 7.6.5 y .NET 10.0.11; el wrapper del intento lanza `pwsh` pero no registró su versión.

Además, [PowerShell 7.6.5 Binders.cs:5028–5041](https://github.com/PowerShell/PowerShell/blob/v7.6.5/src/System.Management.Automation/engine/runtime/Binding/Binders.cs#L5028-L5041) captura excepciones ordinarias de getters de propiedades y devuelve null. Así, una InvalidOperationException del getter ExitCode puede quedar como null sin entrar en el catch exterior, incluso con ErrorActionPreference=Stop. Este mecanismo explica todos esos campos sin necesitar un bloqueo del producto. No se reprodujo aquí para no ejecutar nuevos experimentos.

La cronología aporta evidencia adicional:

| Hecho | UTC |
|---|---|
| started en desktop.log | 13:44:03.785 |
| stopped en desktop.log | 13:44:05.925 |
| CreationTimeUtc de smoke-result.json | 13:44:06.4337886 |
| LastWriteTimeUtc de smoke-result.json | 13:44:06.4347894 |

El JSON se escribe al final del script, después de DELETE session y cleanup. Si estos metadatos preservan la cronología original, no cabe una espera agotada de 15 segundos. Es un indicio fuerte de WaitForExit=true con código no disponible; los timestamps de archivos no sustituyen un cronómetro registrado y no demuestran el código 0. `process-evidence.json` tiene monitorSamples vacío y no aporta presencia del proceso entre el cierre y DELETE session. La ausencia posterior a DELETE tampoco demuestra que DELETE causara su salida.

## 2. Qué significa stopped en c4436bd

En `src-tauri/src/desktop/lifecycle.rs:186–202`, el worker espera mantenimiento inactivo, ejecuta actor.shutdown, escribe Stopped, publica ready y llama on_ready. El callback de `src-tauri/src/lib.rs:83–90` es app.exit(0). Stopped acredita la rama shutdown exitoso; no acredita por sí solo ejecución del callback ni recepción de su mensaje por el loop.

`src-tauri/src/adapters/sqlite/actor.rs:174–190` procesa la cola, cierra la conexión y libera LibraryLock y scratch antes de enviar done. `:249–281` espera done y hace join antes de devolver Ok. Por tanto, el join de research-db ya terminó cuando se escribe Stopped. El cierre de conexión ignora el resultado explícito, así que el log no constituye una prueba independiente de integridad o reapertura; sí descarta ese join como espera posterior al log.

`src-tauri/src/adapters/windows/managed_files.rs:14–21,244–250` conserva Arc<File> y pins, sin Drop propio, thread spawn, oplock ni join. `LibraryLock` contiene File y se libera en el hilo DB. Los únicos Drop explícitos encontrados en producto son OperationPermit (`desktop/maintenance.rs:76`) y TokenPermit (`modules/library/service.rs:47`); actualizan estado protegido por mutex, sin joins. Hay spawn_blocking para operaciones de documentos; mantenimiento ya está inactivo cuando se emite Stopped. No se observó una espera de runtime al finalizar: Tauri conserva su runtime asíncrono en un OnceLock estático (`tauri-2.12.1/src/async_runtime.rs:29`). No hay base para atribuir este intento a workers de guardas F10.

## 3. RequestExit/teardown: hipótesis condicionada a confirmar proceso vivo

Fuentes locales bajo `C:/Users/david/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`:

- `tauri-2.12.1/src/app.rs:581–586`: app.exit solicita salida al runtime; normalmente no ejecuta process::exit inmediatamente.
- `tauri-runtime-wry-2.12.1/src/lib.rs:2597–2603,4300–4312`: encola RequestExit; cuando se procesa, entrega ExitRequested y cambia ControlFlow a ExitWithCode si no se previene. En producto ready=true hace que request_exit devuelva true (`lifecycle.rs:167–168`), por lo que no se observa un veto intencionado posterior al drenaje.
- `tao-0.37.1/src/platform_impl/windows/event_loop.rs:271–283`: sale del loop, llama loop_destroyed y reset_runner. Este último elimina el callback (`event_loop/runner.rs:95–110`).
- `tauri-2.12.1/src/app.rs:1449–1455,1128–1139`: RunEvent::Exit precede cleanup_before_exit, que limpia tablas de recursos y oculta ventanas Windows.
- `wry-0.57.0/src/webview2/mod.rs:70–76`: Drop del WebView llama controller.Close y, para el hijo, DestroyWindow. `tauri-runtime-wry` también tiene Drop de WebviewWrapper (`:2392`), con mutex del almacén de contextos. Estas son rutas nativas reales que podrían requerir stacks si el proceso permaneciera vivo.
- `tao-0.37.1/src/platform_impl/windows/event_loop.rs:221–226`: tras run_return se llama std::process::exit. La mera existencia de hilos detached o de referencias Arc no obliga a esperar su terminación en esa ruta.

No puede localizarse el bloqueo entre envío, recepción, cleanup y destrucción sin una observación adicional. `driver.stderr.log:1` contiene a las 15:44:05.931 local el error Chrome_WidgetWin_0/1412, seis milisegundos después de Stopped. Es compatible con actividad de destrucción de ventana, pero no identifica emisor/PID ni acredita un hang o su causa. T01 tuvo exit0 con otra revisión y otro procedimiento; no es una comparación controlada que señale F10.

## Experimento mínimo discriminante propuesto, sin ejecutar

Primero modificar únicamente el observador QA, manteniendo el mismo binario congelado y una nueva biblioteca/perfil sintéticos:

1. Obtener el Process exacto, comprobar PID/ruta y abrir/retener su SafeHandle o Handle **antes** de CloseMainWindow. Verificar no-null/no-inválido; conservar ese mismo Process hasta guardar ExitCode. No reemplazarlo con Get-Process después del cierre.
2. Guardar por separado CloseMainWindowSent, UTC antes/después, WaitForExitReturned y Stopwatch.ElapsedMilliseconds. Si devuelve true, obtener ExitCode una vez con una lectura que exponga excepciones (por ejemplo llamada al getter mediante reflexión con captura explícita) y exigir un entero no-null; no convertir null a int porque daría 0.
3. Guardar todo antes de DELETE session. Si devuelve false, comprobar inmediatamente presencia por PID+ruta y tomar snapshot de hilos/ventanas propios antes del cleanup. No interpretar el getter null como proceso vivo ni como exit0.

Resultados: true+código0 antes de cleanup confirma cierre normal y el falso negativo anterior; false+PID vivo confirma el problema y justifica investigar runtime; true+código no disponible mantiene un defecto del observador; código no cero conserva un fallo real de salida sin inventar causa.

Solo si se confirma proceso vivo: realizar un lanzamiento directo con el mismo binario y fixture sintética, sin driver, con idéntica observación. Si solo falla con driver, priorizar interacción de automatización/WebView. Si ambos fallan, obtener stacks del proceso propio tras Stopped para distinguir espera del loop, controller.Close/DestroyWindow, cleanup de recursos o salida de Windows. Como alternativa posterior, una build diagnóstica aislada puede registrar marcadores cerrados antes/después de app.exit y en ExitRequested/Exit; se mantendría separada de evidencia del ejecutable congelado. No se propone aún cambiar contratos, guardas, ni forzar process::exit en producto.

## Validación y limitaciones

Comandos de investigación: `git show c4436bd:<ruta>`; `git grep -n -E 'impl Drop|thread::|spawn_blocking|oplock|Condvar|wait_for' c4436bd -- src-tauri/src`; Get-Content de JSON/scripts/logs sintéticos; Get-Item con CreationTimeUtc/LastWriteTimeUtc; rg y lectura de rangos de los crates citados. Fuentes externas: solo código oficial .NET/PowerShell y documentación Microsoft. No tests ejecutados; no commit; no cambios de producto. No se abrieron perfiles ni bibliotecas personales. Autor revisión: las conclusiones separan ruta demostrada por código, evidencia del intento e hipótesis; exit0 y liberación/reapertura del lock permanecen sin verificar.
