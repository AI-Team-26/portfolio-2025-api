use std::{sync::Mutex, time::Duration};

use axum_prometheus::{
    utils::SECONDS_DURATION_BUCKETS, BaseMetricLayer, AXUM_HTTP_REQUESTS_DURATION_SECONDS,
};
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};
use sqlx::PgPool;

/// HTTP metric layer plus the handle used to render `/metrics`.
#[derive(Clone)]
pub struct AppMetrics {
    pub layer: BaseMetricLayer<'static>,
    pub handle: PrometheusHandle,
}

// The prometheus recorder can only be installed once per process, so the
// setup is cached after the first call (tests may call init_metrics again).
static METRICS_CACHE: Mutex<Option<AppMetrics>> = Mutex::new(None);

/// Install the global prometheus recorder (first call only), start polling
/// the sqlx pool gauges and return the layer + render handle for the router.
pub fn init_metrics(pool: PgPool) -> AppMetrics {
    let mut cache = METRICS_CACHE.lock().expect("metrics cache poisoned");
    if let Some(existing) = &*cache {
        return existing.clone();
    }

    let handle = PrometheusBuilder::new()
        .set_buckets_for_metric(
            Matcher::Full(AXUM_HTTP_REQUESTS_DURATION_SECONDS.to_string()),
            SECONDS_DURATION_BUCKETS,
        )
        .expect("valid bucket matcher")
        .install_recorder()
        .expect("failed to install prometheus recorder");

    spawn_pool_gauge_poller(pool);

    let metrics = AppMetrics {
        layer: BaseMetricLayer::new(),
        handle,
    };
    *cache = Some(metrics.clone());
    metrics
}

fn spawn_pool_gauge_poller(pool: PgPool) {
    let size = metrics::gauge!("db_pool_size");
    let idle = metrics::gauge!("db_pool_idle");
    let in_use = metrics::gauge!("db_pool_in_use");

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            let pool_size = pool.size();
            let pool_idle = pool.num_idle() as u32;
            size.set(pool_size as f64);
            idle.set(pool_idle as f64);
            in_use.set(pool_size.saturating_sub(pool_idle) as f64);
        }
    });
}
