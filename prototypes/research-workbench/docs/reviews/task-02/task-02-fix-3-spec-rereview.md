# T02 fix 3 — rerevisión focal SPEC y calidad

SPEC: **NOT APPROVED**. Calidad: **NEEDS FIXES**. F3 y F7 ADDRESSED; C1 corregido en sus negativos originales, pero NOT ADDRESSED completamente por una regresión normativa del delta. Critical: 0; Important: 1; Minor: 0. Estado: DONE_WITH_CONCERNS.

BASE `21fd14ed20ae391518a7dabaaece3dc551a92fa2` → HEAD congelado `0c3799bf6e92fd5d1a588d15f26eb0493549d41d`. Implementación `bfb6e3eeda4019ef3dc758cd0a83350d6ad58ea8`; checkout `.worktrees/task-02-library`. Revisor `/root/task02_spec_review`. Solo escritura de este informe; sin suites nuevas, producto, Git, configuración, subagentes ni merges.

## Verificación y fortalezas

| Hallazgo | Veredicto | Código y evidencia |
|---|---|---|
| F3 | **ADDRESSED** | `library_repository.rs:1034`–1077 valida intención/request, compara error_json previo al actualizar y registra import.cancelled en el mismo closure de with_receipt. `library_integration.rs:981`/1042/1116 demuestra fallo de auditoría con rollback de DONE y receipt, recuperación posterior y replay con un evento y un receipt. El archivo ya retirado se trata como ausencia recuperable, sin fingir rollback filesystem. |
| F7 | **ADDRESSED** | `store.rs:192` comprueba ausencia mediante acceso administrado; archivo existente o error de inspección producen Pending sin borrar. ID no canónico produce Pending antes de consultar namespace; filename inválido y source open fallido utilizan la misma prueba (225/231). `library_integration.rs:1644` usa barrera después de begin_import, inserta bytes desconocidos y verifica persistencia FAILED/PENDING, conservación y diagnóstico tras reapertura real. Control vacío e ID inválido también tienen ejecuciones focales. |
| C1 | **NOT ADDRESSED completamente** | `library_repository.rs:731` valida el wrapper anterior antes de sobrescribirlo y el helper compartido rechaza versión99/payload discordante; `library_integration.rs:1136` comprueba ambos negativos end-to-end. Sin embargo, el helper exige un flag que la norma permite ausente; véase Important 1. |

La corrección permanece acotada a los tres casos y sus pruebas. No introduce DTO/IPC/migración ni framework nuevo. F9/F10/F11/F13/M5 y los anteriores siguen cerrados salvo la regresión de cancelación señalada aquí; no se amplía revisión de Reader T03 o amenazas.

## Important 1 — C1 exige promotionConfirmed y bloquea una cancellation v1 legítima sin promoción

Archivo: `src-tauri/src/application/library.rs:165` y `:187` (helper `valid_pending_cancellation`, 143). Consumidores: `adapters/sqlite/library_repository.rs:731`, `:835` y `:1044`.

MANAGED_FILES vigente:51 establece explícitamente que en casos sin promoción `promotionConfirmed` es **false/ausente**. El nuevo conjunto exacto de keys exige siempre esa propiedad y el as_bool()? también rechaza ausencia. Una cancellation v1/PENDING íntegra, con requestId canónico y receipt/token correcto, pero sin ese campo opcional, devuelve None. El formato inicial de cancelación de TASK02_PORTS:68 tampoco incluía ese campo.

En retry, prepare_cancel rechaza el record FAILED antes de actuar. En recovery, validate_cleanup_plan y mark_recovered_cancelled rechazan la misma intención. Esto deja como incidencia permanente una cancelación ordinaria legítima, aunque no exista destino que requiera autoridad de promoción y todas las comprobaciones de namespace/referencias/hash sean correctas. La ausencia de promotionConfirmed debe significar false; no debe conceder autoridad para eliminar un destino.

Reproducción focal propuesta, no ejecutada por este revisor: seleccionar PDF, dejarlo STAGING sin reserva/destino, ejecutar prepare_cancel para obtener FAILED/cancellation v1/PENDING, quitar únicamente `promotionConfirmed` con json_remove, cerrar/reabrir y reconciliar. El código actual produce issue y conserva PENDING/staging en lugar de completar la cancelación. Un retry con el token admitido, antes de cerrar y sin receipt, también devuelve ImportRecoveryRequired. Esta es una regresión nueva de la validación estricta, no una ampliación del modelo de amenazas.

Corrección acotada: admitir el conjunto de claves conocido con promotionConfirmed opcional y tratar su ausencia como false. Si está presente, exigir boolean; mantener validación íntegra cuando es true, y rechazo de versiones/kinds/payloads/keys desconocidos. Añadir regresión de cancellation PENDING sin campo, staging propio sin destino, recuperación/replay y evento/receipt exactamente una vez. Conservar el negativo de destino que carece de promoción durable.

No es plan-mandated: el brief pide validación íntegra, pero la norma vigente permite ausencia; el brief no ordena hacer obligatorio ese flag. No hay dispensa de este Important.

## Evidencia existente revisada

Lectura única completa del paquete `review-21fd14e..0c3799b.diff` en dos porciones, con commits/stat y las cinco modificaciones. Brief, reporte y plantilla task-reviewer de Superpowers aplicados. Sin relectura de archivos de código cambiados; únicamente búsquedas focales de localizadores. Riesgo concreto externo al diff: opcionalidad normativa de promotionConfirmed y consumidores de retry/recovery; se contrastó MANAGED_FILES:51 y los callers ya conocidos de fix2, sin rastrear el repositorio.

Logs y exitcodes consultados en `.worktrees/task-02-library/work/evidence`:

- F3: `fix3-f3-audit-red2` exit101, 1 test conductual fallido; `fix3-f3-audit-green2` exit0, 1 passed. El rojo revela que recovery cerraba aunque el trigger exigía auditoría; el verde verifica rollback/retry/replay real.
- C1: `fix3-c1-version-red` exit101, 1 test fallido; `fix3-c1-payload-mismatch-green2` exit0, 1 passed con ambos subcasos. No se inventa un rojo independiente del subcaso de payload discordante.
- F7: `fix3-f7-early-stage-red` exit101 (DONE frente a PENDING), `fix3-f7-early-stage-green` exit0/1 passed; `fix3-f7-invalid-id-red` exit101 (Done frente a Pending), `fix3-f7-invalid-id-green` exit0/1 passed; `fix3-f7-empty-stage-green` exit0/1 passed.
- `fix3-final-check2` exit0: UI14/14; Rust12+8+13+53+3+6+4+1 = 100 entradas de runner, equivalentes a 99 comprobaciones de comportamiento y el helper hijo pending_wal_fixture_child. Suites finales sin filtered tests; library53/53. Gate registrado scripts/check.ps1 incluye typecheck/build/fmt/Clippy/contract check.
- `fix3-native-debug-build` exit0 y cargo build dev/debug finalizado.

Los primeros intentos de compilación excluidos por el autor no se cuentan como rojos/verdes de comportamiento. Ningún test se repitió en esta rerevisión.

## Cannot Verify

No hay ejecución existente del caso legítimo sin promotionConfirmed; el fallo se deduce directamente de la comparación de keys/as_bool y sus consumidores. Debe añadirse prueba focal al corregirlo y revisarse el nuevo HEAD. No se acredita gate integrado o merge, selector GUI, NSIS, equipo limpio, Reader T03 ni pérdida de alimentación. La evidencia verde presente no resuelve el Important nuevo.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH / Important | 1 | needs fixes |
| MEDIUM | 0 | pass |
| LOW / Minor | 0 | pass |

Verdict SPEC: NOT APPROVED. Verdict calidad: NEEDS FIXES por una regresión concreta C1; F3/F7 cerrados. El proyecto impide integrar con Important abiertos.
