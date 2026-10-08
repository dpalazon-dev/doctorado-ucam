# T04b — revisión Rust final independiente

Resultado de entrega: **DONE_WITH_CONCERNS**. Revisión funcional Rust: **PASS acotado**. La rúbrica del rol clasifica funciones de más de 50 líneas como HIGH; queda RQ-01 de calidad, separado de cualquier defecto funcional demostrado. Conforme a esa rúbrica estricta, no emito aprobación incondicional de integración. Sol debe adjudicar expresamente esta observación en el contexto de ADR-021.

## Corte y alcance

- BASE: `ce15c95dc6404aae1cb158ca6c4a1fe576b70505`.
- HEAD de producto revisado: `5fb4e084ed943831e2d8aa2bc3584ae076590836`.
- Worktree: `C:/Users/david/Projects/Research-Workbench/.worktrees/task-04b-workflow-backend`.
- HEAD durante los gates propios: `e9a568073c76b8661b8ffd94732e40d118841f6f`. `git diff --exit-code 5fb4e084ed943831e2d8aa2bc3584ae076590836 HEAD -- '*.rs'` no mostró diferencias; los Rust coinciden con el corte congelado.
- Diff revisado: `git diff ce15c95dc6404aae1cb158ca6c4a1fe576b70505...5fb4e084ed943831e2d8aa2bc3584ae076590836 -- '*.rs'`, archivado en `work/task-04b-rust-review/full-rust.diff` del worktree. 29 archivos Rust modificados; foco en producción y pruebas de los invariantes afectados.
- Fuentes actuales: AGENTS, INTENT, STATUS, briefs T04b/fix1, TASK04_PORTS, TASK04_APPLICATION_CASES, WORKFLOW_GATES y ADR-021. Skill aplicada: `C:/Users/david/.agents/skills/rust-patterns/SKILL.md`.
- Única escritura de entrega: este informe. Sin cambios de producto, commits, merges, subagentes ni configuración.

## Gates propios y evidencia

El despacho inicialmente pedía revisión estática sin repetir Cargo. La instrucción explícita del rol Rust exige los cuatro comandos antes de revisar; se comunicó esa discrepancia y Sol confirmó exclusión del target compartido. No se repitió el build Tauri. Se ejecutaron secuencialmente desde el worktree, después de `. ./scripts/development-env.ps1`:

| Comando | Exit | Registro propio en el worktree |
|---|---|---|
| `& 'C:/Users/david/.cargo/bin/cargo.exe' check --manifest-path src-tauri/Cargo.toml` | 0 | `work/task-04b-rust-review/check.log` / `check.exit` |
| `& 'C:/Users/david/.cargo/bin/cargo.exe' clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` | 0 | `work/task-04b-rust-review/clippy.log` / `clippy.exit` |
| `& 'C:/Users/david/.cargo/bin/cargo.exe' fmt --manifest-path src-tauri/Cargo.toml --check` | 0 | `work/task-04b-rust-review/fmt.log` / `fmt.exit` |
| `& 'C:/Users/david/.cargo/bin/cargo.exe' test --manifest-path src-tauri/Cargo.toml` | 0 | `work/task-04b-rust-review/test.log` / `test.exit` |

Cargo test: **181 entradas aprobadas**, sin fallos/ignoradas; no se presentan como 181 comportamientos independientes. El total incluye el helper ya documentado por el orquestador. Suites T04b: acceptance 1, concurrency 5, gates 7, IPC 20 y migration 8. Antes de la captura persistente se realizó un primer `cargo check`, también exit0; su repetición inmediata sólo añadió el registro requerido por Sol.

Evidencia previa central leída: `work/task-04b-fix1/{check-final,debug-final}.exit` ambos 0 y colas de los logs reales. El gate anterior contiene pruebas y el build debug no-bundle termina indicando binario construido; esa evidencia no prueba GUI, instalación o equipo limpio. La revisión SQL y seguridad previas son aportadas por Sol, no atribuidas como ejecutadas por este revisor.

## Resultados funcionales

- **Ownership/concurrencia:** servicio mueve trabajo admitido a spawn/oneshot; abandono IPC no cancela el trabajo. `advance`, `touch` y `evaluate` capturan propietarios de permisos y proof en la closure del actor alrededor de toda la TX/with_receipt. Save/goBack retienen permisos en el future independiente hasta que el actor devuelve el resultado durable. Ignorar el error de envío de oneshot es deliberado: representa receptor IPC abandonado, después del resultado; no silencia un fallo de persistencia.
- **Pruebas de lifetime:** workflow_concurrency interrumpe durante proof y tras encolado, usa registry compartido con Library/Reader y exclusión de maintenance. Drop del handle abre observación independiente y exige que pueda obtener `BEGIN IMMEDIATE`; comprueba estado y receipt confirmados o rollback, por lo que no sólo observa el final de action.
- **TX/replay/CAS:** lookup común conserva hash y comando; prepare realiza replay/colisión antes de proof y advance capability. with_receipt repite lookup antes del caso application. Mutaciones, evento específico, receipt y auditoría legacy comparten IMMEDIATE. Fallos inyectados de audit/receipt y archive comprueban rollback de respuesta, clocks, contexto, snapshot y lifecycle. Evaluación usa DEFERRED consistente, sin receipt/eventos/escrituras.
- **Proof:** apertura fuera del actor; Available/Unavailable conservan referencia original. Ambos se revalidan íntegramente dentro de la TX final. Errores tipados se propagan, sin conversión por código/mensaje a available=false. No lectura/hash completo del PDF en los casos Workflow.
- **Pins:** save/evaluate/touch/advance resuelven definición persistida; P1 utiliza el pin PRE; advance verifica destino P1/P2 antes de aceptar origen. Tests incluyen otra versión instalada sin repin y destinos no soportados sin aceptación ni receipt. getPhase permite leer el pin no interpretable.
- **Application:** cinco firmas públicas y cuatro primitivas corresponden a ADR-021; no SQL, actor, receipts, filesystem o commit propios en application. Plan de avance puro conserva destino ya iniciado. Archive reutiliza `archive_paper_in_tx` prestado y no crea receipt Library anidado.
- **D1/precheck:** invalidación deduplica unión, asigna clocks ordenados y preserva aceptación histórica; input directo IN_PROGRESS conserva estado, pero dependencia anterior fuerza NEEDS_REVIEW. `direct_in_progress_input_keeps_state_but_advances_its_clock` y `overlapping_inputs_deduplicate_each_phase_clock` pasan. `initialized_processing_is_noop_after_phase_context_has_advanced` verifica P1/revisión/contexto conservados. No queda defecto funcional de los dos restos señalados.
- **Migración/importación:** registro0002 mantiene runner; suite populated v1 compara datos bibliográficos/documentales/lifecycle, backup y reopen; fallo tardío revierte DDL/seed/backfill/ledger/user_version. Import inicializa dentro de la confirmación existente antes de proyectar Paper.
- **Composición:** actor, store, RequestRegistry, maintenance y recovery compartidos. Ocho adaptadores cerrados; P2 mutaciones/gate y candidatos siguen UnsupportedCapability. Queries y lifecycle reservado conservan límites.
- No se identificaron nuevos unwrap/expect de producción, unsafe, locks envenenados ignorados, bloqueo de filesystem bajo async Workflow, SQL interpolado con inputs o canal de trabajo nuevo sin límite. Los unwrap de las fixtures se restringen a tests.

## RQ-01 — HIGH por rúbrica de calidad: casos application largos

**Ruta/líneas:** `src-tauri/src/application/workflow.rs:110` evaluate (hasta225), `:226` save (hasta347), `:348` goBack (hasta413), `:482` touch (hasta567), `:568` advance (hasta747). Superan el umbral explícito de 50 líneas del rol; advance concentra ~180 líneas.

**Escenario concreto de mantenimiento:** cambiar una rama P1 obliga a recorrer validación/gate, selección de destino, escritura de aceptación/clocks/lifecycle, proyección y construcción manual de audit before/after en un único cuerpo largo. Esto aumenta el esfuerzo de comprobar correspondencia entre efectos y explicación del historial. **No se ha demostrado un error actual**, mezcla de límites TX/application/adaptador ni violación de ABI; ADR-021 exige precisamente un caso transaccional por operación.

**Corrección mínima si Sol decide atenderlo:** extraer helpers privados con responsabilidades concretas (proyecciones de auditoría y lectura/validación de decisión) manteniendo las cinco firmas públicas y una única TX prestada. No dividir commit/rollback, mover reglas al adaptador ni introducir callbacks/UoW/framework. No se solicita fragmentación mecánica para alcanzar una cifra; el hallazgo de umbral queda declarado para adjudicación de integración.

## Límites de aprobación

No hay CRITICAL/HIGH funcional demostrado. La política estricta del rol impide aprobar sin reservas mientras RQ-01 siga clasificado HIGH por umbral. Esta revisión no acredita SPEC completo, TypeScript, instalación, GUI, ACL real nueva, P2 final ni equipo limpio. Integración presupone gate verde sobre el merge preparado, árbol limpio y conflictos resueltos; el árbol del worktree se observó limpio, pero no se ha preparado ni aprobado un merge por este revisor.
