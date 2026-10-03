use axum::http::{HeaderMap, HeaderName, HeaderValue};
use governor::middleware::NoOpMiddleware;
use std::sync::Arc;
use tower_governor::key_extractor::PeerIpKeyExtractor;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::{
    add_extension::AddExtensionLayer, sensitive_headers::SetSensitiveHeadersLayer,
    set_header::SetResponseHeaderLayer,
};

/// Rate limiter configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Requests per second
    pub requests_per_second: u32,
    /// Burst size
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_second: 100,
            burst_size: 200,
        }
    }
}

/// Creates a rate limiter layer
pub fn rate_limiter_layer(
    config: RateLimitConfig,
) -> GovernorLayer<PeerIpKeyExtractor, NoOpMiddleware> {
    let governor_config = GovernorConfigBuilder::default()
        .per_second(config.requests_per_second as u64)
        .burst_size(config.burst_size)
        .finish()
        .expect("Failed to create governor config");

    GovernorLayer {
        config: Arc::new(governor_config),
    }
}

/// Creates secure headers layer with HSTS, CSP, and other security headers using tower-http
pub fn secure_headers_layer() -> (
    AddExtensionLayer<HeaderMap>,
    SetSensitiveHeadersLayer,
    SetResponseHeaderLayer<HeaderValue>,
) {
    // Add security headers using tower-http
    let mut headers = HeaderMap::new();

    // HSTS: max-age=1 year, include subdomains, preload
    headers.insert(
        HeaderName::from_static("strict-transport-security"),
        HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"),
    );

    // CSP: Restrictive policy
    headers.insert(
        HeaderName::from_static("content-security-policy"),
        HeaderValue::from_static(
            "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'"
        ),
    );

    // X-Frame-Options
    headers.insert(
        HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    );

    // X-Content-Type-Options
    headers.insert(
        HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );

    // Referrer-Policy
    headers.insert(
        HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    // Permissions-Policy
    headers.insert(
        HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static("accelerometer=(), camera=(), geolocation=(), gyroscope=(), magnetometer=(), microphone=(), payment=(), usb=()"),
    );

    // Cross-Origin-Embedder-Policy
    headers.insert(
        HeaderName::from_static("cross-origin-embedder-policy"),
        HeaderValue::from_static("require-corp"),
    );

    // Cross-Origin-Opener-Policy
    headers.insert(
        HeaderName::from_static("cross-origin-opener-policy"),
        HeaderValue::from_static("same-origin"),
    );

    // Cross-Origin-Resource-Policy
    headers.insert(
        HeaderName::from_static("cross-origin-resource-policy"),
        HeaderValue::from_static("same-origin"),
    );

    // X-Permitted-Cross-Domain-Policies
    headers.insert(
        HeaderName::from_static("x-permitted-cross-domain-policies"),
        HeaderValue::from_static("none"),
    );

    let add_headers = AddExtensionLayer::new(headers);
    let sensitive_headers =
        SetSensitiveHeadersLayer::new(std::iter::once(HeaderName::from_static("authorization")));
    let server_header = SetResponseHeaderLayer::overriding(
        HeaderName::from_static("server"),
        HeaderValue::from_static("PortfolioAPI"),
    );

    (add_headers, sensitive_headers, server_header)
}
