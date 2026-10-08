use crate::transport::error::AppError;
pub fn with_transaction<T>(
    connection: &mut rusqlite::Connection,
    action: impl FnOnce(&rusqlite::Transaction<'_>) -> Result<T, AppError>,
) -> Result<T, AppError> {
    let tx = connection.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    let result = action(&tx)?;
    tx.commit()?;
    Ok(result)
}
