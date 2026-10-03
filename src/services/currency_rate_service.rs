use std::sync::Arc;

use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use tokio::sync::RwLock;

use crate::repositories::currency_rate_repository::CurrencyRateRepository;
use crate::repositories::schemas::currency_rate_record::CurrencyRateRecord;
use crate::services::currency_service::CurrencyService;
use crate::services::Coingecko::coingecko_api::CoingeckoApi;
use crate::services::Coingecko::currencies_map::COINGECKO_QUOTE_IDS;
use crate::utils::datetime::{now, today, Date};
use crate::{constants, info, warn};

#[derive(Clone)]
pub struct CurrencyRateService {
    repository: CurrencyRateRepository,
    currency_service: CurrencyService,
    coingecko_api: CoingeckoApi,
    latest_rates: Arc<RwLock<Vec<CurrencyRateRecord>>>,
    latest_rates_date: Arc<RwLock<Date>>,
}

impl CurrencyRateService {
    pub fn new(
        repository: CurrencyRateRepository,
        currency_service: CurrencyService,
        coingecko_api: CoingeckoApi,
    ) -> Self {
        Self {
            repository,
            currency_service,
            coingecko_api,
            latest_rates: Arc::new(RwLock::new(Vec::<CurrencyRateRecord>::new())), // Initialize empty }
            latest_rates_date: Arc::new(RwLock::new(Date::from_epoch_days(0).unwrap())),
        }
    }

    pub async fn get_rates_of_today(
        &self,
    ) -> Result<Vec<CurrencyRateRecord>, crate::repositories::errors::DatabaseError> {
        if self.latest_rates.read().await.is_empty()
            || *self.latest_rates_date.read().await != today()
        {
            self.load_rates_from_coingecko().await?;
        }

        Ok(self.latest_rates.read().await.clone())
    }

    pub async fn create(
        &self,
        record: &CurrencyRateRecord,
    ) -> Result<(), crate::repositories::errors::DatabaseError> {
        self.repository.create(record).await
    }

    pub async fn search(
        &self,
        base_currency_id: i32,
        quote_currency_id: i32,
        date: Option<Date>,
    ) -> Result<Vec<CurrencyRateRecord>, crate::repositories::errors::DatabaseError> {
        self.repository
            .search(base_currency_id, quote_currency_id, date)
            .await
    }

    pub async fn list_at_date(
        &self,
        date: Date,
    ) -> Result<Vec<CurrencyRateRecord>, crate::repositories::errors::DatabaseError> {
        self.repository.list_at_date(date).await
    }

    /// Retrieve the currency rates from Coingecko.
    /// Coingecko uses its ids to identify Base currencies and has only some ids marked valid as Quote currencies.
    /// Base currencies are Cypto and stable coin, and Quote currencies are fiat.
    /// We use a fixed map for mapping Coingecko currency ID to currency symbols.    
    pub async fn load_rates_from_coingecko(
        &self,
    ) -> Result<Vec<CurrencyRateRecord>, crate::repositories::errors::DatabaseError> {
        let base_ids: Vec<String> = self
            .currency_service
            .crypto_and_stable_currencies
            .iter()
            .filter_map(|entry| entry.value().coingecko_id.clone())
            .collect();

        // We use a fixed list of coins as quote
        let quote_ids = COINGECKO_QUOTE_IDS.values().copied().collect::<Vec<_>>();

        let result = self.coingecko_api.get_rates(&base_ids, &quote_ids).await;

        match result {
            Ok(coingecko_rates) => {
                //let rate: f64 = serde_json::from_str(&text).unwrap_or(0.0);
                info!("got coingecko rates");

                let mut new_rates: Vec<CurrencyRateRecord> = vec![];
                for (coin, map) in coingecko_rates {
                    if let Some(base_currency) =
                        self.currency_service.try_get_by_Coingecko_id(&coin)
                    {
                        for (symbol, rate) in map {
                            if let Some(quote_currency) =
                                self.currency_service.try_get_by_symbol_CI(&symbol)
                            {
                                info!(
                                    "Add currency rate for {}/{}: {}",
                                    base_currency.symbol, quote_currency.symbol, rate
                                );

                                let rate = CurrencyRateRecord {
                                    base_currency_id: base_currency.id,
                                    quote_currency_id: quote_currency.id,
                                    date: today(),
                                    rate: Decimal::from_f64(rate).ok_or_else(|| {
                                        crate::repositories::errors::DatabaseError::generic(
                                            format!("f64 to Decimal conversion failed for {rate}"),
                                        )
                                    })?,
                                    source: constants::external_services::COINGECKO.to_owned(),
                                    created_at: now(),
                                };

                                new_rates.push(rate);
                            } else {
                                warn!("Currency not found with symbol: {}", symbol);
                            }
                        }
                    } else {
                        warn!("Currency not found with Coingecko name: {}", coin);
                    }
                }

                // update latest rates
                let mut latest_rates = self.latest_rates.write().await;
                *latest_rates = new_rates.clone();
                let mut date = self.latest_rates_date.write().await;
                *date = today();

                Ok(new_rates)
            }
            Err(e) => Err(crate::repositories::errors::DatabaseError::generic(
                format!("Failed to get rates. {}", e),
            )),
        }
    }
}
