// we use SQLx OffsetDateTime because the other type (PrimitiveDateTime) is ambiguous, it doesnt have timezone so it will be stored as TIMESTAMP, not TIMESTAMPTZ.
// TIMESYAMPTZ is required to have UTC date in Postgres.
//
// usage:
// UtcDateTime::now()
// UtcDateTime::try_from(date as string)

use chrono::{DateTime, NaiveDate, NaiveTime, Utc};

// type alias (does not allow impl)
pub type UtcDateTime = DateTime<Utc>;
pub type Date = chrono::NaiveDate; // Datetime without timezone

pub fn now() -> UtcDateTime {
    chrono::Utc::now()
}

pub fn today() -> Date {
    chrono::Utc::now().date_naive()
}

pub fn try_from(s: String) -> Result<UtcDateTime, String> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(&s) {
        return Ok(dt.with_timezone(&Utc));
    }

    if let Ok(dt) = DateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S%.fZ") {
        return Ok(dt.with_timezone(&Utc));
    }

    if let Ok(dt) = DateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%SZ") {
        return Ok(dt.with_timezone(&Utc));
    }

    if let Ok(date) = NaiveDate::parse_from_str(&s, "%Y-%m-%d") {
        let naive_datetime = date.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap());
        return Ok(DateTime::<Utc>::from_naive_utc_and_offset(
            naive_datetime,
            Utc,
        ));
    }

    Err(format!("Unable to parse datetime from string: {}", s))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, Timelike, Utc};

    #[test]
    fn now_returns_current_time() {
        let now = now();
        let expected = Utc::now();
        let diff = (now - expected).num_seconds().unsigned_abs();
        assert!(diff < 5, "now() was {} seconds off", diff);
    }

    #[test]
    fn today_returns_today() {
        let today = today();
        let expected = Utc::now().date_naive();
        assert_eq!(today, expected);
    }

    #[test]
    fn try_from_parses_rfc3339() {
        let dt = try_from("2024-01-15T10:30:00Z".to_string()).unwrap();
        assert_eq!(dt.year(), 2024);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 15);
        assert_eq!(dt.hour(), 10);
        assert_eq!(dt.minute(), 30);
    }

    #[test]
    fn try_from_parses_iso_with_microseconds() {
        let dt = try_from("2024-01-15T10:30:00.123Z".to_string()).unwrap();
        assert_eq!(dt.year(), 2024);
        assert_eq!(dt.second(), 0);
    }

    #[test]
    fn try_from_parses_date_only() {
        let dt = try_from("2024-01-15".to_string()).unwrap();
        assert_eq!(dt.year(), 2024);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 15);
        assert_eq!(dt.hour(), 0);
    }

    #[test]
    fn try_from_returns_error_on_invalid_input() {
        let result = try_from("not-a-date".to_string());
        assert!(result.is_err());
    }
}

//use serde::{Deserialize, Serialize};
//use sqlx::types::time::OffsetDateTime;

//pub use sqlx::types::time::OffsetDateTime as UtcDateTime;

/*
pub fn utc_now() -> OffsetDateTime {
    OffsetDateTime::now_utc()
}*/

//#[derive(Debug, Clone, Copy)]
//#[derive(Serialize, Deserialize)]
/*
pub struct UtcDateTime(OffsetDateTime);

impl UtcDateTime {
    pub fn now() -> UtcDateTime {
        UtcDateTime(OffsetDateTime::now_utc())
    }

    //pub fn utc() -> Self { OffsetDateTime::now_utc() }

    // Deref to inner type for database operations (OffsetDateTime is )
    pub fn as_timestamptz(&self) -> OffsetDateTime {
        self.0
    }

    pub fn from_timestamptz(value:OffsetDateTime) -> UtcDateTime {
        UtcDateTime(value)
    }
}
*/
