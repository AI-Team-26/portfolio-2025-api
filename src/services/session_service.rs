use crate::{
    constants,
    entities::{session::Session, user::User},
    repositories::{schemas::session_record::SessionRecord, session_repository::SessionRepository},
    services::user_service::UserService,
    utils::datetime,
    utils::token::generate_token,
};

use thiserror::Error;

#[derive(Clone)]
pub struct SessionService {
    repository: SessionRepository,
    #[allow(dead_code)]
    user_service: UserService,
}

#[derive(Error, Debug)]
pub enum CreateError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] crate::repositories::errors::DatabaseError),
}

impl SessionService {
    pub fn new(repository: SessionRepository, user_service: UserService) -> Self {
        Self {
            repository,
            user_service,
        }
    }

    pub async fn create(
        &self,
        user: User,
        ip_address: String,
        user_agent: String,
    ) -> Result<Session, CreateError> {
        let now = datetime::now();
        let access_expires_at = now + constants::auth::ACCESS_TOKEN_LIFETIME;
        let refresh_expires_at = now + constants::auth::REFRESH_TOKEN_LIFETIME;

        let session = Session {
            id: 0, // to be updated
            user,
            access_token: generate_token(),
            access_token_expires_at: access_expires_at,
            refresh_token: generate_token(),
            refresh_token_expires_at: refresh_expires_at,
            created_at: now,
            last_access_at: None,
            last_refresh_at: None,
            creation_ip_address: ip_address,
            creation_user_agent: user_agent,
        };

        let record = SessionRecord::from(session.clone());

        let new_id = self.repository.create(record).await?;

        // update id
        let final_session = Session {
            id: new_id,
            ..session
        };
        Ok(final_session)
    }
}
