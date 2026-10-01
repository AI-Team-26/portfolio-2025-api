use crate::utils::datetime::{try_from as try_get_date_from_string, Date, UtcDateTime};

/**
 * Parse the "YYYY-MM-DD" date to "YYYY-MM-DDT00:00:00Z" (UTC datetime)
 */
pub fn parse_datetime(date: Option<String>) -> Result<Option<UtcDateTime>, String> {
    match date {
        Some(s) => try_get_date_from_string(s).map(Some),
        None => Ok(None),
    }
}

pub fn parse_date(date: Option<String>) -> Result<Option<Date>, String> {
    match parse_datetime(date) {
        Ok(opt_dt) => Ok(opt_dt.map(|dt| dt.date_naive())),
        Err(e) => Err(e),
    }
}
