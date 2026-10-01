use async_trait::async_trait;

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

#[async_trait]
impl RecurringJob for UpdateCurrencyRatesJob {
    async fn run(&self) -> () {
        /*match self.app_state.api.ping().await {
            true => info!("Coingecko Ping OK"),
            false => warn!("Coingecko ping FAIL. ")
        };*/
        crate::info!("Run ");

        match self
            .app_state
            .currency_rate_service
            .load_rates_from_coingecko()
            .await
        {
            Ok(rates) => {
                for rate in rates {
                    match self.app_state.currency_rate_service.create(&rate).await {
                        Ok(()) => (),
                        Err(e) => {
                            crate::error!("Failed to create rate for {}. {}", &rate.display(), e)
                        }
                    }
                }
            }
            Err(e) => crate::error!("Failed to get rates from CoinGecko. {}", e),
        }
    }
}
