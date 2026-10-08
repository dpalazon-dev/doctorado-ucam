# T04c UI Workflow PRE/P1 — checkpoint de implementación

**Estado:** CHECKPOINT. La implementación y los gates de código están en `198f27b281d97810d16fb2955b620af4f99b1490`, pero no se declara funcionalmente cerrada ni integrada. La revisión de Sol está en curso y ha identificado un fallo importante que requiere fix1 del mismo autor.

## Corte y commits

- BASE: `d4a0c77b691d58493f03f622340005de0e6193ed`.
- Rama/worktree: `agent/task-04c-workflow-ui`, `C:/Users/david/Projects/Research-Workbench/.worktrees/task-04c-workflow-ui`.
- Commits de implementación: `6e8ffcbc46e8fa324d6cb448828aca6a9f621d33` (`feat(workflow): add PRE and P1 workspace UI`) y `198f27b281d97810d16fb2955b620af4f99b1490` (`fix(workflow): reconcile retries and refreshed phase tokens`).
- El segundo commit es el SHA congelado para revisión y gates. El worktree estaba limpio antes de ejecutar los gates.

## Cambios incluidos

La entrada Procesamiento de Biblioteca navega por `paperId`, invalida aperturas Reader tardías y no lee el PDF ni muta estado al montar. App conserva en memoria de sesión los drafts por paper/fase/pregunta al volver a Biblioteca o cambiar de paper. El workspace consulta PRE/P1/P2, pide la definición con la versión fijada por `PhaseDto` y diferencia fase consultada de activa. P3/P4, P2, `ARCHIVED` y `COMPLETED` son de solo lectura según la capacidad disponible.

Los formularios usan las resoluciones permitidas por definición, texto sin HTML, confirmación explícita de `PRE.review_type`, UNKNOWN/NOT_APPLICABLE con explicación y la decisión estructurada de P1 sin inferir su rama del texto. Evaluar y avanzar son acciones separadas. Las tres ramas de P1 consumen el resultado del backend. Los guardados preservan ediciones posteriores, serializan mutaciones por paper, manejan conflictos de forma explícita y reintentan solicitudes no confirmadas con el mismo payload. Tras un guardado confirmado se rehidrata paper/fase; un refresco fallido no vuelve a enviar la mutación. Consultar una fase `NOT_STARTED` no permite guardarla, evaluarla ni iniciarla implícitamente.

Archivos dentro del ownership del brief: `src/app/{App.tsx,App.test.tsx,ShellView.ts,tokens.css}`, `src/features/library/{LibraryPage.tsx,library.test.tsx}`, los cinco nuevos archivos de `src/features/workflow/` y este informe. No se cambiaron contratos, Rust, wire, Reader ni archivos fuera del ownership.

## Evidencia TDD y validación ejecutada

Los tests de comportamiento añadidos se probaron en RED antes del cambio correspondiente y en GREEN después. Entre los casos RED/GREEN observados están: entrada Library sin Reader/touch, fase `NOT_STARTED` solo consulta, error tardío de revisión de conflicto tras cambiar de fase, token de fase rehidratado tras save confirmado, retry de transición con el mismo comando y fallo de refresh tras mutación confirmada sin segundo envío. La suite también cubre resolución de conflicto, doble submit, edición mientras guarda, ramas P1, gate bloqueado, consulta P2/archivada, contaminación entre papers, preview tardía, PDF no disponible y atributos accesibles.

Comandos y resultados reales:

- `npm.cmd test -- src/features/workflow/workflow.test.tsx src/features/library/library.test.tsx src/app/App.test.tsx` — exit 0; 3 archivos, 59 tests aprobados.
- `npm.cmd run typecheck` — exit 0.
- `git diff --check` — exit 0.
- `powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/check.ps1` — exit 0. Se dot-sourceó antes `scripts/development-env.ps1`. El gate ejecutó typecheck, Vitest completo (11 archivos/102 tests aprobados), build Vite, `cargo fmt --check`, Clippy con `-D warnings`, Cargo tests y comprobación de contratos generados.
- `npm.cmd run tauri:build -- --debug --no-bundle` — exit 0. Se dot-sourceó `scripts/development-env.ps1`; produjo `C:/Users/david/Projects/Research-Workbench/work/cargo-target/debug/research-workbench.exe`.

Los logs reales de stdout, stderr y exit code están en `C:/Users/david/Projects/Research-Workbench/work/task-04c/`: `check.stdout.log`, `check.stderr.log`, `check.exit.txt`, `tauri-debug-no-bundle.stdout.log`, `tauri-debug-no-bundle.stderr.log` y `tauri-debug-no-bundle.exit.txt`. Tauri/Vite informa un aviso de chunk JavaScript de 872.77 kB antes de gzip; la compilación terminó correctamente.

## Revisión pendiente y limitaciones

Sol ha confirmado que `canAdvance` exige `phase.state === "IN_PROGRESS"` y por ello no permite completar/reconfirmar una fase `NEEDS_REVIEW`; el brief espera que la reconfirmación explícita sea posible. También pidió verificar el remount de A con una operación admitida y sus drafts/error de paper. No se modificó código tras congelar `198f27b`; ambos puntos quedan para el fix1 solicitado por Sol y la revisión posterior.

No se ejecutó QA manual de GUI con biblioteca sintética, escalas DPI, reinstalación ni entorno Windows limpio. El build debug no-bundle y los tests automatizados no demuestran esos escenarios. No se hizo merge, push, release ni modificación de ledger/STATUS.

**Autorrevisión checkpoint:** diff y scope revisados frente al ownership; tests focales y gate completo pasan en el SHA indicado; árbol de este worktree limpio al cerrar gates. No declarar DONE hasta corregir/revisar los hallazgos anteriores y recibir validación final de Sol.
