use crate::{
    constants,
    entities::{currency::Currency, session::Session, user::User},
    repositories::{
        schemas::session_record::{
            SessionRecord, SessionWithUser, UpdateForAccess, UpdateForRefresh,
        },
        session_repository::SessionRepository,
    },
    services::{
        password_hashing::{hash_password, verify_password},
        session_service::{CreateError as SessionCreateError, SessionService},
        user_service::{CreateError, UserService},
    },
    utils::datetime::{self, now},
    utils::token::generate_token,
};

use thiserror::Error;

#[derive(Clone)]
pub struct AuthService {
    user_service: UserService,
    session_service: SessionService,
    session_repository: SessionRepository,
}

#[derive(Error, Debug)]
pub enum LoginError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] crate::repositories::errors::DatabaseError),

    #[error("Wrong username or password")]
    FailedLogin,
}

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] crate::repositories::errors::DatabaseError),

    #[error("Invalid or expired token: {0}")]
    InvalidOrExpiredToken(String),
}

impl AuthService {
    pub fn new(
        user_service: UserService,
        session_service: SessionService,
        session_repository: SessionRepository,
    ) -> Self {
        AuthService {
            user_service,
            session_service,
            session_repository,
        }
    }

    pub async fn signup(
        &self,
        username: String,
        password: String,
        currency: Currency,
    ) -> Result<(), CreateError> {
        let id = uuid::Uuid::new_v4().to_string();
        let hashed_password = hash_password(&password);

        let user: User = User {
            id,
            username,
            hashed_password,
            creation_date: now(),
            currency,
            role: String::from("User"), // default
        };

        self.user_service.create(user).await
    }

    pub async fn login(&self, request: LoginRequest) -> Result<Session, LoginError> {
        let Some(user) = self.user_service.find_by_username(request.username).await? else {
            return Err(LoginError::FailedLogin);
        };

        match verify_password(&request.password, &user.hashed_password) {
            true => {
                // create session
                self.session_service
                    .create(user, request.ip_address, request.user_agent)
                    .await
                    .map_err(|e| match e {
                        SessionCreateError::DatabaseError(e) => LoginError::DatabaseError(e),
                    })
            }
            false => Err(LoginError::FailedLogin),
        }
    }

    /// Prolonge the access and refresh token validity and return the session with user id
    pub async fn validate_access(
        &self,
        access_token: String,
    ) -> Result<SessionWithUser, AuthError> {
        let now = datetime::now();

        let data_for_expired_token = format!(
            "update_for_access: 
        access_token: {},
        now: {},
        access_token_expires_at: {},
        refresh_token_expires_at: {},
        ",
            access_token,
            now,
            now + constants::auth::ACCESS_TOKEN_LIFETIME,
            now + constants::auth::REFRESH_TOKEN_LIFETIME
        );

        match self
            .session_repository
            .update_for_access(UpdateForAccess {
                access_token,
                access_token_expires_at: now + constants::auth::ACCESS_TOKEN_LIFETIME,
                refresh_token_expires_at: now + constants::auth::REFRESH_TOKEN_LIFETIME,
                last_access_at: now,
            })
            .await?
        {
            Some(record) => Ok(record),
            None => Err(AuthError::InvalidOrExpiredToken(data_for_expired_token)), // session not found
        }
    }

    pub async fn refresh_session(&self, refresh_token: String) -> Result<SessionRecord, AuthError> {
        let now = datetime::now();

        let exists = self
            .session_repository
            .exists_by_refresh_token(&refresh_token)
            .await?;

        if !exists {
            return Err(AuthError::InvalidOrExpiredToken(format!(
                "NOT FOUND - 111 | refresh_token: {}",
                refresh_token
            )));
        }

        let session = self
            .session_repository
            .find_by_refresh_token(&refresh_token)
            .await?;

        let (session_id, refresh_token_expires_at) = match session {
            Some(s) => (s.id.to_string(), s.refresh_token_expires_at.to_string()),
            None => (String::new(), String::new()),
        };

        let data_for_expired_token = format!(
            "refresh_session.
            refresh_token: {}, 
            session: {},
            session.refresh_token_expires_at: {}
            ",
            refresh_token, session_id, refresh_token_expires_at
        );

        match self
            .session_repository
            .update_for_refresh(UpdateForRefresh {
                old_refresh_token: refresh_token,
                access_token: generate_token(),
                refresh_token: generate_token(),
                access_token_expires_at: now + constants::auth::ACCESS_TOKEN_LIFETIME,
                refresh_token_expires_at: now + constants::auth::REFRESH_TOKEN_LIFETIME,
                last_refresh_at: now,
            })
            .await?
        {
            Some(record) => Ok(record),
            None => Err(AuthError::InvalidOrExpiredToken(data_for_expired_token)), // session not found
        }
    }
}

pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub ip_address: String,
    pub user_agent: String,
}
