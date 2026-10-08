F1 — ADDRESSED.

# Task 01 · Re-review de SPEC y calidad transversal · ronda 1

**Spec Compliance: Pass en el alcance de esta re-review.**

**Task quality: Approved en el alcance de esta re-review.**

BASE `13f985f212b29008e699fbde1ee00d5a556dd408`; HEAD `892e4e63f4c1b8fef3de054e8f510a1e1926d95a`. Revisión del paquete `review-13f985f..892e4e6.diff`, del brief y del informe de corrección centrales. Aplicado el formato de `superpowers/6.4.2/skills/subagent-driven-development/re-review-prompt.md`. Checkout consultado: `.worktrees/task-01-scaffold`.

El alcance es F1/F2 de mi informe previo y F3/F4, duplicados de la revisión TypeScript, más posibles roturas nuevas introducidas por el fix diff. No sustituye el dictamen especializado Rust de F5/F6/F7 ni el TypeScript de F8. No se amplió a código intacto ni funcionalidades T02+. Solo lecturas; ninguna suite reejecutada, ningún cambio de producto/Git. Único archivo escrito: este informe.

## Hallazgos previos

### F1 · Important · Propagación de exitcode del gate — ADDRESSED

`scripts/check.ps1:11` ejecuta la generación y ahora `scripts/check.ps1:12` comprueba inmediatamente `$LASTEXITCODE` y termina con ese mismo código. Se cierra la vía por la que la última comprobación podía fallar mientras el proceso padre devolvía éxito.

La regresión en `scripts/tests/check-exitcode.ps1:15` inyecta un generador que termina con 37 en una copia temporal del gate; `scripts/tests/check-exitcode.ps1:17` exige observar 37 en el proceso padre. La prueba usa un proceso PowerShell real y sustituye herramientas anteriores únicamente en la copia de prueba. La limpieza comprueba que el destino resuelto pertenece al directorio temporal.

Evidencia inspeccionada: `work/fix1-script-red.log` contiene «Gate hid generator exit37: observed 0»; `work/fix1-script-green.log` y `work/fix1-script-delivery.log` contienen «Parent gate propagated generator exit37.». El diff y la evidencia reproducen y corrigen la garantía del gate de integración. No queda este Important abierto.

### F2 · Important · Fronteras y puerto de Settings — ADDRESSED

`src-tauri/src/application/settings.rs:4`, `:11` y `:16` definen valores internos; `:21` define el puerto mínimo `SettingsQuery`. `src-tauri/src/modules/settings.rs:5`, `:25` y `:28` consumen ese puerto y devuelven valores internos. El módulo deja de depender de `DbActor`, del adaptador SQLite y de DTOs de respuesta.

La implementación concreta está en `src-tauri/src/adapters/sqlite/settings.rs:8` y `:17`: concentra actor, consulta SQL de recuperación y mantenimiento. La composición explícita queda en `src-tauri/src/lib.rs:9`. Las conversiones a los DTOs congelados están en `src-tauri/src/transport/commands/settings.rs:5`, `:21` y `:31`. No se introduce un contenedor de DI ni se altera el puerto UoW prescrito.

Conclusión concreta: las consultas Settings cruzan ahora la frontera mediante un puerto de aplicación y valores internos; SQLite queda detrás de su implementación y transporte construye las respuestas IPC. La corrección conserva el tipo de error compartido existente; este fix no exige rediseñar ese contrato transversal.

`src-tauri/tests/settings_service.rs:8` implementa un puerto simulado sin actor/SQLite y ejercita capacidades, información y estado. `work/fix1-settings-green.log` y el gate final registran `settings_service_uses_only_supplied_query_port ... ok`. No queda este Important abierto.

### F3 · Important · Opcionales numéricos no anulables — ADDRESSED

`src-tauri/src/transport/dto.rs:2` deserializa un valor presente como `T` y lo envuelve en `Some`, por lo que un `null` presente se rechaza. Con `default` y omisión al serializar, `:439`/`:441` y `:1746`/`:1748` conservan omisión o número para `PageDto.total` y `WorkflowGetPhaseDefinitionArgs.version`.

Los tipos generados de `src/shared/contracts/generated/contracts.ts:35` y `:122` son ahora `total?: number` y `version?: number`. `src-tauri/tests/scaffold_contracts.rs:3` cubre omisión/número/null en Rust; `src/shared/contracts/wire.test.ts:11` lo cubre en Zod; `src/shared/contracts/optional-types.test.ts:6` y `:9` exigen errores de compilación para null. El cambio separado del patch anulable `ConceptUpdateArgs.domain` mantiene su categoría distinta y no se hizo una sustitución masiva de campos anulables.

Evidencia: `work/fix1-dto-green.log` registra `optional_numbers_reject_explicit_null ... ok`; `work/fix1-final-check-green.log` incluye typecheck, 14 pruebas UI en 4 archivos, la misma prueba Rust en verde y el resto del gate. `work/fix1-generation-final.log` no contiene salida; se interpreta junto con la ejecución satisfactoria declarada en el informe y el gate, no como prueba independiente de éxito. No queda este Important abierto; se consolida con el duplicado TypeScript.

### F4 · Important · Tamaño agregado de backup — ADDRESSED

`src/shared/contracts/wire.ts:87` elimina el máximo de 524288000 únicamente de `BackupDto.sizeBytes`, que conserva validación numérica entera no negativa. El máximo individual de PDF continúa en `src/shared/contracts/wire.ts:28`.

`contracts/fixtures/backup-aggregate.json:1` representa un backup de 629145600 bytes. `src/shared/contracts/wire.test.ts:7`/`:8` exige aceptarlo y `:9` rechaza un PDF de 524288001 bytes. `work/fix1-wire-green.log` registra 13 pruebas UI verdes en la ejecución enfocada previa; `work/fix1-final-check-green.log` registra 14 en la comprobación final, tras incorporar la prueba de tipos. No queda este Important abierto; se consolida con el duplicado TypeScript.

## Roturas nuevas introducidas por el fix diff

Ninguna confirmada en este ámbito: **Critical 0 / Important 0 / Minor 0**. No se añaden riesgos hipotéticos como hallazgos. La lectura transversal de los cambios de cierre, diagnóstico de esquema y clasificación de errores no encontró una rotura nueva demostrable; sus comprobaciones detalladas permanecen en los dictámenes especializados asignados.

## Evidencia y límites

- `work/fix1-final-check-green.log`: typecheck, 14 pruebas UI/4 archivos, build web, comprobaciones Rust y generación del gate final. Sus pruebas Rust incluyen el puerto Settings, opcionales no anulables, patch de tres estados, cierre con trabajo pendiente y diagnóstico con WAL. Se revisó el log existente; no se ejecutó otra suite.
- `work/fix1-native-build.log`: `tauri build --debug --no-bundle` termina construyendo `research-workbench.exe` tras build web y compilación Rust. Prueba construcción nativa de desarrollo.
- `work/fix1-native-smoke.log`: captura declarada de una ventana Research Workbench, PID 6524, en biblioteca sintética de `work/native-smoke/library`.
- `work/fix1-native-close.log`: ese PID recibe cierre, termina y registra `exitCode: 0`. Prueba cierre ordinario de esa ejecución. El cierre ocupado se ejercita en la prueba del coordinador; no se convierte esa evidencia en una prueba GUI de cierre ocupado.

La instalación en equipo limpio, empaquetado instalable y QA instalada siguen siendo trabajo posterior ya delimitado. No son hallazgos bloqueantes de T01 ni se presentan como verificados aquí. No se generan observaciones nuevas fuera de alcance.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH / Important | 0 abiertos; 4 resueltos | pass |
| MEDIUM / Minor | 0 | pass |
| LOW | 0 | pass |

Verdict: **APPROVED para este ámbito**. F1, F2, F3 y F4 están **ADDRESSED**, sin hallazgos Critical/Important/Minor nuevos confirmados. La aprobación de integración conjunta requiere los restantes dictámenes especializados; este informe no los anticipa.
