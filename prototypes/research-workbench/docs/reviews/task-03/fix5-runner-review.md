# T03 fix5 — revisión focal del runner de geometría

**PASS** del runner `work/qa/t03-native/native-qa-fix5.ps1`, SHA-256 **`1F986F2AA515F6257981122B00E280E0B5D354C5378632206806002C721B119A`**. Fecha: 2026-10-03. No se ejecutó preflight, aplicación ni sesión.

## Corte y alcance

Hashes confirmados del runner nuevo, base `native-qa-fix4.ps1` (`8297BB98FBDF5099E744D81F4A8B4488E546CC91FEBD37E247D26752659346DC`) y candidato `candidate-3bf7cac2/research-workbench.exe` (`3BF7CAC2920E41C3E7294E3CA414313D8DADBBFCF3C4CD48577F994590966E40`). Diff completo revisado: pins y comprobaciones de geometría/scroll; sin reabrir el harness base.

## Evaluación del delta

- La proporción compara `cssWidth/cssHeight` con `rasterWidth/rasterHeight`, exige dimensiones positivas y registra error relativo/tolerancia del 1%. Se evalúa en página 1 al 100%, página 2 al 100%, página 2 al 110% y tras reabrir. Una observación fuera de tolerancia marca fallo visual sin impedir que se recoja el resto de la evidencia y se limpie.
- El zoom conserva el rango existente 1.05–1.15 para ambos ejes raster y CSS. Ahora una escala incorrecta o la falta de raster estable al 110% impide PASS, en lugar de dejar solo una advertencia visual.
- La medición de scroll solo cambia `scrollTop`/`scrollLeft` del wrapper. Guarda ambos valores, intenta alcanzar los máximos y los restaura en `finally`. Comprueba restauración con tolerancia de 1 px, overflow vertical y acceso al extremo vertical; el extremo horizontal se exige cuando hay overflow horizontal. No cambia CSS, dimensiones del canvas, contenido raster, página ni zoom.
- El resultado final consulta `visualLayoutChecksPassed` antes del estado passed. Un fallo visual produce `failed`, o un estado parcial por otra comprobación anterior; ninguno satisface el criterio final de exit0. Una excepción de medición tampoco permite completar el recorrido. Las condiciones de cleanup previas siguen vigentes.
- Perfil, importación, protocolo, identidad de página, reapertura y cleanup no se relajan. Los fallos visuales pueden medirse sin saltarse el cierre, pero no conservar un PASS.

## Verificación independiente sin motor de layout

Parser PowerShell: **0 errores**. Extracción de literales JS del AST actual y compilación con Node `new Function`: **24 scripts, 0 errores**; incluye el nuevo script de scroll. No se ejecutó JS contra DOM ni navegador.

Se extrajo únicamente `Assert-CanvasAspectRatio` y se probó su lógica con datos numéricos: el caso real anterior (raster 612×792, CSS 611.99408×304.58334) devuelve `passed=false`, flag visual false y error relativo ≈1.60025. Casos numéricos proporcionales al 100% y 110% devuelven true con error subpíxel inferior a la tolerancia. Es una comprobación del detector, **no una medición del binario corregido**.

No se añadieron tests al producto ni tests que comparen literalmente CSS. El preflight comunicado por QA no se repitió. La revisión permite ejecutar la medición; la proporción, scroll y legibilidad reales quedan pendientes de esa observación.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH | 0 | pass |
| MEDIUM | 0 | pass |
| LOW | 0 | pass |

**Verdict: PASS focal de `1F986F2AA515F6257981122B00E280E0B5D354C5378632206806002C721B119A`.** Solo se escribió este informe; sin cambios en runner/producto, ejecución app, subagentes o merges.
