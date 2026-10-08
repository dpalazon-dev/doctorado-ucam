# T04c native runner QA2 — corrección estática

Fecha: 2026-10-05. Autor: `/root/task04c_native_runner`.
Estado: **DONE_WITH_CONCERNS**. Runner congelado para revisión independiente; no ejecutar hasta que root cierre su revisión.

## Corte y alcance

- Worktree: `C:/Users/david/Projects/Research-Workbench/.worktrees/task-04c-native-runner`.
- BASE QA1: `17ca5d58790aae1fff4d2a2ee8592c9b41b6e203`.
- Commit QA2: `011f153c90f9f4cef30fc67079eaaebe12519823` (`qa: fix T04c native runner review findings`); rama limpia tras commit. Delta propio: runner `+161/-64`, generador `+11/-7`, evidencia nueva `+54`.
- Referencia de producto consultada sólo por SHA inmutable: `5d35f03b13c4e85a0944296bac20e593cf494b5c`.
- Únicos cambios QA: `work/qa/t04-native/native-qa.ps1`, `work/qa/t04-native/make-synthetic-pdfs.ps1` y evidencia nueva `work/qa/t04-native/evidencias/qa2-static.json`.
- No cambios de producto, fixture PDFs ni manifest. No se ejecutaron el runner, preflight, aplicación, driver, listener, selector nativo, GUI, npm, Cargo o instalador. El generador tampoco se ejecutó.

## Hallazgos cerrados

- **Q6a:** ciclo 2 observa PRE en su vista y no exige P1 simultáneo. Después de `continue`, el runner registra P2 activo, consulta P1 para registrar su cierre y vuelve con `Activa · P2`. En el reinicio de ciclo 4 espera P2 activo antes de consultar P1. El retorno al listado reinicia el filtro, por lo que vuelve a aplicar ARCHIVED antes de Restaurar; tras el único clic, selecciona ALL y exige la tarjeta exacta activa, con Archivar y sin Restaurar.
- **Q5a:** el ciclo 2 guarda la decisión completa, exige gate P1 completo y estado P1 en curso / paper ACTIVE; no espera un error de decisión ausente ni ejecuta la rama.
- **Q5b:** navega de Home/onboarding a Biblioteca antes de usar sus datos. Las esperas de lista, tarjeta y Workflow requieren carga concluida, sin alerta de error, título exacto y fase/controles de la fase solicitada. La restauración exige presencia y estado visible de la tarjeta, no la ausencia de un botón en una tarjeta vacía.
- **Q5c:** el texto largo no tiene espacio final, de modo que la comparación antes/después corresponde a la normalización `trim` del contrato. No se añade un guardado reparador.
- **Q7a:** la excepción primaria se conserva mientras se intenta recuperación y escritura final. Un fallo al guardar JSON o sidecar queda registrado como secundario cuando es posible, y cualquier estado agregado distinto de `passed` termina con excepción/código no cero después del cleanup.
- **Q7b:** readiness requiere que el listener WebDriver pertenezca al PID de tauri-driver. La propiedad WebView2 requiere perfil, ascendencia hasta el PID exacto de la app y coincidencia con ruta/versión del runtime registrado; una capability `userDataDir` discordante se rechaza. Se guardan identidades mientras la app está viva y se detienen sólo observaciones con propiedad demostrada y PID/ruta/fecha de creación aún coincidentes. Las observaciones no verificadas quedan en evidencia y no se detienen.
- **Generador MEDIUM:** sólo acepta la raíz exacta `fixtures`, exige que ya exista, rechaza cualquier destino PDF/manifest ocupado antes de escribir y usa `CreateNew` para rechazar también una colisión tardía.

## Pines y evidencia estática

- Runner SHA-256: `5F542D388D05699AEB3CD3C8C761E1678C59DCCC0747215918ABE109DC798283` (1,090 líneas al corte de evidencia).
- Generador SHA-256: `73CEA6AAFE3C6E38EE71338469D1E274903C2C27981DC947113DF40A271EBE42`.
- Candidato: `C:/Users/david/Projects/Research-Workbench/work/qa/t04-native/candidate-32249b7a/research-workbench.exe`, 27,037,184 bytes, hash `32249B7A168D8B8AA53E65D82BE11A55FA6EE0B2294DE94682A424CC0921BDEC`.
- WebView2 y EdgeDriver fijados a `154.0.4258.53`. Se verificó por lectura que existen y coinciden con los pins: FIX5 `1F986F2AA515F6257981122B00E280E0B5D354C5378632206806002C721B119A`, tauri-driver `2684A7B9E1E667D6689BEA8EAFF2AEFE093C5B1F5CA329B3156B384A9CAB9286`, EdgeDriver `008115B68B38437B1C0138F3CCED648F61F333CF4623A9895296E7C1C01ABCB8`.
- Los tres PDF existentes tienen 1,104 bytes cada uno, dos páginas declaradas y hashes SHA-256 distintos: continue `E224CA384B0B7ACA8FE6BEECC07AA951F7E7EE12F8C430D86C43E9915C525205`; light-read `753BDE30628B0EC9C129FF6B541C1DEC05C484A17C823CADC3F4712AE09B03A9`; archive `E9CEBA49CBFF46EBE7A6916715BF77085CDB76C177EDFFD2E75D3294B27F845E`.
- Evidencia legible por máquina: `work/qa/t04-native/evidencias/qa2-static.json`.

Comprobaciones ejecutadas sin invocar scripts:

- `[System.Management.Automation.Language.Parser]::ParseFile` en runner y generador: runner 11,967 tokens / 0 errores; generador 940 tokens / 0 errores.
- `git diff --check`: sin errores de whitespace (Git avisó que normalizará CRLF del runner al siguiente write).
- `Test-Path`, `Get-Item` y `Get-FileHash` sobre candidato, FIX5 y ambos drivers: las cuatro rutas existen y coinciden con sus hashes fijados.
- `Get-FileHash` sobre los tres PDF: coincide con manifest; contenido y manifest no se modificaron.

## Límites y revisión pendiente

El trabajo acredita preparación y parseo estático, no compatibilidad funcional, comportamiento de GUI, instalación ni QA PASS. Los seis ciclos permanecen sin ejecutar. El parámetro externo `ApprovedRunnerSha256` sigue siendo obligatorio y debe corresponder al script revisado final.

La ruta al FIX5 histórico y a los drivers quedó anclada por indicación de Sol al checkout primario absoluto; `area` y las evidencias siguen en el `PSScriptRoot` del worktree QA. Root debe revisar el diff y fijar el SHA externo aprobado antes de cualquier ejecución.

Sin decisiones de Sol pendientes. No se realizó merge ni se tocó configuración global.
