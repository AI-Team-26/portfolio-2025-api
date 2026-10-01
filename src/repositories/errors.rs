use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum DatabaseError {
    #[error("Duplicate field: {0}")]
    DuplicatedField(String),

    #[error("Record not found")]
    RecordNotFound,

    #[error("Record not found with ID: {0}")]
    #[allow(dead_code)]
    RecordNotFoundWithId(i32),

    #[error("Database error: {0}")]
    Generic(String),
}

impl DatabaseError {
    pub fn duplicated_field(message: String) -> Self {
        DatabaseError::DuplicatedField(message)
    }

    pub fn record_not_found() -> Self {
        DatabaseError::RecordNotFound
    }

    #[allow(dead_code)]
    pub fn record_not_found_with_id(id: i32) -> Self {
        DatabaseError::RecordNotFoundWithId(id)
    }

    pub fn generic(message: String) -> Self {
        DatabaseError::Generic(message)
    }
}

impl From<sqlx::Error> for DatabaseError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::Database(e) if e.is_unique_violation() => {
                DatabaseError::DuplicatedField(e.to_string())
            }
            sqlx::Error::Database(e) => DatabaseError::Generic(match e.code() {
                Some(code) => format!("Code: {}. {}", code, e),
                None => e.to_string(),
            }),
            _ => DatabaseError::Generic(err.to_string()),
        }
    }
}

impl From<String> for DatabaseError {
    fn from(err: String) -> Self {
        DatabaseError::Generic(err)
    }
}

impl From<&str> for DatabaseError {
    fn from(err: &str) -> Self {
        DatabaseError::Generic(err.to_string())
    }
}
