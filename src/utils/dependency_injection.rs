use sqlx::PgPool;

use crate::{
    configuration::Configuration,
    fatal_end_exit,
    repositories::{
        currency_of_user_repository::CurrencyOfUserRepository,
        currency_rate_repository::CurrencyRateRepository, currency_repository::CurrencyRepository,
        custodian_repository::CustodianRepository, holding_repository::HoldingRepository,
        session_repository::SessionRepository, user_repository::UserRepository,
    },
    services::{
        auth_service::AuthService, currency_rate_service::CurrencyRateService,
        currency_service::CurrencyService, custodian_service::CustodianService,
        holding_service::HoldingService, session_service::SessionService,
        user_service::UserService, Coingecko::coingecko_api::CoingeckoApi,
    },
};

#[derive(Clone)]
pub struct AppState {
    //pub config: Configuration,
    pub user_service: UserService,
    //pub session_service: SessionService,
    pub auth_service: AuthService,
    pub currency_service: CurrencyService,
    pub custodian_service: CustodianService,
    pub holding_service: HoldingService,
    pub currency_rate_service: CurrencyRateService,
}

pub async fn inject_services(config: &Configuration, db_pool: PgPool) -> AppState {
    let user_repository = UserRepository::new(db_pool.clone());
    let session_repository = SessionRepository::new(db_pool.clone());
    let currency_repository = CurrencyRepository::new(db_pool.clone());
    let currency_of_user_repository = CurrencyOfUserRepository::new(db_pool.clone());
    let custodian_repository = CustodianRepository::new(db_pool.clone());
    let holding_repository = HoldingRepository::new(db_pool.clone());
    let currency_rate_repository = CurrencyRateRepository::new(db_pool.clone());

    let coingecko_api = CoingeckoApi::new(config);

    let currency_service = CurrencyService::new(
        currency_repository.clone(),
        currency_of_user_repository.clone(),
    );
    // **LOAD THE CACHE ONLY ONCE**
    if let Err(e) = currency_service.init_cache().await {
        fatal_end_exit!("Failed to initialize currency cache: {}", e);
    }

    let user_service = UserService::new(user_repository.clone(), currency_service.clone());
    let session_service = SessionService::new(session_repository.clone(), user_service.clone());
    let auth_service = AuthService::new(
        user_service.clone(),
        session_service.clone(),
        session_repository.clone(),
    );
    let currency_rate_service = CurrencyRateService::new(
        currency_rate_repository,
        currency_service.clone(),
        coingecko_api,
    );
    let holding_service = HoldingService::new(
        holding_repository,
        currency_rate_service.clone(),
        currency_service.clone(),
        custodian_repository.clone(),
    );

    AppState {
        //config: config.clone(),
        user_service: user_service.clone(),
        //session_service: session_service.clone(),
        auth_service: auth_service.clone(),
        currency_service: currency_service.clone(),
        custodian_service: CustodianService::new(custodian_repository.clone()),
        holding_service: holding_service.clone(),
        currency_rate_service: currency_rate_service.clone(),
    }
}
