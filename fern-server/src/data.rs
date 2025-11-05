use std::path::Path;

use rusqlite::Connection;

pub struct Data {
    conn: Connection,
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
    module BLOB NOT NULL
  )
  "#,
        (),
    )
    .expect("failed to create table");
    conn
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct GuestRow {
    pub id: i64,
    pub name: String,
    pub module: Vec<u8>,
}

impl GuestRow {
    pub fn create(data: &Data, name: String, module: Vec<u8>) -> rusqlite::Result<Self> {
        let conn = &data.conn;
        conn.execute(
            "INSERT INTO guests (name, module) VALUES (?1, ?2)",
            (&name, &module),
        )?;

        Ok(Self {
            id: conn.last_insert_rowid(),
            name,
            module,
        })
    }

    pub fn by_id(data: &Data, id: i64) -> rusqlite::Result<Option<GuestRow>> {
        let conn = &data.conn;
        let mut stmt = conn.prepare("SELECT id, name, module FROM guests WHERE id = ?1")?;
        let mut rows = stmt.query_map([id], |row| {
            Ok(GuestRow {
                id: row.get(0)?,
                name: row.get(1)?,
                module: row.get(2)?,
            })
        })?;

        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    pub fn by_name(data: &Data, name: &str) -> rusqlite::Result<Option<GuestRow>> {
        let conn = &data.conn;
        let mut stmt = conn.prepare("SELECT id, name, module FROM guests WHERE name = ?1")?;
        let mut rows = stmt.query_map([name], |row| {
            Ok(GuestRow {
                id: row.get(0)?,
                name: row.get(1)?,
                module: row.get(2)?,
            })
        })?;

        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    pub fn update_module_by_name(data: &Data, name: &str, module: &[u8]) -> rusqlite::Result<bool> {
        let conn = &data.conn;
        let rows_affected = conn.execute(
            "UPDATE guests SET module = ?1 WHERE name = ?2",
            (&module, name),
        )?;
        Ok(rows_affected == 1)
    }

    pub fn all_with_pagination(
        data: &Data,
        limit: i64,
        offset: i64,
    ) -> rusqlite::Result<Vec<GuestRow>> {
        let conn = &data.conn;
        let mut stmt =
            conn.prepare("SELECT id, name, module FROM guests ORDER BY id LIMIT ?1 OFFSET ?2")?;
        let rows = stmt.query_map([limit, offset], |row| {
            Ok(GuestRow {
                id: row.get(0)?,
                name: row.get(1)?,
                module: row.get(2)?,
            })
        })?;

        let mut guests = Vec::new();
        for row in rows {
            guests.push(row?);
        }
        Ok(guests)
    }
}

#[test]
fn sql_guest_row() {
    let data = Data::new_memory();
    let guest = GuestRow::create(
        &data,
        "test module".to_string(),
        uuid::Uuid::new_v4().as_bytes().into(),
    )
    .expect("failed to create guest row");

    let got_guest = GuestRow::by_id(&data, guest.id)
        .expect("failed to query guest row")
        .expect("couldn't find guest row");

    assert_eq!(guest, got_guest);

    let got_guest = GuestRow::by_name(&data, "test module")
        .expect("failed to execute sql")
        .expect("failed to find row");

    assert_eq!(guest, got_guest);

    let new_module = uuid::Uuid::new_v4();
    GuestRow::update_module_by_name(&data, "test module", new_module.as_bytes())
        .expect("failed to update module bytes");

    let got_guest = GuestRow::by_name(&data, "test module")
        .expect("failed to execute sql")
        .expect("failed to find row");

    assert_eq!(got_guest.module, new_module.as_bytes());

    // Test pagination - create a few more guests first
    GuestRow::create(&data, "guest2".to_string(), vec![1, 2, 3]).expect("failed to create guest2");
    GuestRow::create(&data, "guest3".to_string(), vec![4, 5, 6]).expect("failed to create guest3");

    // Test getting all guests with pagination
    let all_guests =
        GuestRow::all_with_pagination(&data, 10, 0).expect("failed to get paginated guests");
    assert_eq!(all_guests.len(), 3); // Should have 3 guests total

    // Test pagination with limit
    let first_two = GuestRow::all_with_pagination(&data, 2, 0).expect("failed to get first page");
    assert_eq!(first_two.len(), 2);

    // Test pagination with offset
    let last_one = GuestRow::all_with_pagination(&data, 2, 2).expect("failed to get second page");
    assert_eq!(last_one.len(), 1);
}
