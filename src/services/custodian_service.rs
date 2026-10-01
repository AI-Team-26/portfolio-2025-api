use crate::{
    entities::custodian::Custodian,
    repositories::{custodian_repository::CustodianRepository, errors::DatabaseError},
};

#[derive(Clone)]
pub struct CustodianService {
    repository: CustodianRepository,
}

#[allow(dead_code)]
pub enum CreateError {
    NameAlreadyExists,
    Unexpected(String),
}

impl CustodianService {
    pub fn new(repository: CustodianRepository) -> Self {
        Self { repository }
    }

    pub async fn create(&self, item: Custodian) -> Result<i32, CreateError> {
        //self.repository.create(item).await
        self.repository
            .create(item)
            .await
            .map_err(|err| CreateError::Unexpected(err.message))
    }

    pub async fn single(&self, id: i32, user_id: &str) -> Result<Custodian, String> {
        self.repository.single(id, user_id).await
    }

    pub async fn update(&self, item: Custodian) -> Result<(), DatabaseError> {
        self.repository.update(item).await
    }

    pub async fn delete(&self, id: i32, user_id: &str) -> Result<(), DatabaseError> {
        self.repository.delete(id, user_id).await
    }

    pub async fn list(&self) -> Result<Vec<Custodian>, String> {
        self.repository.list().await
    }
}
