# T04c fix1 — checkpoint de gate

**Estado:** `DONE_WITH_CONCERNS` para revisión; no declarar integración funcional. La revisión estática encontró un caso S2/F1 pendiente, por lo que el build Tauri no se ejecutó sobre este corte.

## Identidad

- Rama: `agent/task-04c-workflow-ui`
- Base fix1: `c14c97dd8cfa2209e5596ef5ef8c39540973ad99`
- Producto congelado: `37169112abd10d7639265af589b76904e734b290` (`fix(workflow): reconcile confirmed actions across remounts`)
- Gate documental: este informe solamente; HEAD de producto sigue siendo el SHA anterior.

## Cambios y cierres de fix1

El delta de producto aborda S1–S8, TS9/TS10 y R1: reconfirmación NEEDS_REVIEW PRE/P1; relectura/notificación entre vistas; bloqueo por intenciones locales globales; identidad monotónica y replay con argumentos congelados; distinción entre respuesta ausente y PENDING durable; lectura P2 sin mutación; conflicto con valores actuales y borrador local; reevaluación explícita de GateBlocked; fixture Reader tipado; y replay acotado de una operación ARCHIVED ya pendiente.

También se cubre el caso de recibo histórico: el resultado confirma la operación original, no el lifecycle/contexto actual. La vista archivada no adopta lifecycle histórico; una relectura completa gobierna controles y contexto. `refreshRequired` impide transiciones con tokens viejos tras éxito confirmado y refresh fallido, y Retry Load vuelve a reconciliar sin reenviar la mutación.

## Validación ejecutada

Desde el worktree indicado, se dot-sourceó `scripts/development-env.ps1` y se ejecutó:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/check.ps1
```

Resultado: exit code `0`. El gate completo registró **122 tests frontend en 11 archivos** y **181 entradas de pruebas Rust**, incluidas las suites/helper. Los logs preservados son:

- `C:\Users\david\Projects\Research-Workbench\work\task-04c-fix1\check.stdout.log`
- `C:\Users\david\Projects\Research-Workbench\work\task-04c-fix1\check.stderr.log`
- `C:\Users\david\Projects\Research-Workbench\work\task-04c-fix1\check.exit-code.txt`

El producto permanece sin cambios desde el SHA congelado; el commit documental de este informe queda en HEAD de la rama y el worktree queda limpio tras ese commit. No se ejecutó `npm run tauri:build -- --debug --no-bundle`: el orquestador difirió ese gate hasta resolver F1. Tampoco se hizo QA GUI/instalador.

## Limitación abierta: S2/F1

Si una mutación se confirma cuando Paper A está desmontado, no hay listener que solicite reconciliación. El callback local de refresh puede salir al encontrar la vista desmontada. Al volver a montar A, la hidratación inicial carga Paper y fase, pero no limpia su `refreshRequired`; los controles siguen bloqueados y no aparece una acción de recuperación. El mismo callejón sin salida se reproduce al desmontar/remontar tras un refresh fallido. La relectura al montar debe completar la reconciliación vigente y limpiar ese estado sin recrear ni reenviar la mutación confirmada.

Este caso está abierto para corrección/revisión independiente. El corte no es candidato a integración hasta que el revisor confirme cierre de F1 y se ejecute el gate Tauri aplazado en el corte corregido.

## Autorrevisión

- **Cerrado por evidencia del corte:** gate `scripts/check.ps1` exit 0; árbol limpio; identidad de producto trazable.
- **Abierto:** remount de Paper con `refreshRequired` (S2/F1), revisión independiente y Tauri debug no-bundle.
- **Alcance no demostrado:** QA GUI nativa, instalador y máquina limpia.

