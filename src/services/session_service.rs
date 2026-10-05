use crate::{
    constants,
    entities::{session::Session, user::User},
    repositories::{schemas::session_record::SessionRecord, session_repository::SessionRepository},
    services::user_service::UserService,
    utils::datetime,
    utils::token::generate_token,
};

#[derive(Clone)]
pub struct SessionService {
    repository: SessionRepository,
    #[allow(dead_code)]
    user_service: UserService,
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
    ) -> Result<Session, String> {
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

        match self.repository.create(record).await {
            Ok(new_id) => {
                // TODO: log

                // update id
                let final_session = Session {
                    id: new_id,
                    ..session
                };
                Ok(final_session)

                //session.update_id(new_id);
                //Ok(session)
            }
            Err(e) => {
                // TODO: log
                Err(format!("Failed to create user Sessoon. {}", e))
            }
        }
    }

    /*
    pub async fn find_by_access_token(&self, access_token: &str) -> Result<Option<Session>, String> {

        // TODO: optimize with a single query

        let Some(record) = self.repository.find_by_access_token(access_token).await? else {
            return Ok(None); //(format!("Session not found with access token: '{}'", access_token));
        };

        let Some(user ) = self.user_service.get(&record.user_id).await? else {
            return Err(format!("User not found with id: '{}'", &record.user_id));
        };

        let session: Session = (record, user).into();
        Ok(Some(session))
    }

    pub async fn find_by_refresh_token(&self, refresh_token: &str) -> Result<Option<Session>, String> {

        // TODO: optimize with a single query

        let Some(record) = self.repository.find_by_refresh_token(refresh_token).await? else {
            return Ok(None);
        };

        let Some(user ) = self.user_service.get(&record.user_id).await? else {
            return Err(format!("User not found with id: '{}'", &record.user_id));
        };

        let session: Session = (record, user).into();
        Ok(Some(session))
    }
    */

    //pub async fn update(&self, item: Session) -> Result<(), String> {
    //    self.repository.update(item).await
    //}
}
