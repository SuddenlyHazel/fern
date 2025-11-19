use wasmtime::component::HasData;
use wasmtime_wasi::ResourceTable;

pub mod database;
pub mod error;
pub mod rows;
pub mod value;

pub use database::DatabaseResource;
pub use rows::RowsResource;

pub struct SqliteState {
    resource_table: ResourceTable,
}

impl SqliteState {
    pub fn new() -> Self {
        Self {
            resource_table: ResourceTable::new(),
        }
    }
}

impl HasData for SqliteState {
    type Data<'a> = &'a mut Self;
}
