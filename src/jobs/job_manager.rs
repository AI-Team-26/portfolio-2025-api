use crate::{
    configuration::Configuration, info, jobs::update_currency_rates_job::UpdateCurrencyRatesJob,
    utils::dependency_injection::AppState,
};
use async_trait::async_trait;
use tokio_cron_scheduler::{Job, JobScheduler};

pub async fn schedule_jobs(config: &Configuration, app_state: AppState) {
    let scheduler = JobScheduler::new().await.unwrap();

    // Update Exchange Rate
    let update_exchange_rates_job = UpdateCurrencyRatesJob::new(config, app_state.clone());

    scheduler
        .add(
            Job::new_async(
                &config.jobs.update_exchange_rate_cron,
                move |_id, mut _lock| {
                    let job = update_exchange_rates_job.clone();
                    Box::pin(async move {
                        job.run().await;
                    })
                },
            )
            .unwrap(),
        )
        .await
        .unwrap();

    scheduler.start().await.unwrap();

    info!(
        "'Update Exchange Rate' job scheduled ({})",
        &config.jobs.update_exchange_rate_cron
    );

    // Spawn the service in the background instead of awaiting it
    /*tokio::spawn(async move {
        service.await;
    });*/
}

// async_trait] is necessary because Rust async traits are not yet natively supported in stable.
// Added Send + Sync bounds, which are typically required for types shared across threads.
#[async_trait]
pub trait RecurringJob: Send + Sync {
    async fn run(&self) -> ();
}
