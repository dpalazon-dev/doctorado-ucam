# T03 — revisión focal del runner con selector Win32

**Dictamen: BLOCK del corte actual hasta el ajuste mínimo del flag BM_CLICK acordado con el orquestador; resto del delta PASS.** Fecha: 2026-10-03. No se ejecutó aplicación, runner, preflight ni selector.

## Identidad revisada

- Nuevo runner: `work/qa/t03-native/native-qa-win32.ps1`, SHA-256 `47379FFF6D1B2A0344F5C71FF6407420F524C7667E7FD99B4518B83EA3B788DE`.
- Base conservada: `work/qa/t03-native/native-qa.ps1`, SHA-256 `874F2552F3D11A61531C922A50DBD191E9A38612BF6566FEB7CF1530D82BFB19`.
- Candidato integrado fijado: `candidate-69fd578d/research-workbench.exe`, SHA-256 `69FD578D817849DFB60F2B0D15A82DE0A66686EF6D1F9D79F946D5978931D938`, confirmado en disco.

## Ajuste previo acordado: evitar un falso fallo al cerrar el receptor de BM_CLICK

**Líneas 246–249.** El clic usa flags `0x22`, incluyendo `SMTO_ERRORONEXIT`. Microsoft documenta que esa opción produce retorno cero si la ventana receptora se destruye mientras procesa el mensaje. El runner rechaza ese retorno antes de comprobar cierre del diálogo/preview; por tanto, si Aceptar destruye el botón dentro de la llamada, puede clasificar como fallo una selección realizada correctamente. [SendMessageTimeoutW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendmessagetimeoutw).

Esto es un **riesgo semántico de falso negativo**, no una reproducción ni una afirmación sobre el orden de destrucción del IFileDialog concreto. BM_CLICK no tiene valor de retorno propio; el retorno comprobado corresponde al transporte SendMessageTimeout. [BM_CLICK](https://learn.microsoft.com/en-us/windows/win32/controls/bm-click).

El orquestador decide eliminar ese riesgo mediante un único cambio: **flags BM_CLICK `0x22` → `0x02`**, conservando timeout de 1500 ms, comprobación de fallo de transporte, espera de cierre y validaciones posteriores del recorrido. WM_SETTEXT mantiene `0x22`; ahí la destrucción durante la escritura sí es fallo. No se solicita investigación adicional ni cambios en producto/base.

## Resto del delta: PASS focal

- **HWND/PID:** solo reconoce un diálogo #32770 Open/Abrir del PID exacto. No acepta solo por título ni por nombre de proceso. Dos diálogos válidos son error.
- **Controles:** enumera descendientes del diálogo, exige IsChild y PID exacto, clase Edit/ID 1148 y Button/ID 1, unicidad y habilitación. Revalida clase/PID/título y ambos HWND inmediatamente antes de aceptar.
- **Entrada:** limita la escritura a la ruta absoluta de la fixture fijada y vuelve a comprobar su hash. Interop Unicode con tamaño de punteros adecuado. WM_SETTEXT exige éxito de API y resultado TRUE; la documentación define ese resultado como texto establecido. [WM_SETTEXT](https://learn.microsoft.com/en-us/windows/win32/winmsg/wm-settext).
- **Evidencia disponible:** `picker-diagnostic-2d07b20403aa43888475c5140eafef18/picker-inventory.json` muestra diálogo #32770 PID 38780, owner 0; 50 hijos sin truncamiento, un Edit/1148 y un Button/1 del mismo PID. Justifica el mecanismo propuesto, pero no demuestra que la selección funcione todavía en el nuevo candidato.
- **Recorrido y cleanup:** el diff completo respecto a 874F conserva importación UI, preview/confirmación, fetch real del protocolo, marcas de ambas páginas, medición de zoom, reapertura/persistencia, cierre observado y degradación por cleanup incompleto. Fuera del selector cambia solo ruta/hash del candidato y campos/mensajes de evidencia del picker. No se reabre la revisión de la base.
- **Sin ampliar alcance:** no se añaden teclas globales, clipboard, selección directa por IPC, seed o lectura de archivos personales. El mensaje se dirige únicamente al control validado de la sesión sintética.

## Validación sin aplicación

Hashes de runner/base/candidato confirmados. Parser PowerShell: **0 errores**. El bloque C# de interop extraído del AST compiló con Add-Type, **sin invocar sus funciones Win32**. Los **23 literales JS** del archivo compilaron con `new Function`: **0 errores**, sin ejecutarlos. No se repitió preflight.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH | 1 | warn |
| MEDIUM | 0 | pass |
| LOW | 0 | pass |

El único punto pendiente es el ajuste preventivo de BM_CLICK acordado. **BLOCK acotado del hash 47379…788DE; resto PASS.** La lectura del diff de ese único token y del nuevo hash basta para cerrar este dictamen; no se requiere otra revisión general ni ejecución para revisar el cambio.

## Cierre focal por el orquestador
Root verificó SHA39C2102EE00A9ED4F8E2C1A93221AE5A73ABACBE236A4D3B1D85A6D52D772616 y diff exactamente una línea frente snapshot47379FFF: BM_CLICK flags0x02, WM_SETTEXT intacto. Cierra el único ajuste exigido por la revisión independiente. Autorizado un intento completo sobre candidato integrado69FD/fixture sintético; no aceptación anticipada del resultado.
