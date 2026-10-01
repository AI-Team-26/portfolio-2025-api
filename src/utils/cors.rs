use crate::info;
use axum::{
    http::{self, HeaderValue},
    Router,
};
use tower_http::cors::{Any, CorsLayer};

pub trait RouterExtensions<S> {
    fn set_cors(self, domain: &str) -> Self;
}

impl<S> RouterExtensions<S> for Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn set_cors(self, domain: &str) -> Self {
        let production_url = format!("https://{}", domain);

        info!("CORS set for domain '{}'.", production_url);

        self.layer(
            CorsLayer::new()
                .allow_origin([
                    HeaderValue::from_static("http://localhost:5173"), // local development
                    HeaderValue::from_static("http://localhost:8300"),
                    HeaderValue::try_from(production_url.as_str()).unwrap(),
                ])
                .allow_methods([
                    http::Method::GET,
                    http::Method::POST,
                    http::Method::PUT,
                    http::Method::PATCH,
                    http::Method::DELETE,
                ])
                .allow_headers(Any),
        )
    }
}
