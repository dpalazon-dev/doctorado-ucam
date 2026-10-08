use crate::{
    application::{
        reader_ports::RegisteredDocument,
        workflow_ports::{
            WorkflowAdmission, WorkflowContext, WorkflowDocumentProof, WorkflowDocumentReference,
            WorkflowFuture, WorkflowPersistence, WorkflowPhaseAcceptance, WorkflowPhaseRecord,
            WorkflowRepository,
        },
    },
    transport::{
        dto::{
            AnswerResolution, DocumentDto, GateEvaluationDto, PhaseAnswerDto, PhaseCode,
            PhaseDefinitionDto, PhaseDto, PhaseState, UUID, WorkflowEvaluateGateArgs,
            WorkflowGoBackToPhaseArgs, WorkflowGoBackToPhaseOutput, WorkflowSavePhaseAnswerArgs,
            WorkflowTouchPhaseArgs,
        },
        error::{AppError, ErrorCode},
    },
};
use rusqlite::{Connection, OptionalExtension, Transaction};
use serde_json::Value;

#[derive(Default)]
pub struct SqliteWorkflowRepository;

fn phase_code(code: &str) -> Result<PhaseCode, AppError> {
    match code {
        "PRE" => Ok(PhaseCode::PRE),
        "P1" => Ok(PhaseCode::P1),
        "P2" => Ok(PhaseCode::P2),
        _ => Err(AppError::new(ErrorCode::IntegrityFailure)),
    }
}

impl WorkflowRepository for SqliteWorkflowRepository {
    fn context(
        &self,
        tx: &Transaction<'_>,
        paper_id: &str,
    ) -> Result<Option<WorkflowContext>, AppError> {
        let row = tx
            .query_row(
                "SELECT processing_initialized,current_phase FROM papers WHERE id=?1",
                [paper_id],
                |row| Ok((row.get::<_, bool>(0)?, row.get::<_, Option<String>>(1)?)),
            )
            .optional()?;
        Ok(row.map(|(initialized, active_phase)| WorkflowContext {
            initialized,
            active_phase,
        }))
    }

    fn phases(
        &self,
        tx: &Transaction<'_>,
        paper_id: &str,
    ) -> Result<Vec<WorkflowPhaseRecord>, AppError> {
        let mut statement = tx.prepare("SELECT phase_code,definition_version,state,revision,completed_at,accepted_gate_snapshot_json,accepted_gate_snapshot_hash FROM paper_phases WHERE paper_id=?1 ORDER BY CASE phase_code WHEN 'PRE' THEN 0 WHEN 'P1' THEN 1 ELSE 2 END")?;
        let rows = statement.query_map([paper_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, Option<String>>(6)?,
            ))
        })?;
        rows.map(|row| {
            let (
                code,
                definition_version,
                state,
                revision,
                completed_at,
                snapshot_json,
                snapshot_hash,
            ) = row?;
            Ok(WorkflowPhaseRecord {
                code: phase_code(&code)?,
                definition_version,
                state,
                revision,
                completed_at,
                snapshot_json,
                snapshot_hash,
            })
        })
        .collect()
    }

    fn initialize(&self, tx: &Transaction<'_>, paper_id: &str) -> Result<(), AppError> {
        tx.execute("INSERT INTO paper_phases(paper_id,phase_code,definition_version,state,revision) VALUES(?1,'PRE',1,'IN_PROGRESS',1),(?1,'P1',1,'NOT_STARTED',0),(?1,'P2',1,'NOT_STARTED',0)", [paper_id])?;
        Ok(())
    }

    fn set_context(
        &self,
        tx: &Transaction<'_>,
        paper_id: &str,
        phase: &str,
    ) -> Result<(), AppError> {
        if phase != "PRE" {
            return Err(AppError::new(ErrorCode::UnsupportedCapability));
        }
        let changed = tx.execute("UPDATE papers SET processing_initialized=1,current_phase=?2 WHERE id=?1 AND processing_initialized=0 AND current_phase IS NULL", rusqlite::params![paper_id,phase])?;
        if changed != 1 {
            return Err(AppError::new(ErrorCode::IntegrityFailure));
        }
        Ok(())
    }

    fn set_phase_clock(
        &self,
        tx: &Transaction<'_>,
        paper_id: &str,
        phase: &PhaseCode,
        state: &str,
        revision: i64,
    ) -> Result<(), AppError> {
        let code = match phase {
            PhaseCode::PRE => "PRE",
            PhaseCode::P1 => "P1",
            PhaseCode::P2 => "P2",
            _ => return Err(AppError::new(ErrorCode::UnsupportedCapability)),
        };
        let changed = tx.execute(
            "UPDATE paper_phases SET state=?3,revision=?4 WHERE paper_id=?1 AND phase_code=?2",
            rusqlite::params![paper_id, code, state, revision],
        )?;
        if changed != 1 {
            return Err(AppError::new(ErrorCode::IntegrityFailure));
        }
        Ok(())
    }

    fn max_phase_revision(&self, tx: &Transaction<'_>, paper_id: &str) -> Result<i64, AppError> {
        let maximum: Option<i64> = tx.query_row(
            "SELECT max(revision) FROM paper_phases WHERE paper_id=?1",
            [paper_id],
            |row| row.get(0),
        )?;
        maximum.ok_or_else(|| AppError::new(ErrorCode::IntegrityFailure))
    }

    fn answers(
        &self,
        tx: &Transaction<'_>,
        paper_id: &str,
        phase: &PhaseCode,
    ) -> Result<Vec<PhaseAnswerDto>, AppError> {
        answers_in(tx, paper_id, phase)
    }

    fn put_answer(&self, tx: &Transaction<'_>, answer: &PhaseAnswerDto) -> Result<(), AppError> {
        let phase = phase_name(&answer.phase_code)?;
        let resolution = serde_json::to_value(&answer.resolution)
            .map_err(|_| AppError::new(ErrorCode::IntegrityFailure))?
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| AppError::new(ErrorCode::IntegrityFailure))?;
        let structured = answer
            .structured_value
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|_| AppError::new(ErrorCode::IntegrityFailure))?;
        tx.execute(
            "INSERT INTO phase_answers(paper_id,phase_code,question_key,answer_text,structured_value_json,resolution,explanation,revision,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9) ON CONFLICT(paper_id,phase_code,question_key) DO UPDATE SET answer_text=excluded.answer_text,structured_value_json=excluded.structured_value_json,resolution=excluded.resolution,explanation=excluded.explanation,revision=excluded.revision,updated_at=excluded.updated_at",
            rusqlite::params![answer.paper_id.0, phase, answer.question_key, answer.answer_text, structured, resolution, answer.explanation, answer.revision, answer.updated_at],
        )?;
        Ok(())
    }

    fn set_phase_acceptance(
        &self,
        tx: &Transaction<'_>,
        paper_id: &str,
        phase: &PhaseCode,
        acceptance: WorkflowPhaseAcceptance<'_>,
    ) -> Result<(), AppError> {
        let code = phase_name(phase)?;
        let changed = tx.execute(
            "UPDATE paper_phases SET state='COMPLETED',revision=?3,completed_at=?4,accepted_gate_snapshot_json=?5,accepted_gate_snapshot_hash=?6 WHERE paper_id=?1 AND phase_code=?2",
            rusqlite::params![paper_id, code, acceptance.revision, acceptance.completed_at, acceptance.snapshot_json, acceptance.snapshot_hash],
        )?;
        if changed != 1 {
            return Err(AppError::new(ErrorCode::IntegrityFailure));
        }
        Ok(())
    }

    fn set_active_phase(
        &self,
        tx: &Transaction<'_>,
        paper_id: &str,
        phase: &PhaseCode,
    ) -> Result<(), AppError> {
        let code = phase_name(phase)?;
        let changed = tx.execute(
            "UPDATE papers SET current_phase=?2 WHERE id=?1 AND processing_initialized=1",
            rusqlite::params![paper_id, code],
        )?;
        if changed != 1 {
            return Err(AppError::new(ErrorCode::IntegrityFailure));
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct SqliteWorkflowPersistence {
    actor: crate::adapters::sqlite::actor::DbActor,
}

impl SqliteWorkflowPersistence {
    pub fn new(actor: crate::adapters::sqlite::actor::DbActor) -> Self {
        Self { actor }
    }
}

impl WorkflowPersistence for SqliteWorkflowPersistence {
    fn writable(&self) -> bool {
        self.actor.info().writable
    }

    fn phase(&self, paper_id: UUID, phase: PhaseCode) -> WorkflowFuture<'static, PhaseDto> {
        let actor = self.actor.clone();
        Box::pin(async move {
            actor
                .submit(move |connection| phase_in(connection, &paper_id.0, &phase))
                .await
        })
    }

    fn answers(
        &self,
        paper_id: UUID,
        phase: PhaseCode,
    ) -> WorkflowFuture<'static, Vec<PhaseAnswerDto>> {
        let actor = self.actor.clone();
        Box::pin(async move {
            actor
                .submit(move |connection| answers_in(connection, &paper_id.0, &phase))
                .await
        })
    }

    fn definition(
        &self,
        phase: PhaseCode,
        version: Option<i64>,
    ) -> WorkflowFuture<'static, PhaseDefinitionDto> {
        Box::pin(async move { crate::modules::workflow::definitions::get(&phase, version) })
    }

    fn save_answer(
        &self,
        args: WorkflowSavePhaseAnswerArgs,
    ) -> WorkflowFuture<'static, PhaseAnswerDto> {
        let actor = self.actor.clone();
        Box::pin(async move {
            let payload =
                serde_json::to_value(&args).map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
            let request_id = args.request_id.0.clone();
            actor
                .submit(move |connection| {
                    let value = crate::adapters::sqlite::receipts::with_receipt(
                        connection,
                        &request_id,
                        "workflow_save_phase_answer",
                        &payload,
                        |tx| serde_json::to_value(crate::application::workflow::save_phase_answer_in_tx(
                            tx,
                            &SqliteWorkflowRepository,
                            &crate::adapters::sqlite::library_repository::SqliteLibraryRepository,
                            &args,
                        )?).map_err(|_| AppError::new(ErrorCode::IntegrityFailure)),
                    )?;
                    serde_json::from_value(value)
                        .map_err(|_| AppError::new(ErrorCode::IntegrityFailure))
                })
                .await
        })
    }

    fn go_back(
        &self,
        args: WorkflowGoBackToPhaseArgs,
    ) -> WorkflowFuture<'static, WorkflowGoBackToPhaseOutput> {
        let actor = self.actor.clone();
        Box::pin(async move {
            let payload =
                serde_json::to_value(&args).map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
            let request_id = args.request_id.0.clone();
            actor
                .submit(move |connection| {
                    let value = crate::adapters::sqlite::receipts::with_receipt(
                        connection,
                        &request_id,
                        "workflow_go_back_to_phase",
                        &payload,
                        |tx| serde_json::to_value(crate::application::workflow::go_back_to_phase_in_tx(
                            tx,
                            &SqliteWorkflowRepository,
                            &crate::adapters::sqlite::library_repository::SqliteLibraryRepository,
                            &args,
                        )?).map_err(|_| AppError::new(ErrorCode::IntegrityFailure)),
                    )?;
                    serde_json::from_value(value)
                        .map_err(|_| AppError::new(ErrorCode::IntegrityFailure))
                })
                .await
        })
    }

    fn prepare_touch(
        &self,
        args: WorkflowTouchPhaseArgs,
    ) -> WorkflowFuture<'static, crate::application::workflow_ports::PreparedTouch> {
        let actor = self.actor.clone();
        Box::pin(async move {
            actor
                .submit(move |c| {
                    let payload = serde_json::to_value(&args)
                        .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                    if let Some(replay) = crate::adapters::sqlite::receipts::lookup_receipt::<
                        PhaseDto,
                    >(
                        c, &args.request_id.0, "workflow_touch_phase", &payload
                    )? {
                        return Ok(crate::application::workflow_ports::PreparedTouch::Replay(
                            replay,
                        ));
                    }
                    let active: Option<String> = c
                        .query_row(
                            "SELECT current_phase FROM papers WHERE id=?1",
                            [&args.paper_id.0],
                            |r| r.get(0),
                        )
                        .optional()?
                        .flatten();
                    let current = phase_by_name(
                        active
                            .as_deref()
                            .ok_or_else(|| AppError::new(ErrorCode::IntegrityFailure))?,
                    )?;
                    if phase_index(&args.phase_code)? > phase_index(&current)? {
                        Ok(
                            crate::application::workflow_ports::PreparedTouch::NeedsProof(
                                document_reference_in(c, &args.paper_id.0)?,
                            ),
                        )
                    } else {
                        Ok(crate::application::workflow_ports::PreparedTouch::NoProof)
                    }
                })
                .await
        })
    }
    fn touch(
        &self,
        args: WorkflowTouchPhaseArgs,
        proof: Option<WorkflowDocumentProof>,
        admission: WorkflowAdmission,
    ) -> WorkflowFuture<'static, PhaseDto> {
        let actor = self.actor.clone();
        Box::pin(async move {
            actor
                .submit(move |c| {
                    let _operation = admission.operation;
                    let _request = admission.request;
                    let _proof = &proof;
                    let payload = serde_json::to_value(&args)
                        .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                    let request_id = args.request_id.0.clone();
                    let value = crate::adapters::sqlite::receipts::with_receipt(
                        c,
                        &request_id,
                        "workflow_touch_phase",
                        &payload,
                        |tx| {
                            if let Some(proof) = &proof {
                                let (reference, available) = match proof {
                                    WorkflowDocumentProof::Available { reference, handle } => {
                                        if *handle.registered() != reference.registered {
                                            return Err(AppError::new(ErrorCode::IntegrityFailure));
                                        }
                                        (reference, true)
                                    }
                                    WorkflowDocumentProof::Unavailable { reference } => {
                                        (reference, false)
                                    }
                                };
                                let current = document_reference_in(tx, &args.paper_id.0)?;
                                if current.paper_id != reference.paper_id
                                    || current.active_document_id != reference.active_document_id
                                    || current.registered != reference.registered
                                {
                                    return Err(AppError::new(ErrorCode::Conflict));
                                }
                                let result = crate::application::workflow::touch_phase_in_tx(
                                    tx,
                                    &SqliteWorkflowRepository,
                                    &crate::adapters::sqlite::library_repository::SqliteLibraryRepository,
                                    &args,
                                    Some((&current, available)),
                                )?;
                                serde_json::to_value(result)
                                    .map_err(|_| AppError::new(ErrorCode::IntegrityFailure))
                            } else {
                                let result = crate::application::workflow::touch_phase_in_tx(
                                    tx,
                                    &SqliteWorkflowRepository,
                                    &crate::adapters::sqlite::library_repository::SqliteLibraryRepository,
                                    &args,
                                    None,
                                )?;
                                serde_json::to_value(result)
                                    .map_err(|_| AppError::new(ErrorCode::IntegrityFailure))
                            }
                        },
                    )?;
                    serde_json::from_value(value)
                        .map_err(|_| AppError::new(ErrorCode::IntegrityFailure))
                })
                .await
        })
    }

    fn document_reference(
        &self,
        paper_id: String,
    ) -> WorkflowFuture<'static, WorkflowDocumentReference> {
        let actor = self.actor.clone();
        Box::pin(async move {
            actor
                .submit(move |connection| document_reference_in(connection, &paper_id))
                .await
        })
    }

    fn prepare_advance(
        &self,
        args: crate::transport::dto::WorkflowAdvancePhaseArgs,
    ) -> WorkflowFuture<'static, crate::application::workflow_ports::PreparedAdvance> {
        let actor = self.actor.clone();
        Box::pin(async move {
            actor
                .submit(move |c| {
                    let payload = serde_json::to_value(&args)
                        .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                    let command = "workflow_advance_phase";
                    if let Some(replay) =
                        crate::adapters::sqlite::receipts::lookup_receipt::<
                            crate::transport::dto::WorkflowAdvancePhaseOutput,
                        >(c, &args.request_id.0, command, &payload)?
                    {
                        return Ok(crate::application::workflow_ports::PreparedAdvance::Replay(
                            replay,
                        ));
                    }
                    if !matches!(args.from_phase, PhaseCode::PRE | PhaseCode::P1) {
                        return Err(AppError::new(ErrorCode::UnsupportedCapability));
                    }
                    Ok(
                        crate::application::workflow_ports::PreparedAdvance::NeedsProof(
                            document_reference_in(c, &args.paper_id.0)?,
                        ),
                    )
                })
                .await
        })
    }

    fn advance(
        &self,
        args: crate::transport::dto::WorkflowAdvancePhaseArgs,
        proof: WorkflowDocumentProof,
        admission: WorkflowAdmission,
    ) -> WorkflowFuture<'static, crate::transport::dto::WorkflowAdvancePhaseOutput> {
        let actor = self.actor.clone();
        Box::pin(async move {
            actor
                .submit(move |c| {
                    let _operation = admission.operation;
                    let _request = admission.request;
                    let _proof = &proof;
                    let payload = serde_json::to_value(&args)
                        .map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
                    let request_id = args.request_id.0.clone();
                    let value = crate::adapters::sqlite::receipts::with_receipt(
                        c,
                        &request_id,
                        "workflow_advance_phase",
                        &payload,
                        |tx| {
                            let current = document_reference_in(tx, &args.paper_id.0)?;
                            let reference = match &proof {
                                WorkflowDocumentProof::Available { reference, handle } => {
                                    if *handle.registered() != reference.registered {
                                        return Err(AppError::new(ErrorCode::IntegrityFailure));
                                    }
                                    reference
                                }
                                WorkflowDocumentProof::Unavailable { reference } => reference,
                            };
                            if current.paper_id != reference.paper_id
                                || current.active_document_id != reference.active_document_id
                                || current.registered != reference.registered
                            {
                                return Err(AppError::new(ErrorCode::Conflict));
                            }
                            serde_json::to_value(
                                crate::application::workflow::advance_phase_in_tx(
                                    tx,
                                    &SqliteWorkflowRepository,
                                    &crate::adapters::sqlite::library_repository::SqliteLibraryRepository,
                                    &args,
                                    &current,
                                    matches!(&proof, WorkflowDocumentProof::Available { .. }),
                                )?,
                            )
                            .map_err(|_| AppError::new(ErrorCode::IntegrityFailure))
                        },
                    )?;
                    serde_json::from_value(value)
                        .map_err(|_| AppError::new(ErrorCode::IntegrityFailure))
                })
                .await
        })
    }

    fn evaluate(
        &self,
        args: WorkflowEvaluateGateArgs,
        proof: WorkflowDocumentProof,
        admission: WorkflowAdmission,
    ) -> WorkflowFuture<'static, GateEvaluationDto> {
        let actor = self.actor.clone();
        Box::pin(async move {
            actor
                .submit(move |connection| {
                    let _operation = admission.operation;
                    let _request = admission.request;
                    let _proof = &proof;
                    let tx = connection
                        .transaction_with_behavior(rusqlite::TransactionBehavior::Deferred)?;
                    let current = document_reference_in(&tx, &args.paper_id.0)?;
                    let (reference, available) = match &proof {
                        WorkflowDocumentProof::Available { reference, handle } => {
                            if *handle.registered() != reference.registered {
                                return Err(AppError::new(ErrorCode::IntegrityFailure));
                            }
                            (reference, true)
                        }
                        WorkflowDocumentProof::Unavailable { reference } => (reference, false),
                    };
                    if current.paper_id != reference.paper_id
                        || current.active_document_id != reference.active_document_id
                        || current.registered != reference.registered
                    {
                        return Err(AppError::new(ErrorCode::Conflict));
                    }
                    let (result, _snapshot) = crate::application::workflow::evaluate_gate_in_tx(
                        &tx,
                        &SqliteWorkflowRepository,
                        &crate::adapters::sqlite::library_repository::SqliteLibraryRepository,
                        &args,
                        &current,
                        available,
                    )?;
                    tx.commit()?;
                    Ok(result)
                })
                .await
        })
    }
}

type RegisteredDocumentRow = (
    String,
    String,
    String,
    String,
    String,
    String,
    i64,
    String,
    String,
);
fn document_reference_in(
    c: &Connection,
    paper_id: &str,
) -> Result<WorkflowDocumentReference, AppError> {
    let active: Option<String> = c
        .query_row(
            "SELECT active_document_id FROM papers WHERE id=?1",
            [paper_id],
            |r| r.get(0),
        )
        .optional()?
        .ok_or_else(|| AppError::new(ErrorCode::NotFound))?;
    let active = active.ok_or_else(|| AppError::new(ErrorCode::IntegrityFailure))?;
    let row: Option<RegisteredDocumentRow> = c.query_row("SELECT d.id,d.paper_id,d.original_filename,d.sha256,d.imported_at,d.status,d.size_bytes,d.relative_path,li.library_id FROM documents d CROSS JOIN library_identity li WHERE d.id=?1 AND li.singleton=1",[&active],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?,r.get(7)?,r.get(8)?))).optional()?;
    let row = row.ok_or_else(|| AppError::new(ErrorCode::IntegrityFailure))?;
    if row.1 != paper_id
        || row.7 != format!("documents/{}/original.pdf", row.0)
        || row.8.is_empty()
        || !["ACTIVE", "MISSING", "SUPERSEDED"].contains(&row.5.as_str())
        || row.3.len() != 64
        || !row.3.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err(AppError::new(ErrorCode::IntegrityFailure));
    }
    let document = DocumentDto {
        id: UUID(row.0.clone()),
        paper_id: UUID(row.1),
        original_filename: row.2,
        sha256: row.3,
        imported_at: row.4,
        status: serde_json::from_value(Value::String(row.5))
            .map_err(|_| AppError::new(ErrorCode::IntegrityFailure))?,
    };
    Ok(WorkflowDocumentReference {
        paper_id: paper_id.to_owned(),
        active_document_id: active,
        registered: RegisteredDocument {
            library_id: UUID(row.8),
            document,
            relative_path: row.7,
            size_bytes: row.6,
        },
    })
}

fn phase_in(c: &Connection, paper_id: &str, phase: &PhaseCode) -> Result<PhaseDto, AppError> {
    let code = match phase {
        PhaseCode::PRE => "PRE",
        PhaseCode::P1 => "P1",
        PhaseCode::P2 => "P2",
        _ => return Err(AppError::new(ErrorCode::UnsupportedCapability)),
    };
    let row = c.query_row("SELECT definition_version,state,revision,completed_at FROM paper_phases WHERE paper_id=?1 AND phase_code=?2", rusqlite::params![paper_id,code], |r| Ok((r.get::<_,i64>(0)?,r.get::<_,String>(1)?,r.get::<_,i64>(2)?,r.get::<_,Option<String>>(3)?))).optional()?.ok_or_else(|| AppError::new(ErrorCode::NotFound))?;
    let state = match row.1.as_str() {
        "NOT_STARTED" => PhaseState::NOTSTARTED,
        "IN_PROGRESS" => PhaseState::INPROGRESS,
        "COMPLETED" => PhaseState::COMPLETED,
        "NEEDS_REVIEW" => PhaseState::NEEDSREVIEW,
        _ => return Err(AppError::new(ErrorCode::IntegrityFailure)),
    };
    Ok(PhaseDto {
        paper_id: UUID(paper_id.to_owned()),
        code: phase.clone(),
        definition_version: row.0,
        state,
        revision: row.2,
        completed_at: row.3,
    })
}

fn answers_in(
    c: &Connection,
    paper_id: &str,
    phase: &PhaseCode,
) -> Result<Vec<PhaseAnswerDto>, AppError> {
    let code = match phase {
        PhaseCode::PRE => "PRE",
        PhaseCode::P1 => "P1",
        PhaseCode::P2 => "P2",
        _ => return Err(AppError::new(ErrorCode::UnsupportedCapability)),
    };
    let mut statement = c.prepare("SELECT question_key,answer_text,structured_value_json,resolution,explanation,revision,updated_at FROM phase_answers WHERE paper_id=?1 AND phase_code=?2 ORDER BY question_key")?;
    let rows = statement.query_map(rusqlite::params![paper_id, code], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, Option<String>>(4)?,
            row.get::<_, i64>(5)?,
            row.get::<_, String>(6)?,
        ))
    })?;
    rows.map(|row| {
        let (question_key, answer_text, structured, resolution, explanation, revision, updated_at) =
            row?;
        let structured_value = structured
            .map(|raw| {
                serde_json::from_str::<Value>(&raw)
                    .map_err(|_| AppError::new(ErrorCode::IntegrityFailure))
            })
            .transpose()?;
        let resolution = match resolution.as_str() {
            "PENDING" => AnswerResolution::PENDING,
            "ANSWERED" => AnswerResolution::ANSWERED,
            "UNKNOWN" => AnswerResolution::UNKNOWN,
            "NOT_APPLICABLE" => AnswerResolution::NOTAPPLICABLE,
            _ => return Err(AppError::new(ErrorCode::IntegrityFailure)),
        };
        Ok(PhaseAnswerDto {
            paper_id: UUID(paper_id.to_owned()),
            phase_code: phase.clone(),
            question_key,
            answer_text,
            structured_value,
            resolution,
            explanation,
            revision,
            updated_at,
        })
    })
    .collect()
}

fn phase_name(phase: &PhaseCode) -> Result<&'static str, AppError> {
    match phase {
        PhaseCode::PRE => Ok("PRE"),
        PhaseCode::P1 => Ok("P1"),
        PhaseCode::P2 => Ok("P2"),
        _ => Err(AppError::new(ErrorCode::UnsupportedCapability)),
    }
}
fn phase_by_name(phase: &str) -> Result<PhaseCode, AppError> {
    match phase {
        "PRE" => Ok(PhaseCode::PRE),
        "P1" => Ok(PhaseCode::P1),
        "P2" => Ok(PhaseCode::P2),
        _ => Err(AppError::new(ErrorCode::IntegrityFailure)),
    }
}
fn phase_index(phase: &PhaseCode) -> Result<usize, AppError> {
    match phase {
        PhaseCode::PRE => Ok(0),
        PhaseCode::P1 => Ok(1),
        PhaseCode::P2 => Ok(2),
        _ => Err(AppError::new(ErrorCode::UnsupportedCapability)),
    }
}
