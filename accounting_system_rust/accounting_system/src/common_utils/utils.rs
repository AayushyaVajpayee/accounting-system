use std::future::{ Ready};
use std::str::FromStr;
use std::sync::Arc;
use anyhow::{anyhow, bail, Context};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use actix_web::{FromRequest, HttpRequest, ResponseError};
use actix_web::body::MessageBody;
use actix_web::dev::{Payload, ServiceRequest, ServiceResponse};
use actix_web::error::ErrorInternalServerError;
use actix_web::http::StatusCode;
use actix_web_lab::middleware::Next;
use chrono::{Datelike, NaiveDate, TimeZone, Utc};
use serde_json::Value;
use thiserror::Error;
use tokio_postgres::SimpleQueryMessage;
use tracing::error;
use uuid::Uuid;
use validator::{ValidationError, ValidationErrors, ValidationErrorsKind};
use crate::accounting::user::user_service::UserService;

use crate::common_utils::dao_error::DaoError;
use crate::tenant::tenant_service::TenantService;

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

#[allow(dead_code)]
pub fn current_indian_date() -> NaiveDate {
    let utc_now = Utc::now().naive_utc();
    let current_date = chrono_tz::Asia::Kolkata.from_utc_datetime(&utc_now).date_naive();
    current_date
}

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum TenantIdHeaderError {
    #[error("x-acc-tenant-id header not present in request")]
    NotPresent,
    #[error("x-acc-tenant-id header does not have a valid uuid")]
    NotUuid,
    #[error("tenant id not found in system")]
    NotInDb
}
#[derive(Debug, Error)]
pub enum UserIdHeaderError{
    #[error("x-acc-user-id header not present in request")]
    NotPresent,
    #[error("x-acc-user-id header does not have a valid uuid")]
    NotUuid,
    #[error("user id not found in system")]
    NotInDb
}

impl ResponseError for UserIdHeaderError {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}
impl ResponseError for TenantIdHeaderError {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

pub fn extract_tenant_id_from_header(request: &HttpRequest) -> Result<TenantId, TenantIdHeaderError> {
    let p = request.headers()
        .get("x-acc-tenant-id")
        .ok_or(TenantIdHeaderError::NotPresent)?;
    let tenant_id_str = p.to_str().map_err(|_| TenantIdHeaderError::NotUuid)?;
    let tenant_uuid = Uuid::from_str(tenant_id_str).map_err(|_| TenantIdHeaderError::NotUuid)?;
    Ok(TenantId(tenant_uuid))
}
pub fn extract_user_id_from_header(request:&HttpRequest)->Result<UserId,UserIdHeaderError>{
    let p = request.headers()
        .get("x-acc-user-id")
        .ok_or(UserIdHeaderError::NotPresent)?;
    let tenant_id_str = p.to_str().map_err(|_| UserIdHeaderError::NotUuid)?;
    let tenant_uuid = Uuid::from_str(tenant_id_str).map_err(|_| UserIdHeaderError::NotUuid)?;
    Ok(UserId(tenant_uuid))
}
pub struct TenantId(Uuid);
impl TenantId{
   pub fn inner(&self)->Uuid{
        self.0
    }
}
pub struct UserId(Uuid);
impl UserId{
    pub fn inner(&self)->Uuid{
        self.0
    }
}
impl FromRequest for TenantId {
    type Error = TenantIdHeaderError;
    type Future = Ready<Result<TenantId,Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let p = extract_tenant_id_from_header(req);
        std::future::ready(p)
    }
}
impl FromRequest for UserId{
    type Error = UserIdHeaderError;
    type Future = Ready<Result<UserId,Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let p = extract_user_id_from_header(req);
        std::future::ready(p)
    }
}

pub async fn tenant_user_header_middleware(
     req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, actix_web::Error> {
    let tenant_service:&Arc<dyn TenantService> = req.app_data()
        .ok_or(ErrorInternalServerError(anyhow!("tenant service not found")))?;
    let user_service:&Arc<dyn UserService> = req.app_data()
        .ok_or(ErrorInternalServerError(anyhow!("user service not found")))?;
    let tenant_id =extract_tenant_id_from_header(req.request())?;
    let user_id = extract_user_id_from_header(req.request())?;
    let _tenant =tenant_service.get_tenant_by_id(tenant_id.0)
        .await?
        .ok_or(TenantIdHeaderError::NotInDb)?;
    let _user=user_service.get_user_by_id(user_id.0,tenant_id.inner())
        .await?
        .ok_or(UserIdHeaderError::NotInDb)?;
    // pre-processing
    let resp = next.call(req).await;
    resp
    // post-processing
}

pub fn parse_db_output_of_insert_create_and_return_uuid(rows: &[SimpleQueryMessage]) -> Result<Uuid, DaoError> {
    let closure =|a:&str|{
        Uuid::parse_str(a).map_err(|_| {
            DaoError::PostgresQueryError("unable to convert str to uuid".to_string())
        })
    };
    parse_rows(rows,closure)
}
fn parse_rows<T, F>(rows: &[SimpleQueryMessage], parse_fn: F)
                    -> Result<T, DaoError>
    where F: FnOnce(&str) -> Result<T, DaoError> {
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
            parse_fn(uuid_str)
        }
        SimpleQueryMessage::CommandComplete(_) => Err(DaoError::PostgresQueryError(
            "should have returned a result but was a command".to_string(),
        )),
        _ => Err(DaoError::PostgresQueryError(
            "should have returned a result but was a command".to_string(),
        )),
    }
}

pub fn parse_db_output_of_insert_create_and_return_json(rows: &[SimpleQueryMessage]) -> Result<Value, DaoError> {
    let closure =|a:&str|{
        let value = serde_json::from_str(a)
            .context("error during deserialising db value")?;
        Ok(value)
    };
    parse_rows(rows,closure)
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