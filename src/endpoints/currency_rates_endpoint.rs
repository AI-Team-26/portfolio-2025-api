use axum::extract::Query;
use axum::{extract::State, response::IntoResponse};

use super::response_utils::response_not_found;
use crate::endpoints::helper::parse_date;
use crate::endpoints::models::currency_rate_models as models;
use crate::state::AppState;

use crate::endpoints::response_utils::{
    response_bad_request, response_ok_map, response_validation_errors,
};
use validator::Validate;

pub async fn single_for_pair(
    State(state): State<AppState>,
    Query(query): Query<models::SinglePairQuery>,
) -> impl IntoResponse {
    crate::warn!("Received date query parameter: {:?}", query);

    let parsed_date = parse_date(query.date).unwrap();

    match state
        .currency_rate_service
        .search(query.base, query.quote, parsed_date)
        .await
    {
        Ok(rates) => response_ok_map(rates, models::CurrencyRate::from),
        Err(e) => response_not_found(&format!("Currency rate not foun. {e}")),
    }
}

pub async fn list_at_date(
    State(state): State<AppState>,
    Query(query): Query<models::AtDateQuery>,
) -> impl IntoResponse {
    crate::info!("curerncy-rates list_at_date");
    crate::warn!("Received date query parameter: {:?}", query);

    if let Err(errors) = query.validate() {
        return response_validation_errors(&errors);
    }

    let parsed_date = match parse_date(Some(query.date)) {
        Ok(Some(d)) => d,
        Ok(None) => return response_bad_request("Date is missing or invalid"),
        Err(e) => return response_bad_request(&format!("Invalid date format: {e}")),
    };

    match state.currency_rate_service.list_at_date(parsed_date).await {
        Ok(rates) => response_ok_map(rates, models::CurrencyRate::from),
        Err(e) => response_not_found(&format!("Currency rate not foun. {e}")),
    }
}
