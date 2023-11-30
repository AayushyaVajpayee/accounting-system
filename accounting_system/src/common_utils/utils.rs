use std::time::{Duration, SystemTime, SystemTimeError, UNIX_EPOCH};
use thiserror::Error;
use tokio_postgres::SimpleQueryMessage;
use tracing::error;
use uuid::Uuid;
use crate::common_utils::dao_error::DaoError;

#[derive(Debug,Error)]
pub enum TimeError{
    #[error("duration in opposite {:?}",0)]
    ForwardTime(Duration)
}
///in microseconds
pub fn get_current_time_us() -> Result<i64,TimeError> {
  let current_time= SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|a|{
            error!(?a,%a,"error during time fetching");
           TimeError::ForwardTime(a.duration())
        })?.as_micros() as i64;
    Ok(current_time)
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
            Uuid::parse_str(uuid_str).map_err(|a| {
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