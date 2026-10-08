# T03 — revisión del diagnóstico separado del selector

**Resultado: BLOCK.** Fecha: 2026-10-03. No se ejecutó diagnóstico, preflight, aplicación, helper UIA ni sesión. Solo se escribió este informe.

## Identidad y alcance

El encargo/brief citaban `6983DF87505B6D38AF1E9429FA3B53E0787D16C6FE8E1DFF0C52E1BF384BDDE0`, pero el archivo estaba cambiando durante la lectura: primera observación `C15EC78A7C77ED82FF542BC78AEF141EFB740C83EECEB6AF334437EDCE9354A5`; última lectura y parser **`374E1EB988F836B00D55440B3FAE183AEE284BFCA2569A1A577A60312101118C`**. Los tres defectos siguientes persisten en esa última copia. No se emite aprobación para ninguno de esos hashes; las correcciones deben entregarse juntas como un corte congelado.

El autor confirmó después que 374E es el corte congelado. Se releyó su bloque helper/cleanup (379–460) y se volvió a confirmar el mismo hash; este es el corte concreto del dictamen. El preflight de 6983 es evidencia histórica y no se atribuye a 374E.

Archivo revisado: `work/qa/t03-native/picker-diagnostic.ps1`. Runner principal conservado: SHA-256 `874F2552F3D11A61531C922A50DBD191E9A38612BF6566FEB7CF1530D82BFB19`.

## [HIGH] Get-DialogWindows vuelve a escribir la variable automática PID

**Líneas 158–166.** `$pid = ...` colisiona, sin distinción de mayúsculas, con `$PID`, `Constant, AllScope`. La primera enumeración de ventanas falla antes de obtener el diálogo y también impide verificar su cierre mediante el mismo helper.

**Corrección mínima:** renombrar a `$windowProcessId` todos los usos locales. `Get-Variable PID` confirmó los flags en el entorno; el parser no detecta este fallo de ejecución.

## [HIGH] Las propiedades de disponibilidad se consultan en el tipo equivocado

**Líneas 230–231.** `$current` es `AutomationElementInformation`; no tiene `IsValuePatternAvailable` ni `IsInvokePatternAvailable`. Sin StrictMode esas lecturas se convierten desde null a false, por lo que el inventario puede declarar que no hay patrón incluso cuando `TryGetCurrentPattern` devuelve uno. Se falsearía precisamente la evidencia que pretende recoger este diagnóstico.

**Corrección mínima:** consultar `$element.GetCurrentPropertyValue([System.Windows.Automation.AutomationElement]::IsValuePatternAvailableProperty)` y el equivalente Invoke, registrando error/valor no soportado si corresponde, sin leer el Value del control. Reflexión local confirmó que ambas propiedades faltan en AutomationElementInformation y ambos campos `...Property` existen en AutomationElement. No se consultó ninguna ventana real.

## [HIGH] Un cierre fallido del selector no degrada el resultado completo

**Líneas 403–410 y 453–460 de la última copia.** `dialogCloseObserved=false`, `dialogCloseError` o `dialogCloseRefused` no participan en la decisión final. Si DELETE session elimina después todos los procesos, `cleanup.complete=true` permite conservar `picker observed; bounded inventory captured; no file selected` y exit0, aunque no se haya demostrado cerrar el selector mediante WM_CLOSE como exige el brief.

**Corrección mínima:** cuando se observó un diálogo y se intentó su cierre, exigir la desaparición observada tras el cierre validado o degradar a parcial con motivo específico. Conservar la evidencia del inventario. No hace falta exigir exit0 de la app para reconocer un inventario obtenido; sí diferenciar explícitamente inventario capturado de cierre del diálogo no acreditado.

## Resto del alcance revisado

- El inventario se dirige al HWND #32770 con PID/owner propio, revalidado por el helper contra el ejecutable congelado. WM_CLOSE comprueba clase y PID/owner antes de actuar. No se proponen cambios de producto ni selección de archivos.
- No se encontraron escrituras ValuePattern, envío de ruta IPC ni selección de archivo. Los registros de nodos serializan estructura/categorías reconocidas y tipos, sin nombres crudos de listado ni valores de campos; eventos WebDriver no guardan cuerpos ni base64. No se afirma que se haya observado una jerarquía real todavía.
- Límites declarados: 512 hijos Win32, 512 nodos por vista, profundidad 8, presupuesto blando 8 segundos por vista; helper separado con espera de 15 segundos. El presupuesto UIA no interrumpe una llamada individual; el proceso helper aporta el corte externo. La copia 374E retiene el Process helper propio, intenta Kill tras timeout, espera hasta 3 segundos y no bloquea leyendo stdout si no confirmó salida; en finally reintenta únicamente ese Process y exige su ausencia para cleanup completo. No se exige ampliar este mecanismo en la corrección consolidada.
- `Get-Command pwsh` resolvió `C:\Users\david\.cache\codex-runtimes\codex-primary-runtime\dependencies\native\powershell\pwsh.exe`; no hay bloqueo demostrado de localización del helper en el entorno actual.
- Parser PowerShell de la última copia: **0 errores**. Los **5 literales JS** con acceso DOM, extraídos del AST, compilaron mediante Node `new Function`: **0 errores**; no se ejecutaron.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH | 3 | warn |
| MEDIUM | 0 | pass |
| LOW | 0 | pass |

**Verdict: BLOCK** hasta corregir los tres puntos y entregar un hash estable para revisión focal. Este dictamen no altera la revisión del runner principal ni atribuye el fallo anterior al producto.

## Addendum — tres cierres sobre el diagnóstico congelado 50AD

**2026-10-03: PASS para una ejecución del diagnóstico separado**, SHA-256 **`50AD0ADB3AA9E4718562283E4D92B1A3D051D8D29023E532038CFFA199416B82`**, confirmado antes y después de esta revisión. El BLOCK anterior queda como evidencia histórica del corte 374E.

Revisión limitada a las tres regiones corregidas, contrastadas con el texto del corte 374E leído en la revisión anterior:

1. **PID, líneas 163–175:** renombrado completo a `$windowProcessId`. Búsqueda sobre AST del archivo actual: **0 asignaciones a PID** sin distinguir mayúsculas.
2. **Disponibilidad UIA, líneas 238–247:** ahora usa `AutomationElement.GetCurrentPropertyValue` con `IsValuePatternAvailableProperty`/`IsInvokePatternAvailableProperty` y `ignoreDefaultValue=true`. Solo guarda booleanos cuando el tipo es bool; NotSupported, tipos inesperados y errores quedan diferenciados y no convertidos a false. No consulta Value del control.
3. **Resultado de cierre, líneas 131–138 y 470:** `Resolve-PickerCompletionStatus` se ejecuta después del cleanup y antes de guardar JSON/decidir exit. Un diálogo observado exige tanto WM_CLOSE enviado como desaparición confirmada; en caso contrario conserva inventario y devuelve parcial aunque cleanup sea true. El fallo de cleanup sigue degradando por separado.

Se extrajo únicamente `Resolve-PickerCompletionStatus` del AST y se probó con objetos sintéticos: cierre no observado + cleanup true → parcial específico de WM_CLOSE; cierre observado + cleanup true → inventario observado; cierre observado + cleanup false → parcial de cleanup. Tres ramas con resultados esperados. No se ejecutó ninguna función que consulte UIA, inicie procesos o navegue por UI.

No se repitieron preflight, diagnóstico, revisión de fuentes de app ni comprobaciones ya cerradas. La evidencia de parser/JS/reflexión/preflight del autor queda en el brief actualizado; este addendum no la presenta como ejecución propia. Sin hallazgos abiertos en los tres cierres: CRITICAL 0, HIGH 0, MEDIUM 0, LOW 0.

**Verdict focal: PASS de `50AD0ADB3AA9E4718562283E4D92B1A3D051D8D29023E532038CFFA199416B82` para una ejecución diagnóstica acotada.** No acredita todavía inventario real, cierre, importación o calidad del producto.
