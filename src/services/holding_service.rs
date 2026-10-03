use std::collections::HashMap;

use rust_decimal::Decimal;

use crate::{
    endpoints::models::holding_models::{create, search, update},
    entities::{currency::Currency, user::User},
    repositories::{
        custodian_repository::CustodianRepository, errors::DatabaseError,
        holding_repository::HoldingRepository, schemas::holding_record::HoldingRecord,
    },
    services::{currency_rate_service::CurrencyRateService, currency_service::CurrencyService},
};

#[derive(Clone)]
pub struct HoldingService {
    repository: HoldingRepository,
    currency_rate_service: CurrencyRateService,
    _currency_service: CurrencyService,
    _custodian_repository: CustodianRepository,
}

impl HoldingService {
    pub fn new(
        repository: HoldingRepository,
        currency_rate_service: CurrencyRateService,
        _currency_service: CurrencyService,
        _custodian_repository: CustodianRepository,
    ) -> Self {
        Self {
            repository,
            currency_rate_service,
            _currency_service,
            _custodian_repository,
        }
    }

    pub async fn create(
        &self,
        user_id: &str,
        request: create::Request,
    ) -> Result<i32, DatabaseError> {
        let record: HoldingRecord = (request, user_id).into();
        self.repository.create(record).await
    }

    pub async fn update(
        &self,
        user_id: &str,
        id: i32,
        request: update::Request,
    ) -> Result<(), DatabaseError> {
        let record: HoldingRecord = (id, request, user_id).into();
        self.repository.update(record).await
    }

    pub async fn delete(&self, user_id: &str, id: i32) -> Result<(), DatabaseError> {
        self.repository.delete(id, user_id).await
    }

    pub async fn single_for_user(
        &self,
        user_id: &str,
        id: i32,
    ) -> Result<HoldingRecord, DatabaseError> {
        self.repository.single_for_user(id, user_id).await
    }

    pub async fn list_last_balance(
        &self,
        user: &User,
    ) -> Result<Vec<search::Response>, DatabaseError> {
        let records = self.repository.list_last_balance(&user.id).await?;

        self.add_amount_in_main_currency(&records, &user.currency)
            .await
    }

    pub async fn list_for_user(&self, user: &User) -> Result<Vec<search::Response>, DatabaseError> {
        let records = self.repository.list(&user.id).await?;

        self.add_amount_in_main_currency(&records, &user.currency)
            .await
    }

    async fn add_amount_in_main_currency(
        &self,
        records: &[HoldingRecord],
        main_currency: &Currency,
    ) -> Result<Vec<search::Response>, DatabaseError> {
        let rates = self.currency_rate_service.get_rates_of_today().await?;

        let rates_map: HashMap<i32, Decimal> = HashMap::from_iter(
            rates
                .into_iter()
                .filter(|rate| rate.quote_currency_id.eq(&main_currency.id))
                .map(|rate| (rate.base_currency_id, rate.rate)),
        );

        Ok(records
            .iter()
            .map(|r| {
                let amount = match r.currency_id.eq(&main_currency.id) {
                    true => Some(r.amount),
                    false => rates_map
                        .get(&r.currency_id)
                        .map(|rate| (r.amount * rate).round_dp(main_currency.precision as u32)),
                };

                search::Response::from((r.clone(), amount))
            })
            .collect())
    }
}
