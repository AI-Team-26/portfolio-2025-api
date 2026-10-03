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

impl From<String> for DatabaseError {
    fn from(error: String) -> Self {
        Self::Generic(error)
    }
}

impl From<&str> for DatabaseError {
    fn from(error: &str) -> Self {
        Self::Generic(error.to_owned())
    }
}
