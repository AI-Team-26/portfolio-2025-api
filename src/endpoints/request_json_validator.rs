use crate::endpoints::response_utils::response_bad_request;
use axum::{
    body::Body,
    extract::{rejection::JsonRejection, FromRequest, Json},
    http::Request,
    response::IntoResponse,
};
use serde::de::DeserializeOwned;

pub struct ValidJson<T>(pub T);

impl<T, S> FromRequest<S, Body> for ValidJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = axum::response::Response;

    async fn from_request(req: Request<Body>, state: &S) -> Result<Self, Self::Rejection> {
        match Json::<T>::from_request(req, state).await {
            Ok(Json(payload)) => Ok(ValidJson(payload)),
            Err(json_rejection) => {
                let error_message = match json_rejection {
                    JsonRejection::JsonDataError(err) => format!("Invalid JSON data: {}", err),
                    JsonRejection::JsonSyntaxError(err) => format!("JSON syntax error: {}", err),
                    JsonRejection::MissingJsonContentType(err) => {
                        format!("Missing JSON content type: {}", err)
                    }
                    JsonRejection::BytesRejection(err) => {
                        format!("Failed to extract request body: {}", err)
                    }
                    _ => format!("Invalid JSON request: {}", json_rejection),
                };
                Err(response_bad_request(&error_message).into_response())
            }
        }
    }
}

// Implement Deref so you can access the inner value directly
impl<T> std::ops::Deref for ValidJson<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
