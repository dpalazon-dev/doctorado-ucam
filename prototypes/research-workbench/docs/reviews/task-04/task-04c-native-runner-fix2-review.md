# T04c runner — revisión independiente QA2

2026-10-05. Revisor `/root/task04c_runner_review`. **DONE_WITH_CONCERNS**: QA2 todavía **NO APTO para ejecutar**. Lista consolidada restante: **Q5b y Q6a**. No se aprueba un SHA de ejecución.

## Corte y método

BASE QA1 `17ca5d58790aae1fff4d2a2ee8592c9b41b6e203`; HEAD QA2 `011f153c90f9f4cef30fc67079eaaebe12519823`, worktree `.worktrees/task-04c-native-runner` limpio. Leídos brief e informe QA2 centrales y delta completo de los tres archivos (`226+ / 71-`). La revisión cubre cierres de la lista QA1 y regresiones del delta, sin nuevas propuestas cosméticas. Producto de referencia exclusivo `5d35f03b13c4e85a0944296bac20e593cf494b5c` mediante git show; no WIP.

Hash del runner **revisado pero no aprobado**: `5F542D388D05699AEB3CD3C8C761E1678C59DCCC0747215918ABE109DC798283`.
Generador: `73CEA6AAFE3C6E38EE71338469D1E274903C2C27981DC947113DF40A271EBE42`.

## [HIGH] Q5b — espera de tarjeta mal formada y lecturas residuales sin esperar lista

Archivo: `work/qa/t04-native/native-qa.ps1:457`, `451–452`, `916–918`, `925–927` (rutas relativas al worktree QA).

La cadena añadida en Wait-PaperCard usa backtick + backslash + comilla alrededor del título. En PowerShell el backtick escapa el backslash; la comilla termina la cadena. El AST de **la propia línea 457** demuestra que Invoke-PageScript recibe tres argumentos después del nombre, no un único script completo:

1. `StringConstantExpressionAst`: su value termina en `x=>x.innerText===` seguido de un backslash.
2. `SubExpressionAst`: `$($Fixture.title)`.
3. `StringConstantExpressionAst`: su value empieza en backslash + `);const c=...`.

Por tanto, el parámetro Script queda truncado y no constituye JavaScript válido. La primera importación llama este helper y no puede completar esa espera. **Cero errores de parseo PowerShell no acredita una interpolación válida**. Evidencia obtenida únicamente de Parser/AST, sin evaluar el script, DOM ni driver.

Además, Ensure-LibraryView espera la lista antes de cambiar el filtro, pero después sólo espera select.value. Ciclo 3 archive (917) y ciclo 4 (926) siguen llamando Read-PaperCard inmediatamente, sin Wait-PaperCard tras el cambio de filtro. La actualización listPapers es asíncrona; esas comprobaciones pueden leer una lista todavía vacía/antigua y fallar sobre un producto correcto. El nuevo helper sí permite solucionar las esperas restantes cuando su cadena esté corregida.

Fix acotado: corregir el quoting usando el patrón válido ya presente (`backtick + comilla`, sin backslash), o pasar el título como argumento del script a Invoke-PageScript; inspeccionar el AST para que Script sea un solo argumento completo. Aplicar Wait-PaperCard después de todos los cambios de filtro antes de los asserts/clics de tarjeta. Mantener título/lifecycle/botones exactos y fallo explícito ante alertas.

Otros componentes de Q5b sí se corrigen: eliminado assert prematuro en Home; espera de Workflow por título/fase/carga; restore exige tarjeta presente Activo + Archivar y sin Restaurar. Esos cierres no neutralizan el script mal formado.

## [HIGH] Q6a — recorrido continue abandona Workflow y fases esperadas invertidas

Archivo: `native-qa.ps1:899–911`, `939`, `946–947`, `954`.

- En ciclo 3 se llama Return-ToLibrary en **899**, antes de la rama continue. Dentro de esa rama, 902 captura evidencia Workflow sobre Biblioteca y 903 busca `Consultar P1` sin volver a abrir Workflow. El botón no existe en Biblioteca; el recorrido se corta. Conservar la vista hasta acabar las comprobaciones de fase o reabrir explícitamente con P2 esperada antes de consultar P1.
- En ciclo 4, **939** espera P2 al abrir el paper restaurado de archive, cuyo contexto activo conservado es **P1**. **946** espera P1 al abrir continue, cuyo contexto activo es **P2**; no llega a la espera P2 de 947. Open-Workflow/Wait-WorkflowReady sólo esperan y no seleccionan la fase. Corregir las fases esperadas a P1 y P2 respectivamente, sin touch/back ni mutaciones compensatorias.
- Después de consultar P1 completada en continue, **954** rechaza la mera presencia del botón `Completar P1 y continuar`. PhaseWorkspace lo renderiza en toda vista P1 no archivada, con `disabled={!canAdvance}`. En P1 completada y activa P2 es correcto que exista **deshabilitado**. Exigir disabled (y conservar estado/contexto), no ausencia del botón.

Referencia inmutable: PhaseWorkspace líneas 72, 96 y 158–160; useWorkflow líneas 230 y 249 selecciona la activa al montar/reload(true). El cierre normativo archive/restore conserva P1; continue activa P2. Los errores son del runner, no resultados de una ejecución GUI.

La selección ARCHIVED antes de un único restore y ALL para observar la tarjeta restaurada queda corregida por lectura, sujeta al helper Q5b. Las consultas separadas de estados sustituyen las antiguas condiciones simultáneas correctamente donde el recorrido permanece en la vista prevista.

## Cierre de la lista QA1

| ID | Estado QA2 | Evidencia estática |
|---|---|---|
| Q6a | **ABIERTO** | Regresiones del recorrido y expectativas enumeradas arriba. |
| Q5a | CERRADO | Ciclo 2 guarda decisión, espera gate completo y verifica P1 en curso/ACTIVE sin ejecutar rama (855–869). |
| Q5b | **ABIERTO** | Cadena PS/JS fragmentada y lecturas residuales de lista. |
| Q5c | CERRADO | Texto largo de 589 carece de whitespace exterior; comparación exacta de save/restart se conserva. |
| Q7a | CERRADO | Error primario retenido, escritura/sidecar secundarios, throw primario o estado no passed tras finally (1033, 1060–1086). |
| Q7b | CERRADO | Readiness exige listener del driver; WebView exige perfil + ancestry app + ruta/versión runtime; capability discordante falla; cleanup sólo ownershipProven y PID/ruta/creationTime coincidentes. No probados se registran y no se paran. |
| Generador MEDIUM | CERRADO | Raíz exacta preexistente, comprobación de los cuatro destinos antes de writes, CreateNew PDFs/manifest; no se regeneraron fixtures. |

Se conservan seis ciclos, una biblioteca, pin externo obligatorio, selección UI y retirada/restauración PDF entre procesos. No encontré nuevas mutaciones IPC/React en el delta. No se amplía framework, cobertura ni lista a preferencias de estilo.

## Comprobaciones propias

- `git -C .worktrees/task-04c-native-runner diff --staged --stat` y `git diff --stat`: sin cambios sin commit; HEAD y status verificados.
- `git -C .worktrees/task-04c-native-runner diff 17ca5d5 011f153 -- work/qa/t04-native/native-qa.ps1` leído completo por bloques; delta completo del generador y evidencia nueva leído.
- `git -C .worktrees/task-04c-native-runner diff --check 17ca5d5 011f153`: exit 0, sin errores.
- `[System.Management.Automation.Language.Parser]::ParseFile(...)`: **runner 11967 tokens / 0 errores; generador 940 / 0**. Inspección adicional CommandAst/CommandElements de línea 457 demuestra la fragmentación anterior.
- `Get-FileHash` de ambos scripts: hashes indicados arriba. `Get-FileHash`/`Get-Item` del candidato: **32249B7A168D8B8AA53E65D82BE11A55FA6EE0B2294DE94682A424CC0921BDEC**, **27037184 bytes**, coincide con el literal autorizado y brief.
- Lectura manifest + hashes/tamaños: tres fixtures **1104 bytes** cada una y hashes exactos; no modificaciones PDF/manifest en el delta.
- `git show 5d35f03:src/features/workflow/PhaseWorkspace.tsx` y `useWorkflow.ts`: referencia para fases/contexto y botón presente deshabilitado. Continúan válidas las fuentes congeladas leídas en QA1; no se sustituyen por código actual.

No se ejecutaron runner/generador/preflight/app/driver/listener/picker/GUI/npm/Cargo/instalador. No se afirma funcionamiento nativo, compatibilidad o QA PASS. Pin de runtime debe releerse por root antes de la futura ejecución. Sólo se escribió este informe central; sin código/commits/merges/configuración/subagentes ni alteración de cambios ajenos.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH | 2 | warn |
| MEDIUM | 0 | info |
| LOW | 0 | note |

Verdict: **WARNING — Q5b y Q6a deben corregirse y revisarse antes de ejecución. SHA aprobado: ninguno. QA nativa NOT_RUN.**
