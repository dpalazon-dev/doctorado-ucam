//! Transactional Library use cases shared by persistence callers.
//!
//! These functions apply lifecycle and revision rules inside a transaction borrowed from the
//! caller. They never open or commit a transaction themselves.

use crate::{
    application::library_ports::{
        DuplicateCandidate, ImportReservation, LibraryRepository, NewPaperDocument, PreparedImport,
        ResourceInspection,
    },
    application::workflow::{initialize_processing, invalidate_effective_change},
    application::workflow_ports::WorkflowRepository,
    domain::library::normalize_metadata,
    transport::{
        dto::{
            DuplicateCandidateDto, DuplicateCandidateDtoReasonsItem, PaperDto, PaperMetadataInput,
            UUID,
        },
        error::{AppError, ErrorCode},
    },
};
use rusqlite::Transaction;
use serde_json::Value;

pub fn record_open_activity_in_tx(
    tx: &Transaction<'_>,
    repository: &dyn LibraryRepository,
    request_id: &str,
    paper_id: &str,
    now: &str,
) -> Result<PaperDto, AppError> {
    let before = repository.paper(tx, paper_id)?;
    if repository.set_last_opened_at(tx, paper_id, now)? != 1 {
        return Err(AppError::new(ErrorCode::NotFound));
    }
    repository.audit_change(
        tx,
        request_id,
        "reader.paper_opened",
        paper_id,
        &serde_json::json!({"documentId": before.document_id}),
        now,
    )?;
    repository.paper(tx, paper_id)
}

fn duplicate_decision_required(candidates: Vec<DuplicateCandidate>) -> AppError {
    let candidates = candidates
        .into_iter()
        .map(|candidate| {
            let mut reasons = Vec::new();
            if candidate.doi_match {
                reasons.push(DuplicateCandidateDtoReasonsItem::Doi);
            }
            if candidate.sha256_match {
                reasons.push(DuplicateCandidateDtoReasonsItem::Sha256);
            }
            DuplicateCandidateDto {
                paper_id: UUID(candidate.paper_id),
                title: candidate.title,
                reasons,
            }
        })
        .collect::<Vec<_>>();
    AppError {
        code: ErrorCode::DuplicateDecisionRequired,
        details: Some(
            [(
                "candidates".into(),
                serde_json::to_value(candidates).unwrap_or_else(|_| serde_json::json!([])),
            )]
            .into(),
        ),
    }
}

pub fn canonical_hash(value: &Value) -> Result<String, AppError> {
    crate::domain::workflow::canonical_hash(value)
}

/// Returns true only for an intact, non-reuse confirmation whose filesystem promotion was
/// durably recorded before cancellation. File presence or a matching hash never grants authority.
pub fn has_durable_promoted_confirmation(
    record: &crate::application::library_ports::ImportRecord,
) -> bool {
    let Some(request_id) = record.request_id.as_deref() else {
        return false;
    };
    let Some(raw) = record.metadata_json.as_deref() else {
        return false;
    };
    let Ok(intent) = serde_json::from_str::<Value>(raw) else {
        return false;
    };
    let cancelled_after_promotion = record.state == "PROMOTED"
        || (record.state == "FAILED"
            && valid_pending_cancellation(record)
                .is_some_and(|(_, promotion_confirmed)| promotion_confirmed));
    if !cancelled_after_promotion
        || record.result_json.is_some()
        || !canonical_uuid(&record.library_id)
        || !canonical_uuid(request_id)
        || !canonical_uuid(&record.token)
        || record
            .reserved_paper_id
            .as_deref()
            .is_none_or(|id| !canonical_uuid(id))
        || record
            .reserved_document_id
            .as_deref()
            .is_none_or(|id| !canonical_uuid(id))
        || record.destination_path.as_deref()
            != record
                .reserved_document_id
                .as_deref()
                .map(|id| format!("documents/{id}/original.pdf"))
                .as_deref()
        || record.sha256.as_deref().is_none_or(|hash| {
            hash.len() != 64 || !hash.bytes().all(|byte| byte.is_ascii_hexdigit())
        })
        || record
            .size_bytes
            .is_none_or(|size| size <= 0 || size > 524_288_000)
        || intent["version"] != 1
        || intent["kind"] != "confirmation"
        || intent["duplicateResolution"] != Value::Null
        || intent["receipt"]["command"] != "library_confirm_import"
        || canonical_hash(&intent["receipt"]["payload"])
            .ok()
            .as_deref()
            != record.payload_hash.as_deref()
        || intent["receipt"]["payload"]["importToken"] != record.token
    {
        return false;
    }
    let Ok(original) = serde_json::from_value::<PaperMetadataInput>(
        intent["receipt"]["payload"]["metadata"].clone(),
    ) else {
        return false;
    };
    let Ok(normalized) = normalize_metadata(original) else {
        return false;
    };
    intent["normalizedMetadata"] == serde_json::json!(normalized)
}

/// Parses only the known, fully bound v1 PENDING cancellation envelope.
pub fn valid_pending_cancellation(
    record: &crate::application::library_ports::ImportRecord,
) -> Option<(String, bool)> {
    if record.state != "FAILED" {
        return None;
    }
    let intent: Value = serde_json::from_str(record.error_json.as_deref()?).ok()?;
    let required_keys = ["cleanup", "code", "kind", "receipt", "requestId", "version"];
    let mut actual_keys = intent
        .as_object()?
        .keys()
        .map(String::as_str)
        .collect::<Vec<_>>();
    actual_keys.sort_unstable();
    let mut expected_keys = required_keys.to_vec();
    if intent.get("promotionConfirmed").is_some() {
        expected_keys.push("promotionConfirmed");
        expected_keys.sort_unstable();
    }
    if actual_keys != expected_keys
        || intent["version"] != 1
        || intent["kind"] != "cancellation"
        || intent["code"] != "OperationCancelled"
        || intent["cleanup"] != "PENDING"
        || intent["receipt"]["command"] != "library_cancel_import"
        || intent["receipt"]["payload"] != serde_json::json!({"importToken":record.token})
    {
        return None;
    }
    let receipt = intent["receipt"].as_object()?;
    if receipt.len() != 2 || !receipt.contains_key("command") || !receipt.contains_key("payload") {
        return None;
    }
    let payload = intent["receipt"]["payload"].as_object()?;
    if payload.len() != 1 || !payload.contains_key("importToken") {
        return None;
    }
    let request_id = intent["requestId"].as_str()?.to_owned();
    if !canonical_uuid(&request_id) {
        return None;
    }
    let promotion_confirmed = match intent.get("promotionConfirmed") {
        None => false,
        Some(value) => value.as_bool()?,
    };
    Some((request_id, promotion_confirmed))
}

fn canonical_uuid(value: &str) -> bool {
    uuid::Uuid::parse_str(value).is_ok_and(|parsed| parsed.to_string() == value)
}

pub fn prepare_import_in_tx(
    tx: &Transaction<'_>,
    repository: &dyn LibraryRepository,
    request_id: &str,
    token: &str,
    original: &PaperMetadataInput,
    resolution: Option<&crate::transport::dto::DuplicateResolution>,
) -> Result<PreparedImport, AppError> {
    let normalized = normalize_metadata(original.clone())?;
    let mut record = repository.import_record(tx, token)?;
    if record.library_id.is_empty()
        || !matches!(record.state.as_str(), "STAGING" | "PROMOTED" | "COMMITTED")
    {
        return Err(AppError::new(ErrorCode::Conflict));
    }
    let token_uuid = crate::transport::dto::UUID(record.token.clone());
    let payload = {
        let mut object = serde_json::Map::new();
        object.insert("importToken".into(), serde_json::json!(token_uuid));
        object.insert("metadata".into(), serde_json::json!(original));
        if let Some(resolution) = resolution {
            object.insert("duplicateResolution".into(), serde_json::json!(resolution));
        }
        Value::Object(object)
    };
    let payload_hash = canonical_hash(&payload)?;
    if let Some(raw) = record.metadata_json.as_deref() {
        let intent: Value =
            serde_json::from_str(raw).map_err(|_| AppError::new(ErrorCode::IntegrityFailure))?;
        if intent["kind"] == "confirmation" {
            if record.payload_hash.as_deref() != Some(payload_hash.as_str()) {
                return Err(AppError::new(ErrorCode::Conflict));
            }
            if intent["normalizedMetadata"] != serde_json::json!(normalized) {
                return Err(AppError::new(ErrorCode::IntegrityFailure));
            }
            let paper_id = record
                .reserved_paper_id
                .clone()
                .ok_or_else(|| AppError::new(ErrorCode::IntegrityFailure))?;
            let document_id = record
                .reserved_document_id
                .clone()
                .ok_or_else(|| AppError::new(ErrorCode::IntegrityFailure))?;
            let destination_path = record.destination_path.clone().unwrap_or_default();
            let reuse_paper = if resolution.is_some() && destination_path.is_empty() {
                Some(repository.paper(tx, &paper_id)?)
            } else {
                None
            };
            return Ok(PreparedImport {
                record,
                metadata: normalized,
                duplicate_resolution: resolution.cloned(),
                paper_id,
                document_id,
                destination_path,
                confirmation_payload: payload,
                reuse_paper,
            });
        }
    }
    if record.state == "COMMITTED" {
        if record.payload_hash.as_deref() != Some(payload_hash.as_str()) {
            return Err(AppError::new(ErrorCode::Conflict));
        }
        let paper: PaperDto = serde_json::from_str(
            record
                .result_json
                .as_deref()
                .ok_or_else(|| AppError::new(ErrorCode::IntegrityFailure))?,
        )
        .map_err(|_| AppError::new(ErrorCode::IntegrityFailure))?;
        return Ok(PreparedImport {
            record,
            metadata: normalized,
            duplicate_resolution: resolution.cloned(),
            paper_id: paper.id.0.clone(),
            document_id: paper.document_id.0.clone(),
            destination_path: String::new(),
            confirmation_payload: payload,
            reuse_paper: Some(paper),
        });
    }
    if record.expires_at <= chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
    {
        return Err(AppError::new(ErrorCode::OperationCancelled));
    }
    let hash = record
        .sha256
        .as_deref()
        .ok_or_else(|| AppError::new(ErrorCode::ImportRecoveryRequired))?;
    if record.size_bytes.is_none() {
        return Err(AppError::new(ErrorCode::ImportRecoveryRequired));
    }
    let candidates = repository.duplicate_candidates(tx, normalized.doi.as_deref(), hash)?;
    if !candidates.is_empty() {
        let Some(decision) = resolution else {
            return Err(duplicate_decision_required(candidates));
        };
        if serde_json::to_value(&decision.action)
            .ok()
            .and_then(|value| value.as_str().map(str::to_owned))
            .as_deref()
            != Some("reuseExisting")
            || !candidates
                .iter()
                .any(|candidate| candidate.paper_id == decision.paper_id.0)
        {
            return Err(AppError::new(ErrorCode::InvalidInput));
        }
        let existing = repository.paper(tx, &decision.paper_id.0)?;
        let wrapper = serde_json::json!({"version":1,"kind":"confirmation","normalizedMetadata":normalized,"receipt":{"command":"library_confirm_import","payload":payload},"duplicateResolution":resolution});
        repository.reserve_import(
            tx,
            ImportReservation {
                operation_id: &record.operation_id,
                request_id,
                metadata_json: &wrapper,
                payload_hash: &payload_hash,
                paper_id: &existing.id.0,
                document_id: &existing.document_id.0,
                destination_path: None,
            },
        )?;
        record = repository.import_record(tx, token)?;
        repository.audit_change(
            tx,
            request_id,
            "import.confirmation_prepared",
            &record.operation_id,
            &serde_json::json!({"paperId":existing.id,"documentId":existing.document_id,"reuse":true}),
            &chrono::Utc::now().to_rfc3339(),
        )?;
        return Ok(PreparedImport {
            record,
            metadata: normalized,
            duplicate_resolution: resolution.cloned(),
            paper_id: existing.id.0.clone(),
            document_id: existing.document_id.0.clone(),
            destination_path: String::new(),
            confirmation_payload: payload,
            reuse_paper: Some(existing),
        });
    }
    let paper_id = uuid::Uuid::new_v4().to_string();
    let document_id = uuid::Uuid::new_v4().to_string();
    let destination_path = format!("documents/{document_id}/original.pdf");
    let wrapper = serde_json::json!({"version":1,"kind":"confirmation","normalizedMetadata":normalized,"receipt":{"command":"library_confirm_import","payload":payload},"duplicateResolution":resolution});
    repository.reserve_import(
        tx,
        ImportReservation {
            operation_id: &record.operation_id,
            request_id,
            metadata_json: &wrapper,
            payload_hash: &payload_hash,
            paper_id: &paper_id,
            document_id: &document_id,
            destination_path: Some(&destination_path),
        },
    )?;
    record = repository.import_record(tx, token)?;
    repository.audit_change(
        tx,
        request_id,
        "import.confirmation_prepared",
        &record.operation_id,
        &serde_json::json!({"paperId":paper_id,"documentId":document_id,"reuse":false}),
        &chrono::Utc::now().to_rfc3339(),
    )?;
    Ok(PreparedImport {
        record,
        metadata: normalized,
        duplicate_resolution: resolution.cloned(),
        paper_id,
        document_id,
        destination_path,
        confirmation_payload: payload,
        reuse_paper: None,
    })
}

pub fn update_metadata_in_tx(
    tx: &Transaction<'_>,
    repository: &dyn LibraryRepository,
    workflow: &dyn WorkflowRepository,
    request_id: &str,
    paper_id: &str,
    expected_revision: i64,
    metadata: &PaperMetadataInput,
) -> Result<PaperDto, AppError> {
    let metadata = normalize_metadata(metadata.clone())?;
    let before = repository.paper(tx, paper_id)?;
    let Some(current_revision) = repository.current_revision(tx, paper_id)? else {
        return Err(AppError::new(ErrorCode::NotFound));
    };
    if current_revision != expected_revision {
        return Err(AppError::conflict_revision(current_revision));
    }
    let venue_id = repository.resolve_venue(tx, metadata.venue.as_deref())?;
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    let changed = repository.update_paper_metadata(
        tx,
        paper_id,
        expected_revision,
        &metadata,
        venue_id.as_deref(),
        &now,
    )?;
    if changed != 1 {
        let current = repository
            .current_revision(tx, paper_id)?
            .ok_or_else(|| AppError::new(ErrorCode::NotFound))?;
        return Err(AppError::conflict_revision(current));
    }
    repository.replace_paper_authors(tx, paper_id, &metadata.authors)?;
    if before.title != metadata.title || before.review_type != metadata.review_type {
        invalidate_effective_change(
            tx,
            workflow,
            paper_id,
            &[crate::transport::dto::PhaseCode::PRE],
        )?;
    }
    repository.audit_change(
        tx,
        request_id,
        "paper.metadata.updated",
        paper_id,
        &serde_json::json!({"fields":["title","authors","year","doi","venue","reviewType","domain"]}),
        &now,
    )?;
    repository.paper(tx, paper_id)
}

pub fn archive_paper_in_tx(
    tx: &Transaction<'_>,
    repository: &dyn LibraryRepository,
    request_id: &str,
    paper_id: &str,
    expected_revision: i64,
) -> Result<PaperDto, AppError> {
    set_archived_in_tx(
        tx,
        repository,
        request_id,
        paper_id,
        expected_revision,
        true,
    )
}

pub fn restore_paper_in_tx(
    tx: &Transaction<'_>,
    repository: &dyn LibraryRepository,
    request_id: &str,
    paper_id: &str,
    expected_revision: i64,
) -> Result<PaperDto, AppError> {
    set_archived_in_tx(
        tx,
        repository,
        request_id,
        paper_id,
        expected_revision,
        false,
    )
}

/// Applies a previously authorized confirmation under the caller's transaction. `verified`
/// must come from the document-store promotion for a newly created document.
pub fn confirm_in_tx(
    tx: &Transaction<'_>,
    repository: &dyn LibraryRepository,
    workflow: &dyn WorkflowRepository,
    prepared: &PreparedImport,
    verified: &ResourceInspection,
) -> Result<PaperDto, AppError> {
    let normalized_metadata = normalize_metadata(prepared.metadata.clone())?;
    let current = repository.import_record(tx, &prepared.record.token)?;
    if current.state == "COMMITTED" {
        return serde_json::from_str(
            current
                .result_json
                .as_deref()
                .ok_or_else(|| AppError::new(ErrorCode::IntegrityFailure))?,
        )
        .map_err(|_| AppError::new(ErrorCode::IntegrityFailure));
    }
    if let Some(existing) = &prepared.reuse_paper {
        let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        repository.mark_import_committed(tx, &current.operation_id, existing, &now)?;
        let request_id = current
            .request_id
            .as_deref()
            .ok_or_else(|| AppError::new(ErrorCode::IntegrityFailure))?;
        repository.audit_change(
            tx,
            request_id,
            "paper.import_reused",
            &existing.id.0,
            &serde_json::json!({"operationId":current.operation_id}),
            &now,
        )?;
        return Ok(existing.clone());
    }
    if current.state != "PROMOTED"
        || current.destination_path.as_deref() != Some(prepared.destination_path.as_str())
    {
        return Err(AppError::new(ErrorCode::ImportRecoveryRequired));
    }
    if verified.ambiguous || !verified.destination_matches {
        return Err(AppError::new(ErrorCode::ImportRecoveryRequired));
    }
    let hash = current
        .sha256
        .as_deref()
        .ok_or_else(|| AppError::new(ErrorCode::IntegrityFailure))?;
    let size = current
        .size_bytes
        .ok_or_else(|| AppError::new(ErrorCode::IntegrityFailure))?;
    let candidates =
        repository.duplicate_candidates(tx, normalized_metadata.doi.as_deref(), hash)?;
    if !candidates.is_empty() {
        return Err(duplicate_decision_required(candidates));
    }
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    repository.insert_paper_document(
        tx,
        NewPaperDocument {
            paper_id: &prepared.paper_id,
            doc_id: &prepared.document_id,
            metadata: &normalized_metadata,
            sha: hash,
            size,
            filename: &current.original_filename,
            relative: &prepared.destination_path,
            now: &now,
        },
    )?;
    initialize_processing(tx, workflow, &prepared.paper_id)?;
    let created = repository.paper(tx, &prepared.paper_id)?;
    repository.mark_import_committed(tx, &current.operation_id, &created, &now)?;
    let request_id = current
        .request_id
        .as_deref()
        .ok_or_else(|| AppError::new(ErrorCode::IntegrityFailure))?;
    repository.audit_change(
        tx,
        request_id,
        "paper.imported",
        &prepared.paper_id,
        &serde_json::json!({"documentId":prepared.document_id,"operationId":current.operation_id}),
        &now,
    )?;
    Ok(created)
}

fn set_archived_in_tx(
    tx: &Transaction<'_>,
    repository: &dyn LibraryRepository,
    request_id: &str,
    paper_id: &str,
    expected_revision: i64,
    archive: bool,
) -> Result<PaperDto, AppError> {
    let Some(current_revision) = repository.current_revision(tx, paper_id)? else {
        return Err(AppError::new(ErrorCode::NotFound));
    };
    if current_revision != expected_revision {
        return Err(AppError::conflict_revision(current_revision));
    }
    let (current, previous) = repository
        .lifecycle_state(tx, paper_id)?
        .ok_or_else(|| AppError::new(ErrorCode::NotFound))?;
    if current == "TRASHED"
        || expected_revision < 0
        || (archive && current == "ARCHIVED")
        || (!archive && current != "ARCHIVED")
    {
        return Err(AppError::new(ErrorCode::Conflict));
    }
    let (lifecycle, archived_from) = if archive {
        ("ARCHIVED", Some(current.as_str()))
    } else {
        (previous.as_deref().unwrap_or("NEW"), None)
    };
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    let changed = repository.update_lifecycle(
        tx,
        paper_id,
        expected_revision,
        lifecycle,
        archived_from,
        &now,
    )?;
    if changed != 1 {
        let current = repository
            .current_revision(tx, paper_id)?
            .ok_or_else(|| AppError::new(ErrorCode::NotFound))?;
        return Err(AppError::conflict_revision(current));
    }
    repository.audit_change(
        tx,
        request_id,
        if archive {
            "paper.archived"
        } else {
            "paper.restored"
        },
        paper_id,
        &serde_json::json!({"lifecycle":lifecycle,"archivedFrom":archived_from}),
        &now,
    )?;
    repository.paper(tx, paper_id)
}
