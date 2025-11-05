use std::path::Path;

use rusqlite::Connection;

pub mod guest_row;
pub use guest_row::GuestRow;

pub mod module_row;
pub use module_row::ModuleRow;

pub struct Data {
    pub(crate) conn: Connection,
}

impl Data {
    pub fn new_memory() -> Self {
        let conn = Connection::open_in_memory().expect("failed to open memory db");
        let conn = setup(conn);

        Self { conn }
    }

    pub fn new_path(path: &Path) -> Self {
        let conn = Connection::open(path).expect("failed to open file db");
        let conn = setup(conn);
        Self { conn }
    }
}

fn setup(conn: Connection) -> Connection {
    conn.execute(
        r#"
  create table if not exists guests (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL,
    module BLOB NOT NULL,
    module_hash TEXT NOT NULL
  )
  "#,
        (),
    )
    .expect("failed to create guests table");

    conn.execute(
        r#"
  create table if not exists module_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    parent_id INTEGER NOT NULL,
    module BLOB NOT NULL,
    module_hash TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (parent_id) REFERENCES guests (id)
  )
  "#,
        (),
    )
    .expect("failed to create module_history table");

    conn
}
