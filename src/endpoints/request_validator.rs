use chrono::Utc;

use crate::{entities::custodian::KINDS, utils::datetime::UtcDateTime};

#[allow(dead_code)]
pub enum RuleString {
    NotEmpty,
    MinLength(usize),
    MaxLength(usize),
    FixLength(usize),
    Uuid,
    IsValidCustodianKind(),
}

/*
pub enum RuleStringOption {
    MinLength(usize),
    MaxLength(usize),
    FixLength(usize),
}
*/

pub enum RuleNumber {
    NotZero,
}

#[allow(dead_code)]
pub enum RuleDate {
    NotInFuture,
}

impl RuleString {
    pub fn validate(&self, field: &str, value: &str) -> Option<String> {
        match self {
            RuleString::NotEmpty if value.trim().is_empty() => {
                Some(format!("{}: cannot be empty", field))
            }
            RuleString::MinLength(min) if value.len() < *min => {
                Some(format!("{}: must be at least {} characters", field, min))
            }
            RuleString::MaxLength(max) if value.len() > *max => {
                Some(format!("{}: must be at max {} characters", field, max))
            }
            RuleString::FixLength(len) if value.len() > *len => {
                Some(format!("{}: must be {} characters", field, len))
            }
            RuleString::Uuid => None, // TODO: not implemented
            RuleString::IsValidCustodianKind() if !KINDS.contains(&value) => Some(format!(
                "{}: is not a valid kind. Valid values: {}.",
                field,
                KINDS.join(", ")
            )),
            _ => None,
        }
    }
}

/*
impl RuleStringOption {
    #[allow(dead_code)]
    pub fn validate(&self, field: &str, value: &Option<String>) -> Option<String> {
        match value {
            None => None, // None always passes
            Some(value) => match self {
                RuleStringOption::MinLength(min) => {
                    RuleString::MinLength(*min).validate(field, value)
                }
                RuleStringOption::MaxLength(max) => {
                    RuleString::MaxLength(*max).validate(field, value)
                }
                RuleStringOption::FixLength(len) => {
                    RuleString::FixLength(*len).validate(field, value)
                }
            },
        }
    }
}
*/

impl RuleNumber {
    pub fn validate<T: PartialEq + Default>(&self, field: &str, value: T) -> Option<String> {
        match self {
            RuleNumber::NotZero if value == T::default() => {
                Some(format!("{}: cannot be zero", field))
            }
            _ => None,
        }
    }
}

impl RuleDate {
    #[allow(dead_code)]
    pub fn validate(&self, field: &str, value: UtcDateTime) -> Option<String> {
        match self {
            RuleDate::NotInFuture if value > Utc::now() => {
                Some(format!("{}: cannot be in the future", field))
            }
            _ => None,
        }
    }
}

/// Validates a list of `(field, value, rule)` triplets.
///
/// # Example
/// ```ignore
/// validate!(
///     "username", user.username, RuleString::NotEmpty;
///     "email", user.email, rules::email();
/// );
/// ```
#[macro_export]
macro_rules! validate {
    //use crate::endpoints::response_utils::response_validation_errors;
    ($($field:expr, $value:expr, $rule:expr);* $(;)?) => {{
        let mut errors = Vec::new();
        $(
            if let Some(error) = $rule.validate($field, $value) {
                errors.push(error);
            }
        )*
        // returns the HTTP response 4xx
        if !errors.is_empty() {
            return $crate::endpoints::response_utils::response_validation_errors(errors);
        }
    }};
}

#[macro_export]
macro_rules! _get_errors {
    ($($field:expr, $value:expr, $rule:expr);* $(;)?) => {{
        let mut errors = Vec::new();
        $(
            if let Some(error) = $rule.validate($field, $value) {
                errors.push(error);
            }
        )*
        // returns the errors
        errors
    }};
}
