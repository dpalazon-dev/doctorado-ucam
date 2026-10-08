use crate::{
    adapters::sqlite::{
        actor::DbActor,
        receipts::{canonical_hash, with_receipt},
    },
    application::{library_ports::*, unit_of_work::with_transaction},
    transport::{
        dto::*,
        error::{AppError, ErrorCode},
    },
};
use rusqlite::{
    Connection, OptionalExtension, Transaction, params, params_from_iter, types::Value,
};
use serde_json::{Value as Json, json};

#[derive(Clone)]
pub struct SqliteLibraryPersistence {
    actor: DbActor,
    library_id: String,
}
impl SqliteLibraryPersistence {
    pub fn new(actor: DbActor, library_id: String) -> Self {
        Self { actor, library_id }
    }
}

/// SQL implementation of the transaction-scoped Library repository port.
pub struct SqliteLibraryRepository;

impl LibraryRepository for SqliteLibraryRepository {
    fn current_revision(
        &self,
        tx: &Transaction<'_>,
        paper_id: &str,
    ) -> Result<Option<i64>, AppError> {
        Ok(tx
            .query_row(
                "SELECT revision FROM papers WHERE id=?1",
                [paper_id],
                |row| row.get(0),
            )
            .optional()?)
    }

    fn resolve_venue(
        &self,
        tx: &Transaction<'_>,
        name: Option<&str>,
    ) -> Result<Option<String>, AppError> {
        let Some(name) = name else { return Ok(None) };
        let found = tx
            .query_row(
                "SELECT id FROM venues WHERE name=?1 ORDER BY id LIMIT 1",
                [name],
                |row| row.get::<_, String>(0),
            )
            .optional()?;
        Ok(Some(match found {
            Some(id) => id,
            None => {
                let id = uuid::Uuid::new_v4().to_string();
                tx.execute(
                    "INSERT INTO venues(id,name) VALUES(?1,?2)",
                    params![id, name],
                )?;
                id
            }
        }))
    }

    fn update_paper_metadata(
        &self,
        tx: &Transaction<'_>,
        paper_id: &str,
        expected_revision: i64,
        metadata: &PaperMetadataInput,
        venue_id: Option<&str>,
        updated_at: &str,
    ) -> Result<usize, AppError> {
        let review = serde_json::to_value(&metadata.review_type)
            .map_err(|_| AppError::new(ErrorCode::IntegrityFailure))?
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| AppError::new(ErrorCode::IntegrityFailure))?;
        Ok(tx.execute(
            "UPDATE papers SET title=?1,doi=?2,year=?3,review_type=?4,domain=?5,venue_id=?6,revision=revision+1,updated_at=?7 WHERE id=?8 AND revision=?9",
            params![metadata.title,metadata.doi,metadata.year,review,metadata.domain,venue_id,updated_at,paper_id,expected_revision],
        )?)
    }

    fn replace_paper_authors(
        &self,
        tx: &Transaction<'_>,
        paper_id: &str,
        authors: &[String],
    ) -> Result<(), AppError> {
        tx.execute("DELETE FROM paper_authors WHERE paper_id=?1", [paper_id])?;
        for (order, name) in authors.iter().enumerate() {
            let author_id = uuid::Uuid::new_v4().to_string();
            tx.execute(
                "INSERT INTO authors(id,display_name) VALUES(?1,?2)",
                params![author_id, name],
            )?;
            tx.execute(
                "INSERT INTO paper_authors(paper_id,author_id,author_order) VALUES(?1,?2,?3)",
                params![paper_id, author_id, order as i64],
            )?;
        }
        Ok(())
    }

    fn lifecycle_state(
        &self,
        tx: &Transaction<'_>,
        paper_id: &str,
    ) -> Result<Option<(String, Option<String>)>, AppError> {
        Ok(tx
            .query_row(
                "SELECT lifecycle,archived_from_lifecycle FROM papers WHERE id=?1",
                [paper_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?)
    }

    fn update_lifecycle(
        &self,
        tx: &Transaction<'_>,
        paper_id: &str,
        expected_revision: i64,
        lifecycle: &str,
        archived_from: Option<&str>,
        updated_at: &str,
    ) -> Result<usize, AppError> {
        Ok(tx.execute(
            "UPDATE papers SET lifecycle=?1,archived_from_lifecycle=?2,revision=revision+1,updated_at=?3 WHERE id=?4 AND revision=?5",
            params![lifecycle,archived_from,updated_at,paper_id,expected_revision],
        )?)
    }

    fn audit_change(
        &self,
        tx: &Transaction<'_>,
        request_id: &str,
        action: &str,
        entity_id: &str,
        changes: &Json,
        created_at: &str,
    ) -> Result<(), AppError> {
        tx.execute(
            "INSERT INTO audit_events(id,request_id,action,entity_id,changes_json,created_at) VALUES(?1,?2,?3,?4,?5,?6)",
            params![
                uuid::Uuid::new_v4().to_string(),
                request_id,
                action,
                entity_id,
                changes.to_string(),
                created_at,
            ],
        )?;
        Ok(())
    }

    fn paper(&self, tx: &Transaction<'_>, paper_id: &str) -> Result<PaperDto, AppError> {
        paper(tx, paper_id)
    }

    fn set_last_opened_at(
        &self,
        tx: &Transaction<'_>,
        paper_id: &str,
        now: &str,
    ) -> Result<usize, AppError> {
        Ok(tx.execute(
            "UPDATE papers SET last_opened_at=?1 WHERE id=?2",
            params![now, paper_id],
        )?)
    }

    fn import_record(&self, tx: &Transaction<'_>, token: &str) -> Result<ImportRecord, AppError> {
        get_record(tx, token)
    }

    fn has_duplicate(
        &self,
        tx: &Transaction<'_>,
        doi: Option<&str>,
        sha256: &str,
    ) -> Result<bool, AppError> {
        Ok(tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM papers WHERE doi=?1 AND ?1 IS NOT NULL) OR EXISTS(SELECT 1 FROM documents WHERE sha256=?2)",
            params![doi, sha256],
            |row| row.get(0),
        )?)
    }

    fn duplicate_candidates(
        &self,
        tx: &Transaction<'_>,
        doi: Option<&str>,
        sha256: &str,
    ) -> Result<Vec<DuplicateCandidate>, AppError> {
        let mut statement = tx.prepare(
            "SELECT DISTINCT p.id,p.title,EXISTS(SELECT 1 FROM documents d2 WHERE d2.paper_id=p.id AND d2.sha256=?1),COALESCE(p.doi=?2,0) FROM papers p WHERE (?2 IS NOT NULL AND p.doi=?2) OR EXISTS(SELECT 1 FROM documents d2 WHERE d2.paper_id=p.id AND d2.sha256=?1) ORDER BY p.id",
        )?;
        let candidates = statement
            .query_map(params![sha256, doi], |row| {
                Ok(DuplicateCandidate {
                    paper_id: row.get(0)?,
                    title: row.get(1)?,
                    sha256_match: row.get(2)?,
                    doi_match: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(candidates)
    }

    fn reserve_import(
        &self,
        tx: &Transaction<'_>,
        reservation: ImportReservation<'_>,
    ) -> Result<(), AppError> {
        let changed = tx.execute(
            "UPDATE import_operations SET request_id=?1,metadata_json=?2,payload_hash=?3,reserved_paper_id=?4,reserved_document_id=?5,destination_path=?6,updated_at=?7 WHERE id=?8 AND state IN('STAGING','PROMOTED')",
            params![reservation.request_id,reservation.metadata_json.to_string(),reservation.payload_hash,reservation.paper_id,reservation.document_id,reservation.destination_path,chrono::Utc::now().to_rfc3339(),reservation.operation_id],
        )?;
        if changed != 1 {
            return Err(AppError::new(ErrorCode::Conflict));
        }
        Ok(())
    }

    fn insert_paper_document(
        &self,
        tx: &Transaction<'_>,
        document: NewPaperDocument<'_>,
    ) -> Result<(), AppError> {
        insert_metadata(tx, document)
    }

    fn mark_import_committed(
        &self,
        tx: &Transaction<'_>,
        operation_id: &str,
        paper: &PaperDto,
        updated_at: &str,
    ) -> Result<(), AppError> {
        tx.execute(
            "UPDATE import_operations SET state='COMMITTED',result_json=?1,updated_at=?2 WHERE id=?3",
            params![serde_json::to_string(paper).map_err(|_| AppError::new(ErrorCode::IntegrityFailure))?,updated_at,operation_id],
        )?;
        Ok(())
    }
}

fn review(s: &str) -> Result<ReviewType, AppError> {
    serde_json::from_value(json!(s)).map_err(|_| AppError::new(ErrorCode::IntegrityFailure))
}
fn lifecycle(s: &str) -> Result<PaperLifecycle, AppError> {
    serde_json::from_value(json!(s)).map_err(|_| AppError::new(ErrorCode::IntegrityFailure))
}
fn review_type_value(review: &ReviewType) -> &'static str {
    match review {
        ReviewType::Survey => "survey",
        ReviewType::TopicalReview => "topical_review",
        ReviewType::Slr => "slr",
        ReviewType::MappingStudy => "mapping_study",
        ReviewType::Tutorial => "tutorial",
        ReviewType::Other => "other",
        ReviewType::Unknown => "unknown",
    }
}
fn phase(s: Option<String>) -> Result<Option<PhaseCode>, AppError> {
    s.map(|x| {
        serde_json::from_value(json!(x)).map_err(|_| AppError::new(ErrorCode::IntegrityFailure))
    })
    .transpose()
}
fn phase_filter_value(phase: PhaseCode) -> &'static str {
    match phase {
        PhaseCode::PRE => "PRE",
        PhaseCode::P1 => "P1",
        PhaseCode::P2 => "P2",
        PhaseCode::P3 => "P3",
        PhaseCode::P4 => "P4",
    }
}
fn archived(s: Option<String>) -> Result<Option<PaperDtoArchivedFromLifecycle>, AppError> {
    s.map(|x| {
        serde_json::from_value(json!(x)).map_err(|_| AppError::new(ErrorCode::IntegrityFailure))
    })
    .transpose()
}
fn paper(c: &Connection, id: &str) -> Result<PaperDto, AppError> {
    let row=c.query_row("SELECT p.id,p.title,p.year,p.doi,p.review_type,p.domain,v.name,p.lifecycle,p.archived_from_lifecycle,p.current_phase,p.processing_initialized,p.revision,p.created_at,p.updated_at,p.last_opened_at,COALESCE(p.active_document_id,'') FROM papers p LEFT JOIN venues v ON v.id=p.venue_id WHERE p.id=?1",[id],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,Option<i64>>(2)?,r.get::<_,Option<String>>(3)?,r.get::<_,String>(4)?,r.get::<_,Option<String>>(5)?,r.get::<_,Option<String>>(6)?,r.get::<_,String>(7)?,r.get::<_,Option<String>>(8)?,r.get::<_,Option<String>>(9)?,r.get::<_,bool>(10)?,r.get::<_,i64>(11)?,r.get::<_,String>(12)?,r.get::<_,String>(13)?,r.get::<_,Option<String>>(14)?,r.get::<_,String>(15)?))).optional()?.ok_or_else(||AppError::new(ErrorCode::NotFound))?;
    let mut q=c.prepare("SELECT a.display_name FROM paper_authors pa JOIN authors a ON a.id=pa.author_id WHERE pa.paper_id=?1 ORDER BY pa.author_order")?;
    let authors = q
        .query_map([id], |r| r.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(PaperDto {
        id: UUID(row.0),
        title: row.1,
        year: row.2,
        doi: row.3,
        review_type: review(&row.4)?,
        domain: row.5,
        venue: row.6,
        authors,
        lifecycle: lifecycle(&row.7)?,
        archived_from_lifecycle: archived(row.8)?,
        active_phase_code: phase(row.9)?,
        processing_initialized: row.10,
        revision: row.11,
        created_at: row.12,
        updated_at: row.13,
        last_opened_at: row.14,
        document_id: UUID(row.15),
    })
}
fn insert_metadata(tx: &Transaction<'_>, document: NewPaperDocument<'_>) -> Result<(), AppError> {
    let NewPaperDocument {
        paper_id,
        doc_id,
        metadata,
        sha,
        size,
        filename,
        relative,
        now,
    } = document;
    let venue_id = if let Some(name) = &metadata.venue {
        tx.query_row(
            "SELECT id FROM venues WHERE name=?1 ORDER BY id LIMIT 1",
            [name],
            |r| r.get::<_, String>(0),
        )
        .optional()?
        .or_else(|| Some(uuid::Uuid::new_v4().to_string()))
    } else {
        None
    };
    if let (Some(venue_id), Some(name)) = (&venue_id, &metadata.venue) {
        tx.execute(
            "INSERT OR IGNORE INTO venues(id,name) VALUES(?1,?2)",
            params![venue_id, name],
        )?;
    }
    tx.execute("INSERT INTO papers(id,title,doi,year,review_type,domain,venue_id,lifecycle,revision,current_phase,processing_initialized,active_document_id,created_at,updated_at,last_opened_at) VALUES(?1,?2,?3,?4,?5,?6,?7,'NEW',0,NULL,0,?8,?9,?9,NULL)",params![paper_id,metadata.title,metadata.doi,metadata.year,review_type_value(&metadata.review_type),metadata.domain,venue_id,doc_id,now])?;
    for (index, name) in metadata.authors.iter().enumerate() {
        let author_id = uuid::Uuid::new_v4().to_string();
        tx.execute(
            "INSERT INTO authors(id,display_name) VALUES(?1,?2)",
            params![author_id, name],
        )?;
        tx.execute(
            "INSERT INTO paper_authors(paper_id,author_id,author_order) VALUES(?1,?2,?3)",
            params![paper_id, author_id, index as i64],
        )?;
    }
    tx.execute("INSERT INTO documents(id,paper_id,original_filename,relative_path,sha256,media_type,size_bytes,imported_at,status) VALUES(?1,?2,?3,?4,?5,'application/pdf',?6,?7,'ACTIVE')",params![doc_id,paper_id,filename,relative,sha,size,now])?;
    Ok(())
}
fn parse_record(r: &rusqlite::Row<'_>) -> rusqlite::Result<ImportRecord> {
    Ok(ImportRecord {
        operation_id: r.get(0)?,
        token: r.get(1)?,
        request_id: r.get(2)?,
        library_id: r.get(3)?,
        state: r.get(4)?,
        staging_path: r.get(5)?,
        destination_path: r.get(6)?,
        original_filename: r.get(7)?,
        sha256: r.get(8)?,
        size_bytes: r.get(9)?,
        reserved_paper_id: r.get(10)?,
        reserved_document_id: r.get(11)?,
        metadata_json: r.get(12)?,
        payload_hash: r.get(13)?,
        result_json: r.get(14)?,
        error_json: r.get(15)?,
        expires_at: r.get(16)?,
    })
}
const RECORD_SELECT: &str = "SELECT id,import_token,request_id,library_id,state,staging_path,destination_path,original_filename,sha256,size_bytes,reserved_paper_id,reserved_document_id,metadata_json,payload_hash,result_json,error_json,expires_at FROM import_operations";
fn get_record(c: &Connection, token: &str) -> Result<ImportRecord, AppError> {
    c.query_row(
        &format!("{RECORD_SELECT} WHERE import_token=?1"),
        [token],
        parse_record,
    )
    .optional()?
    .ok_or_else(|| AppError::new(ErrorCode::NotFound))
}
fn as_paper(value: Json) -> Result<PaperDto, AppError> {
    serde_json::from_value(value).map_err(|_| AppError::new(ErrorCode::IntegrityFailure))
}
fn preview(c: &Connection, record: &ImportRecord) -> Result<ImportPreviewDto, AppError> {
    let hash = record
        .sha256
        .as_deref()
        .ok_or_else(|| AppError::new(ErrorCode::IntegrityFailure))?;
    let mut stmt=c.prepare("SELECT DISTINCT p.id,p.title,EXISTS(SELECT 1 FROM documents d WHERE d.paper_id=p.id AND d.sha256=?1),COALESCE(p.doi=?2,0) FROM papers p LEFT JOIN documents d ON d.paper_id=p.id WHERE (?2 IS NOT NULL AND p.doi=?2) OR EXISTS(SELECT 1 FROM documents d2 WHERE d2.paper_id=p.id AND d2.sha256=?1) ORDER BY p.id")?;
    let candidates = stmt
        .query_map(params![hash, Option::<String>::None], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, bool>(2)?,
                r.get::<_, bool>(3)?,
            ))
        })?
        .map(|r| {
            let (id, title, by_hash, by_doi) = r?;
            let mut reasons = Vec::new();
            if by_doi {
                reasons.push(DuplicateCandidateDtoReasonsItem::Doi)
            }
            if by_hash {
                reasons.push(DuplicateCandidateDtoReasonsItem::Sha256)
            }
            Ok(DuplicateCandidateDto {
                paper_id: UUID(id),
                title,
                reasons,
            })
        })
        .collect::<Result<Vec<_>, rusqlite::Error>>()?;
    Ok(ImportPreviewDto {
        import_token: UUID(record.token.clone()),
        original_filename: record.original_filename.clone(),
        size_bytes: record.size_bytes.unwrap_or(0),
        sha256: hash.to_owned(),
        candidates,
        expires_at: record.expires_at.clone(),
    })
}
fn payload_metadata(
    token: &UUID,
    original: &PaperMetadataInput,
    resolution: &Option<DuplicateResolution>,
) -> Json {
    let mut object = serde_json::Map::new();
    object.insert("importToken".into(), json!(token));
    object.insert("metadata".into(), json!(original));
    if let Some(resolution) = resolution {
        object.insert("duplicateResolution".into(), json!(resolution));
    }
    Json::Object(object)
}
fn commit_confirmation(
    tx: &Transaction<'_>,
    prepared: &PreparedImport,
    verified: &ResourceInspection,
) -> Result<PaperDto, AppError> {
    crate::application::library::confirm_in_tx(
        tx,
        &SqliteLibraryRepository,
        &super::workflow_repository::SqliteWorkflowRepository,
        prepared,
        verified,
    )
}

impl LibraryPersistence for SqliteLibraryPersistence {
    fn library_id(&self) -> &str {
        &self.library_id
    }
    fn writable(&self) -> bool {
        self.actor.info().writable
    }
    fn begin_import(
        &self,
        request_id: UUID,
        operation_id: String,
        token: String,
        filename: String,
        expires_at: String,
    ) -> LibraryFuture<'static, ()> {
        let actor = self.actor.clone();
        let library = self.library_id.clone();
        Box::pin(async move {
            actor.submit(move|c|{let now=chrono::Utc::now().to_rfc3339();c.execute("INSERT INTO import_operations(id,import_token,request_id,library_id,state,staging_path,destination_path,original_filename,source_path_private,sha256,size_bytes,reserved_paper_id,reserved_document_id,metadata_json,payload_hash,result_json,error_json,created_at,updated_at,expires_at) VALUES(?1,?2,?3,?4,'STAGING',?5,NULL,?6,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,?7,?7,?8)",params![operation_id,token,request_id.0,library,format!("staging/{operation_id}/source.pdf"),filename,now,expires_at])?;Ok(())}).await
        })
    }
    fn record_stage_failure(
        &self,
        operation_id: String,
        failure: StageFailure,
    ) -> LibraryFuture<'static, ()> {
        let actor = self.actor.clone();
        Box::pin(async move {
            let code = serde_json::to_value(failure.error.code)
                .ok()
                .and_then(|value| value.as_str().map(str::to_owned))
                .ok_or_else(|| AppError::new(ErrorCode::IntegrityFailure))?;
            let cleanup = match failure.cleanup {
                StageCleanup::Pending => "PENDING",
                StageCleanup::Done => "DONE",
            };
            let error_json =
                json!({"version":1,"kind":"stageFailure","code":code,"cleanup":cleanup});
            actor
                .submit(move |connection| {
                    with_transaction(connection, |tx| {
                        let request_id: Option<String> = tx
                            .query_row(
                                "SELECT request_id FROM import_operations WHERE id=?1 AND state='STAGING'",
                                [&operation_id],
                                |row| row.get(0),
                            )
                            .optional()?;
                        let request_id = request_id
                            .ok_or_else(|| AppError::new(ErrorCode::Conflict))?;
                        let now = chrono::Utc::now().to_rfc3339();
                        let changed = tx.execute(
                            "UPDATE import_operations SET state='FAILED',error_json=?1,updated_at=?2 WHERE id=?3 AND state='STAGING'",
                            params![error_json.to_string(), now, operation_id],
                        )?;
                        if changed != 1 {
                            return Err(AppError::new(ErrorCode::Conflict));
                        }
                        SqliteLibraryRepository.audit_change(
                            tx,
                            &request_id,
                            "import.stage_failed",
                            &operation_id,
                            &json!({"code":code,"cleanup":cleanup}),
                            &now,
                        )?;
                        Ok(())
                    })
                })
                .await
        })
    }
    fn complete_stage_failure_cleanup(&self, operation_id: String) -> LibraryFuture<'static, ()> {
        let actor = self.actor.clone();
        Box::pin(async move {
            actor
                .submit(move |connection| {
                    with_transaction(connection, |tx| {
                        let request_id: Option<String> = tx
                            .query_row(
                                "SELECT request_id FROM import_operations WHERE id=?1 AND state='FAILED' AND json_extract(error_json,'$.version')=1 AND json_extract(error_json,'$.kind')='stageFailure' AND json_extract(error_json,'$.cleanup')='PENDING'",
                                [&operation_id],
                                |row| row.get(0),
                            )
                            .optional()?;
                        let request_id = request_id
                            .ok_or_else(|| AppError::new(ErrorCode::Conflict))?;
                        let now = chrono::Utc::now().to_rfc3339();
                        let changed = tx.execute(
                            "UPDATE import_operations SET error_json=json_set(error_json,'$.cleanup','DONE'),updated_at=?1 WHERE id=?2 AND state='FAILED' AND json_extract(error_json,'$.version')=1 AND json_extract(error_json,'$.kind')='stageFailure' AND json_extract(error_json,'$.cleanup')='PENDING'",
                            params![now, operation_id],
                        )?;
                        if changed != 1 {
                            return Err(AppError::new(ErrorCode::Conflict));
                        }
                        SqliteLibraryRepository.audit_change(
                            tx,
                            &request_id,
                            "import.stage_failure_reconciled",
                            &operation_id,
                            &json!({"cleanup":"DONE"}),
                            &now,
                        )?;
                        Ok(())
                    })
                })
                .await
        })
    }
    fn previous_selection(
        &self,
        request_id: UUID,
    ) -> LibraryFuture<'static, Option<ImportPreviewDto>> {
        let actor = self.actor.clone();
        Box::pin(async move {
            actor.submit(move|c|{let previous:Option<(String,String,String)>=c.query_row("SELECT command,payload_hash,result_json FROM operation_receipts WHERE request_id=?1",[request_id.0],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional()?;let Some((command,hash,result))=previous else{return Ok(None)};if command!="library_select_pdf"||hash!=canonical_hash(&json!({}))?{return Err(AppError::new(ErrorCode::Conflict));}serde_json::from_str(&result).map(Some).map_err(|_|AppError::new(ErrorCode::IntegrityFailure))}).await
        })
    }
    fn previous_confirmation(
        &self,
        request_id: UUID,
        token: UUID,
        metadata: PaperMetadataInput,
        resolution: Option<DuplicateResolution>,
    ) -> LibraryFuture<'static, Option<PaperDto>> {
        let actor = self.actor.clone();
        Box::pin(async move {
            actor.submit(move|c|{let previous:Option<(String,String,String)>=c.query_row("SELECT command,payload_hash,result_json FROM operation_receipts WHERE request_id=?1",[request_id.0],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional()?;let Some((command,hash,result))=previous else{return Ok(None)};let payload=payload_metadata(&token,&metadata,&resolution);if command!="library_confirm_import"||hash!=canonical_hash(&payload)?{return Err(AppError::new(ErrorCode::Conflict));}serde_json::from_str(&result).map(Some).map_err(|_|AppError::new(ErrorCode::IntegrityFailure))}).await
        })
    }
    fn previous_cancel(&self, request_id: UUID, token: UUID) -> LibraryFuture<'static, bool> {
        let actor = self.actor.clone();
        Box::pin(async move {
            actor.submit(move|c|{let previous:Option<(String,String)>=c.query_row("SELECT command,payload_hash FROM operation_receipts WHERE request_id=?1",[request_id.0],|r|Ok((r.get(0)?,r.get(1)?))).optional()?;let Some((command,hash))=previous else{return Ok(false)};if command!="library_cancel_import"||hash!=canonical_hash(&json!({"importToken":token}))?{return Err(AppError::new(ErrorCode::Conflict));}Ok(true)}).await
        })
    }
    fn record_staged(
        &self,
        request_id: UUID,
        staged: StagedPdf,
        expires_at: String,
    ) -> LibraryFuture<'static, ImportPreviewDto> {
        let actor = self.actor.clone();
        Box::pin(async move {
            let value = actor
                .submit(move |connection| {
                    with_receipt(
                        connection,
                        &request_id.0,
                        "library_select_pdf",
                        &json!({}),
                        |tx| {
                            let now = chrono::Utc::now().to_rfc3339();
                            tx.execute(
                                "UPDATE import_operations SET sha256=?1,size_bytes=?2,updated_at=?3 WHERE id=?4 AND state='STAGING'",
                                params![staged.sha256, staged.size_bytes, now, staged.operation_id],
                            )?;
                            let token = tx.query_row(
                                "SELECT import_token FROM import_operations WHERE id=?1",
                                [&staged.operation_id],
                                |row| row.get::<_, String>(0),
                            )?;
                            SqliteLibraryRepository.audit_change(
                                tx,
                                &request_id.0,
                                "import.staged",
                                &staged.operation_id,
                                &json!({"sizeBytes":staged.size_bytes,"sha256":staged.sha256}),
                                &now,
                            )?;
                            let mut record = get_record(tx, &token)?;
                            record.expires_at = expires_at;
                            serde_json::to_value(preview(tx, &record)?)
                                .map_err(|_| AppError::new(ErrorCode::IntegrityFailure))
                        },
                    )
                })
                .await?;
            serde_json::from_value(value).map_err(|_| AppError::new(ErrorCode::IntegrityFailure))
        })
    }
    fn preview(&self, token: String) -> LibraryFuture<'static, ImportPreviewDto> {
        let actor = self.actor.clone();
        Box::pin(async move {
            actor
                .submit(move |c| {
                    let record = get_record(c, &token)?;
                    preview(c, &record)
                })
                .await
        })
    }
    fn prepare_confirmation(
        &self,
        request_id: UUID,
        token: UUID,
        original_metadata: PaperMetadataInput,
        resolution: Option<DuplicateResolution>,
    ) -> LibraryFuture<'static, PreparedImport> {
        let actor = self.actor.clone();
        Box::pin(async move {
            actor
                .submit(move |c| {
                    with_transaction(c, |tx| {
                        crate::application::library::prepare_import_in_tx(
                            tx,
                            &SqliteLibraryRepository,
                            &request_id.0,
                            &token.0,
                            &original_metadata,
                            resolution.as_ref(),
                        )
                    })
                })
                .await
        })
    }
    fn record_promoted(&self, prepared: PreparedImport) -> LibraryFuture<'static, ()> {
        let actor = self.actor.clone();
        Box::pin(async move {
            actor
                .submit(move |connection| {
                    with_transaction(connection, |tx| {
                        let request_id = prepared
                            .record
                            .request_id
                            .as_deref()
                            .ok_or_else(|| AppError::new(ErrorCode::IntegrityFailure))?;
                        let now = chrono::Utc::now().to_rfc3339();
                        let changed = tx.execute(
                            "UPDATE import_operations SET state='PROMOTED',updated_at=?1 WHERE id=?2 AND state IN('STAGING','PROMOTED')",
                            params![now, prepared.record.operation_id],
                        )?;
                        if changed != 1 {
                            return Err(AppError::new(ErrorCode::Conflict));
                        }
                        SqliteLibraryRepository.audit_change(
                            tx,
                            request_id,
                            "import.promoted",
                            &prepared.record.operation_id,
                            &json!({"documentId":prepared.document_id}),
                            &now,
                        )?;
                        Ok(())
                    })
                })
                .await
        })
    }
    fn commit_confirmation(
        &self,
        request_id: UUID,
        prepared: PreparedImport,
        verified: ResourceInspection,
    ) -> LibraryFuture<'static, PaperDto> {
        let actor = self.actor.clone();
        Box::pin(async move {
            let payload = prepared.confirmation_payload.clone();
            let result = actor
                .submit(move |c| {
                    with_receipt(c, &request_id.0, "library_confirm_import", &payload, |tx| {
                        serde_json::to_value(commit_confirmation(tx, &prepared, &verified)?)
                            .map_err(|_| AppError::new(ErrorCode::IntegrityFailure))
                    })
                })
                .await?;
            as_paper(result)
        })
    }
    fn prepare_cancel(&self, request_id: UUID, token: UUID) -> LibraryFuture<'static, CancelPlan> {
        let actor = self.actor.clone();
        Box::pin(async move {
            actor
                .submit(move |connection| {
                    with_transaction(connection, |tx| {
                        let mut record = get_record(tx, &token.0)?;
                        if !matches!(record.state.as_str(), "STAGING" | "PROMOTED" | "FAILED") {
                            return Err(AppError::new(ErrorCode::Conflict));
                        }
                        let payload = json!({"importToken":token});
                        let promotion_confirmed_from_previous = if record.state == "FAILED" {
                            let Some((_, previously_confirmed)) =
                                crate::application::library::valid_pending_cancellation(&record)
                            else {
                                return Err(AppError::new(ErrorCode::ImportRecoveryRequired));
                            };
                            if previously_confirmed
                                && !crate::application::library::has_durable_promoted_confirmation(&record)
                            {
                                return Err(AppError::new(ErrorCode::ImportRecoveryRequired));
                            }
                            previously_confirmed
                        } else if record.state == "PROMOTED" {
                            if !crate::application::library::has_durable_promoted_confirmation(&record) {
                                return Err(AppError::new(ErrorCode::ImportRecoveryRequired));
                            }
                            true
                        } else {
                            false
                        };
                        let confirmation_receipt: i64 = match record.request_id.as_deref() {
                            Some(confirmation_request) => tx.query_row(
                                "SELECT count(*) FROM operation_receipts WHERE request_id=?1 AND command='library_confirm_import'",
                                [confirmation_request],
                                |row| row.get(0),
                            )?,
                            None => 0,
                        };
                        if promotion_confirmed_from_previous && confirmation_receipt != 0 {
                            return Err(AppError::new(ErrorCode::ImportRecoveryRequired));
                        }
                        let promotion_confirmed = promotion_confirmed_from_previous
                            && confirmation_receipt == 0;
                        let error = json!({"version":1,"kind":"cancellation","code":"OperationCancelled","requestId":request_id.0,"receipt":{"command":"library_cancel_import","payload":payload},"cleanup":"PENDING","promotionConfirmed":promotion_confirmed});
                        let now = chrono::Utc::now().to_rfc3339();
                        tx.execute(
                            "UPDATE import_operations SET state='FAILED',error_json=?1,updated_at=?2 WHERE id=?3",
                            params![error.to_string(), now, record.operation_id],
                        )?;
                        SqliteLibraryRepository.audit_change(
                            tx,
                            &request_id.0,
                            "import.cancellation_prepared",
                            &record.operation_id,
                            &json!({"cleanup":"PENDING"}),
                            &now,
                        )?;
                        record.state = "FAILED".into();
                        record.error_json = Some(error.to_string());
                        Ok(CancelPlan {
                            staging_path: record.staging_path.clone(),
                            destination_path: record.destination_path.clone(),
                            expected_sha256: record.sha256.clone(),
                            payload,
                            record,
                        })
                    })
                })
                .await
        })
    }
    fn validate_cleanup_plan(&self, plan: CancelPlan) -> LibraryFuture<'static, ()> {
        let actor = self.actor.clone();
        let library_id = self.library_id.clone();
        Box::pin(async move {
            actor
                .submit(move |connection| {
                    with_transaction(connection, |tx| {
                        let current = get_record(tx, &plan.record.token)?;
                        let canonical = |value: &str| {
                            uuid::Uuid::parse_str(value)
                                .is_ok_and(|parsed| parsed.to_string() == value)
                        };
                        let expected_staging =
                            format!("staging/{}/source.pdf", current.operation_id);
                        let destination_is_consistent = current.destination_path.as_ref().is_none_or(
                            |destination| {
                                current.reserved_document_id.as_ref().is_some_and(|document_id| {
                                    destination == &format!("documents/{document_id}/original.pdf")
                                })
                            },
                        );
                        let reserved_document_without_destination_is_consistent = current
                            .destination_path
                            .is_some()
                            || current.reserved_document_id.is_none()
                            || current
                                .metadata_json
                                .as_deref()
                                .and_then(|raw| serde_json::from_str::<Json>(raw).ok())
                                .is_some_and(|intent| {
                                    intent["version"] == 1
                                        && intent["kind"] == "confirmation"
                                        && intent["duplicateResolution"]["action"]
                                            == "reuseExisting"
                                        && intent["duplicateResolution"]["paperId"].as_str()
                                            == current.reserved_paper_id.as_deref()
                                });
                        let cancellation_intent = current
                            .error_json
                            .as_deref()
                            .and_then(|raw| serde_json::from_str::<Json>(raw).ok());
                        let cancellation_request = cancellation_intent
                            .as_ref()
                            .and_then(|intent| intent["requestId"].as_str());
                        let valid_pending_cancellation =
                            crate::application::library::valid_pending_cancellation(&current);
                        let has_cancellation_intent = cancellation_intent
                            .as_ref()
                            .is_some_and(|intent| intent["kind"] == "cancellation");
                        let cleanup_plan_is_valid = if has_cancellation_intent {
                            valid_pending_cancellation
                                .as_ref()
                                .is_some_and(|(request_id, _)| {
                                    cancellation_request == Some(request_id.as_str())
                                        && canonical(request_id)
                                })
                                && cancellation_intent.as_ref().is_some_and(|intent| {
                                    intent["version"] == 1
                                        && intent["cleanup"] == "PENDING"
                                        && intent["receipt"]["command"] == "library_cancel_import"
                                        && intent["receipt"]["payload"] == plan.payload
                                        && plan.payload == json!({"importToken":current.token})
                                })
                        } else {
                            cancellation_intent.is_none()
                                && current.state != "FAILED"
                                && current.destination_path.is_none()
                                && plan.payload.is_null()
                                && current.metadata_json.as_deref().is_some_and(|raw| {
                                    serde_json::from_str::<Json>(raw).ok().is_some_and(|intent| {
                                        intent["version"] == 1
                                            && intent["kind"] == "confirmation"
                                            && intent["duplicateResolution"]["action"] == "reuseExisting"
                                    })
                                })
                        };
                        let promotion_confirmed = cancellation_intent
                            .as_ref()
                            .is_some_and(|intent| intent["promotionConfirmed"] == true);
                        if !canonical(&library_id)
                            || current.library_id != library_id
                            || !canonical(&current.operation_id)
                            || !canonical(&current.token)
                            || !matches!(current.state.as_str(), "FAILED" | "STAGING" | "PROMOTED")
                            || current.operation_id != plan.record.operation_id
                            || current.state != plan.record.state
                            || current.library_id != plan.record.library_id
                            || current.staging_path != expected_staging
                            || current.staging_path != plan.staging_path
                            || current.staging_path != plan.record.staging_path
                            || !destination_is_consistent
                            || current.destination_path != plan.destination_path
                            || current.destination_path != plan.record.destination_path
                            || current.sha256 != plan.expected_sha256
                            || current.sha256 != plan.record.sha256
                            || current.error_json != plan.record.error_json
                            || (has_cancellation_intent
                                && plan.payload != json!({"importToken":current.token}))
                            || !cleanup_plan_is_valid
                            || current.reserved_document_id != plan.record.reserved_document_id
                            || current.reserved_document_id.as_deref().is_some_and(|id| !canonical(id))
                            || current.reserved_paper_id.as_deref().is_some_and(|id| !canonical(id))
                            || !reserved_document_without_destination_is_consistent
                            || (promotion_confirmed
                                && (current.state != "FAILED"
                                    || !crate::application::library::has_durable_promoted_confirmation(&current)))
                        {
                            return Err(AppError::new(ErrorCode::ImportRecoveryRequired));
                        }
                        if promotion_confirmed {
                            let confirmation_request = current.request_id.as_deref()
                                .ok_or_else(|| AppError::new(ErrorCode::ImportRecoveryRequired))?;
                            let successful_confirmation: i64 = tx.query_row(
                                "SELECT count(*) FROM operation_receipts WHERE request_id=?1 AND command='library_confirm_import'",
                                [confirmation_request],
                                |row| row.get(0),
                            )?;
                            if successful_confirmation != 0 || current.result_json.is_some() {
                                return Err(AppError::new(ErrorCode::ImportRecoveryRequired));
                            }
                        }
                        let paths = std::iter::once(current.staging_path.as_str())
                            .chain(current.destination_path.as_deref());
                        for path in paths {
                            let other_intents: i64 = tx.query_row(
                                "SELECT count(*) FROM import_operations WHERE id<>?1 AND (staging_path=?2 OR destination_path=?2)",
                                params![current.operation_id, path],
                                |row| row.get(0),
                            )?;
                            let documents: i64 = tx.query_row(
                                "SELECT count(*) FROM documents WHERE relative_path=?1",
                                [path],
                                |row| row.get(0),
                            )?;
                            if other_intents != 0 || documents != 0 {
                                return Err(AppError::new(ErrorCode::ImportRecoveryRequired));
                            }
                        }
                        Ok(())
                    })
                })
                .await
        })
    }
    fn commit_cancel(
        &self,
        request_id: UUID,
        plan: CancelPlan,
        cleanup: ResourceInspection,
    ) -> LibraryFuture<'static, ()> {
        let actor = self.actor.clone();
        Box::pin(async move {
            actor
                .submit(move |connection| {
                    with_receipt(
                        connection,
                        &request_id.0,
                        "library_cancel_import",
                        &plan.payload,
                        |tx| {
                            if cleanup.ambiguous || cleanup.staged_matches || cleanup.destination_matches {
                                return Err(AppError::new(ErrorCode::ImportRecoveryRequired));
                            }
                            let now = chrono::Utc::now().to_rfc3339();
                            let updated = tx.execute(
                                "UPDATE import_operations SET error_json=json_set(error_json,'$.cleanup','DONE'),updated_at=?1 WHERE id=?2 AND state='FAILED' AND json_extract(error_json,'$.kind')='cancellation' AND json_extract(error_json,'$.cleanup')='PENDING' AND json_extract(error_json,'$.requestId')=?3",
                                params![now, plan.record.operation_id, request_id.0],
                            )?;
                            if updated != 1 {
                                return Err(AppError::new(ErrorCode::ImportRecoveryRequired));
                            }
                            SqliteLibraryRepository.audit_change(
                                tx,
                                &request_id.0,
                                "import.cancelled",
                                &plan.record.operation_id,
                                &json!({"cleanup":"DONE"}),
                                &now,
                            )?;
                            Ok(Json::Null)
                        },
                    )
                })
                .await
                .map(|_| ())
        })
    }
    fn recoverable_imports(&self) -> LibraryFuture<'static, Vec<ImportRecord>> {
        let actor = self.actor.clone();
        Box::pin(async move {
            actor.submit(move|c|{let mut s=c.prepare(&format!("{RECORD_SELECT} WHERE (state IN('STAGING','PROMOTED')) OR (state='FAILED' AND COALESCE(json_extract(error_json,'$.cleanup'),'PENDING')!='DONE') OR (state='COMMITTED' AND destination_path IS NOT NULL) ORDER BY created_at,id"))?;Ok(s.query_map([],parse_record)?.collect::<Result<Vec<_>,_>>()?)}).await
        })
    }
    fn confirmed_recovery(
        &self,
        record: ImportRecord,
    ) -> LibraryFuture<'static, Option<ConfirmedRecovery>> {
        let actor = self.actor.clone();
        Box::pin(async move {
            actor
                .submit(move |c| {
                    with_transaction(c, |tx| {
                        let Some(raw) = record.metadata_json.as_deref() else {
                            return Ok(None);
                        };
                        let value: Json = serde_json::from_str(raw)
                            .map_err(|_| AppError::new(ErrorCode::IntegrityFailure))?;
                        if value.get("version").and_then(Json::as_i64) != Some(1)
                            || value.get("kind").and_then(Json::as_str) != Some("confirmation")
                        {
                            return Ok(None);
                        }
                        let Some(request_id) = record.request_id.clone() else {
                            return Ok(None);
                        };
                        let receipt = &value["receipt"];
                        if receipt["command"] != "library_confirm_import" {
                            return Ok(None);
                        }
                        let payload = receipt["payload"].clone();
                        let original: PaperMetadataInput =
                            serde_json::from_value(payload["metadata"].clone())
                                .map_err(|_| AppError::new(ErrorCode::IntegrityFailure))?;
                        let resolution: Option<DuplicateResolution> =
                            serde_json::from_value(payload["duplicateResolution"].clone())
                                .map_err(|_| AppError::new(ErrorCode::IntegrityFailure))?;
                        let request = UUID(request_id);
                        let prepared = crate::application::library::prepare_import_in_tx(
                            tx,
                            &SqliteLibraryRepository,
                            &request.0,
                            &record.token,
                            &original,
                            resolution.as_ref(),
                        )?;
                        Ok(Some(ConfirmedRecovery {
                            request_id: request,
                            prepared,
                        }))
                    })
                })
                .await
        })
    }
    fn mark_recovered_cancelled(
        &self,
        record: ImportRecord,
        request_id: UUID,
    ) -> LibraryFuture<'static, ()> {
        let actor = self.actor.clone();
        Box::pin(async move {
            actor
                .submit(move |connection| {
                    let Some((intent_request_id, _)) =
                        crate::application::library::valid_pending_cancellation(&record)
                    else {
                        return Err(AppError::new(ErrorCode::ImportRecoveryRequired));
                    };
                    if intent_request_id != request_id.0 {
                        return Err(AppError::new(ErrorCode::ImportRecoveryRequired));
                    }
                    let previous_error = record
                        .error_json
                        .clone()
                        .ok_or_else(|| AppError::new(ErrorCode::IntegrityFailure))?;
                    let error: Json = serde_json::from_str(
                        &previous_error,
                    )
                    .map_err(|_| AppError::new(ErrorCode::IntegrityFailure))?;
                    let payload = error["receipt"]["payload"].clone();
                    with_receipt(
                        connection,
                        &request_id.0,
                        "library_cancel_import",
                        &payload,
                        |tx| {
                            let now = chrono::Utc::now().to_rfc3339();
                            let updated = tx.execute(
                                "UPDATE import_operations SET error_json=json_set(error_json,'$.cleanup','DONE'),updated_at=?1 WHERE id=?2 AND state='FAILED' AND error_json=?3 AND json_extract(error_json,'$.kind')='cancellation' AND json_extract(error_json,'$.cleanup')='PENDING' AND json_extract(error_json,'$.requestId')=?4",
                                params![now, record.operation_id, previous_error, request_id.0],
                            )?;
                            if updated != 1 {
                                return Err(AppError::new(ErrorCode::ImportRecoveryRequired));
                            }
                            SqliteLibraryRepository.audit_change(
                                tx,
                                &request_id.0,
                                "import.cancelled",
                                &record.operation_id,
                                &json!({"cleanup":"DONE"}),
                                &now,
                            )?;
                            Ok(Json::Null)
                        },
                    )
                })
                .await
                .map(|_| ())
        })
    }
    fn list_papers(
        &self,
        _request_id: UUID,
        filter: PaperFilterDto,
    ) -> LibraryFuture<'static, PageDto<PaperDto>> {
        let actor = self.actor.clone();
        Box::pin(async move { actor.submit(move |c| list_papers(c, filter)).await })
    }
    fn get_paper(&self, _request_id: UUID, paper_id: UUID) -> LibraryFuture<'static, PaperDto> {
        let actor = self.actor.clone();
        Box::pin(async move { actor.submit(move |c| paper(c, &paper_id.0)).await })
    }
    fn update_metadata(
        &self,
        request_id: UUID,
        paper_id: UUID,
        revision: i64,
        original: PaperMetadataInput,
    ) -> LibraryFuture<'static, PaperDto> {
        let actor = self.actor.clone();
        Box::pin(async move {
            let payload =
                json!({"paperId":paper_id,"expectedRevision":revision,"metadata":original});
            let result = actor
                .submit(move |c| {
                    with_receipt(
                        c,
                        &request_id.0,
                        "library_update_metadata",
                        &payload,
                        |tx| {
                            let value = update_metadata(
                                tx,
                                &request_id.0,
                                &paper_id.0,
                                revision,
                                &original,
                            )?;
                            Ok(value)
                        },
                    )
                })
                .await?;
            as_paper(result)
        })
    }
    fn archive_paper(
        &self,
        request_id: UUID,
        paper_id: UUID,
        revision: i64,
    ) -> LibraryFuture<'static, PaperDto> {
        let actor = self.actor.clone();
        Box::pin(async move {
            let payload = json!({"paperId":paper_id,"expectedRevision":revision});
            let result = actor
                .submit(move |c| {
                    with_receipt(c, &request_id.0, "library_archive_paper", &payload, |tx| {
                        serde_json::to_value(archive(
                            tx,
                            &request_id.0,
                            &paper_id.0,
                            revision,
                            true,
                        )?)
                        .map_err(|_| AppError::new(ErrorCode::IntegrityFailure))
                    })
                })
                .await?;
            as_paper(result)
        })
    }
    fn restore_paper(
        &self,
        request_id: UUID,
        paper_id: UUID,
        revision: i64,
    ) -> LibraryFuture<'static, PaperDto> {
        let actor = self.actor.clone();
        Box::pin(async move {
            let payload = json!({"paperId":paper_id,"expectedRevision":revision});
            let result = actor
                .submit(move |c| {
                    with_receipt(c, &request_id.0, "library_restore_paper", &payload, |tx| {
                        serde_json::to_value(archive(
                            tx,
                            &request_id.0,
                            &paper_id.0,
                            revision,
                            false,
                        )?)
                        .map_err(|_| AppError::new(ErrorCode::IntegrityFailure))
                    })
                })
                .await?;
            as_paper(result)
        })
    }
}

fn update_metadata(
    tx: &Transaction<'_>,
    request_id: &str,
    id: &str,
    revision: i64,
    metadata: &PaperMetadataInput,
) -> Result<Json, AppError> {
    let value = crate::application::library::update_metadata_in_tx(
        tx,
        &SqliteLibraryRepository,
        &super::workflow_repository::SqliteWorkflowRepository,
        request_id,
        id,
        revision,
        metadata,
    )?;
    serde_json::to_value(value).map_err(|_| AppError::new(ErrorCode::IntegrityFailure))
}
fn archive(
    tx: &Transaction<'_>,
    request_id: &str,
    id: &str,
    revision: i64,
    archive: bool,
) -> Result<PaperDto, AppError> {
    if archive {
        crate::application::library::archive_paper_in_tx(
            tx,
            &SqliteLibraryRepository,
            request_id,
            id,
            revision,
        )
    } else {
        crate::application::library::restore_paper_in_tx(
            tx,
            &SqliteLibraryRepository,
            request_id,
            id,
            revision,
        )
    }
}
fn encode_cursor(key: &str) -> String {
    key.as_bytes().iter().map(|b| format!("{b:02x}")).collect()
}
fn decode_cursor(cursor: &str) -> Result<String, AppError> {
    if cursor.is_empty()
        || !cursor.len().is_multiple_of(2)
        || !cursor.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err(AppError::new(ErrorCode::InvalidInput));
    }
    let bytes = (0..cursor.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&cursor[i..i + 2], 16)
                .map_err(|_| AppError::new(ErrorCode::InvalidInput))
        })
        .collect::<Result<Vec<_>, _>>()?;
    String::from_utf8(bytes).map_err(|_| AppError::new(ErrorCode::InvalidInput))
}
fn list_papers(c: &Connection, filter: PaperFilterDto) -> Result<PageDto<PaperDto>, AppError> {
    if filter.query.chars().count() > 1000
        || filter.limit < 1
        || filter.limit > 500
        || filter
            .year_from
            .is_some_and(|n| !(1000..=9999).contains(&n))
        || filter.year_to.is_some_and(|n| !(1000..=9999).contains(&n))
        || filter
            .year_from
            .zip(filter.year_to)
            .is_some_and(|(a, b)| a > b)
    {
        return Err(AppError::new(ErrorCode::InvalidInput));
    }
    let mut sql =
        String::from("SELECT p.id FROM papers p LEFT JOIN venues v ON v.id=p.venue_id WHERE 1=1");
    let mut vals: Vec<Value> = Vec::new();
    match filter.lifecycle {
        PaperFilterDtoLifecycle::ACTIVE => {
            sql.push_str(" AND p.lifecycle IN('NEW','ACTIVE','COMPLETED')")
        }
        PaperFilterDtoLifecycle::ARCHIVED => sql.push_str(" AND p.lifecycle='ARCHIVED'"),
        PaperFilterDtoLifecycle::ALL => sql.push_str(" AND p.lifecycle!='TRASHED'"),
    }
    if !filter.query.is_empty() {
        vals.push(Value::Text(format!("%{}%", filter.query.to_lowercase())));
        let i = vals.len();
        sql.push_str(&format!(" AND (lower(p.title) LIKE ?{i} ESCAPE '\\' OR lower(COALESCE(v.name,'')) LIKE ?{i} ESCAPE '\\' OR EXISTS(SELECT 1 FROM paper_authors pa JOIN authors a ON a.id=pa.author_id WHERE pa.paper_id=p.id AND lower(a.display_name) LIKE ?{i} ESCAPE '\\'))"));
    }
    if let Some(year) = filter.year_from {
        vals.push(Value::Integer(year));
        sql.push_str(&format!(" AND p.year>=?{}", vals.len()));
    }
    if let Some(year) = filter.year_to {
        vals.push(Value::Integer(year));
        sql.push_str(&format!(" AND p.year<=?{}", vals.len()));
    }
    if let Some(domain) = filter.domain {
        vals.push(Value::Text(domain.trim().to_owned()));
        sql.push_str(&format!(" AND p.domain=?{}", vals.len()));
    }
    if let Some(phase) = filter.phase {
        let phase = phase_filter_value(phase);
        vals.push(Value::Text(phase.to_owned()));
        sql.push_str(&format!(" AND p.current_phase=?{}", vals.len()));
    }
    if !filter.review_types.is_empty() {
        let start = vals.len() + 1;
        for r in filter.review_types {
            vals.push(Value::Text(review_type_value(&r).to_owned()));
        }
        sql.push_str(" AND p.review_type IN(");
        for i in start..=vals.len() {
            if i > start {
                sql.push(',')
            }
            sql.push_str(&format!("?{i}"));
        }
        sql.push(')');
    }
    if let Some(cursor) = filter.cursor {
        let key = decode_cursor(&cursor)?;
        let Some((updated, id)) = key.split_once('|') else {
            return Err(AppError::new(ErrorCode::InvalidInput));
        };
        vals.push(Value::Text(updated.to_owned()));
        vals.push(Value::Text(id.to_owned()));
        sql.push_str(&format!(
            " AND (p.updated_at < ?{} OR (p.updated_at=?{} AND p.id>?{}))",
            vals.len() - 1,
            vals.len() - 1,
            vals.len()
        ));
    }
    sql.push_str(" ORDER BY p.updated_at DESC,p.id ASC LIMIT ?");
    vals.push(Value::Integer(filter.limit + 1));
    let mut stmt = c.prepare(&sql)?;
    let ids = stmt
        .query_map(params_from_iter(vals), |r| r.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    drop(stmt);
    let more = ids.len() as i64 > filter.limit;
    let ids: Vec<_> = ids.into_iter().take(filter.limit as usize).collect();
    let items = ids
        .iter()
        .map(|id| paper(c, id))
        .collect::<Result<Vec<_>, _>>()?;
    let next_cursor = if more {
        items
            .last()
            .map(|item| encode_cursor(&format!("{}|{}", item.updated_at, item.id.0)))
    } else {
        None
    };
    Ok(PageDto {
        items,
        next_cursor,
        total: None,
    })
}
