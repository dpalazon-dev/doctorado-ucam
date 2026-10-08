# T03 fix2 — revisión focal de conformidad

Estado: DONE_WITH_CONCERNS. Veredicto del alcance F2-4: **PASS**, con una observación menor de cobertura. No constituye aprobación global del delta.

Revisor: /root/task03_spec_review. Fecha: 2026-10-03.
BASE: `b36ae5ae0529ba429edf7a360ade7319e353e8fe`.
HEAD: `30d532714d053c09d4021bc89bafbb0168b1602a`.
Worktree: `.worktrees/task-03-reader`, limpio al inicio y después de comprobar.

## Alcance y fuentes

Leídos brief de corrección/revisión fix2, reporte del autor y continuidad del informe SPEC fix1. Inspeccionados App, ImportPaperDialog, LibraryPage, su delta y sus pruebas. Se conserva la interpretación normativa de TASK03_BOUNDARIES/REQ-002 y F2-4 ya revisada. Sin cambios de producto/tests/configuración, subagentes, merges, GUI o bibliotecas personales; solo se crea este informe central.

F2-1/F2-2 y las barreras R2 tienen revisión especializada. No se repiten probes de los tres Important TypeScript ni de la señal de admisión Rust comunicados por el orquestador. Sus hallazgos permanecen independientes y deben cerrarse antes de integrar.

## F2-4 — cerrado

- ImportPaperDialog notifica onOperationFinished en finally de selección, confirmación y cancelación; LibraryPage lo conecta con el refresco de Settings en App. Por ello se consulta también tras errores que pueden dejar importaciones pendientes, además de éxitos y cancelación que limpia la última intención.
- Entrar en Configuración consulta nuevamente el estado; el aviso de recuperación ofrece recomprobación manual aun si la consulta anterior tuvo éxito.
- statusRequest descarta respuestas y errores de consultas antiguas. App conserva la distinción entre fallo de consulta, writable y recoveryRequired.
- No se añadió IPC ni una API de recuperación de drafts. El estado procede del SettingsApi existente.

Se verificaron en pruebas persistentes de App: selección fallida que cambia recuperación false→true; consulta al entrar en Settings; cancelación que elimina el último pendiente; acción manual true→false; respuesta antigua que no sustituye el estado reciente.

Además, un probe inline cargó App real mediante Vite SSR/JSDOM con APIs sintéticas y seam solo para no cargar PaperReader/PDF.js. Secuencia: arranque sin recuperación → selección con preview → confirmación fallida → cancelación fallida → cancelación correcta. Resultados, exit0:

```text
CONFIRM_FAILED_STATUS {"statusCalls":3,"recovery":true,"notice":true}
CANCEL_FAILED_STATUS {"statusCalls":4,"recovery":true,"draft":"Draft"}
CANCEL_SUCCESS_STATUS {"statusCalls":5,"recovery":false,"dialog":false,"notice":false}
```

Así se comprueba el cableado real de las terminales no ejercitadas directamente por las cuatro pruebas persistentes de App. La API y el estado backend siguen simulados: no es prueba de IPC/WebView2.

## Regresiones de importación

Las regresiones añadidas cubren confirmación tardía de duplicado después de introducir DOI, presentación de dos candidatos, elección B con cancelación pendiente/Escape, bloqueo de salida durante selección y confirmación, y conservación del borrador cuando esa confirmación termina con error. El defecto S2 no se reabre por el delta observado.

### [LOW / Minor] M1 — La prueba de details inválidos no ejerce la respuesta de duplicados

Ruta: `src/features/library/library.test.tsx`, test `invalid_import_details_keep_the_draft_and_do_not_confirm`.

El brief F2-4 pidió incorporar la regresión inline previa donde confirmImport devuelve DuplicateDecisionRequired con error.details.candidates inválido. El nuevo test introduce año `20x4` y comprueba que confirmImport nunca se llama. Verifica validación local de metadatos, pero no la validación de candidatos del backend ni la conservación del borrador en esa rama. El informe del autor menciona «detalles inválidos», que puede interpretarse como cobertura de la rama pedida cuando no existe.

Corrección: conservar el caso de año inválido y añadir el caso acotado de respuesta con candidato/UUID inválido, comprobando error accesible, borrador conservado, ausencia de apertura y cancelación disponible. Ajustar la descripción de evidencia. No se ha demostrado un defecto nuevo de producción en esa rama; el probe previo de fix1 pasó y el código de validación no cambió. Esta observación no bloquea F2-4 ni reabre S2 como bug.

## Comandos y evidencia propia

- `git status --short`, `git rev-parse HEAD`: limpio; HEAD indicado.
- `git diff --stat b36ae5ae0529ba429edf7a360ade7319e353e8fe 30d532714d053c09d4021bc89bafbb0168b1602a`: 15 archivos, 1779 inserciones/274 eliminaciones; revisión focal de archivos indicados.
- `node node_modules/vitest/vitest.mjs run src/app/App.test.tsx src/features/library/library.test.tsx -t 'recovery|late_duplicate|candidate_choice|invalid_import|escape_does'`: exit0, **9 pasadas / 16 omitidas por filtro**, dos archivos. No se ejecutó la suite completa.
- Un primer intento equivalente mediante npm.cmd falló por interpretación de los separadores `|` en Windows; no llegó a comprobar comportamiento. Se corrigió invocando directamente Vitest mediante Node como figura arriba.
- Probe inline `node --input-type=module`: exit0; script completo en las herramientas de esta revisión. Vite SSR/JSDOM, sin escribir scripts ni pruebas al checkout. Resultados transcritos arriba.

El gate59 UI/128 entradas Rust y build0 son evidencia verificada por el orquestador, no nuevas ejecuciones de este revisor. Las entradas Rust incluyen helpers; no se adoptan como 128 comportamientos independientes. La ruta correcta de los logs es `work/evidence` bajo el worktree; la errata del reporte que señala la raíz ya está identificada para corrección editorial.

No se afirma render PDF real, layout, selector nativo, instalación, rendimiento ni equipo limpio. Tampoco se atribuye a fix2 la exploración WebView2 de otro binario/corte.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH | 0 | pass |
| MEDIUM | 0 | pass |
| LOW | 1 | note |

Verdict: **APPROVE / PASS para F2-4 y el alcance SPEC revisado**. S5 residual queda cerrado. Resolver la observación menor de cobertura; la integración global sigue condicionada a los Important de las otras revisiones, no incluidos de nuevo en este conteo.
