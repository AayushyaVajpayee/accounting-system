use std::time::{Duration, SystemTime, SystemTimeError, UNIX_EPOCH};
use thiserror::Error;
use tracing::error;
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