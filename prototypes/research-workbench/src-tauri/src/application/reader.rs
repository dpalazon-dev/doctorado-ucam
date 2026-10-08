use crate::{
    application::{
        library::record_open_activity_in_tx,
        library_ports::LibraryRepository,
        reader_ports::{OpenPaperCommand, ReaderRepository, SaveReadingPositionCommand},
    },
    transport::{
        dto::{OpenPaperDto, ReadingPositionDto, UUID},
        error::{AppError, ErrorCode},
    },
};
use rusqlite::Transaction;

pub fn open_paper_in_tx(
    tx: &Transaction<'_>,
    reader: &dyn ReaderRepository,
    library: &dyn LibraryRepository,
    command: &OpenPaperCommand,
    verified_document: &crate::application::reader_ports::RegisteredDocument,
    now: &str,
) -> Result<OpenPaperDto, AppError> {
    let current = reader.registered_document(tx, &verified_document.document.id.0)?;
    if current != *verified_document || current.document.paper_id != command.paper_id {
        return Err(AppError::new(ErrorCode::PathNotAllowed));
    }
    let paper = library.paper(tx, &command.paper_id.0)?;
    if paper.document_id != current.document.id {
        return Err(AppError::new(ErrorCode::PathNotAllowed));
    }
    let paper =
        record_open_activity_in_tx(tx, library, &command.request_id.0, &command.paper_id.0, now)?;
    reader.set_last_opened_paper(tx, &command.paper_id.0)?;
    let position = reading_position_in_tx(tx, reader, current.document.id.clone())?;
    Ok(OpenPaperDto {
        paper,
        document: current.document,
        reading_position: position,
        document_url: format!(
            "http://research.localhost/{}",
            verified_document.document.id.0
        ),
    })
}

pub fn reading_position_in_tx(
    tx: &Transaction<'_>,
    reader: &dyn ReaderRepository,
    document_id: UUID,
) -> Result<ReadingPositionDto, AppError> {
    if let Some(position) = reader.position(tx, &document_id.0)? {
        return Ok(position);
    }
    let document = reader.registered_document(tx, &document_id.0)?;
    Ok(ReadingPositionDto {
        document_id,
        page_index: 1,
        zoom: 1.0,
        revision: 0,
        updated_at: document.document.imported_at,
    })
}

pub fn save_reading_position_in_tx(
    tx: &Transaction<'_>,
    reader: &dyn ReaderRepository,
    command: &SaveReadingPositionCommand,
    now: &str,
) -> Result<ReadingPositionDto, AppError> {
    validate_reading_position(command.expected_revision, command.page_index, command.zoom)?;
    let current = reader.position(tx, &command.document_id.0)?;
    let revision = current.as_ref().map_or(0, |position| position.revision);
    if revision != command.expected_revision {
        return Err(AppError::conflict_revision(revision));
    }
    // Check the FK/registered document before writing. Each PDF has an independent row.
    reader.registered_document(tx, &command.document_id.0)?;
    let next_revision = revision
        .checked_add(1)
        .ok_or_else(|| AppError::new(ErrorCode::InvalidInput))?;
    let next = ReadingPositionDto {
        document_id: command.document_id.clone(),
        page_index: command.page_index,
        zoom: command.zoom,
        revision: next_revision,
        updated_at: now.to_owned(),
    };
    if reader.write_position(tx, revision, &next)? != 1 {
        return Err(AppError::conflict_revision(revision));
    }
    reader.audit_position(tx, &command.request_id.0, &next)?;
    Ok(next)
}

fn validate_reading_position(
    expected_revision: i64,
    page_index: i64,
    zoom: f64,
) -> Result<(), AppError> {
    if expected_revision < 0 || page_index < 1 || !zoom.is_finite() || !(0.25..=5.0).contains(&zoom)
    {
        return Err(AppError::new(ErrorCode::InvalidInput));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_reading_position;
    use crate::transport::error::ErrorCode;

    #[test]
    fn actual_position_validation_rejects_invalid_ranges_and_accepts_edges() {
        for zoom in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 0.249, 5.001] {
            assert_eq!(
                validate_reading_position(0, 1, zoom).unwrap_err().code,
                ErrorCode::InvalidInput
            );
        }
        assert_eq!(
            validate_reading_position(-1, 1, 1.0).unwrap_err().code,
            ErrorCode::InvalidInput
        );
        assert_eq!(
            validate_reading_position(0, 0, 1.0).unwrap_err().code,
            ErrorCode::InvalidInput
        );
        assert!(validate_reading_position(0, 1, 0.25).is_ok());
        assert!(validate_reading_position(0, 1, 5.0).is_ok());
    }
}
