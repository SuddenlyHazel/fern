use turso::params::IntoValue;

use crate::fern::base::sqlite::SqliteValue;

pub struct GuestValue(turso::Value);

impl Into<turso::Value> for GuestValue {
    fn into(self) -> turso::Value {
        self.0
    }
}

impl IntoValue for SqliteValue {
    fn into_value(self) -> turso::Result<turso::Value> {
        Ok(match self {
            SqliteValue::Null => turso::Value::Null,
            SqliteValue::Integer(v) => turso::Value::Integer(v),
            SqliteValue::Real(v) => turso::Value::Real(v),
            SqliteValue::Text(v) => turso::Value::Text(v),
            SqliteValue::Blob(items) => turso::Value::Blob(items),
        })
    }
}
