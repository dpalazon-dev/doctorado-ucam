# T04c native runner QA4 — espera del raster de página 1

Fecha: 2026-10-05. Autor nuevo Sol `/root/task04c_native_raster_fix`.
Estado: **DONE_WITH_CONCERNS**; delta congelado para revisión independiente scoped. No es aprobación de ejecución ni PASS nativo.

## Corte y propiedad

- Worktree: `C:/Users/david/Projects/Research-Workbench/.worktrees/task-04c-native-runner`.
- Rama: `agent/task-04c-native-runner`.
- BASE: `31631fd3641bc52a6403144a0a05558262cbc6a1`.
- HEAD: `2025189869ae8c1b49b31bbb1071066aca1a3056`, `qa: wait for distinctive T04c page-one raster`; árbol limpio después del commit.
- Runner SHA-256 de los bytes actuales: `B8C28716B00C1743B529A1F39A38A76069CFD996BB4C90950FC72BFB2D80427C`.
- Producto de referencia: `5d35f03b13c4e85a0944296bac20e593cf494b5c`; candidato permanece fijado a `32249B7A168D8B8AA53E65D82BE11A55FA6EE0B2294DE94682A424CC0921BDEC`.
- Delta: runner `+29/-11`, probe focal de 60 líneas, logs RED/GREEN y JSON estático. No producto, generador, fixtures/manifiesto, configuración, dependencias, subagentes o merges.

## Investigación y límite causal

Se leyó el fallo archivado `work/task-04c-integration/native-qa3.stderr.log` del primary y el ciclo 1 de `native-run-b4f1fe3c9f0f4fe8b4c83230f75bd526` en el worktree QA. La excepción provino de la aserción posterior a la espera de página 1. El ciclo guardó `reader` ausente y screenshots/aspect observations vacíos; cierre normal false/false/exit null, cleanup verificado sin errores. El fallo histórico y su biblioteca no se modificaron ni reabrieron.

La espera antigua buscaba cualquier componente roja menor de 245 en cualquier píxel, sin comprobar alpha. Un bitmap transparente contiene componentes RGBA cero: satisface esa espera aunque `Get-ReaderCanvasMetrics` compute cero píxeles de regla, pues la métrica exige alpha positivo y RGB oscuros dentro de la línea esperada. También permitía un píxel opaco fuera de la regla o una regla incompleta de sólo 100 píxeles.

La prueba focal ejecuta el JavaScript real de métricas extraído por AST con un adaptador de canvas en memoria y bitmaps controlados de 612×792; no usa un DOM real. Después ejecuta el scriptblock real de la espera extraído del runner con esas métricas. Antes del parche, ejecuta también el JavaScript real de la espera antigua con el mismo bitmap. Las expectativas son literales, incluidos el límite 100/101 y la página equivocada.

RED confirmó tres falsos positivos; GREEN confirmó rechazo de canvas transparente, blanco, píxel ajeno, 100 píxeles y página 2; aceptación de 101 y 468 píxeles de regla en página 1. Esto demuestra el defecto del predicado. **No demuestra el estado exacto del canvas del intento QA3 ni atribuye el fallo al producto.**

Comparación focal: FIX5 esperaba la regla distintiva con alpha positivo antes de avanzar; el helper actual de página 2 exige su regla exclusiva y cinco muestras de geometría estable. No se alteró ese helper ni se copió una nueva espera general.

## Corrección mínima e instrumentación

La espera inicial conserva su plazo de 50 segundos y ahora devuelve la métrica sólo si `ready`, `page == 1` y `page1RulePixels > 100`. Usa la métrica existente de la línea PDF y620. La aserción posterior permanece, con el mismo umbral.

La última métrica de la espera se almacena inmediatamente en `cycleResult.reader.firstPage`, por lo que un timeout conserva la observación previa. Página 1 y página 2 se capturan antes de sus aserciones diagnósticas y de proporción. Las métricas de página 2 se guardan antes de las aserciones. Ante un fallo del recorrido Reader, el catch guarda el error original, intenta registrar `failureCanvas` y una captura `*-reader-failure.png`, y sólo entonces propaga el error hacia el cleanup de sesión. Si esas operaciones diagnósticas fallan, sus mensajes quedan en `diagnosticErrors` y el error original sigue siendo el principal.

Se preservaron la regla exclusiva y estabilidad de página 2, proporción, scroll, seis ciclos y guardas de propiedad/cleanup. La instrumentación se revisó estáticamente; no se afirma que una captura nativa haya sido producida por ella.

## Comandos y resultados

Desde el worktree QA, antes del cambio de runner:

```powershell
& pwsh -NoProfile -File work/qa/t04-native/evidencias/qa4-raster-predicate-probe.ps1 *> work/qa/t04-native/evidencias/qa4-raster-red.log
$LASTEXITCODE # 1: tres regresiones previstas del predicado
```

Después del cambio:

```powershell
& pwsh -NoProfile -File work/qa/t04-native/evidencias/qa4-raster-predicate-probe.ps1 *> work/qa/t04-native/evidencias/qa4-raster-green.log
$LASTEXITCODE # 0: siete casos correctos
```

El probe parsea la AST sin ejecutar el runner y también ejecuta `node --check` para el adaptador temporal. Verificación AST/JS independiente realizada:

```powershell
$qaRunnerPath = (Resolve-Path work/qa/t04-native/native-qa.ps1).Path
$qaTokens = $null; $qaParseErrors = $null
$qaAst = [System.Management.Automation.Language.Parser]::ParseFile($qaRunnerPath, [ref]$qaTokens, [ref]$qaParseErrors)
if ($qaParseErrors.Count) { throw 'Runner AST parse failed' }
$qaMetricFunction = $qaAst.Find({ param($n) $n -is [System.Management.Automation.Language.FunctionDefinitionAst] -and $n.Name -eq 'Get-ReaderCanvasMetrics' }, $true)
$qaMetricCommand = $qaMetricFunction.Body.Find({ param($n) $n -is [System.Management.Automation.Language.CommandAst] -and $n.GetCommandName() -eq 'Invoke-PageScript' }, $true)
$qaJs = $qaMetricCommand.CommandElements[1].SafeGetValue()
$qaTempJs = Join-Path ([IO.Path]::GetTempPath()) ('rw-qa4-metric-' + [Guid]::NewGuid().ToString('N') + '.cjs')
try {
    [IO.File]::WriteAllText($qaTempJs, ('function probe(){' + $qaJs + '}'))
    & node --check $qaTempJs
    if ($LASTEXITCODE -ne 0) { throw 'Reader metric JS syntax failed' }
} finally { Remove-Item -LiteralPath $qaTempJs -ErrorAction SilentlyContinue }
git diff --check
git diff --cached --check
```

Resultados: runner 12,165 tokens / cero errores AST; literal métrico de 895 caracteres, `node --check` exit 0; ambos diff checks exit 0. Git advierte normalización CRLF→LF en futura escritura; el SHA anterior corresponde a los bytes actuales.

Commit exacto:

```powershell
git add -f -- work/qa/t04-native/native-qa.ps1 work/qa/t04-native/evidencias/qa4-raster-predicate-probe.ps1 work/qa/t04-native/evidencias/qa4-raster-red.log work/qa/t04-native/evidencias/qa4-raster-green.log work/qa/t04-native/evidencias/qa4-static.json
git commit -m 'qa: wait for distinctive T04c page-one raster'
git rev-parse HEAD
git status --short
```

Evidencia versionada: `work/qa/t04-native/evidencias/qa4-static.json` y los tres archivos `qa4-raster-*`. El JSON conserva hashes de probe y logs y distingue la hipótesis histórica no demostrada de los falsos positivos reproducidos.

## Autorrevisión y pendientes

El predicado original falló antes de corregirse; el nuevo pasa los mismos casos. El diff completo se revisó contra BASE, sin ampliar esperas generales ni alterar otros escenarios. No se ejecutaron runner, preflight, app, driver, listener, picker, GUI, generador, npm ni Cargo. El alcance autorizado era la prueba focal y AST/JS; el siguiente paso pertenece a root: revisión independiente scoped y un nuevo intento nativo completo con evidencia nueva. No se declara cierre normal, Workflow, persistencia, QA instalada ni funcionamiento en máquina limpia.
