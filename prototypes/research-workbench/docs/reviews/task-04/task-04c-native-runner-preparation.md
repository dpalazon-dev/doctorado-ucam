# T04c — preparación del runner QA nativo

Estado: **DONE_WITH_CONCERNS — preparado para revisión interna; ejecución bloqueada**. Rama `agent/task-04c-native-runner`, BASE `198f27b` (checkpoint T04c). No se leyó WIP. El worktree del autor de producto no se modificó.

## Cambios

- `work/qa/t04-native/native-qa.ps1`: runner preparado a partir de los helpers FIX5; la entrada de Reader se adapta a Workflow y al recorrido persistido. Conserva el selector Win32 nativo con asociación PID/HWND y ahora admite exclusivamente las tres rutas y hashes del manifiesto. Usa una biblioteca sintética común y seis ciclos con perfiles nuevos: cuatro ciclos principales para importación/PRE, PRE→P1 y guardado sin decisión, ejecución de las tres ramas, y verificación tras reinicio más restauración explícita; dos ciclos adicionales prueban un PDF temporalmente indisponible, un guardado permitido, bloqueo de gate y reevaluación tras restaurar los bytes.
- El runner registra identidad del proceso de app y driver, SafeHandle, sesión, puertos y perfil por ciclo; requiere `CloseMainWindow=true`, `WaitForExit=true` y `ExitCode=0` antes de `DELETE session`. El cleanup queda acotado a PID/ruta/creation time registrados. Las interacciones de formulario usan WebDriver; `execute/sync` solo lee DOM.
- El recorrido incorpora Reader (canvas de dos páginas, proporción y scroll), valores persistidos PRE/P1, `UNKNOWN` con explicación separada, decisiones estructuradas completas, consulta P2, lectura archivada, restauración explícita, labels, foco, texto largo y resize. DPI queda pendiente de disponibilidad observada.
- `work/qa/t04-native/make-synthetic-pdfs.ps1` adapta la estructura PDF de dos páginas del generador FIX5 en un generador independiente. Genera tres papers sintéticos, más `fixture-manifest.json` y los PDF bajo `fixtures/`. No modificó ni ejecutó el generador histórico sobre su fixture.
- `work/qa/t04-native/evidencias/preparacion-static.json` conserva hashes y resultados de parseo/integridad.

## Comparación con el runner histórico

El runner y generador T03 permanecen intactos y sus hashes coinciden con los fijados en el brief: `native-qa-fix5.ps1` `1F986F2AA515F6257981122B00E280E0B5D354C5378632206806002C721B119A`; `make-synthetic-pdf.ps1` `45A7F37D90982CBB761127EF0915804858174202692762E50E1EAC165374AFA9`.

La copia conserva las primitivas WebDriver, espera, interacción, captura, observación Reader y selector nativo de FIX5. Sustituye el pin de un PDF y una sesión por allowlist de tres PDFs, root compartido, seis perfiles/sesiones y un viaje explícito entre procesos. Retira el flujo Reader como camino principal y reutiliza sus medidas como regresión. El hash del runner nuevo es `E132D64B1AA492545424733124373EF9253309EFB06C933A78DC83F277943C0E`; el generador nuevo es `C61B2063D58C707224F04823BEE3112C7A1E2B72BD78A7C230771F3C5373CF9F`.

Los selectores Workflow proceden únicamente del DOM de `git show 198f27b` y están comentados como provisionales. El fix1 opcional `37169112` apareció después de iniciar esta preparación y conserva un residual de remount según Sol; no se leyó su WIP. Los selectores finales deberán contrastarse en el próximo corte corregido congelado.

## Comprobaciones ejecutadas

- Generación autorizada de fixtures: `& '.\work\qa\t04-native\make-synthetic-pdfs.ps1' -OutputDirectory '<worktree>\work\qa\t04-native\fixtures'` — terminó correctamente.
- Parseo estático: `System.Management.Automation.Language.Parser.ParseFile` — 0 errores para ambos scripts (`native-qa.ps1`: 9,359 tokens; generador: 788 tokens).
- Integridad local: tres PDF de 1,104 bytes; dos páginas cada uno; offsets `startxref` válidos en 881; SHA-256 iguales al manifiesto y distintos entre sí:
  - `RW-T04C-A` / `continue`: `E224CA384B0B7ACA8FE6BEECC07AA951F7E7EE12F8C430D86C43E9915C525205`
  - `RW-T04C-B` / `light_read`: `753BDE30628B0EC9C129FF6B541C1DEC05C484A17C823CADC3F4712AE09B03A9`
  - `RW-T04C-C` / `archive`: `E9CEBA49CBFF46EBE7A6916715BF77085CDB76C177EDFFD2E75D3294B27F845E`
- `git diff --check` — exit 0. No se ejecutaron tests, npm, Cargo, app, listener, driver, picker ni runner. No se atribuye PASS nativo.

## Bloqueos y decisiones para Sol

- `sourceExe` y su SHA siguen `PENDING_SOL_FREEZE`; la comprobación inicial hace fallar el runner antes de leer manifiesto o iniciar procesos. El hash de runner revisado y los pins WebView2/EdgeDriver también siguen `PENDING`.
- Antes de cualquier ejecución, fijar exe debug/commit/SHA y rutas, revisar el diff del runner y su aislamiento, comparar selectores contra el DOM final congelado y colocar el hash aprobado en `expectedReviewedRunnerHash`. Releer versión/ruta/hash de runtime y drivers justo antes de autorizar `-ExecuteAfterReview`.
- Confirmar si los dos ciclos extra de PDF indisponible/recuperación se conservan junto a los cuatro ciclos principales; el brief los contempla como ciclos adicionales.
- DPI 100/150/200 no está automatizado porque la disponibilidad no se consultó ni se cambiará configuración global. Instalador, entorno limpio, offline, upgrade y desinstalación siguen fuera del alcance.
- El parseo PowerShell y el encabezado/xref no prueban el render PDF real; esa evidencia solo se generará al importar desde la UI tras revisión y autorización interna.

Autorrevisión: el runner permanece fail-closed en el estado actual y no invoca comandos de versión en modo de preparación. Los selectores, el orden real de confirmaciones, respuesta de gate y persistencia siguen sin comprobarse en la app; el artefacto requiere revisión independiente de Sol antes de habilitar ejecución.
