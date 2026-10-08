# T04c fix2 — informe de corrección

**Estado:** `DONE_WITH_CONCERNS` para revisión independiente acotada. F1 y F2 están implementados y sus regresiones son verdes en UI simulada. No se declara integración: `scripts/check.ps1` y Tauri debug no-bundle quedan diferidos hasta que el orquestador apruebe el corte.

## Identidad

- Worktree: `.worktrees/task-04c-workflow-ui`
- Rama: `agent/task-04c-workflow-ui`
- BASE fix2: `1f0fbdf7d4a422bdf14b9aafdcd74dbffdcfe8db`
- BASE producto fix1: `37169112abd10d7639265af589b76904e734b290`
- Commit de producto: `5d35f03b13c4e85a0944296bac20e593cf494b5c` (`fix(workflow): reconcile refresh state after remount`)
- Commit de este informe: documental y separado del producto.

## Correcciones

F1: el montaje ahora ejecuta la misma relectura completa usada para recuperar una operación confirmada. Lee Paper, la fase vigente y, si es distinta, la fase activa antes de limpiar `refreshRequired`. Cada confirmación incrementa `refreshVersion`; una lectura sólo puede limpiar la obligación que observó al comenzar, y una confirmación posterior la mantiene abierta. Si falla la relectura, la acción explícita «Reintentar carga» permanece visible aunque una consulta parcial de fase haya limpiado el error anterior. El retry vuelve a leer el estado canónico y no reenvía la mutación.

F2: un `GateBlocked` confirmado elimina la preview completa previa. No crea `pendingAction` ni evalúa automáticamente; sólo una reevaluación explícita puede producir una nueva preview que habilite el avance. Se conserva el borrador y el contexto de fase.

No se tocaron contratos, Rust, Reader, dependencias, configuración ni el fixture App opcional.

## Regresiones y verificación

Pruebas TDD focales en `src/features/workflow/workflow.test.tsx` cubren save y transición confirmados sin consumidor seguidos de remount; tokens actuales tras remount; refresh fallido seguido de remount; recuperación después de consultar otra fase; fallo al releer la fase activa y recuperación completa con lifecycle ARCHIVED; una hidratación que empezó antes de otra confirmación; y GateBlocked con invalidación de preview y reevaluación explícita.

La ejecución RED antes de corregir F1/F2 registró **5 fallos esperados y 49 pruebas verdes**. Tras añadir los casos de fase activa e hidratación concurrente, el corte actual pasó la suite focal: **56/56**, exit `0`. `npm run typecheck` pasó (`tsc -b`), exit `0`. `git diff --check` pasó antes del commit de producto; el árbol quedó limpio tras los commits.

Comandos ejecutados desde el worktree, con `. .\scripts\development-env.ps1` antes de cada comando:

```powershell
npm.cmd test -- --run --reporter=dot src/features/workflow/workflow.test.tsx
npm.cmd run typecheck
```

Stdout, stderr y exit codes inmediatos conservados en `C:\Users\david\Projects\Research-Workbench\work\task-04c-fix2\` (`red.*`, `green.*`, `typecheck.*`). `red.*` conserva la reproducción previa al cambio; `green.*` y `typecheck.*` son del corte verde. No se ejecutó la suite completa ni Cargo ni Tauri, conforme al brief mientras espera la revisión acotada. No se hizo QA nativa, instalador o prueba de equipo limpio.

## Autorrevisión y límites

Los botones de mutación siguen bloqueados mientras `refreshRequired` esté pendiente; los formularios conservan borradores. El contador protege lecturas concurrentes de una confirmación posterior, y las guardas de generación ya existentes impiden que una lectura obsoleta pinte Paper/fase. `GateBlocked` sólo descarta la preview; la relectura del gate depende de la acción expresa del usuario.

La evidencia demuestra los escenarios de comportamiento en React/Vitest y el typecheck del corte. Revisión independiente pendiente; después, el orquestador decidirá y ejecutará una sola vez el gate/build integrados. No se afirma funcionamiento nativo ni integración.
