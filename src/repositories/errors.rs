#[derive(PartialEq)]
#[allow(dead_code)]
pub enum ErrorKind {
    DuplicatedField,
    RecordNotFound,
    Generic,
}

#[allow(dead_code)]
pub struct DatabaseError {
    pub message: String,
    pub kind: ErrorKind,
}

#[allow(dead_code)]
impl DatabaseError {
    pub fn duplicated_field(message: String) -> Self {
        DatabaseError {
            message,
            kind: ErrorKind::DuplicatedField,
        }
    }

    pub fn record_not_found() -> Self {
        DatabaseError {
            message: "Record not found.".to_string(),
            kind: ErrorKind::RecordNotFound,
        }
    }

    pub fn record_not_found_with_id(id: i32) -> Self {
        DatabaseError {
            message: format!("Record not found. ID: {}.", id),
            kind: ErrorKind::RecordNotFound,
        }
    }

    pub fn generic(message: String) -> Self {
        DatabaseError {
            message,
            kind: ErrorKind::Generic,
        }
    }
}

/* example with thiserror crate to define the behaviour of Display (for having .to_string()) */
/*
#[derive(thiserror::Error, Debug)]
pub enum CustodianError {
    #[error("Duplicate custodian name: {0}")]
    DuplicateName(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}
*/
