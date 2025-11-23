use log::debug;
use turso::params_from_iter;
use wasmtime::component::Resource;

use crate::{
    fern::{
        self,
        base::sqlite::{DbError, DbOpenError, HostDatabase, SqliteValue},
    },
    sqlite::{RowsResource, SqliteState},
};

pub struct DatabaseResource {
    pub(crate) name: String,
    pub(crate) db: turso::Database,
    pub(crate) connection: turso::Connection,
}

impl HostDatabase for &mut SqliteState {
    async fn execute(
        &mut self,
        self_: Resource<DatabaseResource>,
        statement: String,
        params: Vec<SqliteValue>,
    ) -> Result<u64, DbError> {
        let db_resource = self
            .resource_table
            .get(&self_)
            .expect("failed to get db resource");

        let res = db_resource
            .connection
            .execute(&statement, params_from_iter(params))
            .await
            .map_err(|e| {
                debug!("db returned error {e}");
                e
            })?;
        Ok(res)
    }
    async fn query(
        &mut self,
        self_: Resource<DatabaseResource>,
        statement: String,
        params: Vec<SqliteValue>,
    ) -> Result<Resource<fern::base::sqlite::Rows>, DbError> {
        debug!("host executing {statement}");
        let db_resource = self
            .resource_table
            .get(&self_)
            .expect("failed to find database resource");

        let rows = db_resource
            .connection
            .query(&statement, params_from_iter(params))
            .await
            .map_err(|e| {
                debug!("db returned error {e}");
                e
            })?;

        let rows_resource = RowsResource { rows };

        let resource = self.resource_table.push(rows_resource).unwrap();
        Ok(resource)
    }

    async fn drop(&mut self, rep: Resource<DatabaseResource>) -> wasmtime::Result<()> {
        debug!(
            "dropping database resource {} owned {}",
            rep.rep(),
            rep.owned()
        );
        let db_resource = self.resource_table.delete(rep)?;
        // I'm not sure we really need to do this. But, it feels like good practice.
        db_resource.connection.cacheflush()?;
        Ok(())
    }
}

impl fern::base::sqlite::Host for &mut SqliteState {
    async fn open_db(&mut self, name: String) -> Result<Resource<DatabaseResource>, DbOpenError> {
        // TODO: Initialize the actual turso database
        let db = turso::Builder::new_local(":memory:")
            .build()
            .await
            .map_err(|_| DbOpenError::ConnectionFailure)?;

        let connection = db.connect().map_err(|_| DbOpenError::ConnectionFailure)?;

        let db_resource = DatabaseResource {
            name: name.clone(),
            db,
            connection,
        };

        // Return the resource handle
        Ok(self
            .resource_table
            .push(db_resource)
            .expect("failed to store database resource in table"))
    }
}
