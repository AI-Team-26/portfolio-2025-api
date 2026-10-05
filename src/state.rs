use sqlx::PgPool;

use crate::services::{
    auth_service::AuthService, currency_rate_service::CurrencyRateService,
    currency_service::CurrencyService, custodian_service::CustodianService,
    holding_service::HoldingService, user_service::UserService,
};

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub user_service: UserService,
    pub auth_service: AuthService,
    pub currency_service: CurrencyService,
    pub custodian_service: CustodianService,
    pub holding_service: HoldingService,
    pub currency_rate_service: CurrencyRateService,
}
