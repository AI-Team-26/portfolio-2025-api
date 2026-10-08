use std::time::Instant;

use crate::{configuration::Configuration, jobs::job_manager::RecurringJob, state::AppState};

#[derive(Clone)]
pub struct UpdateCurrencyRatesJob {
    app_state: AppState,
}

impl UpdateCurrencyRatesJob {
    pub fn new(_config: &Configuration, app_state: AppState) -> Self {
        Self { app_state }
    }
}

impl RecurringJob for UpdateCurrencyRatesJob {
    async fn run(&self) -> () {
        crate::info!("Run ");
        let started = Instant::now();

        match self
            .app_state
            .currency_rate_service
            .load_rates_from_coingecko()
            .await
        {
            // single multi-row upsert instead of one INSERT per coin
            Ok(rates) => match self
                .app_state
                .currency_rate_service
                .create_many(&rates)
                .await
            {
                Ok(rows_affected) => crate::info!(
                    "Upserted {} currency rates ({} rows affected) in {} ms",
                    rates.len(),
                    rows_affected,
                    started.elapsed().as_millis()
                ),
                Err(e) => {
                    // the batch is atomic, so every rate in it failed
                    let symbol_of = |id: i32| {
                        self.app_state
                            .currency_service
                            .try_get(id)
                            .map(|c| c.symbol)
                            .unwrap_or_else(|| id.to_string())
                    };
                    let samples: Vec<String> = rates
                        .iter()
                        .take(10)
                        .map(|r| {
                            format!(
                                "{}/{}",
                                symbol_of(r.base_currency_id),
                                symbol_of(r.quote_currency_id)
                            )
                        })
                        .collect();
                    crate::error!(
                        "Failed to upsert {} currency rates in one batch: {}. Sample pairs: [{}]",
                        rates.len(),
                        e,
                        samples.join(", ")
                    );
                }
            },
            Err(e) => crate::error!("Failed to get rates from CoinGecko. {}", e),
        }
    }
}
