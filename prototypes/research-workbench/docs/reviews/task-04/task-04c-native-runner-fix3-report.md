# T04c native runner QA3 — residuales Q5b/Q6a

Fecha: 2026-10-05. Mismo autor QA.
Estado: **DONE_WITH_CONCERNS**. Congelado para revisión scoped de root; ningún SHA queda aprobado para ejecutar.

## Corte y alcance

- Worktree: `C:/Users/david/Projects/Research-Workbench/.worktrees/task-04c-native-runner`.
- BASE QA2: `011f153c90f9f4cef30fc67079eaaebe12519823`.
- Commit QA3: `31631fd3641bc52a6403144a0a05558262cbc6a1` (`qa: repair T04c runner phase waits`); worktree limpio.
- Delta propio: runner `+19/-7`; evidencia estática nueva `+36`.
- Únicos cambios: `work/qa/t04-native/native-qa.ps1` y `work/qa/t04-native/evidencias/qa3-static.json`. Sin cambios de producto, generador, PDFs, manifest, dependencias o configuración.
- Referencia de producto sólo por SHA inmutable: `5d35f03b13c4e85a0944296bac20e593cf494b5c`.

## Hallazgos corregidos

- **Q5b — argumento de tarjeta:** el título ya no se interpola dentro del literal JavaScript. `Wait-PaperCard` conserva el JS como string constante y pasa el título mediante `Invoke-PageScript` y `arguments[0]`. La AST ahora muestra exactamente tres CommandElements: comando, variable del script y array de argumentos.
- **Q5b — listas tras filtros:** los dos sitios que leen tarjetas después de cambiar lifecycle/filter esperan antes la tarjeta exacta. En ciclo 3 archive se espera tarjeta Archivado con Restaurar; en ciclo 4 ALL se espera la tarjeta antes de `Read-PaperCard`. El chequeo posterior a restore sigue esperando la misma tarjeta Activa con Archivar y sin Restaurar.
- **Q6a — rama continue ciclo 3:** Workflow permanece montado en P2 mientras captura P2, consulta P1 completada, vuelve a P2 y registra esa vista. Sólo entonces regresa a Biblioteca. Las ramas light-read y archive regresan una vez cada una después de sus observaciones.
- **Q6a — fases ciclo 4:** archive restaurado espera contexto P1; continue espera P2; light-read espera P1. No se requiere que una vista presente estados de dos fases.
- **Q6a — botón P1 completada:** la comprobación ahora permite que `Completar P1 y continuar` esté presente deshabilitado; falla sólo si está habilitado o el estado ACTIVE/P1-completada no coincide. `PhaseWorkspace.tsx` en el SHA de referencia renderiza el botón P1 con `disabled={!canAdvance}`.

Se conservan seis ciclos y las guardas QA2. No se tocaron Q5a/Q5c/Q7a/Q7b ni el generador.

## Verificación estática

- Runner SHA-256: `1B95DC08050BAEE82FD134939EC37A10952693F85437CF0CA49D9E05A9501089`.
- `[System.Management.Automation.Language.Parser]::ParseFile`: runner 11,985 tokens / 0 errores; generador sin cambios desde QA2, 940 tokens / 0 errores.
- Inspección AST focal de `Wait-PaperCard`: tres elementos de llamada; el script es `$cardStateScript` y el argumento es `@([string]$Fixture.title)`.
- El literal JS de 481 caracteres se extrajo de la AST y se validó con `node --check`; no se evaluó, no accedió a DOM y no se llamó al runner.
- Las dos lecturas de tarjetas se contrastaron con sus esperas anteriores en el source.
- `git diff --check`: sin errores. Git informa normalización CRLF del runner en una futura escritura.
- Evidencia: `work/qa/t04-native/evidencias/qa3-static.json`.

## Límites

No se ejecutaron runner, generador, preflight, aplicación, driver, listener, picker, GUI, npm ni Cargo. Estos resultados acreditan parseo, aridad de argumentos, sintaxis JS y secuencia del código; no prueban carga React real, selección visual ni QA nativa. El runner sigue sin aprobación de ejecución y requiere revisión independiente de root.
