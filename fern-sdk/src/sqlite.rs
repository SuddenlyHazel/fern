use crate::bindings::fern::base::sqlite::SqliteValue;


// Convert helpers

impl PartialEq for SqliteValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Integer(l0), Self::Integer(r0)) => l0 == r0,
            (Self::Real(l0), Self::Real(r0)) => l0 == r0,
            (Self::Text(l0), Self::Text(r0)) => l0 == r0,
            (Self::Blob(l0), Self::Blob(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl Eq for SqliteValue {

}

impl Into<SqliteValue> for u64 {
    fn into(self) -> SqliteValue {
        SqliteValue::Integer(self as i64)
    }
}

impl Into<SqliteValue> for u32 {
    fn into(self) -> SqliteValue {
        SqliteValue::Integer(self as i64)
    }
}

impl Into<SqliteValue> for u16 {
    fn into(self) -> SqliteValue {
        SqliteValue::Integer(self as i64)
    }
}

impl Into<SqliteValue> for u8 {
    fn into(self) -> SqliteValue {
        SqliteValue::Integer(self as i64)
    }
}

impl Into<SqliteValue> for String {
    fn into(self) -> SqliteValue {
        SqliteValue::Text(self)
    }
}

impl Into<SqliteValue> for &str {
    fn into(self) -> SqliteValue {
        SqliteValue::Text(self.to_string())
    }
}

impl Into<SqliteValue> for Vec<u8> {
    fn into(self) -> SqliteValue {
        SqliteValue::Blob(self)
    }
}

impl Into<SqliteValue> for i64 {
    fn into(self) -> SqliteValue {
        SqliteValue::Integer(self as i64)
    }
}

impl Into<SqliteValue> for i32 {
    fn into(self) -> SqliteValue {
        SqliteValue::Integer(self as i64)
    }
}

impl Into<SqliteValue> for i16 {
    fn into(self) -> SqliteValue {
        SqliteValue::Integer(self as i64)
    }
}

impl Into<SqliteValue> for i8 {
    fn into(self) -> SqliteValue {
        SqliteValue::Integer(self as i64)
    }
}