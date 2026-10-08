use crate::{
    adapters::sqlite::{
        actor::DbActor,
        receipts::{canonical_hash, with_receipt},
    },
    application::{
        library_ports::LibraryRepository,
        reader::{open_paper_in_tx, reading_position_in_tx, save_reading_position_in_tx},
        reader_ports::*,
    },
    transport::{
        dto::*,
        error::{AppError, ErrorCode},
    },
};
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde_json::json;

#[derive(Clone)]
pub struct SqliteReaderPersistence {
    actor: DbActor,
    library_id: UUID,
}
impl SqliteReaderPersistence {
    pub fn new(actor: DbActor, library_id: String) -> Self {
        Self {
            actor,
            library_id: UUID(library_id),
        }
    }
}
pub struct SqliteReaderRepository;

impl ReaderRepository for SqliteReaderRepository {
    fn registered_document(
        &self,
        tx: &Transaction<'_>,
        id: &str,
    ) -> Result<RegisteredDocument, AppError> {
        let row: (String,String,String,String,String,String,i64,String,String) = tx.query_row(
            "SELECT d.id,d.paper_id,d.original_filename,d.sha256,d.imported_at,d.status,d.size_bytes,d.relative_path,li.library_id FROM documents d JOIN papers p ON p.id=d.paper_id JOIN library_identity li ON li.singleton=1 WHERE d.id=?1 AND d.status='ACTIVE' AND p.active_document_id=d.id",
            [id], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?,r.get(7)?,r.get(8)?)))
            .optional()?.ok_or_else(|| AppError::new(ErrorCode::NotFound))?;
        let expected_path = format!("documents/{}/original.pdf", row.0);
        if row.7 != expected_path || row.8 != self_library_id(tx)? {
            return Err(AppError::new(ErrorCode::PathNotAllowed));
        }
        Ok(RegisteredDocument {
            library_id: UUID(row.8),
            document: DocumentDto {
                id: UUID(row.0),
                paper_id: UUID(row.1),
                original_filename: row.2,
                sha256: row.3,
                imported_at: row.4,
                status: serde_json::from_value(json!(row.5))
                    .map_err(|_| AppError::new(ErrorCode::IntegrityFailure))?,
            },
            relative_path: row.7,
            size_bytes: row.6,
        })
    }
    fn position(
        &self,
        tx: &Transaction<'_>,
        id: &str,
    ) -> Result<Option<ReadingPositionDto>, AppError> {
        Ok(tx.query_row("SELECT document_id,page_index,zoom,revision,updated_at FROM reading_positions WHERE document_id=?1",[id],|r|Ok(ReadingPositionDto{document_id:UUID(r.get(0)?),page_index:r.get(1)?,zoom:r.get(2)?,revision:r.get(3)?,updated_at:r.get(4)?})).optional()?)
    }
    fn write_position(
        &self,
        tx: &Transaction<'_>,
        expected: i64,
        next: &ReadingPositionDto,
    ) -> Result<usize, AppError> {
        if expected == 0 {
            Ok(tx.execute("INSERT INTO reading_positions(document_id,page_index,zoom,revision,updated_at) VALUES(?1,?2,?3,?4,?5) ON CONFLICT(document_id) DO UPDATE SET page_index=excluded.page_index,zoom=excluded.zoom,revision=excluded.revision,updated_at=excluded.updated_at WHERE reading_positions.revision=0",params![next.document_id.0,next.page_index,next.zoom,next.revision,next.updated_at])?)
        } else {
            Ok(tx.execute("UPDATE reading_positions SET page_index=?1,zoom=?2,revision=?3,updated_at=?4 WHERE document_id=?5 AND revision=?6",params![next.page_index,next.zoom,next.revision,next.updated_at,next.document_id.0,expected])?)
        }
    }
    fn last_opened_paper_id(&self, tx: &Transaction<'_>) -> Result<Option<UUID>, AppError> {
        Ok(tx
            .query_row(
                "SELECT last_opened_paper_id FROM app_session WHERE singleton=1",
                [],
                |r| r.get::<_, Option<String>>(0),
            )
            .optional()?
            .flatten()
            .map(UUID))
    }
    fn set_last_opened_paper(&self, tx: &Transaction<'_>, paper_id: &str) -> Result<(), AppError> {
        tx.execute("INSERT INTO app_session(singleton,last_opened_paper_id) VALUES(1,?1) ON CONFLICT(singleton) DO UPDATE SET last_opened_paper_id=excluded.last_opened_paper_id",[paper_id])?;
        Ok(())
    }
    fn audit_position(
        &self,
        tx: &Transaction<'_>,
        request_id: &str,
        position: &ReadingPositionDto,
    ) -> Result<(), AppError> {
        tx.execute("INSERT INTO audit_events(id,request_id,action,entity_id,changes_json,created_at) VALUES(?1,?2,'reader.position_saved',?3,?4,?5)",params![uuid::Uuid::new_v4().to_string(),request_id,position.document_id.0,json!({"pageIndex":position.page_index,"zoom":position.zoom,"revision":position.revision}).to_string(),position.updated_at])?;
        Ok(())
    }
}
fn self_library_id(tx: &Transaction<'_>) -> Result<String, AppError> {
    Ok(tx.query_row(
        "SELECT library_id FROM library_identity WHERE singleton=1",
        [],
        |r| r.get(0),
    )?)
}

fn current_document(
    connection: &Connection,
    paper_id: &str,
) -> Result<RegisteredDocument, AppError> {
    let id: String = connection
        .query_row(
            "SELECT active_document_id FROM papers WHERE id=?1",
            [paper_id],
            |r| r.get(0),
        )
        .optional()?
        .flatten()
        .ok_or_else(|| AppError::new(ErrorCode::NotFound))?;
    let tx = connection.unchecked_transaction()?;
    let result = SqliteReaderRepository.registered_document(&tx, &id);
    tx.rollback()?;
    result
}
impl ReaderPersistence for SqliteReaderPersistence {
    fn writable(&self) -> bool {
        self.actor.info().writable
    }
    fn prepare_open(&self, command: OpenPaperCommand) -> ReaderFuture<'static, RegisteredDocument> {
        let actor = self.actor.clone();
        Box::pin(async move {
            actor.submit(move |c| {
            let payload=json!({"paperId":command.paper_id});
            if let Some((prior,hash,result))=c.query_row("SELECT command,payload_hash,result_json FROM operation_receipts WHERE request_id=?1",[&command.request_id.0],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?))).optional()? {
                if prior!="reader_open_paper"||hash!=canonical_hash(&payload)? {return Err(AppError::new(ErrorCode::Conflict));}
                let opened:OpenPaperDto=serde_json::from_str(&result).map_err(|_|AppError::new(ErrorCode::IntegrityFailure))?;
                if opened.paper.id != command.paper_id {return Err(AppError::new(ErrorCode::PathNotAllowed));}
                let doc=current_document(c,&command.paper_id.0)?;
                if doc.document.id != opened.document.id || doc.document.paper_id != command.paper_id {return Err(AppError::new(ErrorCode::PathNotAllowed));}
                return Ok(doc)
            }
            current_document(c,&command.paper_id.0)
        }).await
        })
    }
    fn confirm_open(
        &self,
        command: OpenPaperCommand,
        verified: VerifiedDocument,
    ) -> ReaderFuture<'static, OpenPaperDto> {
        let actor = self.actor.clone();
        let library_id = self.library_id.clone();
        Box::pin(async move {
            actor.submit(move |c| {
            let payload=json!({"paperId":command.paper_id});
            let opened:OpenPaperDto=with_receipt(c,&command.request_id.0,"reader_open_paper",&payload,|tx| {
                let doc=verified.registered().clone();
                serde_json::to_value(open_paper_in_tx(tx,&SqliteReaderRepository,&crate::adapters::sqlite::library_repository::SqliteLibraryRepository,&command,&doc,&chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis,true))?).map_err(|_|AppError::new(ErrorCode::IntegrityFailure))
            }).and_then(|value|serde_json::from_value(value).map_err(|_|AppError::new(ErrorCode::IntegrityFailure)))?;
            let tx=c.unchecked_transaction()?;
            if opened.paper.id != command.paper_id || opened.document.paper_id != command.paper_id {
                tx.rollback()?;
                return Err(AppError::new(ErrorCode::PathNotAllowed));
            }
            let current=SqliteReaderRepository.registered_document(&tx,&opened.document.id.0)?;
            let paper=LibraryRepository::paper(&crate::adapters::sqlite::library_repository::SqliteLibraryRepository,&tx,&command.paper_id.0)?;
            tx.rollback()?;
            if paper.document_id!=opened.document.id || current.document.paper_id!=command.paper_id || current.library_id!=library_id || current!=*verified.registered() {return Err(AppError::new(ErrorCode::PathNotAllowed));}
            Ok(opened)
        }).await
        })
    }
    fn last_opened_paper(&self) -> ReaderFuture<'static, Option<PaperDto>> {
        let actor = self.actor.clone();
        Box::pin(async move {
            actor.submit(move |c| { let tx=c.unchecked_transaction()?; let id=SqliteReaderRepository.last_opened_paper_id(&tx)?; let result=id.map(|id|LibraryRepository::paper(&crate::adapters::sqlite::library_repository::SqliteLibraryRepository,&tx,&id.0)).transpose()?; tx.rollback()?; Ok(result) }).await
        })
    }
    fn registered_document(&self, document_id: UUID) -> ReaderFuture<'static, RegisteredDocument> {
        let actor = self.actor.clone();
        Box::pin(async move {
            actor
                .submit(move |c| {
                    let tx = c.unchecked_transaction()?;
                    let result = SqliteReaderRepository.registered_document(&tx, &document_id.0);
                    tx.rollback()?;
                    result
                })
                .await
        })
    }
    fn reading_position(&self, document_id: UUID) -> ReaderFuture<'static, ReadingPositionDto> {
        let actor = self.actor.clone();
        Box::pin(async move {
            actor
                .submit(move |c| {
                    let tx = c.unchecked_transaction()?;
                    let result = reading_position_in_tx(&tx, &SqliteReaderRepository, document_id);
                    tx.rollback()?;
                    result
                })
                .await
        })
    }
    fn save_reading_position(
        &self,
        command: SaveReadingPositionCommand,
    ) -> ReaderFuture<'static, ReadingPositionDto> {
        let actor = self.actor.clone();
        Box::pin(async move {
            actor.submit(move |c| { let payload=json!({"documentId":command.document_id,"expectedRevision":command.expected_revision,"pageIndex":command.page_index,"zoom":command.zoom}); let value=with_receipt(c,&command.request_id.0,"reader_save_reading_position",&payload,|tx| { let result=save_reading_position_in_tx(tx,&SqliteReaderRepository,&command,&chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis,true))?; serde_json::to_value(result).map_err(|_|AppError::new(ErrorCode::IntegrityFailure)) })?; serde_json::from_value(value).map_err(|_|AppError::new(ErrorCode::IntegrityFailure)) }).await
        })
    }
}
