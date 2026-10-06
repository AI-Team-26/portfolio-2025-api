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
        session_service::SessionService,
        user_service::{CreateError, UserService},
    },
    utils::datetime::{self, now},
    utils::token::generate_token,
};

#[derive(Clone)]
pub struct AuthService {
    user_service: UserService,
    session_service: SessionService,
    session_repository: SessionRepository,
}

pub enum LoginError {
    DatabaseError(String),
    FailedLogin,
}

#[derive(Debug)]
pub enum AuthError {
    DatabaseError(String),
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
        let Some(user) = self
            .user_service
            .find_by_username(request.username)
            .await
            .map_err(LoginError::DatabaseError)?
        else {
            return Err(LoginError::FailedLogin);
        };

        match verify_password(&request.password, &user.hashed_password) {
            true => {
                // create session
                self.session_service
                    .create(user, request.ip_address, request.user_agent)
                    .await
                    .map_err(LoginError::DatabaseError)
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

        let data_for_expired_token = "access token is invalid or expired".to_string();

        match self
            .session_repository
            .update_for_access(UpdateForAccess {
                access_token,
                access_token_expires_at: now + constants::auth::ACCESS_TOKEN_LIFETIME,
                refresh_token_expires_at: now + constants::auth::REFRESH_TOKEN_LIFETIME,
                last_access_at: now,
            })
            .await
            .map_err(AuthError::DatabaseError)?
        {
            Some(record) => Ok(record),
            None => Err(AuthError::InvalidOrExpiredToken(data_for_expired_token)), // session not found
        }
    }

    pub async fn refresh_session(&self, refresh_token: String) -> Result<SessionRecord, AuthError> {
        let now = datetime::now();

        // Single query: update doubles as lookup; None means unknown/expired token.
        // Fixed message: never include the raw token in errors/logs.
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
            .await
            .map_err(AuthError::DatabaseError)?
        {
            Some(record) => Ok(record),
            None => Err(AuthError::InvalidOrExpiredToken(
                "invalid or expired refresh token".to_string(),
            )),
        }
    }
}

pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub ip_address: String,
    pub user_agent: String,
}
