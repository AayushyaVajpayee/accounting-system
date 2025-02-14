use anyhow::{anyhow, Error as AnyhowError};
use deadpool_postgres::PoolError;
use std::fmt::Error;
use tokio_postgres::error::SqlState;

#[derive(Debug, thiserror::Error)]
pub enum DaoError {
    #[error("error while fetching db connection. {0}")]
    ConnectionPool(String),
    #[error("error while executing query. {0}")]
    PostgresQueryError(String),
    #[error("cannot convert entity to db row {0}")]
    InvalidEntityToDbRowConversion(&'static str),
    #[error("unique constraint violated {constraint_name:?}")]
    UniqueConstraintViolated { constraint_name: String },
    #[error("query returned nothing")]
    ReturnedValueNone,
    #[error(transparent)]
    AnyhowError(#[from] AnyhowError),
}

impl From<PoolError> for DaoError {
    fn from(value: PoolError) -> Self {
        DaoError::ConnectionPool(value.to_string())
    }
}

impl From<std::fmt::Error> for DaoError {
    fn from(value: Error) -> Self {
        DaoError::AnyhowError(anyhow!(value))
    }
}

impl From<tokio_postgres::Error> for DaoError {
    fn from(value: tokio_postgres::Error) -> Self {
        if let Some(k) = value.as_db_error() {
            if k.code().code() == SqlState::UNIQUE_VIOLATION.code() {
                return DaoError::UniqueConstraintViolated {
                    constraint_name: k.constraint().unwrap().to_string(),
                };
            }
        }
        DaoError::PostgresQueryError(value.to_string())
    }
}
