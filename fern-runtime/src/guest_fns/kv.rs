use crate::guest::GuestConfig;
use extism::{PTR, PluginBuilder, UserData, host_fn};
use extism_convert::Json;
use redb::{Database, ReadableDatabase, TableDefinition};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KvStoreInput {
    pub table: String,
    pub key: String,
    pub value: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KvReadInput {
    pub table: String,
    pub key: String,
}

pub struct GuestKvData {
    pub db: Database,
}

impl GuestKvData {
    pub fn new() -> Self {
        let file = tempfile::NamedTempFile::new().expect("failed to get temp file");
        let db = Database::create(file.path()).expect("failed to create db");
        Self { db }
    }

    pub fn new_with_config(config: &GuestConfig) -> Self {
        if let Some(ref host_data_path) = config.host_data_path {
            // Create absolute path: host_data_path + guest_name + "db.redb"
            let mut full_path = host_data_path.clone();
            full_path.push(&config.name);
            full_path.push("db.redb");

            // Ensure the directory exists
            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent).expect("failed to create database directory");
            }

            let db = Database::create(&full_path).expect("failed to create file-based db");
            Self { db }
        } else {
            // Fall back to in-memory database
            Self::new()
        }
    }
}

pub fn attach_guest_kv(builder: PluginBuilder, config: GuestConfig) -> PluginBuilder {
    let user_data = UserData::new(GuestKvData::new_with_config(&config));
    builder
        .with_function("kv_store", [PTR], [PTR], user_data.clone(), kv_store)
        .with_function("kv_read", [PTR], [PTR], user_data.clone(), kv_read)
}

host_fn!(kv_store(user_data : GuestKvData; input: Json<KvStoreInput>) -> bool {
  store(user_data, input.0.table, input.0.key, input.0.value)
});

fn store(
    user_data: UserData<GuestKvData>,
    table: String,
    key: String,
    value: Value,
) -> Result<bool, extism::Error> {
    let data = user_data.get()?;
    let data = data.lock().unwrap();

    let tx = data.db.begin_write()?;
    {
        let mut table: redb::Table<'_, String, &[u8]> =
            tx.open_table(TableDefinition::new(&table))?;
        let bytes = serde_json::to_vec(&value)?;
        table.insert(key, bytes.as_slice())?;
    }
    tx.commit()?;
    Ok(true)
}

fn read(
    user_data: UserData<GuestKvData>,
    table: String,
    key: String,
) -> Result<Option<Value>, extism::Error> {
    let data = user_data.get()?;
    let data = data.lock().unwrap();

    let tx = data.db.begin_read()?;

    let table: redb::ReadOnlyTable<String, &[u8]> = tx.open_table(TableDefinition::new(&table))?;
    let res = match table.get(key)? {
        Some(res) => {
            let res: Value = serde_json::from_slice(res.value())?;
            Some(res)
        }
        None => None,
    };

    Ok(res)
}

host_fn!(kv_read(user_data : GuestKvData; input: Json<KvReadInput>) -> Option<Value> {
  read(user_data, input.0.table, input.0.key)
});
