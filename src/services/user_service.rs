use crate::{
    entities::user::User,
    repositories::{errors::DatabaseError, user_repository::UserRepository},
    services::currency_service::CurrencyService,
};

#[derive(Clone)]
pub struct UserService {
    user_repository: UserRepository,
    currency_service: CurrencyService,
}

#[derive(Debug)]
pub enum CreateError {
    DatabaseError(String),
    UsernameAlreadyInUse,
}

impl UserService {
    pub fn new(user_repository: UserRepository, currency_service: CurrencyService) -> Self {
        Self {
            user_repository,
            currency_service,
        }
    }

    pub async fn create(&self, user: User) -> Result<(), CreateError> {
        match self.find_by_username(user.username.clone()).await {
            Ok(None) => (),
            Ok(Some(_)) => return Err(CreateError::UsernameAlreadyInUse),
            Err(e) => return Err(CreateError::DatabaseError(e)),
        }

        /* if self.find_by_username(user.username.clone()).await
        .map_err(|e| CreateError::DatabaseError(e.to_string()))?
        .is_some()  {
            return Err(CreateError::UsernameAlreadyInUse);
        }
        */

        self.user_repository
            .create(user)
            .await
            .map_err(|e| CreateError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn get(&self, id: &str) -> Result<Option<User>, String> {
        let user_result = self.user_repository.get(id).await?;
        Ok(user_result.map(|record| User {
            id: record.id,
            username: record.username,
            hashed_password: record.hashed_password.clone(),
            creation_date: record.creation_date,
            currency: self.currency_service.get(record.currency_id),
            role: record.role,
        }))
    }

    pub async fn find_by_username(&self, username: String) -> Result<Option<User>, String> {
        let user_result = self.user_repository.find_by_username(username).await?;
        Ok(user_result.map(|record| User {
            id: record.id,
            username: record.username,
            hashed_password: record.hashed_password.clone(),
            creation_date: record.creation_date,
            currency: self.currency_service.get(record.currency_id),
            role: record.role,
        }))
    }

    pub(crate) async fn update_currency(
        &self,
        user_id: String,
        currency_id: i32,
    ) -> Result<(), DatabaseError> {
        self.user_repository
            .update_currency(user_id, currency_id)
            .await
            .map_err(DatabaseError::generic)
    }
}
