use chrono::{DateTime, Utc};

use crate::data::Data;

// Module rows which have been previously deployed but now removed
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ModuleRow {
    pub id: i64,
    pub parent_id: i64,
    pub module: Vec<u8>,
    pub module_hash: String,
    pub created_at: DateTime<Utc>,
}

impl ModuleRow {
    /// Create a new module history entry
    pub fn create(
        data: &Data,
        parent_id: i64,
        module: Vec<u8>,
        module_hash: String,
    ) -> rusqlite::Result<Self> {
        let created_at = Utc::now();
        let created_at_str = created_at.to_rfc3339();

        let conn = &data.conn;
        conn.execute(
            "INSERT INTO module_history (parent_id, module, module_hash, created_at) VALUES (?1, ?2, ?3, ?4)",
            (&parent_id, &module, &module_hash, &created_at_str),
        )?;

        Ok(Self {
            id: conn.last_insert_rowid(),
            parent_id,
            module,
            module_hash,
            created_at,
        })
    }

    /// Get the latest module row by guest id (for potential rollback functionality)
    pub fn latest_by_guest_id(data: &Data, guest_id: i64) -> rusqlite::Result<Option<ModuleRow>> {
        let conn = &data.conn;
        let mut stmt = conn.prepare(
            "SELECT id, parent_id, module, module_hash, created_at
             FROM module_history
             WHERE parent_id = ?1
             ORDER BY created_at DESC
             LIMIT 1"
        )?;
        
        let mut rows = stmt.query_map([guest_id], |row| {
            let created_at_str: String = row.get(4)?;
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                    4,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                ))?
                .with_timezone(&Utc);

            Ok(ModuleRow {
                id: row.get(0)?,
                parent_id: row.get(1)?,
                module: row.get(2)?,
                module_hash: row.get(3)?,
                created_at,
            })
        })?;

        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{Data, GuestRow};

    #[test]
    fn test_module_row_operations() {
        let data = Data::new_memory();
        
        // Create a guest first
        let guest = GuestRow::create(
            &data,
            "test_guest".to_string(),
            vec![1, 2, 3, 4],
        ).expect("failed to create guest");

        // Create a module history entry
        let module_data = vec![5, 6, 7, 8];
        let module_hash = blake3::hash(&module_data).to_string();
        
        let module_row = ModuleRow::create(
            &data,
            guest.id,
            module_data.clone(),
            module_hash.clone(),
        ).expect("failed to create module row");

        assert_eq!(module_row.parent_id, guest.id);
        assert_eq!(module_row.module, module_data);
        assert_eq!(module_row.module_hash, module_hash);

        // Test getting latest module by guest id
        let latest = ModuleRow::latest_by_guest_id(&data, guest.id)
            .expect("failed to query latest module")
            .expect("no module found");

        assert_eq!(latest.id, module_row.id);
        assert_eq!(latest.parent_id, guest.id);
        assert_eq!(latest.module, module_data);

        // Create another module entry for the same guest
        let module_data2 = vec![9, 10, 11, 12];
        let module_hash2 = blake3::hash(&module_data2).to_string();
        
        let module_row2 = ModuleRow::create(
            &data,
            guest.id,
            module_data2.clone(),
            module_hash2.clone(),
        ).expect("failed to create second module row");

        // Latest should now return the second module
        let latest = ModuleRow::latest_by_guest_id(&data, guest.id)
            .expect("failed to query latest module")
            .expect("no module found");

        assert_eq!(latest.id, module_row2.id);
        assert_eq!(latest.module, module_data2);

        // Test with non-existent guest id
        let no_module = ModuleRow::latest_by_guest_id(&data, 999)
            .expect("failed to query non-existent guest");
        assert!(no_module.is_none());
    }
}