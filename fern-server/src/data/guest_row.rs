use rusqlite::Connection;

use crate::data::{Data, ModuleRow};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct GuestRow {
    pub id: i64,
    pub name: String,
    pub module: Vec<u8>,
    pub module_hash: String,
}

impl GuestRow {
    pub fn create(data: &Data, name: String, module: Vec<u8>) -> rusqlite::Result<Self> {
        let module_hash = blake3::hash(&module).to_string();

        let conn = &data.conn;
        conn.execute(
            "INSERT INTO guests (name, module, module_hash) VALUES (?1, ?2, ?3)",
            (&name, &module, &module_hash),
        )?;

        Ok(Self {
            id: conn.last_insert_rowid(),
            name,
            module,
            module_hash,
        })
    }

    pub fn by_id(data: &Data, id: i64) -> rusqlite::Result<Option<GuestRow>> {
        let conn = &data.conn;
        let mut stmt = conn.prepare("SELECT id, name, module, module_hash FROM guests WHERE id = ?1")?;
        let mut rows = stmt.query_map([id], |row| {
            Ok(GuestRow {
                id: row.get(0)?,
                name: row.get(1)?,
                module: row.get(2)?,
                module_hash: row.get(3)?,
            })
        })?;

        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    pub fn by_name(data: &Data, name: &str) -> rusqlite::Result<Option<GuestRow>> {
        let conn = &data.conn;
        let mut stmt = conn.prepare("SELECT id, name, module, module_hash FROM guests WHERE name = ?1")?;
        let mut rows = stmt.query_map([name], |row| {
            Ok(GuestRow {
                id: row.get(0)?,
                name: row.get(1)?,
                module: row.get(2)?,
                module_hash: row.get(3)?,
            })
        })?;

        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    pub fn update_module_by_name(data: &Data, name: &str, module: &[u8]) -> rusqlite::Result<bool> {
        let new_module_hash = blake3::hash(module).to_string();
        
        // First, get the current guest to save their old module to history
        if let Some(current_guest) = Self::by_name(data, name)? {
            // Save the current module to history before updating
            ModuleRow::create(
                data,
                current_guest.id,
                current_guest.module,
                current_guest.module_hash,
            )?;
        }
        
        // Now update the guest with the new module
        let conn = &data.conn;
        let rows_affected = conn.execute(
            "UPDATE guests SET module = ?1, module_hash = ?2 WHERE name = ?3",
            (&module, &new_module_hash, name),
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
            conn.prepare("SELECT id, name, module, module_hash FROM guests ORDER BY id LIMIT ?1 OFFSET ?2")?;
        let rows = stmt.query_map([limit, offset], |row| {
            Ok(GuestRow {
                id: row.get(0)?,
                name: row.get(1)?,
                module: row.get(2)?,
                module_hash: row.get(3)?,
            })
        })?;

        let mut guests = Vec::new();
        for row in rows {
            guests.push(row?);
        }
        Ok(guests)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::Data;

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
        GuestRow::create(&data, "guest2".to_string(), vec![1, 2, 3])
            .expect("failed to create guest2");
        GuestRow::create(&data, "guest3".to_string(), vec![4, 5, 6])
            .expect("failed to create guest3");

        // Test getting all guests with pagination
        let all_guests =
            GuestRow::all_with_pagination(&data, 10, 0).expect("failed to get paginated guests");
        assert_eq!(all_guests.len(), 3); // Should have 3 guests total

        // Test pagination with limit
        let first_two =
            GuestRow::all_with_pagination(&data, 2, 0).expect("failed to get first page");
        assert_eq!(first_two.len(), 2);

        // Test pagination with offset
        let last_one =
            GuestRow::all_with_pagination(&data, 2, 2).expect("failed to get second page");
        assert_eq!(last_one.len(), 1);
    }
}