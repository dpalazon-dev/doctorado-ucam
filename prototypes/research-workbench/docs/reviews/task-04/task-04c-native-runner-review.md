# T04c — revisión independiente del runner QA1

Fecha: 2026-10-05. Revisor: `/root/task04c_runner_review`.
Estado de entrega: **DONE_WITH_CONCERNS**. **Runner NO APTO para ejecución** hasta corregir los seis hallazgos HIGH de esta lista cerrada y revisar el SHA posterior. No se ejecutó QA nativa.

## Corte y alcance

- BASE QA: `1da396562a940c629ac5e85b372988e802bf3944`; HEAD QA1: `17ca5d58790aae1fff4d2a2ee8592c9b41b6e203`, árbol QA limpio.
- Runner revisado completo: `.worktrees/task-04c-native-runner/work/qa/t04-native/native-qa.ps1`, SHA256 `8208020894536F688F3B5BD39909779D320255228BF0F8462CC6DC5D5AA38AF5`, 993 líneas.
- Generador revisado completo: `make-synthetic-pdfs.ps1`, 66 líneas; fixtures y manifest actuales leídos.
- Producto de referencia exclusivo: `5d35f03b13c4e85a0944296bac20e593cf494b5c`, mediante `git show`; no WIP. Contexto: App, LibraryPage/useLibrary/ImportPaperDialog, PhaseWorkspace/PhaseAnswerForm/GateIssues/useWorkflow y normalización Rust.
- Contrastados runner FIX5 y generador T03 históricos, con hashes actuales iguales a los del brief: `1F986F2AA515F6257981122B00E280E0B5D354C5378632206806002C721B119A` y `45A7F37D90982CBB761127EF0915804858174202692762E50E1EAC165374AFA9`.
- Leídos AGENTS, INTENT, inicio STATUS, ambos briefs, plan/preflight, informe QA1 y revisión root Q1–Q7; contratos/gates relevantes. Aplicadas Karpathy y verification-before-completion. Peer OpenViking no verificado: fuentes locales únicamente.
- Delta completo QA1: runner `162+ / 67-` y JSON estático nuevo `38+`. No cambios de producto en ese delta. La rama QA deriva de un checkpoint no aprobado y **no debe fusionarse en bloque**.

Los pins PENDING del ejecutable/runtime son deliberados y no son hallazgos. La guarda de línea 20 falla antes de efectos cuando falta el candidato; Invoke-QARun vuelve a comprobar candidato/runtime y SHA externo aprobado antes de crear el área de ejecución. Cambiar los literales de configuración requerirá revisar/pinear el SHA resultante. La procedencia debug/root override del binario concreto sigue siendo un gate de Sol, no evidencia aportada por esta revisión.

## Lista cerrada de hallazgos importantes

### [HIGH] Q6a — estados y navegación incompatibles con la vista de fase

Archivo: `native-qa.ps1:777`, `812–825`, `845–851`, `860`.

Issue: ciclo 2 exige simultáneamente `Estado de PRE: en curso` y `Estado de P1: sin iniciar`. PhaseWorkspace sólo renderiza `Estado de {phaseCode}` de la fase consultada. En continue, advance ejecuta `reload(true)` y muestra P2; por ello la espera `Estado de P1: completada` de 812–813 y su comprobación de 819 no se satisfacen. Al reabrir, el botón se llama `Activa · P2`, no `Consultar P2` (821); ciclo 4 vuelve a exigir estados P1/P2 simultáneos (860).

Además, App desmonta LibraryPage al abrir Workflow. Volver a Biblioteca monta useLibrary con filtro ACTIVE nuevo. Tras consultar el archivado en 845, Return-ToLibrary en 848 pierde ARCHIVED y 849 busca Restaurar en una lista que excluye ese paper.

Fix: observar cada estado en su fase mediante los botones de consulta, esperar su carga y conservar evidencia separada sin ejecutar touch/back/advance compensatorios. En continue comprobar primero P2 activa/en curso, consultar P1 para comprobar completada y volver mediante `Activa · P2`. Antes de Restaurar seleccionar otra vez ARCHIVED y esperar la tarjeta exacta. No debilitar las comprobaciones a texto genérico.

Referencia congelada: `src/features/workflow/PhaseWorkspace.tsx` (nav y único estado por fase), `useWorkflow.ts` (reload preferActive tras advance), `src/app/App.tsx` y `src/features/library/useLibrary.ts` (desmontaje/default ACTIVE).

### [HIGH] Q5a — se espera una decisión ausente después de guardarla completa

Archivo: `native-qa.ps1:557–565`, `782–786`.

Issue: Fill-And-SaveP1 llama Save-P1Decision, que guarda sufficient + la rama con resolución ANSWERED. Todos los campos requeridos quedan procesados. Sin embargo, ciclo 2 espera el issue `Completa esta salida para continuar.` y rechaza el gate completo como si faltara la decisión. Un producto correcto termina en timeout y ciclo 3, que requiere la decisión persistida, queda inaccesible.

Fix: para el viaje autorizado de persistencia, evaluar y exigir gate completo después de los saves, comprobar que P1 sigue en curso y que no se ejecutó la rama. Si se conserva un paso negativo sin decisión, hacerlo antes de Save-P1Decision y después guardar la decisión dentro de ciclo 2; no añadir un ciclo ni un guardado reparador en ciclo 3.

Referencia: CONTRACTS/WORKFLOW_GATES, P1.v1.json; PhaseAnswerForm convierte ambos enums completos en ANSWERED.

### [HIGH] Q5b — los helpers consideran cargadas vistas todavía vacías

Archivo: `native-qa.ps1:437–439`, `456–460`, `463–466`, `653`, `850`.

Issue: cada proceso arranca en Home/onboarding (App inicializa view Home), sin `.library-page` ni `.workflow-workspace`. La llamada de 653 a Assert-WorkflowDomRevision198 se ejecuta antes de cycleAction y de Ensure-LibraryView, por lo que rechaza ese inicio válido antes de la navegación a Biblioteca.

Después de navegar, Ensure-LibraryView espera únicamente section y select.value, que cambia antes de terminar listPapers. Open-Workflow espera únicamente `.workflow-workspace`, que existe antes de getPaper/getPhase/answers/definition. Los asserts y clics inmediatamente posteriores pueden leer campos/tarjetas ausentes y fallar sobre producto correcto. La espera de restauración en 850 también pasa cuando la tarjeta está ausente, porque `buttons -notcontains Restaurar` es true para un array vacío; no acredita restore.

Fix: resolver onboarding/Home→Biblioteca antes del assert Library/Workflow. Después esperar resultados DOM propios: carga concluida sin alertas, título exacto y controles/estado de la fase requerida; en Biblioteca esperar la tarjeta exacta bajo el filtro seleccionado antes de actuar. Para restore exigir tarjeta presente con lifecycle Activo y acción Archivar, además de desaparición de Restaurar. Reutilizar Wait-Until existente.

Referencia congelada: LibraryPage renderiza Cargando documentos mientras loading; PhaseWorkspace existe con loading/paper null; useWorkflow obtiene sus datos de forma asíncrona.

### [HIGH] Q5c — texto largo esperado contradice la normalización canónica

Archivo: `native-qa.ps1:543–545`, `887–894`, `908–909`.

Issue: Get-ExpectedP1LongScope termina en espacio por `('αβγ12345 ' * 400)`. normalize_answer reemplaza CRLF/CR y recorta whitespace exterior; useWorkflow reemplaza el draft por la respuesta confirmada. Tras un save correcto, 894 compara contra texto con ese espacio final y falla; 909 repite la misma expectativa incompatible al reiniciar.

Fix: separar texto enviado de valor canónico esperado o generar el texto largo sin whitespace exterior. Mantener la comparación exacta del contenido canónico y comprobar longitud/foco antes de save con el texto enviado. No volver a guardar para compensar.

Referencia congelada: `src-tauri/src/domain/workflow.rs:246–260`, `src/features/workflow/useWorkflow.ts` (fromAnswer de result.data).

### [HIGH] Q7a — fallo de evidencia final puede devolver éxito al invocador

Archivo: `native-qa.ps1:979–985`, `993`.

Issue: si los seis ciclos terminan y falla Set-Content de native-result.json, catch marca failed-evidence-write y prueba un sidecar, pero no propaga error ni devuelve código distinto de cero. Invoke-QARun termina normalmente; una ejecución con evidencia final ausente/incompleta puede tener exit 0. El fallo del sidecar también se silencia. Los fallos de ciclo sí se propagan; esta rama final no.

Fix: después de terminar preservación/cleanup, hacer que cualquier resultado final distinto de passed falle explícitamente. Mantener la excepción primaria cuando ya existe y agregar el fallo de escritura como secundario; un fallo de evidencia tras recorrido exitoso debe producir exit no cero. No lanzar desde finally ocultando indiscriminadamente el error inicial.

### [HIGH] Q7b — se han reducido guardas de ownership respecto a FIX5

Archivo: `native-qa.ps1:122–126`, `625`, `642–652`, `700–708`.

Issue: readiness acepta GET /status sin comprobar que el listener pertenezca al PID del driver registrado (FIX5 sí lo comprobaba). Los puertos se liberan antes de Start-Process; la comprobación previa de ausencia no asegura la propiedad del endpoint usado.

Get-ProfileWebViewProcesses identifica cualquier msedgewebview2 por substring de perfil, omitiendo su ancestría al app PID, concordancia del runtime efectivo con ruta/versión fijadas y validación del userDataDir reportado. La observación de archivos + ese conjunto basta para acreditar perfil y después el mismo conjunto se usa en Stop-ObservedProcess. Coincidir en PID/creation/path al parar conserva identidad de lo observado, pero no demuestra que se adquirió como proceso propio. Consultar el runtime registrado en preflight no demuestra cuál cargó la app.

Fix: restaurar las guardas ya existentes en FIX5: readiness sobre listener propio; correlación perfil + ancestry al app identificado + runtime efectivo y rechazo de capability de perfil discordante cuando exista. Recoger esas identidades durante el ciclo para limpieza posterior aunque la app termine. Conservar fail-closed y no parar procesos cuya propiedad no se demuestra; no requiere nuevo harness.

Referencia histórica: FIX5 líneas 405–408, 436–466 y 655–673.

## Observación acotada del generador

### [MEDIUM] publicación inconsistente si se usa un subdirectorio permitido

Archivo: `make-synthetic-pdfs.ps1:6–11`, `49–65`.

Issue: OutputDirectory acepta descendants de fixtures y escribe allí los PDFs, pero manifest siempre se publica en fixtures/fixture-manifest.json y declara las tres rutas de la raíz. Con un subdirectorio permitido, el manifest puede señalar PDFs anteriores o ausentes en vez de los recién generados. WriteAllBytes/Set-Content tampoco rechazan destinos existentes; regenerar puede reemplazar evidencia anterior. Las tres fixtures actuales generadas en la raíz sí son coherentes y verificadas.

Fix mínimo: restringir a la carpeta exacta destinada a esta publicación y rechazar archivos/manifest existentes antes de cualquier write, o hacer que manifest/rutas correspondan exactamente al destino nuevo permitido. No editar T03 ni ampliar framework/dependencias.

## Cierre Q1–Q7

| Hallazgo previo | Estado por lectura de QA1 |
|---|---|
| Q1 | Cerrado estáticamente: área PSScriptRoot real del worktree; SHA aprobado externo elimina autorreferencia; PENDING bloquea. |
| Q2 | Cerrado estáticamente: restore entre cierre/cleanup 5 y lanzamiento 6; hash, contención y rechazo de overwrite; finally conserva recuperación. |
| Q3 | Cerrado estáticamente: observaciones inicializadas por ciclo; geometría/scroll fallan el ciclo; regla y=620; página2 espera raster distintivo; Process se dispone después de preservar evidencia. |
| Q4 | Corregido el selector option/value, etiqueta Tipo de revisión y resolución explícita. Exactitud de valores largos aún requiere Q5c. |
| Q5 | Parcial: hay waits de evaluación/avance e issue PDF exacto, pero quedan Q5a/Q5b/Q5c. |
| Q6 | Parcial: tres decisiones y draft de sesión presentes; quedan Q6a y carga real Q5b. |
| Q7 | Parcial: try cubre creación driver, recuperación de identidad temprana, errores secundarios, agregado en finally y restauración de env; quedan Q7a/Q7b. |

Interacciones de selección/guardado usan UI/WebDriver real; no encontré writes IPC ni inyección React. Test-ReaderScrollability mueve/restaura offsets DOM; eso es una interacción de scroll permitida, no una observación estrictamente sin efectos ni una mutación de estado del producto. Una biblioteca sintética compartida, seis perfiles/ciclos y cierre true/true/0 antes de DELETE están representados; las contradicciones anteriores impiden aún acreditarlos por ejecución. No ampliar el alcance a instalación o DPI no observado.

## Verificación propia y límites

Comandos de lectura ejecutados, con exit 0 salvo git show de dos nombres inexistentes y git diff --no-index que devuelve 1 al haber diferencias:

- `git diff --staged --stat`, `git diff --stat` en primary y worktree QA; primary contiene sólo cambio ajeno AGENTS, preservado.
- `git -C .worktrees/task-04c-native-runner diff 1da3965 17ca5d58790aae1fff4d2a2ee8592c9b41b6e203 -- work/qa/t04-native/native-qa.ps1 work/qa/t04-native/evidencias/qa1-static-review.json` (lectura del delta por bloques).
- `git -C .worktrees/task-04c-native-runner diff --check 1da3965 17ca5d5`: sin errores.
- `[System.Management.Automation.Language.Parser]::ParseFile(path,[ref]tokens,[ref]parseErrors)` para ambos scripts: **runner 10777 tokens / 0 errores; generador 788 / 0**. Sólo AST, sin invocar sus funciones ni dot-source.
- `Get-FileHash` de runner e históricos: coincidencias exactas con pins del brief.
- `Get-Item`, `ReadAllBytes`, SHA256 y comprobación ASCII header/pages/startxref/xref de las tres fixtures: cada una **1104 bytes**, hash exacto de manifest, **2 páginas declaradas estructuralmente**, startxref apunta a xref. No demuestra parsing/render PDF real.
- `git show 5d35f03:<archivo>` para los archivos de producto citados; no lectura de WIP. Intentos iniciales `src/App.tsx` y `WorkflowPage.tsx` inexistentes se resolvieron a App real y PhaseWorkspace; no son resultados de producto.

Único archivo escrito por este revisor: el presente informe central. Sin scripts/producto/commits/merges/configuración ni subagentes. No se ejecutaron runner/preflight/app/driver/listener/picker/GUI/npm/Cargo/instalador. El root puede disponer de gates/build distintos; este informe no les atribuye ejecución propia ni QA PASS. No se usa memoria histórica como autoridad para estos hallazgos.

Autorrevisión: lista consolidada por comportamiento, sin hallazgos cosméticos ni exigencia de evidencia futura. La siguiente corrección debe conservar seis ciclos, fixture allowlist, mutaciones sólo por UI, cierre normal y excepción primaria; volver a revisar SHA/delta antes de ejecutar.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH | 6 | warn |
| MEDIUM | 1 | info |
| LOW | 0 | note |

Verdict: **WARNING — 6 HIGH deben resolverse antes de ejecutar el runner.** Revisión terminada; ejecución nativa **NOT_RUN**.
