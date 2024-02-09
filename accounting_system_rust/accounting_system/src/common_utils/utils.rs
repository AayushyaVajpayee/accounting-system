use std::str::FromStr;
use anyhow::bail;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use actix_web::{HttpRequest, ResponseError};
use actix_web::http::StatusCode;
use chrono::{Datelike, NaiveDate, TimeZone, Utc};
use thiserror::Error;
use tokio_postgres::SimpleQueryMessage;
use tracing::error;
use uuid::Uuid;
use validator::{ValidationError, ValidationErrors, ValidationErrorsKind};

use crate::common_utils::dao_error::DaoError;

#[derive(Debug, Error)]
pub enum TimeError {
    #[error("duration in opposite {:?}", 0)]
    ForwardTime(Duration)
}

///in microseconds
pub fn get_current_time_us() -> Result<i64, TimeError> {
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|a| {
            error!(?a,%a,"error during time fetching");
            TimeError::ForwardTime(a.duration())
        })?.as_micros() as i64;
    Ok(current_time)
}

pub fn current_indian_financial_year() -> u32 {
    let utc_now = Utc::now().naive_utc();
    let current_date = chrono_tz::Asia::Kolkata.from_utc_datetime(&utc_now).date_naive();

    let current_year = current_date.year();
    let start_year = if current_date.month() < 4 {
        current_year - 1
    } else {
        current_year
    };

    start_year as u32
}

pub fn current_indian_date() -> NaiveDate {
    let utc_now = Utc::now().naive_utc();
    let current_date = chrono_tz::Asia::Kolkata.from_utc_datetime(&utc_now).date_naive();
    current_date
}
#[derive(Debug,Error)]
pub enum TenantHeaderError{
    #[error("x-tenant-id header not present in request")]
    NotPresent,
    #[error("x-tenant-id header does not have a valid uuid")]
    NotUuid
}
impl ResponseError for TenantHeaderError {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

pub fn extract_tenant_id_from_header(request:&HttpRequest)->Result<Uuid,TenantHeaderError>{
    let p = request.headers()
        .get("x-tenant-id")
        .ok_or(TenantHeaderError::NotPresent)?;
    let tenant_id_str = p.to_str().map_err(|_| TenantHeaderError::NotUuid)?;
    let tenant_uuid = Uuid::from_str(tenant_id_str).map_err(|_| TenantHeaderError::NotUuid)?;
    Ok(tenant_uuid)
}

pub fn parse_db_output_of_insert_create_and_return_uuid(rows: &[SimpleQueryMessage]) -> Result<Uuid, DaoError> {
    let row = rows.get(1).ok_or_else(|| {
        DaoError::PostgresQueryError("no 2nd statement in script but required".to_string())
    })?;
    match row {
        SimpleQueryMessage::Row(a) => {
            let uuid_str = a.get(0).ok_or_else(|| {
                DaoError::PostgresQueryError(
                    "should have returned a result but was none".to_string(),
                )
            })?;
            Uuid::parse_str(uuid_str).map_err(|_| {
                DaoError::PostgresQueryError("unable to convert str to uuid".to_string())
            })
        }
        SimpleQueryMessage::CommandComplete(_) => Err(DaoError::PostgresQueryError(
            "should have returned a result but was a command".to_string(),
        )),
        _ => Err(DaoError::PostgresQueryError(
            "should have returned a result but was a command".to_string(),
        )),
    }
}


pub fn flatten_errors(validation_errors: &ValidationErrors) -> anyhow::Result<Vec<ValidationError>> {
    let mut result = Vec::new();
    let mut stack = vec![(validation_errors, 0)]; // Each element is a tuple (errors, depth)

    while let Some((errors, depth)) = stack.pop() {
        if depth >= 9 {
            bail!("can't handle this much depth");
        }

        for kind in errors.errors().values() {
            match kind {
                ValidationErrorsKind::Struct(inner) => stack.push((inner.as_ref(), depth + 1)),
                ValidationErrorsKind::List(inner_map) => {
                    for inner in inner_map.values() {
                        stack.push((inner.as_ref(), depth + 1));
                    }
                }
                ValidationErrorsKind::Field(inner_errors) => {
                    result.extend(inner_errors.clone());
                }
            }
        }
    }

    Ok(result)
}


#[cfg(test)]
mod utils_tests {
    use serde::{Deserialize, Serialize};
    use spectral::assert_that;
    use spectral::prelude::VecAssertions;
    use validator::Validate;

    use crate::common_utils::utils::flatten_errors;

    #[derive(Debug, Serialize, Deserialize, Validate)]
    pub struct TestVal {
        #[validate(range(min = 1, max = 2000, message = "page no should be cannot be less than 1 and more than 2000"))]
        pub page_no: u32,
        #[validate(range(min = 1, max = 100, message = "per_page count cannot be less than 1 and more than 2000"))]
        pub per_page: u32,
        #[validate]
        pub jk: Option<Box<TestVal>>,
    }


    #[test]
    fn test_flatten_errors() {
        let kp = TestVal { per_page: 0, page_no: 0, jk: Some(Box::new(TestVal { page_no: 0, per_page: 0, jk: None })) };
        let errs = kp.validate().unwrap_err();
        let fla = flatten_errors(&errs).unwrap();
        assert_that!(fla).has_length(4)
    }
}