F3 — ADDRESSED. F4 — ADDRESSED. F8 — ADDRESSED.

## Finding Verdicts

- **F3: opcionales numéricos no-null** — **ADDRESSED**. `src-tauri/src/transport/dto.rs:2` deserializa un valor presente como T y lo envuelve en Some: null ya no se convierte en ausencia. `:439` y `:1746` aplican este helper a total/version manteniendo default para omisión y serialización omitida de None. `src/shared/contracts/generated/contracts.ts:35` y `:122` ahora declaran `total?: number` y `version?: number`, en conformidad con puertos y wire. `src-tauri/tests/scaffold_contracts.rs:3` prueba omisión, número y rechazo de null; `src/shared/contracts/wire.test.ts:11` comprueba esas mismas alternativas runtime; `optional-types.test.ts:6` y `:9` hacen fallar el typecheck si los tipos vuelven a admitir null. El helper nullable distinto en `dto.rs:7` y su uso en `:1967` preservan el caso legítimo de borrar domain con null; el fix no aplica una sustitución global que destruya esa semántica.

- **F4: backup agregado frente a límite por PDF** — **ADDRESSED**. `src/shared/contracts/wire.ts:87` elimina solamente el máximo de 524288000 de BackupDto.sizeBytes y conserva número entero no negativo; z.number/int sigue rechazando no finitos. `contracts/fixtures/backup-aggregate.json:1` aporta un backup de 629145600 bytes. `src/shared/contracts/wire.test.ts:7` valida ese backup y rechaza un ImportPreview de 524288001, por lo que el límite individual del PDF se conserva. No se simula un backup ejecutado ni se amplía la funcionalidad.

- **F8: errores wire separados de errores de transporte** — **ADDRESSED**. `src/shared/adapters/tauri/client.ts:16` limita try/catch al invoke. `:18` valida aparte la respuesta recibida; `:19` devuelve IntegrityFailure para envelope/payload inválido o requestId inesperado, sin convertirlo en StorageUnavailable. safeFailure marca reintentable únicamente StorageUnavailable, así que IntegrityFailure es no reintentable. `client.test.ts:5` prueba versión y payload inválidos, código, retryable=false y ausencia de contenido privado; `:12` comprueba excepción del transporte y preservación exacta del failure Busy recibido o lanzado.

## New Breakage in the Fix Diff

Ninguna Critical, Important o Minor identificada en el ámbito TypeScript/contratos de esta corrección. La conversión nueva de valores internos Settings a DTO en `src-tauri/src/transport/commands/settings.rs:6`, `:23` y `:34` mantiene los campos, contractVersion y estados que consume el adaptador; no introduce un cambio de formato público.

## Out-of-Scope Observations

Ninguna. No se reabrió la revisión de código intacto ni se emitió dictamen sobre F1/F2/F5/F6/F7 como hallazgos independientes; corresponden a las otras revisiones asignadas. Se comprobó el cambio nullable conectado con F3 únicamente para descartar una regresión de esa corrección.

## Evidence y límites

Fix BASE: `13f985f212b29008e699fbde1ee00d5a556dd408`. HEAD: `892e4e63f4c1b8fef3de054e8f510a1e1926d95a`. Revisados brief, informe y hunks pertinentes del paquete `review-13f985f..892e4e6.diff` siguiendo Scoped Re-Review de Superpowers. Sin comandos Git, subagentes, modificaciones de producto ni suites nuevas.

Evidencia existente inspeccionada: `work/fix1-dto-green.log` muestra seis tests de contratos Rust verdes, incluidos optional_numbers_reject_explicit_null y concept_domain_patch_preserves_three_states; `work/fix1-wire-green.log` registra 13 tests/3 archivos antes de añadir el test de declaraciones. `work/fix1-final-check-green.log:2` registra npm typecheck/tsc -b y `:13` cuatro archivos/14 tests verdes; `:97` y `:98` vuelven a confirmar los dos tests Rust en el gate final. Se localizó el log silencioso `work/fix1-generation-final.log`; su exit0 y el gate completo exit0 constan en el informe de corrección. No se reutilizó el rojo inicial de clippy como evidencia del árbol final.

Las pruebas TS son simulación de invoke y contratos/typecheck; no prueban interacción React/WebView2 ni instalación. No apareció una duda nueva sin respuesta en diff/logs que requiriese un test enfocado. La revisión local no verifica metadata de PR/CI ni autoriza merge por sí sola.

## Verdict

**Fix round, ámbito TS:** All findings addressed, no new Critical/Important breakage — F3, F4 y F8 cerrados. **Spec Compliance del fix:** conforme. **Task Quality del fix:** Approved.

