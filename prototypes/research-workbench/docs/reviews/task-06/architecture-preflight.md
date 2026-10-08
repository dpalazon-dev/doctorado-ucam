# T06 — preflight arquitectónico independiente

**Resultado: DONE_WITH_CONCERNS.** 2026-10-03. Sólo informe; ninguna implementación, commit, merge, prueba, build o llamada de red. Runtime inspeccionado exclusivamente con `git show d4a0c77:<path>` en `.worktrees/integration`: **d4a0c77b691d58493f03f622340005de0e6193ed**. No leído WIP de T04c. T05a/b son contratos aceptados, todavía no código. Las propuestas siguientes necesitan adjudicación de Sol y promoción al brief/contratos internos antes del despacho; este informe no las acepta.

Fuentes locales: AGENTS, INTENT, STATUS, IMPLEMENTATION, brief T06, DOMAIN, WORKFLOW_GATES, P2.v1.json, CONTRACTS, DATA, SPEC-004, ADR-022, TASK04_PORTS, TASK04_APPLICATION_CASES y TASK05_PORTS. Karpathy aplicado a alcance/supuestos; Superpowers a revisión y evidencia, conservando las instrucciones de ejecución ya autorizadas. Memoria histórica consultada únicamente para continuidad del método; decisiones y capacidades verificadas contra fuentes locales actuales.

## 1. Frontera demostrada y discrepancias

- `application/workflow.rs` ya contiene los cinco casos TX y `initialize_processing`/`invalidate_effective_change`; `workflow_ports.rs` tiene las diez primitivas del Repository y el protocolo proof/admission. PRE/P1 son ejecutables. Evaluate/save/advance rechazan P2; `domain/workflow.rs::answer_matches_output` devuelve false para cualquier P2 y `advance_decision` no tiene cierre P2. No basta quitar la guarda de una command.
- `WorkflowPersistence` y `WorkflowService` no tienen métodos de candidatos. `transport/dto.rs`, `src/shared/contracts/ports.ts` y los branches cerrados de `transport/commands/mod.rs` sí declaran ambos Args/DTO wire; los branches hoy devuelven UnsupportedCapability. No hace falta ampliar wire, generated DTO, registry o permisos.
- Settings expone `workflow=true` para lo integrado; Knowledge/Concept/Relations/Provenance permanecen false en el corte. No existe un flag granular P2 en ese runtime. Un esquema0003 o una lectura de listas vacías no permite activar gate P2.
- STATUS todavía identifica T04b con29108ee; el corte solicitado es d4a0c77. TASK04_APPLICATION_CASES conserva la etiqueta histórica «implementación pendiente» aunque sus casos ya están en este corte. Sol debe actualizar estado/evidencia al consolidar; ninguna de estas etiquetas autoriza ignorar el código real.
- El brief asigna `modules/workflow/{p2,candidates}.rs`, pero exige casos application y mantiene shared bajo Sol. Es necesario transferir explícitamente los archivos de integración a un único autor; Sol decide/revisa, no escribe producto en paralelo. No crear una segunda implementación en módulos para eludir esa transferencia.

## 2. Decisiones críticas para Sol

| Punto no cerrado suficientemente | Propuesta mínima | Riesgo / coste y alternativa |
|---|---|---|
| setP3Candidate con P2 NOT_STARTED o no activa | Nueva mutación exige P2 iniciada, igual que save; puede operar sobre P2 iniciada aunque no activa, sin touch ni aceptación implícita. Replay primero; Paper ARCHIVED/COMPLETED reservado conserva las guardas Workflow. Summary es consulta, incluso NOT_STARTED/archivado. | Evita que selección cree una respuesta en fase no iniciada. Permitir selección antes de P2 es una alternativa de producto; requiere regla explícita distinta de ADR-020, no inferirla desde captura T05. |
| Último candidato deseleccionado; PENDING sin justificación | set expresa una decisión completa: al terminar con0 exige justificación no vacía del mismo request; con>0 exige null. Genera resolución ANSWERED. Save permite el borrador PENDING, incluido null, sin fabricar item ni cambiar asociaciones. | El wire set no lleva resolución. No aceptar una retirada parcial silenciosa ni inventar campo. Save PENDING/null debe conservar null como borrador y summary derivar asociaciones+justificación null; gate pendiente, no inconsistencia por ausencia de proyección todavía no confirmada. Adjudicar explícitamente esta excepción de borrador frente al requisito de «proyección comprobada». |
| set sobre fila ya deseleccionada; priority/rationale irrelevantes | selected=false exige priority/rationale=null; no ignorar valores. selected=true requiere item ACTIVE claim/question con paper_items directo, prioridad null/1..5 y rationale canónico1..5000. No-op conserva revisión/tiempo/clock sólo después CAS y validación del resultado de decisión. | Rechazo de parámetros irrelevantes frente a borrarlos silenciosamente. La norma sólo exige motivo «al seleccionar»; esta precisión necesita aceptación. Un no-op sobre una decisión de0 todavía incompleta no sustituye save PENDING. |
| Archive/restore T05 de un item seleccionado | Conservar selección y count de paper_items, incluso si el item se archiva. Summary no filtra seleccionados archivados; el gate los considera inválidos como artefactos vigentes. Deseleccionar un claim/question archivado sigue permitido si Paper/P2 permiten editar; volver a seleccionarlo requiere ACTIVE. | Sin cascada ni segunda autoridad; body/confidence/lifecycle cambia snapshot/clock, no candidata PhaseAnswer si proyección igual. Filtrar archivados produciría divergencia con la respuesta y una modificación de candidatura invisible durante archive. UI necesita resolver item y etiquetar archivado. |
| Guardar la decisión candidata por save | ANSWERED acepta sólo objeto cerrado exactamente igual a candidatos/count DB y justificación única; explanation=null, answerText opcional sin gobernar selección. PENDING acepta null o objeto completo válido; si hay objeto, candidates/count coinciden DB. Set conserva answerText ya guardado, sustituye sólo proyección/resolution/explanation pertinentes. | Ningún segundo editor de selección. Al pasar de borrador a set, primera fila revision1; cambio efectivo posterior +1. Límites de la justificación ausente aún no expresos: proponer5000, canonical_text, límite antes de normalizar. No inventar truncado. |
| «Preguntas conectadas con PRE» sin vínculo wire/schema | Mostrar las preguntas de lectura PRE como contexto y permitir captura Question enlazada al Paper; asociar explícitamente esas Questions a main_questions_processed. No interpretar automáticamente prosa PRE ni persistir FK PRE-question nueva. | Conexión contextual y manual cabe en contratos existentes. Si se exige identidad por pregunta PRE, falta contrato/ADR: no hay asociación equivalente en0003 ni API para ella. |
| Gate real depende T07, pero T06 debe entregar partes utilizables | Activar captura y candidatos reales tras T05b/T06a; guardar salidas de items ya resolubles. Salidas de relaciones devuelven UnsupportedCapability hasta T07. Evaluate/advance P2 permanecen UnsupportedCapability hasta resolutores completos Relations+Provenance. | No basta justificar arrays vacíos. Mantener workflow=true existente y derivar disponibilidad UI de composición y errores UnsupportedCapability; no añadir nombres de capability wire aquí. Alternativa all-or-nothing mantiene todos saves P2 apagados hasta T07, pero reduce utilidad de T06. |

Las siete keys, tipos admisibles, alternativas por key, closure/transitividad, snapshots, clocks y cadena ya están normativamente cerrados: no requieren rediseño. No cambiar JSON P2.v1 ni ontology core1.0.0.

## 3. ABI interno propuesto, concreto y acotado

### Candidatos: una autoridad y una transacción

Nuevos `application/candidate_ports.rs`, `application/candidates.rs`, `adapters/sqlite/candidate_repository.rs`. Los tipos siguientes son internos, no derivados TS. `UUID`, PhaseCode y DTOs vienen de transport **sólo en application/adapters**, nunca en dominio nuevo.

```rust
pub struct PaperCandidateLink {
    pub paper_id: UUID,
    pub item_id: UUID,
    pub phase_code: PhaseCode,
    pub selected_for_p3: bool,
    pub priority: Option<i64>,
    pub rationale: Option<String>,
}
pub trait CandidateRepository {
    fn paper_links(&self, tx: &Transaction<'_>, paper_id: &str)
        -> Result<Vec<PaperCandidateLink>, AppError>;
    fn write_selection(&self, tx: &Transaction<'_>, next: &PaperCandidateLink)
        -> Result<usize, AppError>;
}
pub fn candidate_summary_in_tx(
    tx: &Transaction<'_>, candidates: &dyn CandidateRepository,
    workflow: &dyn WorkflowRepository, library: &dyn LibraryRepository,
    args: &WorkflowGetP3CandidateSummaryArgs,
) -> Result<P3CandidateSummaryDto, AppError>;
pub fn set_candidate_in_tx(
    tx: &Transaction<'_>, candidates: &dyn CandidateRepository,
    knowledge: &dyn KnowledgeRepository, workflow: &dyn WorkflowRepository,
    library: &dyn LibraryRepository, args: &WorkflowSetP3CandidateArgs, now: &str,
) -> Result<P3CandidateSummaryDto, AppError>;
```

`paper_links` incluye todas las asociaciones directas, ordenadas por itemId; no filtra selected ni lifecycle. `write_selection` actualiza **únicamente** selected_for_p3/priority/rationale de la fila existente, conserva phase_code, no inserta ni cambia item.revision. Exige una fila. La pertenencia a un closure indirecto no hace seleccionable un Concept/item sin paper_items directo. El caso TX decide guardas, normalización/CAS/no-op y usa KnowledgeRepository.item para el tipo/lifecycle; repositorio no decide regla de negocio.

Summary combina esa consulta con WorkflowRepository.answers(P2) y phases(P2): workflowRevision es revision P2, nunca máximo/answer/item revision. Lee justificación exclusivamente de la candidata PhaseAnswer; cero sin respuesta devuelve candidates=[],count0,justification=null, no crea fila. Una fila estructurada completa discordante con DB es IntegrityFailure en lectura, no reparación; valores proporcionados discordantes en save son InvalidInput. Pending/null es la excepción propuesta arriba.

Set: replay durable en wrapper → pin P2/lifecycle/fase iniciada → CAS expectedWorkflowRevision → validar estado final → comparar proyección → escribir asociación/PhaseAnswer si efectivo → invalidate_effective_change(P2) **una vez** → audit específico → receipt/commit. Incremento PhaseAnswer sólo si realmente cambian sus cuatro valores canónicos; cambio body de item no lo hace. La proyección candidata cambia answer.revision/updatedAt y clock conjuntamente. No segundo receipt ni touch. Si sólo se cambia justificación mediante set sobre un item deseleccionado, sigue siendo una modificación efectiva de esa misma fuente; la UI usará save para0 sin item ficticio.

Añadir a **WorkflowPersistence existente** y al mismo WorkflowService:

```rust
fn set_candidate(&self, args: WorkflowSetP3CandidateArgs)
    -> WorkflowFuture<'static, P3CandidateSummaryDto>;
fn candidate_summary(&self, args: WorkflowGetP3CandidateSummaryArgs)
    -> WorkflowFuture<'static, P3CandidateSummaryDto>;
```

Service retiene RequestPermit/OperationPermit en owned spawn/oneshot para set, igual que save; lectura retiene mantenimiento. SqliteWorkflowPersistence usa actor/with_receipt ya existentes y llama casos TX. `commands/p2.rs` puede contener sólo delegaciones a WorkflowService y conectarse a los dos branches existentes. No CandidateService/actor/scheduler nuevos.

### Artefactos: lectura única del cierre, validación y snapshot

Nuevos `application/{p2,p2_ports}.rs` y `adapters/sqlite/p2_repository.rs`; regla pura nueva en `domain/p2.rs`. El brief debe transferirlos o elegir ubicación equivalente una vez; no doble código en modules/workflow/p2.rs. Dominio recibe valores propios cerrados/índices de identidad-tipo-lifecycle y devuelve resultados puros; ninguna Transaction, DTO transport, query o acceso documental allí.

```rust
pub struct P2ItemScope {
    pub items: Vec<KnowledgeItemDto>,
    pub concepts: Vec<ConceptDto>,
    pub paper_items: Vec<PaperCandidateLink>,
    pub item_concepts: Vec<(UUID, UUID)>,
}
pub struct P2Artifacts {
    pub scope: P2ItemScope,
    pub relations: Vec<RelationDto>,
    pub provenance: Vec<ProvenanceDto>,
    pub documents: Vec<DocumentDto>,
    pub item_provenance: Vec<(UUID, UUID)>,
    pub relation_provenance: Vec<(UUID, UUID)>,
}
pub trait P2Repository {
    fn item_scope(&self, tx: &Transaction<'_>, paper_id: &str)
        -> Result<P2ItemScope, AppError>;
    fn artifacts(&self, tx: &Transaction<'_>, paper_id: &str)
        -> Result<P2Artifacts, AppError>;
}
pub struct P2Context<'a> {
    pub repository: &'a dyn P2Repository,
    pub candidates: &'a dyn CandidateRepository,
    pub relation_provenance_ready: bool,
}
```

`item_scope` reutiliza decoder único T05 y paper_links; semillas son todos paper_items. Sigue item_concepts hasta punto fijo, incluso Concept→Concept/ciclos; no filtra archivados. Concepts están representados como items y sus campos específicos para proyectar normalizedName/alias/domain sin segunda autoridad. `artifacts` completa ese mismo scope con relaciones de dos extremos alcanzados, provenance de items/relaciones y Documents registrados correspondientes; ninguna relación expande scope. Consultas bajo una TX consistente, sin N+1 público ni paginación que omita el final. Reutilizar decoders T05/T07; comprobar después T05 que realmente son compartibles antes de fijar visibilidad Rust. No copiar su SQL/decoder al módulo de dominio.

Ampliación propuesta de casos existentes: añadir `p2: Option<&P2Context<'_>>` inmediatamente después de `library` en `save_phase_answer_in_tx`, `evaluate_gate_in_tx` y `advance_phase_in_tx`; restantes parámetros/retornos intactos. PRE/P1 ignoran el contexto; P2 sin contexto devuelve UnsupportedCapability. Touch/goBack mantienen firmas y cadena existente. Las llamadas recursivas PRE/P1 propagan el mismo contexto, sin ejecutar queries0003 en esos recorridos. Es una extensión interna explícita que necesita transferencia de callsites y fixtures, no cambio wire.

- Save P2 reutiliza el caso común CAS/no-op/put/invalidation/audit; las primeras seis keys decodifican sólo `{itemIds,relationIds}` y validan mapping exacto WORKFLOW_GATES. Items directos Question/Insight/Reference; Concepts alcanzables; Relations ambos extremos alcanzables y contradicts para esa key. NA sólo arrays vacíos, ausencia usa explanation incluso ANSWERED; UNKNOWN/NA no requieren answerText. Candidata valida proyección desde el repositorio, sin escribir selección. PENDING/null guarda borrador y nunca satisface gate. Resolutores de relación pendientes no se sustituyen por arrays vacíos.
- Evaluate P2 exige relation_provenance_ready antes de consultar/aceptar artefactos; P1 aceptada COMPLETED+snapshot JSON/hash actual, toda PRE vigente y readingDecision=continue. Reutiliza ensure_accepted_chain y evaluación de los pins PRE/P1, ampliando la lógica P2 sin un segundo algoritmo de habilitación. Devuelve gate/hash actual; no guarda aceptación.
- Snapshot v1 conserva answers completas/pins; inputs exactos `{p1AcceptedGateSnapshotHash,p1DefinitionHash,readingDecision,document:{id,status,sha256,available}}`. Propuesta de claves internas artifacts: `{items,concepts,relations,provenance,documents,associations:{paperItems,itemConcepts,itemProvenance,relationProvenance},outputs,candidates}`. Cada entidad proyecta exclusivamente campos WORKFLOW_GATES; nunca serializar DTO entero con tiempos/rutas. outputs conserva los vínculos por key; candidates es proyección comprobada de la misma autoridad, no selección nueva. Arrays UUID/tuplas ordenados y alias ordenados; snapshots históricos nunca se reconstruyen desde filas actuales.
- Advance reevalúa bajo IMMEDIATE con documento vivo y el mismo contexto; ampliar el enum puro AdvanceDecision con `CompleteP2` y plan target=None/activate_paper=false. Completar conserva active P2/Paper ACTIVE y nextPhase=null; aceptación, clock, snapshot, audit/receipt juntos. No branch P3 ni cambio bibliográfico gratuito. El JSON/hash ya no incluye clock/contexto.

`relation_provenance_ready` es disponibilidad **interna de composición**, false tras T06 y true sólo después de conectar/revisar T07. No es confianza científica ni otro registry público. El coste es una condición explícita en los casos P2; evita convertir el hecho de que las tablas existen en disponibilidad de resolutores reales. Tests fixture pueden habilitarla en entorno sintético etiquetado, sin presentar eso como capabilities de producto.

El protocolo documental T04 se reutiliza íntegro: prepare/replay, prove_access fuera del actor, reference revalidada dentro, handle/admission alrededor de with_receipt hasta commit/rollback. No nueva apertura, hash, path UI ni proof a partir de bool inventado. P2 usa el Document vigente del Paper; los Documents de provenance se proyectan con estado/hash registrado, sin extender proof físico a cada fuente ni atribuir validación de citas.

## 4. Cortes, ownership y límites de aceptación

**T06a backend/candidatos**, tras T04c y T05a/b integradas y revisadas. Un autor nuevo posee domain/p2.rs, application/p2/p2_ports/candidates/candidate_ports, adapters/sqlite/p2_repository/candidate_repository y tests/p2_candidates.rs. Transferencias puntuales de Sol: workflow.rs/workflow_ports.rs, domain/workflow.rs para cierre P2, WorkflowService, SqliteWorkflowPersistence, commands/p2.rs y branches/mod.rs pertinentes, composition lib/desktop y módulos de declaraciones. No migración nueva: consumidor0003, sin editar0001/2/3, catálogos, actor, receipts, Windows, proof, wire, manifests o configuración. Revisión Rust/SQL/especificación completa antes de integrar. Resultado aceptable: candidatos y saves permitidos reales, policy/snapshot fixture probado, gate de producto final P2 pendiente T07.

**T06b UI conocimiento/P2**, autor nuevo sólo después T06a y APIs T05b reales. Posee features/workflow/{P2Workspace,P3Candidates,p2.test}; features/knowledge archivos del brief y tests. Transferencia puntual de integración a App/PhaseWorkspace/adapter sólo donde el corte T04c real la requiera; no leer hoy su WIP ni diseñar un shell paralelo. APIs por props, sin invoke en panels. Doce tipos incluye ruta ConceptApi distinta; concepto global recién creado se vincula al item mediante linkConcept para entrar en el closure, sin paperId inexistente en createConcept. Un Concept aislado no se muestra asociado al Paper hasta vínculo real. UI permite reuse/nuevo sentido sin merge; atributos required, origin/confidence visibles editables; literatura NONE pendiente de atribuir, PENDING/STALE sin etiqueta de cita verificada. Las citas iniciales usan ProvenanceInput; panels attach/update/hash y relaciones son T07.

Prop interna propuesta: `P2Workspace({paperId, workflow, knowledge, concepts, capabilities})` con APIs existentes y capacidades existentes recibidas por composición. Primero hidrata por getPhase/getPhaseAnswers/getP3CandidateSummary; UnsupportedCapability de summary mantiene cola pendiente, nunca la convierte en count0. Relaciones/provenance false deshabilitan completar P2 con razón visible. Gate llamado explícitamente puede devolver UnsupportedCapability; no fabricar GateEvaluation completo. Borradores por paper/key/item; respuesta tardía y Conflict conservan el draft y refrescan tokens de respuesta/P2 sin reenvío silencioso. Seleccionar requiere workflowRevision último; guardar respuesta usa answer.revision. Revisión TS/React y experiencia de captura antes de integración.

**T07 posterior** entrega resolutores/servicios/panels Relations/Provenance reales y activa full gate bajo transferencia explícita. El journey P2→reapertura→cambio compartido→NEEDS_REVIEW→reconfirmación no se declara aceptado antes. T06a/b no cambia ese orden ni requiere otro implementador concurrente.

## 5. Verificación necesaria y comprobaciones pendientes de T05

Antes del despacho, confirmar en BASE integrada T05: DDL0003/constraints reales, decoder único y su accesibilidad, KnowledgeRepository.{item,affected_papers}/ConceptRepository y firmas finales, función canonical_text/relojes existentes, alias normalizado/domain, conservación de seleccionados al archive, catorce handlers/capabilities/replay y closure/inversa idénticos. Si cambia ABI, Sol resuelve y actualiza propuesta/brief; no asumir TASK05_PORTS implementado por su título ACCEPTED.

Pruebas prioritarias del autor, sin harness nuevo:

1. Una sola fuente de selección/justificación: primera respuesta revision1, set/save/summary iguales tras reopen; deselección final y borrador PENDING/null; stale no-op Conflict, replay previo a archivo/CAS y payload distinto Conflict;0 sin item ficticio.
2. Rollback tardío en answer/audit/receipt conserva asociación, answer, clock/snapshot y eventos; set efectivo renueva P2 una vez sin item.revision; body/archive de candidato renueva artefacto/clock sin candidata answer; summary de archivado no filtra selección.
3. Mapping siete keys; IDs desconocidos/tipos/lifecycle/enlaces incompatibles rechazados; ausencia explicada, UNKNOWN/NA permitidos, síntesis Insight y ningún mínimo de counts; resolutor ausente UnsupportedCapability aun con arrays vacíos.
4. Closure A/B compartido, cadena/ciclo Concept, relación a extremo externo sin expansión y unión antes/después; igualdad canónica del snapshot pese a orden SQL/tiempos/contexto, sensibilidad a todos los campos contratados, locator pendiente incluido.
5. Cadena aceptada completa/P1 continue; P2 iniciada no activa vs NOT_STARTED; documento vivo inaccesible y cambio DB entre proof/TX; commit/rollback retiene permisos/handle; P2 completo ACTIVE/nextPhase null y hash aceptado preservado al invalidar.
6. UI todos los tipos y ConceptApi correcto, defaults visibles, filtros/archive alcance global, literaria NONE pendiente, quote separado, reuse/nuevo sentido, candidato archivado etiquetado; Conflict/IPC tardío conserva drafts y cola/revisiones tras reapertura real.

Comandos futuros fijados por scaffold/ENVIRONMENT: tests focalizados Rust/Vitest, generación/typecheck, `scripts/check.ps1` en merge preparado y native no-bundle donde corresponda. No ejecutados aquí. Revisiones independientes Rust/SQL y TS obligatorias según delta, revisión SPEC y diff BASE..HEAD; pruebas web, SQLite fixture y build no acreditan P2 GUI instalada ni equipo limpio.

**Autorrevisión:** ownership restringido al presente informe; supuestos etiquetados; propuestas pendientes separadas de contratos cerrados; sin DTO/wire/JSON/ontología/framework nuevos, sin segundo actor/harness ni éxito ficticio. Punto de salida para Sol: adjudicar tabla de decisiones, fijar ABI/transferencias al confirmar T05 y preparar briefs T06a/b; no iniciar producto desde este informe aislado.
