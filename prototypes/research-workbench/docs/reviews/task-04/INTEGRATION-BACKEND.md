# T04b — integración del backend PRE/P1

Estado: integrado en `29108ee02afad288d1c6bb4ce1aace66fdd8aa66`, rama `integration/v0.1`. Autor `/root/task04b_workflow_backend` (Luna high); producto `5fb4e084ed943831e2d8aa2bc3584ae076590836`, entrega `e9a568073c76b8661b8ffd94732e40d118841f6f`, BASE `ce15c95dc6404aae1cb158ca6c4a1fe576b70505`.

## Revisión y preparación

Orquestador revisó diff completo y deltas. SPEC S1–S9 y TypeScript TS-01 cerrados; seguridad PASS final y SQL idéntico al corte aprobado. Rust funcional PASS con checks propios; observación de longitud adjudicada como menor en `task-04b-final-adjudication.md`. No hallazgos críticos/importantes aceptados abiertos. Dictámenes originales conservados.

El merge documental desde main se preparó sin commit; conflicto único de STATUS por EOF resuelto conservando la nueva evidencia, sin producto afectado. Checks de diff y árbol pasaron; commit `b1aa73c84e8dc31d24db53ba6be45aac4044f4b0`. Sobre ese corte limpio se preparó `git merge --no-ff --no-commit agent/task-04b-workflow-backend`, sin conflictos. Código src/src-tauri/contracts/scripts/config de producto idéntico a la entrega revisada.

## Gate del merge preparado

Desde `.worktrees/integration`: `powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/check.ps1`, exit **0** observado por el orquestador. Log real: `work/evidence/task04b-integration-gate.log`; sidecar `task04b-integration-gate.exit`. Typecheck, 69 pruebas frontend, build Vite, formato, Clippy, 181 entradas Rust incluidos helpers y contratos sin drift. No builds concurrentes. Diff cached check mediante opción local `core.whitespace=-blank-at-eol` para conservar hardbreaks Markdown históricos; sin cambios de configuración. Sin diferencias unstaged. Sólo después se confirmó el merge; árbol limpio.

El build debug sin bundle del autor sobre el mismo producto pasó, log real `work/task-04b-fix1/debug-final.log/.exit`, comprobado por root. No se repitió build debug sobre documentación idéntica al producto. Advertencia conocida de chunk Vite >500 kB.

## Capacidad y límites

Ocho comandos Workflow reales, definiciones fijadas y migración0002; guardado/evaluación/avance PRE/P1, ramas continue/light_read/archive, navegación con cadena aceptada, clocks/CAS/receipt/auditoría atómicos. P2 sólo contexto/lecturas; gates, escritura y candidatos P2 siguen no disponibles. No hay todavía formularios Workflow de usuario: T04c los incorpora.

Reapertura de DbActor y conservación de datos se probaron con bibliotecas sintéticas. No se atribuye a estas pruebas una sesión GUI, instalación, proceso desktop reiniciado o Windows limpio. T04c debe verificar la persistencia visible entre procesos; QA instalada sigue pendiente por separado. Los cambios externos de AGENTS.md en main se preservaron sin incluirlos en commits.
