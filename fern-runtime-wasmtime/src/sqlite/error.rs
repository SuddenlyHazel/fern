use crate::fern::base::sqlite::DbError;

impl From<turso::Error> for DbError {
    fn from(value: turso::Error) -> Self {
        match value {
            turso::Error::ToSqlConversionFailure(_) => DbError::ToSqlConversionFailure,
            turso::Error::MutexError(e) => DbError::LockError(e),
            turso::Error::SqlExecutionFailure(e) => DbError::SqlExecutionFailure(e),
            turso::Error::WalOperationError(e) => DbError::WalOperationError(e),
            turso::Error::QueryReturnedNoRows => DbError::QueryReturnedNoRows,
            turso::Error::ConversionFailure(e) => DbError::ConversionFailure(e),
        }
    }
}
