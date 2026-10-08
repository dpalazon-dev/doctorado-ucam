# T02 fix 4 — rerevisión focal SPEC y calidad

SPEC: **APPROVE**. Calidad: **APPROVE**. C1 opcionalidad: **ADDRESSED**. Critical: 0; Important: 0; Minor: 0. Estado: DONE.

BASE `0c3799bf6e92fd5d1a588d15f26eb0493549d41d` → HEAD congelado `c2b35e21f7bd2f785670b4544b741908d58fc472`. Código `1a691c2aa67bb287bb75ca7262e773cd577b8b85`; checkout `.worktrees/task-02-library`. Revisor `/root/task02_spec_review`. Única escritura: este informe; sin suites nuevas, producto, Git, configuración, merges o subagentes.

## Conformidad y calidad verificadas

- `application/library.rs:150`–162 mantiene el conjunto cerrado de campos obligatorios y añade únicamente promotionConfirmed cuando existe. Las propiedades adicionales siguen rechazándose. Las comprobaciones de version/kind/code/request/receipt/payload permanecen.
- `application/library.rs:185` trata ausencia como false; `:186` exige as_bool si existe. null, string, número y otros tipos presentes no equivalen a ausencia. No concede autoridad por hash/ruta ni cambia la validación vigente de promoción durable cuando es true. Cambio mínimo de un helper puro, sin duplicar reglas ni añadir wire/puerto/migración.
- `library_integration.rs:1136` prepara una intención cuyo recurso físico está solo en staging, retira el flag y cancela/reproduce por servicio. Comprueba ausencia de staging y destino físico, DONE/false y un receipt/evento de entidad. La reserva de destino sigue existiendo en el record, ejercitando también la cancelación preparada ordinaria previamente corregida.
- `library_integration.rs:2197` ahora cierra/reabre SQLite realmente, recupera una cancellation PENDING sin flag, retira staging, conserva el original y reproduce por servicio con un único receipt/import.cancelled. Recovery conserva la ausencia del flag; retry normal puede materializar false de forma válida.
- `library_integration.rs:1217` parte de promoción durable, quita el flag y reabre: no recupera como éxito, comunica issue, conserva destino/bytes y PENDING, sin receipt. La ausencia no se interpreta como autoridad sobre el destino.
- `library_integration.rs:1303` conserva negativos versión99/payload discordante y añade promotionConfirmed de tipo string "false": falla de forma segura, conserva bytes/wrapper y no publica receipt.

No se detectaron regresiones concretas del delta. F3/F7 y los demás hallazgos previamente cerrados permanecen cerrados; no se reabren T03 ni amenazas ajenas a la tarea. Ninguna decisión del plan impone un defecto nuevo.

## Evidencia revisada

Brief/contexto/reporte fix4 y paquete completo `review-0c3799b..c2b35e2.diff` leídos una vez. El paquete contiene commits, stat y tres archivos. Se aplicó la plantilla task-reviewer Superpowers, contrastando las afirmaciones contra fuente y tests. No se releyó código cambiado; las búsquedas posteriores solo obtuvieron localizadores. No fue necesaria exploración externa al delta.

Logs y exitcodes existentes de `.worktrees/task-02-library/work/evidence`:

| Evidencia | Resultado observado |
|---|---|
| fix4-missing-flag-recovery-red | exit101, 1 test conductual; recovered0 frente a1 antes del cambio |
| fix4-missing-flag-reopen-green | exit0, 1 passed; recuperación tras reapertura |
| fix4-missing-flag-retry-green | exit0, 1 passed; retry/replay y unicidad |
| fix4-missing-flag-target-control-green | exit0, 1 passed; destino conservado sin autoridad |
| fix4-invalid-promotion-boolean-green | exit0, 1 passed; tipo inválido y negativos anteriores |
| fix4-final-check | exit0; UI14/14, library55/55, Rust102 entradas de runner = 101 comprobaciones de comportamiento + helper pending_wal_fixture_child. Suites finales sin tests filtrados. |
| fix4-native-debug-build | exit0, cargo build dev/debug finalizado |

El gate registrado scripts/check.ps1 cubre TypeScript, Vite, fmt, Clippy, Rust y contract check. No se volvió a ejecutar ningún comando de validación.

`fix4-missing-flag-retry-red` exit101 se conserva como fallo de expectativa posterior al cambio, no como rojo pre-cambio: el log falla porque esperaba flag ausente, mientras retry válido escribe false. El ajuste de assertion corresponde al contrato false/ausente; no acepta eliminación de un destino sin autoridad ni debilita el test de unicidad. El rojo conductual que prueba la regresión original es recovery-red.

## Cannot Verify

Esta es la aprobación del delta fix4 de SPEC/calidad, no la constatación de merge/gate integrado ni un sustituto de las revisiones Rust/seguridad; seguridad revisa fix3+fix4 acumulados. No se verifica UI del selector, Reader T03, NSIS, equipo limpio o pérdida de alimentación. No se consultó OpenViking ni se usó memoria auxiliar.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH / Important | 0 | pass |
| MEDIUM | 0 | pass |
| LOW / Minor | 0 | pass |

Verdict SPEC: APPROVE. Verdict calidad: APPROVE. C1 ADDRESSED; sin hallazgos abiertos en esta rerevisión focal.
