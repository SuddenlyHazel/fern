use log::debug;

use crate::{
    fern::base::sqlite::{DbError, HostRows, HostWithStore, Row as GuestRow, Rows, SqliteValue},
    sqlite::SqliteState,
};

pub struct RowsResource {
    pub rows: turso::Rows,
}

impl HostRows for SqliteState {
    async fn next(
        &mut self,
        self_: wasmtime::component::Resource<Rows>,
    ) -> Result<Option<GuestRow>, DbError> {
        let rows = self
            .resource_table
            .get_mut(&self_)
            .expect("failed to find rows resource");

        let next_row = rows
            .rows
            .next()
            .await
            .map_err(|e| {
                debug!("failed to get next row {e}");
                e
            })?
            .map(|row| {
                let mut values = vec![];
                for i in 0..row.column_count() {
                    values.push(SqliteValue::from(
                        row.get_value(i).expect("failed to get value from sql row"),
                    ));
                }
                GuestRow { values }
            });

        debug!("{next_row:?}");

        Ok(next_row)
    }

    async fn drop(
        &mut self,
        rep: wasmtime::component::Resource<Rows>,
    ) -> Result<(), wasmtime::Error> {
        debug!("dropping rows resource {} owned {}", rep.rep(), rep.owned());
        self.resource_table.delete(rep)?;
        Ok(())
    }
}

impl From<turso::Value> for SqliteValue {
    fn from(value: turso::Value) -> Self {
        match value {
            turso::Value::Null => SqliteValue::Null,
            turso::Value::Integer(v) => SqliteValue::Integer(v),
            turso::Value::Real(v) => SqliteValue::Real(v),
            turso::Value::Text(v) => SqliteValue::Text(v),
            turso::Value::Blob(items) => SqliteValue::Blob(items),
        }
    }
}
