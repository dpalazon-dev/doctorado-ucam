# T03 — revisión focal de observación Win32

**PASS** del script `work/qa/t03-native/native-qa-win32-diagnostics.ps1`, SHA-256 **`189CFC5AC23A3E9B760320A6A5AD37142089E1DCDD1C21AA0AF2D03AF31F6655`**. Fecha: 2026-10-03. Solo revisión del delta frente a `native-qa-win32.ps1`, SHA-256 `39C2102EE00A9ED4F8E2C1A93221AE5A73ABACBE236A4D3B1D85A6D52D772616`; no se reabre la base.

## Comprobaciones del delta

- La ambigüedad continúa deteniendo el recorrido. Antes de lanzar el error registra los diálogos Open/Abrir #32770 ya filtrados por PID propio, su owner, visibilidad y habilitación. No pulsa un diálogo alternativo ni elige automáticamente entre ellos.
- Por diálogo inspecciona como máximo 512 descendientes. Solo lee texto de controles clase `Static` con IsChild y PID exacto; cada texto queda limitado a 512 caracteres. Registra límite de hijos y posible truncamiento de texto. No enumera nombres de archivos/listados ni cambia controles durante esa observación. El texto estático puede contener el mensaje de error del selector y se conserva localmente como evidencia diagnóstica autorizada.
- El estado del HWND original conserva existencia, identidad y categoría de título; no guarda un título arbitrario en bruto. No actúa sobre ese HWND durante esta observación.
- WM_GETTEXT lee únicamente el Edit/1148 ya validado después de escribir la fixture. Usa StringBuilder Unicode de capacidad 2–512, wParam igual a esa capacidad, argumento out de ancho de puntero y timeout de 1500 ms. No expone punteros manuales ni asignaciones sin liberar. El JSON guarda éxito, longitud y coincidencia exacta, **no el texto leído**.
- Precisión de alcance: sí añade una condición conservadora nueva antes de BM_CLICK: si el readback no coincide con la fixture, se detiene sin aceptar. Esto no relaja el criterio ni introduce nuevas acciones; evita aceptar un valor no confirmado. El clic conserva `0x02`, timeout y revalidación de controles; protocolo, recorrido restante y cleanup no cambian.

## Verificación sin ejecución

Hashes de ambos scripts confirmados; diff completo revisado. Parser PowerShell: **0 errores**. Bloque C# de interop extraído del AST y compilado con Add-Type: **correcto**, sin invocar funciones Win32. El delta no modifica JavaScript. No se ejecutó preflight, app, selector, lectura de controles reales ni otro recorrido. Solo se escribió este informe.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH | 0 | pass |
| MEDIUM | 0 | pass |
| LOW | 0 | pass |

**Verdict: PASS focal de 189CFC5A…F31F6655.** La revisión permite observar el próximo resultado; no establece la causa de los dos diálogos del intento anterior ni acredita importación/render.

## Addendum — adquisición visible y espera del HWND original

**PASS focal**, 2026-10-03, de `native-qa-win32-visible-dialog.ps1`, SHA-256 **`30A4538128A320CF7FB8C0A08D541DA182B83028BD7E9B46602883EEA667046D`**, frente al corte 189C confirmado. Se revisó exclusivamente el delta completo de ocho líneas añadidas y una sustitución de espera.

La adquisición omite diálogos ocultos y conserva los filtros de PID, clase, título y unicidad. Tras el clic, la espera se refiere al HWND exacto seleccionado: admite que ya no exista o que esté oculto; si existe con otro PID, falla explícitamente. No elige otro diálogo ni vuelve a clicar. El timeout sigue siendo 15 segundos y las comprobaciones posteriores de preview, importación y protocolo se conservan. La desaparición/ocultación por sí sola no se usa para declarar importación exitosa.

Parser PowerShell: **0 errores**. Sin preflight, app ni lecturas de ventanas reales. Sin hallazgos bloqueantes del delta; conserva los límites de la revisión anterior. **Verdict: PASS de 30A45381…A667046D para el intento exploratorio autorizado por el orquestador.**
