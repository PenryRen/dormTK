use crate::error::ApiError;
use serde::de::DeserializeOwned;
use sqlx::Row;

pub fn text(row: &sqlx::postgres::PgRow, name: &str) -> Result<String, sqlx::Error> {
    row.try_get(name)
}

pub fn opt_text(row: &sqlx::postgres::PgRow, name: &str) -> Result<Option<String>, sqlx::Error> {
    row.try_get(name)
}

pub fn enum_value<T>(row: &sqlx::postgres::PgRow, name: &str) -> Result<T, sqlx::Error>
where
    T: DeserializeOwned,
{
    let value: String = row.try_get(name)?;
    serde_json::from_value(serde_json::Value::String(value))
        .map_err(|error| sqlx::Error::Decode(Box::new(error)))
}

pub fn validate_non_empty(value: &str, field: &str) -> Result<(), ApiError> {
    if value.trim().is_empty() {
        return Err(ApiError::bad_request(format!("{field} must not be empty")));
    }

    Ok(())
}
