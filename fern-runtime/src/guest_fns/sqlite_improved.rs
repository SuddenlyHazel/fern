use base64;
use extism::{FromBytes, PTR, PluginBuilder, ToBytes, UserData, host_fn};
use extism_convert::Json;
use log::info;
use rusqlite::{ToSql, params_from_iter, types::ToSqlOutput};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Instant;
use crate::guest::GuestConfig;

pub struct GuestSqliteDbImproved {
    pub db: rusqlite::Connection,
    pub stats: QueryStats,
}

#[derive(Debug, Default)]
pub struct QueryStats {
    pub total_queries: u64,
    pub total_execution_time_ms: f64,
    pub query_count_by_type: HashMap<String, u64>,
}

impl GuestSqliteDbImproved {
    pub fn new() -> Self {
        let db = rusqlite::Connection::open_in_memory().expect("failed to create in-memory db");

        // Set SQLite pragmas for better performance and safety
        db.execute_batch(
            "
            PRAGMA foreign_keys = ON;
            PRAGMA journal_mode = MEMORY;
            PRAGMA synchronous = FULL;
            PRAGMA temp_store = MEMORY;
            PRAGMA cache_size = -64000;
        ",
        )
        .expect("failed to set SQLite pragmas");

        Self {
            db,
            stats: QueryStats::default(),
        }
    }

    pub fn new_with_config(config: &GuestConfig) -> Self {
        let db = if let Some(ref host_data_path) = config.host_data_path {
            // Create absolute path: host_data_path + guest_name + "db.sqlite"
            let mut full_path = host_data_path.clone();
            full_path.push(&config.name);
            full_path.push("db.sqlite");
            
            // Ensure the directory exists
            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent).expect("failed to create database directory");
            }
            
            rusqlite::Connection::open(&full_path).expect("failed to create file-based SQLite db")
        } else {
            // Fall back to in-memory database
            rusqlite::Connection::open_in_memory().expect("failed to create in-memory db")
        };

        // Set SQLite pragmas for better performance and safety
        let pragma_batch = if config.host_data_path.is_some() {
            // File-based database pragmas
            "
            PRAGMA foreign_keys = ON;
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA temp_store = MEMORY;
            PRAGMA cache_size = -64000;
            PRAGMA busy_timeout = 30000;
            "
        } else {
            // In-memory database pragmas
            "
            PRAGMA foreign_keys = ON;
            PRAGMA journal_mode = MEMORY;
            PRAGMA synchronous = FULL;
            PRAGMA temp_store = MEMORY;
            PRAGMA cache_size = -64000;
            "
        };

        db.execute_batch(pragma_batch)
            .expect("failed to set SQLite pragmas");

        Self {
            db,
            stats: QueryStats::default(),
        }
    }

    pub fn record_query(&mut self, query_type: &str, execution_time_ms: f64) {
        self.stats.total_queries += 1;
        self.stats.total_execution_time_ms += execution_time_ms;
        *self
            .stats
            .query_count_by_type
            .entry(query_type.to_string())
            .or_insert(0) += 1;
    }
}

// Input struct for table name operations
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TableNameInput {
    #[serde(rename = "tableName")]
    pub table_name: String,
}

// Empty input for functions that don't require parameters
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmptyInput {
    pub _dummy: Option<String>,
}

// Input struct for transaction ID operations
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransactionIdInput {
    #[serde(rename = "transactionId")]
    pub transaction_id: String,
}

// Enhanced parameter type with type hints for better cross-language support
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TypedSqlParam {
    pub value: Value,
    #[serde(rename = "typeHint")]
    pub type_hint: Option<SqlTypeHint>, // "text", "integer", "real", "blob", "boolean", "datetime", "nullValue"
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub enum SqlTypeHint {
    #[default]
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "integer")]
    Integer,
    #[serde(rename = "real")]
    Real,
    #[serde(rename = "blob")]
    Blob,
    #[serde(rename = "boolean")]
    Boolean,
    #[serde(rename = "datetime")]
    Datetime,
    #[serde(rename = "_null")]
    Null,
}

impl FromBytes<'_> for TypedSqlParam {
    fn from_bytes(bytes: &[u8]) -> Result<Self, extism::Error> {
        let json_str = std::str::from_utf8(bytes)?;
        serde_json::from_str(json_str).map_err(|e| extism::Error::msg(e.to_string()))
    }
}

impl ToSql for TypedSqlParam {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        // Extract the actual value from the nested "value" field if it exists
        let actual_value = if let Some(obj) = self.value.as_object() {
            obj.get("value").unwrap_or(&self.value)
        } else {
            &self.value
        };

        match &self.type_hint {
            Some(hint) => match hint {
                SqlTypeHint::Null => Ok(ToSqlOutput::Owned(rusqlite::types::Value::Null)),
                SqlTypeHint::Boolean => {
                    if let Some(b) = actual_value.as_bool() {
                        Ok(ToSqlOutput::Owned(rusqlite::types::Value::Integer(if b {
                            1
                        } else {
                            0
                        })))
                    } else {
                        Ok(ToSqlOutput::Owned(rusqlite::types::Value::Null))
                    }
                }
                SqlTypeHint::Integer => {
                    if let Some(i) = actual_value.as_i64() {
                        Ok(ToSqlOutput::Owned(rusqlite::types::Value::Integer(i)))
                    } else {
                        Ok(ToSqlOutput::Owned(rusqlite::types::Value::Null))
                    }
                }
                SqlTypeHint::Real => {
                    if let Some(f) = actual_value.as_f64() {
                        Ok(ToSqlOutput::Owned(rusqlite::types::Value::Real(f)))
                    } else {
                        Ok(ToSqlOutput::Owned(rusqlite::types::Value::Null))
                    }
                }
                SqlTypeHint::Text | SqlTypeHint::Datetime => {
                    if let Some(s) = actual_value.as_str() {
                        Ok(ToSqlOutput::Owned(rusqlite::types::Value::Text(
                            s.to_string(),
                        )))
                    } else {
                        Ok(ToSqlOutput::Owned(rusqlite::types::Value::Null))
                    }
                }
                SqlTypeHint::Blob => {
                    if let Some(s) = actual_value.as_str() {
                        // For now, just store as text. Could add base64 decoding later if needed
                        Ok(ToSqlOutput::Owned(rusqlite::types::Value::Text(
                            s.to_string(),
                        )))
                    } else {
                        Ok(ToSqlOutput::Owned(rusqlite::types::Value::Null))
                    }
                }
            },
            None => self.value_to_sql_with_actual(actual_value),
        }
    }
}

impl TypedSqlParam {
    fn value_to_sql_with_actual(&self, actual_value: &Value) -> rusqlite::Result<ToSqlOutput<'_>> {
        match actual_value {
            Value::Null => Ok(ToSqlOutput::Owned(rusqlite::types::Value::Null)),
            Value::Bool(b) => Ok(ToSqlOutput::Owned(rusqlite::types::Value::Integer(if *b {
                1
            } else {
                0
            }))),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(ToSqlOutput::Owned(rusqlite::types::Value::Integer(i)))
                } else if let Some(f) = n.as_f64() {
                    Ok(ToSqlOutput::Owned(rusqlite::types::Value::Real(f)))
                } else {
                    Ok(ToSqlOutput::Owned(rusqlite::types::Value::Null))
                }
            }
            Value::String(s) => Ok(ToSqlOutput::Owned(rusqlite::types::Value::Text(s.clone()))),
            _ => Ok(ToSqlOutput::Owned(rusqlite::types::Value::Text(
                actual_value.to_string(),
            ))),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, FromBytes)]
#[encoding(Json)]
pub struct EnhancedSqlParams {
    pub sql: String,
    pub params: Vec<TypedSqlParam>,
}

// Rich column metadata for better tooling support
#[derive(Debug, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub r#type: String, // 'type' is reserved in some languages
    pub nullable: bool,
    #[serde(rename = "primaryKey")]
    pub primary_key: bool,
    #[serde(rename = "autoIncrement")]
    pub auto_increment: bool,
    #[serde(rename = "defaultValue")]
    pub default_value: Option<String>,
}

// Comprehensive query metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct QueryMetadata {
    #[serde(rename = "executionTimeMs")]
    pub execution_time_ms: f64,
    #[serde(rename = "rowsReturned")]
    pub rows_returned: u64,
    #[serde(rename = "rowsAffected")]
    pub rows_affected: u64,
    #[serde(rename = "lastInsertRowid")]
    pub last_insert_rowid: Option<i64>,
    #[serde(rename = "queryPlan")]
    pub query_plan: Option<String>,
    #[serde(rename = "sqliteVersion")]
    pub sqlite_version: String,
}

// Enhanced result with rich metadata
#[derive(Debug, Serialize, Deserialize, ToBytes)]
#[encoding(Json)]
pub struct EnhancedSqlResult {
    pub data: Vec<Value>,
    pub columns: Vec<ColumnInfo>,
    pub metadata: QueryMetadata,
}

// Table schema for introspection
#[derive(Debug, Serialize, Deserialize, ToBytes)]
#[encoding(Json)]
pub struct TableInfo {
    pub name: String,
    pub columns: Vec<ColumnInfo>,
    pub indexes: Vec<IndexInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexInfo {
    pub name: String,
    pub columns: Vec<String>,
    pub unique: bool,
}

// Transaction support
#[derive(Debug, Serialize, Deserialize, ToBytes)]
#[encoding(Json)]
pub struct TransactionResult {
    #[serde(rename = "transactionId")]
    pub transaction_id: String,
    pub success: bool,
}

// Database statistics
#[derive(Debug, Serialize, Deserialize, ToBytes)]
#[encoding(Json)]
pub struct DatabaseStats {
    #[serde(rename = "totalQueries")]
    pub total_queries: u64,
    #[serde(rename = "totalExecutionTimeMs")]
    pub total_execution_time_ms: f64,
    #[serde(rename = "averageExecutionTimeMs")]
    pub average_execution_time_ms: f64,
    #[serde(rename = "queryCountByType")]
    pub query_count_by_type: HashMap<String, u64>,
    #[serde(rename = "databaseSizeBytes")]
    pub database_size_bytes: u64,
    #[serde(rename = "sqliteVersion")]
    pub sqlite_version: String,
}

// Query explanation for debugging
#[derive(Debug, Serialize, Deserialize, ToBytes)]
#[encoding(Json)]
pub struct QueryExplanation {
    #[serde(rename = "queryPlan")]
    pub query_plan: String,
    #[serde(rename = "estimatedCost")]
    pub estimated_cost: f64,
    #[serde(rename = "estimatedRows")]
    pub estimated_rows: u64,
    #[serde(rename = "sqliteVersion")]
    pub sqlite_version: String,
}

pub fn attach_guest_sqlite_improved(
    builder: PluginBuilder,
    config: GuestConfig,
    existing_user_data: Option<UserData<GuestSqliteDbImproved>>,
) -> (PluginBuilder, UserData<GuestSqliteDbImproved>) {
    let user_data = existing_user_data.unwrap_or_else(|| UserData::new(GuestSqliteDbImproved::new_with_config(&config)));
    let builder = builder
        .with_function(
            "sqlite_execute_enhanced",
            [PTR],
            [PTR],
            user_data.clone(),
            sqlite_execute_enhanced,
        )
        .with_function(
            "sqlite_query_enhanced",
            [PTR],
            [PTR],
            user_data.clone(),
            sqlite_query_enhanced,
        )
        .with_function(
            "sqlite_describe_table",
            [PTR],
            [PTR],
            user_data.clone(),
            sqlite_describe_table,
        )
        .with_function(
            "sqlite_list_tables",
            [PTR],
            [PTR],
            user_data.clone(),
            sqlite_list_tables,
        )
        .with_function(
            "sqlite_explain_query",
            [PTR],
            [PTR],
            user_data.clone(),
            sqlite_explain_query,
        )
        .with_function(
            "sqlite_get_stats",
            [PTR],
            [PTR],
            user_data.clone(),
            sqlite_get_stats,
        )
        .with_function(
            "sqlite_begin_transaction",
            [PTR],
            [PTR],
            user_data.clone(),
            sqlite_begin_transaction,
        )
        .with_function(
            "sqlite_commit_transaction",
            [PTR],
            [PTR],
            user_data.clone(),
            sqlite_commit_transaction,
        )
        .with_function(
            "sqlite_rollback_transaction",
            [PTR],
            [PTR],
            user_data.clone(),
            sqlite_rollback_transaction,
        );

    (builder, user_data)
}

host_fn!(sqlite_execute_enhanced(user_data: GuestSqliteDbImproved; params: EnhancedSqlParams) -> EnhancedSqlResult {
    info!("sqlite_execute_enhanced received params: {:?}", params);
    execute_enhanced(user_data, params)
});

host_fn!(sqlite_query_enhanced(user_data: GuestSqliteDbImproved; params: EnhancedSqlParams) -> EnhancedSqlResult {
    info!("sqlite_query_enhanced received params: {:?}", params);
    query_enhanced(user_data, params)
});

host_fn!(sqlite_describe_table(user_data: GuestSqliteDbImproved; input: Json<TableNameInput>) -> TableInfo {
    info!("sqlite_describe_table received input: {:?}", input.0);
    info!("table_name field: {}", input.0.table_name);
    describe_table(user_data, input.0.table_name)
});

host_fn!(sqlite_list_tables(user_data: GuestSqliteDbImproved; input: Json<EmptyInput>) -> Json<String> {
    info!("sqlite_list_tables received input: {:?}", input.0);
    list_tables_json(user_data).map(Json)
});

host_fn!(sqlite_explain_query(user_data: GuestSqliteDbImproved; params: EnhancedSqlParams) -> QueryExplanation {
    explain_query(user_data, params)
});

host_fn!(sqlite_get_stats(user_data: GuestSqliteDbImproved;) -> DatabaseStats {
    get_stats(user_data)
});

host_fn!(sqlite_begin_transaction(user_data: GuestSqliteDbImproved;) -> TransactionResult {
    begin_transaction(user_data)
});

host_fn!(sqlite_commit_transaction(user_data: GuestSqliteDbImproved; input: Json<TransactionIdInput>) -> TransactionResult {
    commit_transaction(user_data, input.0.transaction_id)
});

host_fn!(sqlite_rollback_transaction(user_data: GuestSqliteDbImproved; input: Json<TransactionIdInput>) -> TransactionResult {
    rollback_transaction(user_data, input.0.transaction_id)
});

fn execute_enhanced(
    user_data: UserData<GuestSqliteDbImproved>,
    params: EnhancedSqlParams,
) -> Result<EnhancedSqlResult, extism::Error> {
    let start = Instant::now();
    let user_data_guard = user_data.get()?;
    let user_data = user_data_guard.lock().unwrap();

    let rows_affected = user_data
        .db
        .execute(&params.sql, params_from_iter(&params.params))? as u64;
    let last_insert_rowid = if params.sql.trim_start().to_lowercase().starts_with("insert") {
        Some(user_data.db.last_insert_rowid())
    } else {
        None
    };

    let execution_time = start.elapsed();
    let execution_time_ms = execution_time.as_secs_f64() * 1000.0;

    // Get SQLite version
    let sqlite_version = user_data
        .db
        .prepare("SELECT sqlite_version()")
        .and_then(|mut stmt| stmt.query_row([], |row| row.get::<_, String>(0)))
        .unwrap_or_else(|_| "unknown".to_string());

    // TODO: Record stats - need to fix borrowing issue
    // let query_type = get_query_type(&params.sql);
    // user_data.record_query(&query_type, execution_time_ms);

    Ok(EnhancedSqlResult {
        data: vec![], // Execute operations don't return data
        columns: vec![],
        metadata: QueryMetadata {
            execution_time_ms,
            rows_returned: 0,
            rows_affected,
            last_insert_rowid,
            query_plan: None,
            sqlite_version,
        },
    })
}

fn query_enhanced(
    user_data: UserData<GuestSqliteDbImproved>,
    params: EnhancedSqlParams,
) -> Result<EnhancedSqlResult, extism::Error> {
    let start = Instant::now();
    let user_data_guard = user_data.get()?;
    let user_data = user_data_guard.lock().unwrap();

    // Get query plan for debugging
    let query_plan = if params.sql.trim_start().to_lowercase().starts_with("select") {
        let plan_stmt = format!("EXPLAIN QUERY PLAN {}", params.sql);
        user_data
            .db
            .prepare(&plan_stmt)
            .and_then(|mut stmt| {
                let rows: Result<Vec<String>, _> = stmt
                    .query_map([], |row| {
                        Ok(format!(
                            "{}: {}",
                            row.get::<_, i32>(0)?,
                            row.get::<_, String>(3)?
                        ))
                    })?
                    .collect();
                rows.map(|r| r.join(" -> "))
            })
            .ok()
    } else {
        None
    };

    let mut stmt = user_data.db.prepare(&params.sql)?;

    // Extract rich column information
    let columns: Vec<ColumnInfo> = stmt
        .columns()
        .iter()
        .map(|col| ColumnInfo {
            name: col.name().to_string(),
            r#type: col.decl_type().unwrap_or("UNKNOWN").to_string(),
            nullable: true,        // SQLite doesn't provide this easily without PRAGMA
            primary_key: false,    // Would need PRAGMA table_info
            auto_increment: false, // Would need PRAGMA table_info
            default_value: None,   // Would need PRAGMA table_info
        })
        .collect();

    let mut result = stmt.query(params_from_iter(&params.params))?;
    let mut results = Vec::new();
    let mut row_count = 0;

    while let Ok(Some(row)) = result.next() {
        let mut row_map = serde_json::Map::new();
        for (i, col_info) in columns.iter().enumerate() {
            let value = match row.get_ref(i)? {
                rusqlite::types::ValueRef::Null => Value::Null,
                rusqlite::types::ValueRef::Integer(i) => Value::Number(serde_json::Number::from(i)),
                rusqlite::types::ValueRef::Real(f) => {
                    if let Some(num) = serde_json::Number::from_f64(f) {
                        Value::Number(num)
                    } else {
                        Value::Null
                    }
                }
                rusqlite::types::ValueRef::Text(s) => {
                    Value::String(String::from_utf8_lossy(s).to_string())
                }
                rusqlite::types::ValueRef::Blob(b) => {
                    // Convert blob to base64 string for JSON representation
                    Value::String(base64::encode(b))
                }
            };
            row_map.insert(col_info.name.clone(), value);
        }
        results.push(Value::Object(row_map));
        row_count += 1;
    }

    let execution_time = start.elapsed();
    let execution_time_ms = execution_time.as_secs_f64() * 1000.0;

    // Get SQLite version
    let sqlite_version = user_data
        .db
        .prepare("SELECT sqlite_version()")
        .and_then(|mut stmt| stmt.query_row([], |row| row.get::<_, String>(0)))
        .unwrap_or_else(|_| "unknown".to_string());

    // TODO: Record stats - need to fix borrowing issue
    // let query_type = get_query_type(&params.sql);
    // user_data.record_query(&query_type, execution_time_ms);

    Ok(EnhancedSqlResult {
        data: results,
        columns,
        metadata: QueryMetadata {
            execution_time_ms,
            rows_returned: row_count,
            rows_affected: 0,
            last_insert_rowid: None,
            query_plan,
            sqlite_version,
        },
    })
}

fn describe_table(
    user_data: UserData<GuestSqliteDbImproved>,
    table_name: String,
) -> Result<TableInfo, extism::Error> {
    let user_data = user_data.get()?;
    let user_data = user_data.lock().unwrap();

    // Get column information
    let sql = format!("PRAGMA table_info('{}')", table_name);
    let mut stmt = user_data.db.prepare(&sql)?;
    let column_rows = stmt.query_map([], |row| {
        Ok(ColumnInfo {
            name: row.get(1)?,
            r#type: row.get(2)?,
            nullable: !row.get::<_, bool>(3)?,
            primary_key: row.get::<_, bool>(5)?,
            auto_increment: false, // SQLite doesn't have true auto-increment
            default_value: row.get::<_, Option<String>>(4)?,
        })
    })?;

    let columns: Result<Vec<_>, _> = column_rows.collect();
    let columns = columns?;

    // Get index information
    let sql = format!("PRAGMA index_list('{}')", table_name);
    let mut stmt = user_data.db.prepare(&sql)?;
    let index_rows = stmt.query_map([], |row| {
        let index_name: String = row.get(1)?;
        let unique: bool = row.get(2)?;
        Ok((index_name, unique))
    })?;

    let mut indexes = Vec::new();
    for index_result in index_rows {
        let (index_name, unique) = index_result?;

        // Get columns for this index
        let sql = format!("PRAGMA index_info('{}')", index_name);
        let mut stmt = user_data.db.prepare(&sql)?;
        let col_rows = stmt.query_map([], |row| Ok(row.get::<_, String>(2)?))?;

        let index_columns: Result<Vec<_>, _> = col_rows.collect();
        indexes.push(IndexInfo {
            name: index_name,
            columns: index_columns?,
            unique,
        });
    }

    Ok(TableInfo {
        name: table_name,
        columns,
        indexes,
    })
}

fn list_tables_json(user_data: UserData<GuestSqliteDbImproved>) -> Result<String, extism::Error> {
    let user_data = user_data.get()?;
    let user_data = user_data.lock().unwrap();

    let mut stmt = user_data.db.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'",
    )?;
    let table_rows = stmt.query_map([], |row| Ok(row.get::<_, String>(0)?))?;

    let tables: Result<Vec<_>, _> = table_rows.collect();
    let tables = tables?;

    serde_json::to_string(&tables).map_err(|e| extism::Error::msg(e.to_string()))
}

fn explain_query(
    user_data: UserData<GuestSqliteDbImproved>,
    params: EnhancedSqlParams,
) -> Result<QueryExplanation, extism::Error> {
    let user_data = user_data.get()?;
    let user_data = user_data.lock().unwrap();

    // Get execution plan
    let plan_query = format!("EXPLAIN QUERY PLAN {}", params.sql);
    let query_plan = user_data
        .db
        .prepare(&plan_query)
        .and_then(|mut stmt| {
            let rows: Result<Vec<String>, _> = stmt
                .query_map([], |row| {
                    Ok(format!(
                        "{}: {}",
                        row.get::<_, i32>(0)?,
                        row.get::<_, String>(3)?
                    ))
                })?
                .collect();
            rows.map(|r| r.join("\n"))
        })
        .unwrap_or_else(|_| "Unable to generate query plan".to_string());

    // Get SQLite version
    let sqlite_version = user_data
        .db
        .prepare("SELECT sqlite_version()")
        .and_then(|mut stmt| stmt.query_row([], |row| row.get::<_, String>(0)))
        .unwrap_or_else(|_| "unknown".to_string());

    Ok(QueryExplanation {
        query_plan,
        estimated_cost: 0.0, // SQLite doesn't provide cost estimates easily
        estimated_rows: 0,   // SQLite doesn't provide row estimates easily
        sqlite_version,
    })
}

fn get_stats(user_data: UserData<GuestSqliteDbImproved>) -> Result<DatabaseStats, extism::Error> {
    let user_data = user_data.get()?;
    let user_data = user_data.lock().unwrap();

    let average_execution_time_ms = if user_data.stats.total_queries > 0 {
        user_data.stats.total_execution_time_ms / user_data.stats.total_queries as f64
    } else {
        0.0
    };

    // Get database size (page_count * page_size)
    let database_size_bytes = user_data
        .db
        .prepare("PRAGMA page_count")
        .and_then(|mut stmt| stmt.query_row([], |row| row.get::<_, u64>(0)))
        .unwrap_or(0)
        * user_data
            .db
            .prepare("PRAGMA page_size")
            .and_then(|mut stmt| stmt.query_row([], |row| row.get::<_, u64>(0)))
            .unwrap_or(4096);

    // Get SQLite version
    let sqlite_version = user_data
        .db
        .prepare("SELECT sqlite_version()")
        .and_then(|mut stmt| stmt.query_row([], |row| row.get::<_, String>(0)))
        .unwrap_or_else(|_| "unknown".to_string());

    Ok(DatabaseStats {
        total_queries: user_data.stats.total_queries,
        total_execution_time_ms: user_data.stats.total_execution_time_ms,
        average_execution_time_ms,
        query_count_by_type: user_data.stats.query_count_by_type.clone(),
        database_size_bytes,
        sqlite_version,
    })
}

fn begin_transaction(
    user_data: UserData<GuestSqliteDbImproved>,
) -> Result<TransactionResult, extism::Error> {
    let user_data = user_data.get()?;
    let user_data = user_data.lock().unwrap();

    match user_data.db.execute("BEGIN TRANSACTION", []) {
        Ok(_) => {
            // Generate a simple transaction ID using timestamp
            let transaction_id = format!(
                "tx_{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos()
            );
            Ok(TransactionResult {
                transaction_id,
                success: true,
            })
        }
        Err(e) => Err(extism::Error::msg(format!(
            "Failed to begin transaction: {}",
            e
        ))),
    }
}

fn commit_transaction(
    user_data: UserData<GuestSqliteDbImproved>,
    _transaction_id: String,
) -> Result<TransactionResult, extism::Error> {
    let user_data = user_data.get()?;
    let user_data = user_data.lock().unwrap();

    match user_data.db.execute("COMMIT", []) {
        Ok(_) => Ok(TransactionResult {
            transaction_id: _transaction_id,
            success: true,
        }),
        Err(e) => Err(extism::Error::msg(format!(
            "Failed to commit transaction: {}",
            e
        ))),
    }
}

fn rollback_transaction(
    user_data: UserData<GuestSqliteDbImproved>,
    _transaction_id: String,
) -> Result<TransactionResult, extism::Error> {
    let user_data = user_data.get()?;
    let user_data = user_data.lock().unwrap();

    match user_data.db.execute("ROLLBACK", []) {
        Ok(_) => Ok(TransactionResult {
            transaction_id: _transaction_id,
            success: true,
        }),
        Err(e) => Err(extism::Error::msg(format!(
            "Failed to rollback transaction: {}",
            e
        ))),
    }
}

// fn get_query_type(sql: &str) -> String {
//     let sql_lower = sql.trim_start().to_lowercase();
//     if sql_lower.starts_with("select") {
//         "SELECT".to_string()
//     } else if sql_lower.starts_with("insert") {
//         "INSERT".to_string()
//     } else if sql_lower.starts_with("update") {
//         "UPDATE".to_string()
//     } else if sql_lower.starts_with("delete") {
//         "DELETE".to_string()
//     } else if sql_lower.starts_with("create") {
//         "CREATE".to_string()
//     } else if sql_lower.starts_with("drop") {
//         "DROP".to_string()
//     } else if sql_lower.starts_with("alter") {
//         "ALTER".to_string()
//     } else {
//         "OTHER".to_string()
//     }
// }
