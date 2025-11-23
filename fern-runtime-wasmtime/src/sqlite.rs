//! SQLite functionality using Turso's Rust rewrite for SQLite.
//! For more information, see: <https://github.com/tursodatabase/turso>
//!
//! It's worth noting that Turso's implmentation is still very much a WIP..
//!
//!

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
