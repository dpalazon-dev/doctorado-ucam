pub use crate::application::library::canonical_hash;
use crate::{
    application::unit_of_work::with_transaction,
    transport::error::{AppError, ErrorCode},
};
use rusqlite::{Connection, OptionalExtension, Transaction};
use serde::de::DeserializeOwned;
use serde_json::Value;
pub(crate) fn lookup_receipt<T: DeserializeOwned>(
    c: &Connection,
    request_id: &str,
    command: &str,
    payload: &Value,
) -> Result<Option<T>, AppError> {
    let id =
        uuid::Uuid::parse_str(request_id).map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
    if id.to_string() != request_id {
        return Err(AppError::new(ErrorCode::InvalidInput));
    }
    let hash = canonical_hash(payload)?;
    let previous: Option<(String, String, String)> = c
        .query_row(
            "SELECT command,payload_hash,result_json FROM operation_receipts WHERE request_id=?1",
            [request_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?;
    let Some((prior_command, prior_hash, result)) = previous else {
        return Ok(None);
    };
    if prior_command != command || prior_hash != hash {
        return Err(AppError::new(ErrorCode::Conflict));
    }
    serde_json::from_str(&result)
        .map(Some)
        .map_err(|_| AppError::new(ErrorCode::IntegrityFailure))
}

pub fn with_receipt(
    connection: &mut Connection,
    request_id: &str,
    command: &str,
    payload: &Value,
    action: impl FnOnce(&Transaction<'_>) -> Result<Value, AppError>,
) -> Result<Value, AppError> {
    with_transaction(connection, |tx| {
        if let Some(previous) = lookup_receipt::<Value>(tx, request_id, command, payload)? {
            return Ok(previous);
        }
        let result = action(tx)?;
        let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let serialized =
            serde_json::to_string(&result).map_err(|_| AppError::new(ErrorCode::InvalidInput))?;
        let hash = canonical_hash(payload)?;
        tx.execute("INSERT INTO operation_receipts(request_id,command,payload_hash,result_json,created_at) VALUES(?1,?2,?3,?4,?5)",rusqlite::params![request_id,command,hash,serialized,now])?;
        tx.execute("INSERT INTO audit_events(id,request_id,action,entity_id,changes_json,created_at) VALUES(?1,?2,?3,NULL,'{}',?4)",rusqlite::params![uuid::Uuid::new_v4().to_string(),request_id,command,now])?;
        Ok(result)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_distinguishes_absent_replay_conflict_and_corrupt_result_without_writes() {
        let connection = Connection::open_in_memory().unwrap();
        connection.execute_batch("CREATE TABLE operation_receipts(request_id TEXT PRIMARY KEY,command TEXT,payload_hash TEXT,result_json TEXT);").unwrap();
        let request_id = uuid::Uuid::new_v4().to_string();
        let payload = serde_json::json!({"n":1});
        let absent: Option<Value> =
            lookup_receipt(&connection, &request_id, "test", &payload).unwrap();
        assert_eq!(absent, None);
        assert_eq!(
            connection
                .query_row("SELECT count(*) FROM operation_receipts", [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
        let hash = canonical_hash(&payload).unwrap();
        connection
            .execute(
                "INSERT INTO operation_receipts VALUES(?1,'test',?2,'{\"ok\":true}')",
                rusqlite::params![request_id, hash],
            )
            .unwrap();
        let replay: Option<Value> =
            lookup_receipt(&connection, &request_id, "test", &payload).unwrap();
        assert_eq!(replay, Some(serde_json::json!({"ok":true})));
        assert_eq!(
            lookup_receipt::<Value>(&connection, &request_id, "other", &payload)
                .unwrap_err()
                .code,
            ErrorCode::Conflict
        );
        connection
            .execute(
                "UPDATE operation_receipts SET result_json='{' WHERE request_id=?1",
                [&request_id],
            )
            .unwrap();
        assert_eq!(
            lookup_receipt::<Value>(&connection, &request_id, "test", &payload)
                .unwrap_err()
                .code,
            ErrorCode::IntegrityFailure
        );
        assert_eq!(
            connection
                .query_row("SELECT count(*) FROM operation_receipts", [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            1
        );
    }
}
