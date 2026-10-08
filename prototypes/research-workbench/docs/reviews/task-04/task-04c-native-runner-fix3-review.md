# T04c runner — revisión independiente QA3

2026-10-05. Revisor `/root/task04c_runner_review`. **DONE — APROBADO por revisión estática scoped.** Q5b y Q6a cerrados; sin hallazgos nuevos importantes en el delta. **QA nativa NOT_RUN**; la ejecución concreta sigue bajo el gate del orquestador.

BASE QA2: `011f153c90f9f4cef30fc67079eaaebe12519823`.
HEAD QA3: `31631fd3641bc52a6403144a0a05558262cbc6a1`, worktree QA limpio.
**SHA256 aprobado del runner:** `1B95DC08050BAEE82FD134939EC37A10952693F85437CF0CA49D9E05A9501089`.
Archivo: `C:/Users/david/Projects/Research-Workbench/.worktrees/task-04c-native-runner/work/qa/t04-native/native-qa.ps1`.

## Cierres de la lista

| ID | Estado | Localizadores QA3 y evidencia |
|---|---|---|
| Q5b: script/argumento | CERRADO | 457–466: literal JS constante, título por arguments[0], llamada Invoke-PageScript con Script y array Arguments completos; AST verifica aridad y tipos. |
| Q5b: tarjeta tras filtro | CERRADO | 925–927 archive y 935–937 ALL: Wait-PaperCard precede cada lectura; 945–949 restore conserva tarjeta exacta/lifecycle/botones y un único restore. |
| Q6a: continue ciclo 3 | CERRADO | 907–918: permanece en Workflow P2, consulta P1 completada, vuelve a P2 y sólo después retorna a Biblioteca. Light-read/archive también retornan al acabar sus observaciones. |
| Q6a: fases ciclo 4 | CERRADO | 950 espera P1 tras restore archive; 957 espera P2 para continue; 975 espera P1 para light-read. Consulta posterior P1 y retorno P2 explícitos, sin touch/back/avance compensatorio. |
| Q6a: botón completado | CERRADO | 965–966 rechaza botón habilitado, permite presencia disabled, manteniendo ACTIVE/P1 completada como aserciones. Concordante con PhaseWorkspace del producto inmutable5d35f03. |

Se leyó el delta completo de runner y JSON nuevo: `55+ / 7-` totales; runner `19+ / 7-`. Sin cambios de generador/PDF/manifest/producto/configuración/dependencias. Las guardas y los seis ciclos revisados en QA2 no se alteran. Los cierres Q5a/Q5c/Q7a/Q7b/generador del informe QA2 permanecen; no se reabrió el ámbito ni se añadieron propuestas cosméticas. No encontré regresiones directas de estas correcciones.

## Verificación propia

- Leídos brief/informe centrales task-04c-native-runner-fix3 y revisión QA2; contexto de producto exclusivo `5d35f03b13c4e85a0944296bac20e593cf494b5c`, conservado de la revisión anterior sin sustituirlo por WIP.
- `git -C .worktrees/task-04c-native-runner diff --staged --stat` y `git diff --stat`: sin cambios pendientes. `git rev-parse HEAD`: SHA QA3 indicado arriba.
- `git -C .worktrees/task-04c-native-runner diff 011f153 31631fd -- work/qa/t04-native/native-qa.ps1` y delta del JSON estático: leídos completos.
- `git -C .worktrees/task-04c-native-runner diff --check 011f153 31631fd`: exit 0, sin errores.
- `[System.Management.Automation.Language.Parser]::ParseFile(...)`: runner **11985 tokens / 0 errores**. AST focal de Wait-PaperCard: `[StringConstantExpressionAst(command), VariableExpressionAst(script), ArrayExpressionAst(arguments)]`; Script es `$cardStateScript`, Arguments `@([string]$Fixture.title)`.
- Literal JavaScript de **481 caracteres** extraído del AST, usa `arguments[0]`. Se envolvió únicamente como cuerpo de `function webdriverScript(){...}` y se envió por stdin a `C:/nvm4w/nodejs/node.exe --check -`: **exit 0**. Sólo comprobación sintáctica; fuente no evaluada, sin DOM/driver.
- `Get-FileHash` del runner: coincidencia exacta con el SHA aprobado arriba. Sólo lectura de fuentes y análisis estático; único archivo escrito por el revisor, este informe central.

No se ejecutaron runner/generador/preflight/app/driver/listener/picker/GUI/npm/Cargo/instalador. Sin cambios de código, commits, merges, subagentes o configuración. Esta aprobación cubre el runner exacto y la lista de revisión; no acredita funcionamiento nativo, persistencia observada, instalación o QA PASS. Root debe conservar candidato/pins y releer runtime antes de la ejecución autorizada. Si cambia el script, el SHA aprobado deja de corresponder y requiere revisión del delta.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH | 0 | pass |
| MEDIUM | 0 | info |
| LOW | 0 | note |

Verdict: **APPROVE — Q5b/Q6a cerrados; SHA exacto aprobado para el gate de ejecución de root. QA nativa NOT_RUN.**
