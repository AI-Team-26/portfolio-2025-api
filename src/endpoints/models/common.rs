use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

impl ErrorResponse {
    pub fn error(message: &str) -> Self {
        ErrorResponse {
            error: message.to_string(),
            code: None,
        }
    }
    pub fn error_code(error: &str, code: &str) -> Self {
        ErrorResponse {
            error: error.to_string(),
            code: Some(code.to_string()),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigResponse {
    pub(crate) custodian_kinds: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewIdResponse {
    pub new_id: i32,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationErrorsResponse {
    pub errors: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SuccessErrorResponse {
    is_success: bool,
    error: Option<String>,
}

impl SuccessErrorResponse {
    pub fn success() -> Self {
        Self {
            is_success: true,
            error: None,
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            is_success: false,
            error: Some(message.to_string()),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct _DataResponse<T> {
    #[serde(flatten)]
    pub base: SuccessErrorResponse,
    pub data: Option<T>,
}

impl<T> _DataResponse<T> {
    pub fn _success(data: T) -> Self {
        _DataResponse {
            base: SuccessErrorResponse::success(),
            data: Some(data),
        }
    }

    pub fn _error(message: &str) -> Self {
        _DataResponse {
            base: SuccessErrorResponse::error(message),
            data: None,
        }
    }
}

/* Usage

    /*pub type Response = DataResponse<Session>;

    #[derive(serde::Serialize)]
    pub struct Session {
        pub access_token: String,
        pub access_token_expires_at: UtcDateTime,
        pub refresh_token: String,
        pub refresh_token_expires_at: UtcDateTime
    }*/
*/
