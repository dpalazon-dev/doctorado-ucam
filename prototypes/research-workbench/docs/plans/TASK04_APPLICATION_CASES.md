# T04b — extracción acotada de application (S9, S3 y S5)

**Estado: ACCEPTED por Sol, ADR-021, 2026-10-03; implementación pendiente.** Corte inspeccionado mediante `git show`: `a051d88ca4c8127fe4749293b0b03cecc2923a14`, worktree `.worktrees/task-04b-workflow-backend`. Fuentes: AGENTS, INTENT, STATUS, TASK04_PORTS, brief central T04b, revisión S3/S5/S9 y WORKFLOW_GATES. No se consultó WIP de producto; el informe ajeno sin seguimiento y AGENTS modificado en el repo principal se preservan. No builds, commits ni merges.

## Decisión aceptada

Extraer los cinco casos de uso al módulo `application/workflow.rs`, con TX prestada y repositorios inyectados. Añadir **cuatro** primitivas SQL a `WorkflowRepository`; reutilizar sus seis métodos actuales y `LibraryRepository` íntegro. Application decide CAS, validación, gate, aceptación, clocks, contexto, lifecycle y eventos efectivos. El adaptador conserva actor, transacción/receipt, revalidación documental, ownership de proof/permisos y serialización del resultado para el receipt.

No cambiar `WorkflowPersistence`, IPC, DTOs, esquema, Reader, actor, UnitOfWork, Windows ni receipts. No ampliar `LibraryRepository` ni crear Clock/IdGenerator: application Library ya utiliza `chrono::Utc`; el adaptador Library ya genera IDs de auditoría. Mantener ese patrón.

## Primitivas nuevas exactas

En `application/workflow_ports.rs`, junto a `WorkflowPhaseRecord`/`WorkflowContext` existentes. Tipos importados ya disponibles: `PhaseAnswerDto`, `PhaseCode`, `AppError`, `Transaction`.

```rust
pub struct WorkflowPhaseAcceptance<'a> {
    pub revision: i64,
    pub completed_at: &'a str,
    pub snapshot_json: &'a str,
    pub snapshot_hash: &'a str,
}

// Adiciones a WorkflowRepository; métodos actuales intactos.
fn answers(
    &self, tx: &Transaction<'_>, paper_id: &str, phase: &PhaseCode,
) -> Result<Vec<PhaseAnswerDto>, AppError>;

fn put_answer(
    &self, tx: &Transaction<'_>, answer: &PhaseAnswerDto,
) -> Result<(), AppError>;

fn set_phase_acceptance(
    &self, tx: &Transaction<'_>, paper_id: &str, phase: &PhaseCode,
    acceptance: WorkflowPhaseAcceptance<'_>,
) -> Result<(), AppError>;

fn set_active_phase(
    &self, tx: &Transaction<'_>, paper_id: &str, phase: &PhaseCode,
) -> Result<(), AppError>;
```

- `answers`: reutiliza SQL/mapping de `answers_in`, incluyendo revisión y updatedAt; cero filas es válido. Application comprueba existencia de Paper/fase previamente y construye `StoredGateAnswer` normalizado. No hace validación de gate ni selección de definición.
- `put_answer`: INSERT/UPDATE de los valores y revisión/timestamp **ya decididos**; serializa structuredValue y resolución. No calcula CAS/no-op, clocks, contexto o auditoría.
- `set_phase_acceptance`: UPDATE de state=COMPLETED, revision, completedAt, snapshot JSON/hash; no cambia `definition_version`. Un solo UPDATE, exige una fila afectada. No acepta el gate por su cuenta.
- `set_active_phase`: UPDATE exclusivo de `papers.current_phase`, exige una fila afectada. No modifica initialized, lifecycle, Paper.revision ni timestamps. No sustituye `set_context`, cuya semántica actual es inicialización exclusiva PRE.

`phases` ya entrega pin/state/revision/completedAt/snapshot; no añadir `phase`, `pin`, `current_revision` Workflow redundantes. `set_phase_clock` ya actualiza state/revision preservando aceptación; se reutiliza para navegación/inicio de destino e invalidación. `max_phase_revision` ya permite asignar clocks sin SQL en application.

`LibraryRepository::paper` entrega título/reviewType/lifecycle/revisión/contexto. `update_lifecycle` aplica NEW→ACTIVE con CAS bibliográfico y devuelve número de filas afectadas. `audit_change` admite identidad y JSON de cambios específicos. Reutilizar esos métodos; no añadir auditoría o lifecycle paralelos a WorkflowRepository.

## Casos de uso exactos en application/workflow.rs

```rust
pub fn save_phase_answer_in_tx(
    tx: &Transaction<'_>, workflow: &dyn WorkflowRepository,
    library: &dyn LibraryRepository, args: &WorkflowSavePhaseAnswerArgs,
) -> Result<PhaseAnswerDto, AppError>;

pub fn evaluate_gate_in_tx(
    tx: &Transaction<'_>, workflow: &dyn WorkflowRepository,
    library: &dyn LibraryRepository, args: &WorkflowEvaluateGateArgs,
    reference: &WorkflowDocumentReference, available: bool,
) -> Result<(GateEvaluationDto, serde_json::Value), AppError>;

pub fn advance_phase_in_tx(
    tx: &Transaction<'_>, workflow: &dyn WorkflowRepository,
    library: &dyn LibraryRepository, args: &WorkflowAdvancePhaseArgs,
    reference: &WorkflowDocumentReference, available: bool,
) -> Result<WorkflowAdvancePhaseOutput, AppError>;

pub fn go_back_to_phase_in_tx(
    tx: &Transaction<'_>, workflow: &dyn WorkflowRepository,
    library: &dyn LibraryRepository, args: &WorkflowGoBackToPhaseArgs,
) -> Result<WorkflowGoBackToPhaseOutput, AppError>;

pub fn touch_phase_in_tx(
    tx: &Transaction<'_>, workflow: &dyn WorkflowRepository,
    library: &dyn LibraryRepository, args: &WorkflowTouchPhaseArgs,
    proof: Option<(&WorkflowDocumentReference, bool)>,
) -> Result<PhaseDto, AppError>;
```

El segundo valor de evaluate es el snapshot actual **no aceptado**, como en `evaluate_in` existente: advance lo consume, el wrapper evaluate devuelve sólo el DTO. Ninguno abre/commit/rollback de TX, consulta actor/receipts ni posee handles/permisos. `reference` ya fue revalidada bajo esta misma TX por el adaptador; `available` procede exclusivamente del proof tipado. Application nunca infiere disponibilidad desde errores ni consulta filesystem.

### Pin persistido (S3)

Antes de validar/guardar/evaluar/aceptar: seleccionar el `WorkflowPhaseRecord` de `workflow.phases(tx, paper_id)` y resolver `definitions::get(&record.code, Some(record.definition_version))`. Nunca usar `None` en estos casos de uso. La función existente ya sirve; no crear resolver/repository de definiciones nuevo. Para pin persistido positivo no soportado por este binario, convertir su `NotFound` a `UnsupportedCapability` en el helper interno de application; inconsistencias de filas siguen `IntegrityFailure`. No cambiar el contrato público getPhaseDefinition ni implementar v2.

P1 obtiene `preDefinitionHash` resolviendo el pin de la **fila PRE**, y reevalúa PRE con ese mismo pin. Snapshot, GateEvaluationDto y PhaseDto usan la versión correspondiente a su fila, también touch y resultado de advance. Touch y advance validan que el pin destino P1/P2 sea soportado antes de escribir cualquier fase; iniciar P2 no evalúa su gate pendiente. En advance, origen válido con destino de versión desconocida devuelve UnsupportedCapability sin aceptar origen ni cambiar clocks/contexto/audit/receipt. Esta precisión cierra S3: activación no debe interpretar una fase cuyo pin no entiende el binario. La presencia de otra definición en DB no repina ni cambia un Paper v1. Pin no soportado falla sin escritura/evento/receipt de éxito.

### Plan puro mínimo para avance

Conservar reglas existentes de `domain/workflow.rs` (normalización, salida, snapshot vigente, cadena, lifecycle y `advance_decision`). Añadir sólo este plan concreto; no un motor genérico de comandos o efectos:

```rust
pub struct AdvancePlan {
    pub decision: AdvanceDecision,
    pub target: Option<(PhaseCode, PhaseState)>,
    pub activate_paper: bool,
}

pub fn plan_advance(
    from: &PhaseCode, reading_decision: Option<&ReadingDecision>,
    lifecycle: &str, target_state: Option<&PhaseState>,
) -> Result<AdvancePlan, AppError>;
```

Precondición application: fase activa/CAS/gate/cadena válidos; para PRE se pasa estado P1, para P1 continue estado P2, para light_read/archive `None`. El plan valida lifecycle mediante la regla actual y usa `advance_decision`: PRE→P1, continue→P2; destino NOT_STARTED→IN_PROGRESS y otro estado iniciado se preserva; light_read/archive sin destino; `activate_paper=true` sólo PRE con NEW. Falta de estado para un destino esperado es IntegrityFailure. El plan no calcula timestamps, clocks, pin, snapshot, SQL ni auditoría. Esto permite probar primera aceptación y reconfirmación preservando P1/P2 sin SQLite.

Application asigna los clocks desde max+1 y aplica el plan: una revisión final por fase afectada, origen y después destino en orden PRE/P1/P2. Acepta origen con el snapshot evaluado; aplica destino con `set_phase_clock` y `set_active_phase`. Light_read/archive conservan contexto P1 y P2 previo. NEW→ACTIVE reutiliza `library.update_lifecycle(..., paper.revision, "ACTIVE", None, &now)` y comprueba una fila; archive llama **exactamente** a `archive_paper_in_tx(tx, library, request_id, paper_id, paper.revision)`. No copiar `set_archived_in_tx`, no invocar wrapper público y no añadir receipt Library. El helper existente conserva archivedFrom, revisión y `paper.archived` en la TX común.

### Historial específico (S5)

Application llama `library.audit_change` después de las escrituras efectivas y antes de devolver; entidad siempre Paper ID y requestId existente. Acciones nuevas internas, sin cambios wire:

| Acción | Cambios JSON mínimos para explicar el historial |
|---|---|
| `workflow.answer_changed` | phaseCode, definitionVersion, questionKey; before=null o cuatro valores normalizados+answer revision; after=cuatro valores+revision; fases afectadas before/after state/revision por invalidación. |
| `workflow.phase_accepted` | phaseCode/pin; before/after state/revision/completedAt/snapshotHash; decision; contexto y lifecycle before/after; destino afectado before/after state/revision. Snapshot completo permanece en su fila. |
| `workflow.context_changed` | operación touch/goBack, contexto before/after y destino before/after state/revision; explica inicio o navegación preservando aceptación histórica. |

No guardar payloadHash como sustituto de cambios. No-op canónico de respuesta conserva todos los tiempos/clocks y no llama put/invalidate/audit efectivo; CAS ocurre antes de comprobar igualdad. No-op de contexto no escribe ni crea context_changed. Una aceptación/reconfirmación que renueva aceptación/clock sí registra phase_accepted aunque el hash coincida. Replay no llega al caso de uso y no repite evento. Mantener intacta la auditoría genérica legacy de `with_receipt`; `paper.archived` procede sólo del helper Library. El evento phase_accepted explica NEW→ACTIVE y la decisión archive sin duplicar el algoritmo/evento paper.archived.

## Movimiento y permanencia de funciones

| Función actual del adaptador | Destino/responsabilidad |
|---|---|
| `save_answer_in` | save_phase_answer_in_tx: normalización, pin/output, guards de negocio, CAS/no-op, put, invalidation y evento. SQL de respuestas pasa a primitivas. |
| `evaluate_in` | evaluate_gate_in_tx: pin, StoredGateAnswer, evaluate_outputs, cadena PRE y composición/hash de snapshot; sólo lectura. |
| `advance_in` | advance_phase_in_tx: CAS/gate, plan puro, aceptación, clocks/contexto, lifecycle/Library archive y evento. |
| `go_back_in`, `touch_in` | Casos application: CAS activo, dirección/cadena, preservación de estados, clock/contexto y evento. |
| `ensure_accepted_chain`, `next_clock` | Helpers privados application usando casos/repo y reglas puras. No SQL ni segundo algoritmo de invalidación. |
| `phase_in`, `answers_in`, `document_reference_in` | Adaptador: lecturas/mapping SQL. `answers_in` se comparte con nueva primitiva. Conversión de strings DB a enums permanece junto al mapping SQL. |

En `SqliteWorkflowPersistence` permanecen `prepare_*` y lookup/replay; actor.submit; with_receipt; TX DEFERRED de evaluate; revalidación completa de referencia/handle bajo TX; ownership de admission/proof **fuera de action y alrededor de toda la llamada with_receipt**, incluyendo commit/rollback. Action sólo llama application y serializa su resultado tipado a Value. Replays no llaman application. Evaluate llama application bajo la TX DEFERRED existente, confirma y devuelve el DTO; no evento/receipt ni escritura.

Los guards de propiedad documental/admission no se mueven a application. Las guardas de negocio (CAS, lifecycle, capability, cadena) se deciden allí después de replay; no confundir ambas responsabilidades. La resolución S4 de prioridad receipt/capability y S8 de rango puede reutilizar esta frontera, pero esta propuesta no cambia servicios/protocolo ni declara esos hallazgos cerrados.

## Pruebas sin infraestructura nueva

1. Unitarias en domain/workflow: plan PRE inicial NEW, PRE reconfirmada ACTIVE con P1 NEEDS_REVIEW/COMPLETED, P1 continue con P2 iniciado, light_read/archive con P2 previo; estados preservados y ausencia de dependencia rusqlite/reloj.
2. Extender suites T04b existentes: filas v2 sintéticas sin repinar→mismo resultado v1; pin no soportado en PRE/P1→error seguro, sin resultado v1, escritura/evento/receipt; P1 usa pin PRE. No instalar binario/SQLite externo ni nueva suite/harness.
3. Recorridos IPC reales existentes: save efectivo/nuevo no-op/stale no-op/replay; aceptación/reconfirmación; navegación efectiva/no-op. Consultar audit_events por acción+entity_id, comprobar contenido before/after e identidad, ignorando la auditoría genérica del wrapper.
4. Fallo tardío mediante fixture/trigger SQLite sintético existente: abortar INSERT del evento específico y después INSERT de receipt. Comprobar rollback conjunto de respuesta/fases/snapshots/contexto/lifecycle/eventos; max no consumido. Archive conserva su evento Library en la misma atomicidad. Mantener pruebas de drop/commit/rollback de workflow_concurrency para demostrar guardas fuera de action.
5. Revisor inspecciona que application no contiene SQL ni referencias a adaptadores; WorkflowRepository sólo persiste valores decididos. Orquestador ejecuta los comandos ya fijados y el gate después de implementación. Esta propuesta **no acredita** esas ejecuciones ni corrige producto por sí misma.

**Alternativa descartada:** extraer un helper que siga recibiendo Connection y emitiendo SQL cambia ubicación pero deja S9 intacto. Introducir unidad de trabajo, puertos de tiempo/IDs, framework de efectos o repositorios paralelos amplía el alcance sin necesidad. Coste elegido: cuatro métodos, un struct prestado de aceptación, un plan puro concreto y cinco casos de uso sobre la TX vigente.
