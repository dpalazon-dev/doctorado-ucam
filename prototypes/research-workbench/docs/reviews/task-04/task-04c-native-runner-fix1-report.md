# T04c runner QA1: correcciones de preparación

Fecha: 2026-10-05  
BASE: `1da396562a940c629ac5e85b372988e802bf3944`  
Referencia DOM congelada: `5d35f03b13c4e85a0944296bac20e593cf494b5c` (fix2)  
Estado: `DONE_WITH_CONCERNS` — corrección y revisión estática listas; QA nativa no ejecutada.

## Cambios Q1–Q7

- **Q1:** el área se resuelve desde `$PSScriptRoot`, por lo que runner y fixtures se leen del checkout QA real. El SHA revisado se recibe como parámetro externo obligatorio `-ApprovedRunnerSha256`; se quitó el pin autorreferente. La ruta y el hash del ejecutable siguen `PENDING_SOL_FREEZE`, con guardas fail-closed.
- **Q2:** el PDF se mueve fuera del dataRoot sintético antes del ciclo 5. Tras su cierre normal y gates de limpieza se restaura, con hash exacto, contención de rutas y rechazo de sobrescritura, antes de iniciar el ciclo 6. El `finally` conserva recuperación ante fallo.
- **Q3:** las observaciones de canvas y layout se inicializan por ciclo; proporción de canvas, raster de ambas páginas y scroll son aserciones que fallan el ciclo. La línea de fixture se mide en y=620, igual que el PDF. Se dispone del objeto `Process` después de capturar evidencia.
- **Q4:** las opciones se localizan por `<option value>` y se comprueba el valor del `<select>`. La etiqueta de revisión usa `starts-with` para no incluir el texto de opciones. Las resoluciones se seleccionan explícitamente; estados y transiciones corresponden a las etiquetas de UI observadas.
- **Q5:** los gates esperan botón habilitado, resultado y transición exactos antes de avanzar o capturar evidencia. Los ciclos 5 y 6 comprueban el issue exacto de PDF no disponible y el gate recuperado.
- **Q6:** el ciclo 4 selecciona `ALL` y `ARCHIVED` explícitamente; continue comprueba P1 completada y P2 en curso, y P2 sigue en consulta readonly. Ciclo 2 comprueba que un borrador local sobrevive Biblioteca→Paper; los ciclos con proceso nuevo comprueban valores guardados sin esperar persistencia del borrador. Las decisiones P1 requieren ambos valores y resolución guardada.
- **Q7:** el límite de cleanup cubre desde la creación del driver. En un fallo temprano se registra la ventana temporal, candidatos exactos y ancestría; sólo se para una identidad propia validada. Cada cleanup/escritura captura su error sin ocultar la excepción principal; el agregado se escribe en `finally`, y `Process` se dispone después de guardar evidencia. Se restaura el valor previo de la variable de entorno del proceso.

Se conservan seis ciclos y una biblioteca sintética compartida: cuatro perfiles/sesiones originales más los ciclos de PDF no disponible y recuperación, todos con cierres normales antes de `DELETE`. Las tres fixtures PDF son distintas y sus hashes coinciden con el manifiesto. Se siguen reutilizando, inmutables, el runner T03 FIX5 (`1F986F2AA515F6257981122B00E280E0B5D354C5378632206806002C721B119A`) y el generador T03 (`45A7F37D90982CBB761127EF0915804858174202692762E50E1EAC165374AFA9`).

Los selectores y etiquetas se contrastaron sólo por lectura contra `5d35f03b13c4e85a0944296bac20e593cf494b5c`; la diferencia relevante respecto a `37169112` añade estado/reintento de carga, sin cambiar los selectores usados. El helper de scroll desplaza y restaura únicamente offsets DOM como observación Reader; no cambia estado React ni llama IPC.

## Verificación y límites

Hashes QA1: runner 8208020894536F688F3B5BD39909779D320255228BF0F8462CC6DC5D5AA38AF5; evidencia estática E7FEE9D448864725D66D2CEF5FB5D3E9E8DA5FA5D63FEEC5632D9D0497DC3460.

Comandos ejecutados en el worktree QA:

- `[System.Management.Automation.Language.Parser]::ParseFile(...)` para `native-qa.ps1` y `make-synthetic-pdfs.ps1`: cero errores; 10.777 y 788 tokens respectivamente.
- `Get-FileHash` y longitud de los tres PDF contra `fixture-manifest.json`: las tres coincidencias exactas, 1.104 bytes cada una.
- `git diff --check`: sin errores.
- Lectura estática mediante `git show` de DOM congelado fix2.

No se invocó el runner ni se lanzó producto, driver, listener, picker, GUI o proceso auxiliar. No se ejecutaron `npm`, Cargo, instalador ni pruebas nativas. Por tanto, no hay QA PASS ni afirmación de instalación. El ejecutable candidato y su SHA siguen pendientes; antes de usar el runner aún hace falta el pin de Sol y la revisión independiente del runner. El JSON de evidencia está en `work/qa/t04-native/evidencias/qa1-static-review.json`.

Commit QA1: `17ca5d58790aae1fff4d2a2ee8592c9b41b6e203` (`qa(t04c): address runner review findings`). Worktree limpio; sin merge.

