use crate::repositories::errors::DatabaseError;
use sqlx::postgres::{PgQueryResult, PgRow};
use sqlx::{Error as SqlxError, Row};

pub trait BaseRepository {
    /**
     * Check used for CREATE database operation
     */
    fn check_result_for_new_id(
        &self,
        result: Result<PgRow, sqlx::Error>,
    ) -> Result<i32, DatabaseError> {
        match result {
            //Ok(row) => Ok(row.get(0)), // assumes id is the first column
            Ok(row) => Ok(row.get("id")),
            Err(SqlxError::Database(e)) => match e.is_unique_violation() {
                true => Err(DatabaseError::duplicated_field(e.to_string())),
                _ => match e.code() {
                    Some(code) => Err(DatabaseError::generic(format!(
                        "Code: {}. {}",
                        code,
                        e.to_string()
                    ))),
                    None => Err(DatabaseError::generic(e.to_string())),
                },
            },
            Err(err) => Err(DatabaseError::generic(err.to_string())),
        }
    }

    /**
     * Check used for UPDATE, DELETE database operation
     */
    fn check_result(&self, result: PgQueryResult) -> Result<(), DatabaseError> {
        match result.rows_affected() == 0 {
            true => Err(DatabaseError::record_not_found()),
            false => Ok(()),
        }
    }
}
