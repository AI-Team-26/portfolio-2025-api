use crate::{
    entities::custodian::Custodian,
    repositories::{custodian_repository::CustodianRepository, errors::DatabaseError},
};

use thiserror::Error;

#[derive(Clone)]
pub struct CustodianService {
    repository: CustodianRepository,
}

#[derive(Error, Debug)]
pub enum CreateError {
    #[error("Custodian name already exists")]
    NameAlreadyExists,

    #[error("Database error: {0}")]
    DatabaseError(#[from] DatabaseError),
}

impl CustodianService {
    pub fn new(repository: CustodianRepository) -> Self {
        Self { repository }
    }

    pub async fn create(&self, item: Custodian) -> Result<i32, CreateError> {
        match self.repository.create(item).await {
            Ok(id) => Ok(id),
            Err(DatabaseError::DuplicatedField(_)) => Err(CreateError::NameAlreadyExists),
            Err(e) => Err(CreateError::DatabaseError(e)),
        }
    }

    pub async fn single(&self, id: i32, user_id: &str) -> Result<Custodian, DatabaseError> {
        self.repository.single(id, user_id).await
    }

    pub async fn update(&self, item: Custodian) -> Result<(), DatabaseError> {
        self.repository.update(item).await
    }

    pub async fn delete(&self, id: i32, user_id: &str) -> Result<(), DatabaseError> {
        self.repository.delete(id, user_id).await
    }

    pub async fn list(&self) -> Result<Vec<Custodian>, DatabaseError> {
        self.repository.list().await
    }
}