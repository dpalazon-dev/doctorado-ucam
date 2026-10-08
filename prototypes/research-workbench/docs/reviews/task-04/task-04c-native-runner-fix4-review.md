# T04c runner — revisión independiente QA4

2026-10-05. Revisor `/root/task04c_runner_review`. **DONE — APPROVE, delta scoped de espera/instrumentación Reader.** Sin hallazgos importantes nuevos. El probe aislado pasa; el runner/aplicación **no se ejecutaron por este revisor**.

BASE: `31631fd3641bc52a6403144a0a05558262cbc6a1`.
HEAD QA4: `2025189869ae8c1b49b31bbb1071066aca1a3056`, worktree QA limpio.
**SHA256 aprobado del runner:** `B8C28716B00C1743B529A1F39A38A76069CFD996BB4C90950FC72BFB2D80427C`.
Archivo exacto: `C:/Users/david/Projects/Research-Workbench/.worktrees/task-04c-native-runner/work/qa/t04-native/native-qa.ps1`.

## Resultado de la revisión

Se leyeron brief/informe QA4 y delta completo: runner `29+ / 11-`, probe focal y sus logs RED/GREEN/JSON. Únicamente espera de primera página e instrumentación Reader; no se reabrieron otros hallazgos ni se propuso ampliar framework.

- **Predicado corregido:** la espera usa Get-ReaderCanvasMetrics y exige ready, page==1 y page1RulePixels>100. Conserva plazo 50s y umbral de la aserción posterior; la métrica existente requiere alpha>0 y RGB oscuros sobre la regla y620. Canvas transparente o un píxel opaco ajeno dejan de acreditar raster listo.
- **Evidencia preservada:** la última métrica de primera página se guarda dentro de cada intento de espera; métricas/capturas de página 1 y página 2 preceden las aserciones diagnósticas/de aspecto. Si falla Reader, catch guarda el error original, intenta failureCanvas/captura y agrega errores diagnósticos secundarios antes de propagar el original. No convierte fallo de captura/observación en PASS ni oculta el error principal.
- **Sin regresión directa detectada:** página 2 conserva regla exclusiva, espera estable y umbral; aspecto/scroll siguen siendo aserciones decisivas. No cambian seis ciclos, UI, ownership/cleanup ni el orden del PDF indisponible/restaurado.
- **Probe pertinente:** extrae por AST el JS real de métricas y el scriptblock de la espera; ejecuta sólo éstos con bitmaps/canvas adaptados en memoria. Expectativas de casos explícitas, incluido límite100/101 y página equivocada. No importa/dot-source/invoca runner ni crea DOM/app/driver. Su archivo temporal de Node es único y se elimina mediante LiteralPath en finally.

## Evidencia propia

Leídos los archivos históricos del intento root `native-run-b4f1fe3c9f0f4fe8b4c83230f75bd526/cycle-1/cycle-result.json` y `work/task-04c-integration/native-qa3.stderr.log`: failure posterior a la espera de página1, reader=null, screenshots vacíos, cleanup sin errores y con ausencia de procesos/puertos observados. Estos archivos no demuestran el bitmap exacto ni una causa de producto. No se reabrieron ni modificaron esa biblioteca o sus resultados.

Desde el worktree QA se ejecutó únicamente el probe autorizado:

```powershell
& ./work/qa/t04-native/evidencias/qa4-raster-predicate-probe.ps1
```

**Exit0, siete casos correctos**: transparente/blanco/píxel ajeno/regla100/página2 rechazados; regla101 y regla468 en página1 aceptadas. Incluye node --check de su adaptador. El RED del autor sobre BASE se leyó como evidencia archivada (tres falsos positivos); no se presenta como ejecución independiente. El fallo del predicado antiguo es también verificable por lectura: RGBA0 satisface red<245 sin alpha, mientras la métrica de regla lo rechaza.

Comprobaciones adicionales propias:

- Parser::ParseFile del runner: **12165 tokens, cero errores**.
- Get-FileHash: coincide exactamente con SHA aprobado arriba.
- `git -C .worktrees/task-04c-native-runner diff --check 31631fd 2025189`: exit0.
- HEAD/status y diffs staged/unstaged: corte indicado, árbol QA limpio antes/después del probe.

No runner/preflight/app/driver/listener/picker/GUI/generador/npm/Cargo/instalador. Sólo se escribió este informe central; sin producto/scripts/commits/merges/configuración/subagentes ni alteración de trabajo ajeno. El probe acredita el predicado controlado; esta aprobación estática no acredita un recorrido nativo completo, cierre normal, Workflow o persistencia. Root mantiene el gate de ejecución sobre este SHA y el candidato congelado.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH | 0 | pass |
| MEDIUM | 0 | info |
| LOW | 0 | note |

Verdict: **APPROVE — espera/instrumentación QA4 y probe focal conformes; SHA exacto aprobado para el gate de root. QA nativa nueva NOT_RUN por este revisor.**
