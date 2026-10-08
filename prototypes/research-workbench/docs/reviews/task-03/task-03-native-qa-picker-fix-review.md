# T03 QA — revisión focal de la corrección del selector

**Resultado: PASS para un segundo intento exploratorio**, con runner SHA-256 **`AB334368DD6DF34B8A981622E4F4E5F18D5B869F08DB3864DBFD23D9C9C6DF6C`**. Fecha: 2026-10-03. Sin ejecución de runner, preflight, app, sesión ni selector durante esta revisión.

## Alcance y cierre del diagnóstico

Se leyó el brief actualizado y se revisó el diff completo contra `work/qa/t03-native/evidence/native-qa-3006-before-picker-filter.ps1`, cuyo SHA-256 coincide con `3006ECAA112C717F5D3FF8A6BB81B5DA43F2DA4945D2610EEC955DFA08C9EC52`. El candidato y el recorrido posterior permanecen fuera del delta.

El defecto descrito en `task-03-picker-diagnosis.md` queda cerrado estáticamente:

- **Líneas 418–425:** antes de clicar «Seleccionar PDF», captura el HWND principal del Process retenido, exige que no sea cero y verifica el PID. Conserva esa identidad en el resultado.
- **Líneas 180–185, 193 y 232–235:** mantiene el filtro inicial por PID propio u owner directo propio; excluye el HWND principal capturado y admite otro HWND del mismo PID aunque su owner sea nulo. Una ventana ajena sin owner de la app sigue descartada antes de inspeccionar sus controles.
- **Líneas 199–244:** solo selecciona un control identificado de nombre de archivo con ValuePattern y un botón identificado de aceptación con InvokePattern. Elimina el fallback que aceptaba cualquier único Edit y la selección del primer botón. Controles o ventanas ambiguos detienen el flujo antes de escribir/pulsar.
- **Líneas 226–230:** registra ventana, ownership, clasificación y controles acotados antes del descarte final; deduplica por estado. No lee ValuePattern.Current.Value ni valores de campos. Los errores UIA permanecen registrados y no se confunden con selección exitosa.
- **Líneas 256–261:** escribe únicamente la ruta sintética exacta en el control previamente validado e invoca el botón previamente validado. Mantiene la comprobación posterior de preview/confirmación del recorrido ya revisado.
- **Línea 335:** conserva ambos puertos escogidos en el JSON, sin cambiar selección, consultas o cleanup.

No se identifican defectos bloqueantes de confianza alta en este delta. La disponibilidad de controles/patrones UIA y la identidad real del HWND siguen pendientes de observación: el inventario nuevo permite explicarlas si el intento no progresa.

## Verificación independiente

- Hash del runner actual confirmado.
- Parser PowerShell: **0 errores**.
- Extracción de 23 literales JS desde el AST PowerShell actual y compilación con Node `new Function`: **23 analizados, 0 errores**. No se ejecutó el JS ni se abrió navegador.
- Snapshot anterior confirmado sin cambios. JSON original del primer intento conserva SHA-256 `05992D846834B05F8C42127BA8DCDEB5D0D0363F922F33FDC2F45625C9E58A7F`.
- El preflight de `2026-10-03T02:35:48Z`, exit 0, se cita como evidencia del autor en el brief; no se repitió en esta revisión.

El primer timeout solo demostró que el helper anterior no devolvió un candidato aceptado. No se atribuye ese fallo al producto ni se afirma que un diálogo ownerless fuera observado. Este PASS autoriza evaluar el runner corregido en un segundo intento; no acredita importación, render, persistencia, cierre normal o integración del producto.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH | 0 | pass |
| MEDIUM | 0 | pass |
| LOW | 0 | pass |

**Verdict: PASS del hash `AB334368DD6DF34B8A981622E4F4E5F18D5B869F08DB3864DBFD23D9C9C6DF6C` para un segundo intento exploratorio.** Solo se escribió este informe; no se modificó producto, runner, configuración global o evidencia anterior.

## Addendum — actualización de runtime/driver a .53

**2026-10-03: PASS del runner `874F2552F3D11A61531C922A50DBD191E9A38612BF6566FEB7CF1530D82BFB19` para el intento exploratorio pendiente.** El orquestador informa que el intento con AB334 se detuvo en preflight por actualización automática del runtime, sin iniciar app; no se le atribuye una nueva ejecución nativa.

Diff completo contra `evidence/native-qa-ab334-before-edgedriver53.ps1`: exactamente cuatro líneas modificadas, limitadas a ruta versionada de EdgeDriver, SHA-256 del driver y versiones esperadas de driver/WebView2. No cambia lógica del picker, ownership, cleanup o criterios del recorrido.

Comprobación independiente de solo lectura del ejecutable `work/tools/msedgedriver/154.0.4258.53/msedgedriver.exe`: SHA-256 `008115B68B38437B1C0138F3CCED648F61F333CF4623A9895296E7C1C01ABCB8`, ProductVersion `154.0.4258.53`, Authenticode **Valid**, firmante **Microsoft Corporation**. Driver anterior `.48` y snapshot AB334 conservan sus hashes anteriores.

Evidencia guardada en `work/qa/t03-native/evidence/preflight-20261003-0505-edgedriver53`: `stdout.json` corresponde al runner nuevo y registra driver, WebView2 registrado y ejecutable `.53`, hashes de candidato/fixture/tauri-driver intactos y `prepared; app launch intentionally gated` a las `05:05:35Z`. `exitcode.txt=0`, `stderr.log=0 bytes`; `provenance.json` registra descarga desde `https://msedgedriver.microsoft.com/154.0.4258.53/edgedriver_win64.zip`. Se verificaron hash, versión y firma localmente; no se repitió descarga ni preflight.

Sin hallazgos nuevos: CRITICAL 0, HIGH 0, MEDIUM 0, LOW 0. **Verdict del addendum: PASS focal del hash 874F…FB19**, conservando límites de la revisión anterior. No se ejecutó app, sesión, driver ni runner durante este addendum.
