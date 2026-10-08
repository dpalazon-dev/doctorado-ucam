# T03 — diagnóstico acotado del timeout del selector

**Conclusión:** existe una exclusión incorrecta en el filtro QA para ventanas del mismo PID sin owner de la app. Es una explicación plausible del timeout, **no su causa observada**: el runner no registró los HWND descartados. La revisión estática anterior pasó por alto esta combinación. No se propone modificar producto ni iniciar un reintento en este diagnóstico.

## Evidencia

- Runner verificado: SHA-256 `3006ECAA112C717F5D3FF8A6BB81B5DA43F2DA4945D2610EEC955DFA08C9EC52`.
- Run `work/qa/t03-native/native-run-0c31da2bcfed42f0bcd918ae06ebef29/native-result.json`: PID app `47344`; `failureStage=choose exact synthetic PDF through app-owned Windows picker`; timeout `nativePickerNotAutomatable`; `pickerWindow=null`, `picker=null`, `uiaErrors=[]`; `cleanup.complete=true`, sin errores ni listeners restantes. Esto demuestra que el helper no devolvió candidato aceptado; no demuestra que el diálogo no apareciese o que UIA no lo pudiera leer.
- `git show b36ae5ae0529ba429edf7a360ade7319e353e8fe:src-tauri/src/adapters/windows/pdf_picker.rs`, método `select_pdf`, líneas 29–32: `app.dialog().file().add_filter(...).pick_file(...)`, sin `set_parent`. La fuente no establece explícitamente una ventana padre. No se infiere de esa ausencia cuál fue el HWND/owner real en la ejecución.

## [HIGH] El filtro final excluye una ventana propia que el filtro inicial admite

**Archivo:** `work/qa/t03-native/native-qa.ps1:184`, `:192`.

El primer filtro admite `windowProcessId == AppPid` **o** `ownerPid == AppPid`. El último exige además `windowProcessId != AppPid` **o** `ownerPid == AppPid`. Para un diálogo cuyo PID es el de la app y cuyo owner es nulo/otro PID, la condición final siempre es falsa, aunque exponga todos los controles necesarios. El descarte es silencioso y se repite hasta timeout. La combinación de ambos filtros equivale, en la práctica, a exigir owner de la app.

## Corrección QA mínima propuesta

1. Antes de clicar «Seleccionar PDF», refrescar el Process propio ya retenido y capturar su `MainWindowHandle` no nulo; verificar que ese HWND pertenece al mismo PID y conservarlo como `mainHwndBeforePicker`.
2. Mantener la enumeración restringida al PID propio o owner directo del PID propio. Excluir siempre el HWND principal capturado. Aceptar como candidato un HWND distinto del principal si pertenece al PID propio, **o** si `GW_OWNER` pertenece al PID propio. No ampliar a títulos, nombres de proceso o ventanas globales ajenas.
3. Dentro de ese conjunto, exigir un control inequívoco de nombre de archivo con ValuePattern y un botón inequívoco de aceptar con InvokePattern. No elegir el primer control ambiguo ni usar como criterio suficiente «hay algún edit y algún button». Si hay varios diálogos/controles candidatos, registrar la ambigüedad y parar sin escribir ni pulsar.
4. Registrar antes de descartar un inventario acotado de las ventanas que pasaron el filtro PID/owner: HWND, PID, owner HWND/PID, clase, título, si es ventana principal y motivo de aceptación/descarte; para los controles relevantes, AutomationId/Name/tipo y patrones disponibles. No leer valores de campos ni inventariar ventanas ajenas. Deduplicar por HWND/estado para evitar multiplicar el log en cada poll.
5. Conservar la selección de la ruta sintética exacta, la misma verificación de preview/confirmación y el cleanup existente. Revisar el nuevo hash antes de cualquier nueva observación.

El inventario permitirá distinguir en la siguiente observación autorizada entre diálogo propio sin owner, controles diferentes, ambigüedad, error UIA y ausencia efectiva de ventana dentro del alcance permitido. No hace falta todavía introducir seed, entrada manual, nuevos escenarios ni cambios de producto.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH | 1 | warn |
| MEDIUM | 0 | pass |
| LOW | 0 | pass |

**Verdict: corregir el filtro QA antes de reintentar.** El timeout actual no acredita que el selector nativo sea inautomatizable. Diagnóstico realizado solo con JSON y fuente congelada; sin app, sesión ni edición de scripts/producto.
