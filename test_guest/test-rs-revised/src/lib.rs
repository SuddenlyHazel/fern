mod pdk;

use extism_pdk::*;
use pdk::*;

// Test entry point that demonstrates all SQLite functionality
pub(crate) fn test_enhanced_sql(_input: String) -> Result<types::SqliteTestResult, Error> {
    let mut results = Vec::new();
    let mut performance_summary = types::PerformanceSummary {
        total_queries: 0,
        total_execution_time_ms: 0.0,
        average_query_time_ms: 0.0,
        database_operations: Vec::new(),
        fastest_operation: None,
        slowest_operation: None,
    };

    let test_name = "Comprehensive SQLite Test Suite".to_string();
    guest_info(format!("{test_name}"))?;
    let mut overall_success = true;

    // Test 1: List tables (should be empty initially)
    let step_result = test_list_tables();
    overall_success &= step_result.success;
    performance_summary.total_queries += 1;
    performance_summary.total_execution_time_ms += step_result.execution_time_ms;
    performance_summary.database_operations.push("list_tables".to_string());
    results.push(step_result);

    // Test 2: Create a test table
    let step_result = test_create_table();
    overall_success &= step_result.success;
    performance_summary.total_queries += 1;
    performance_summary.total_execution_time_ms += step_result.execution_time_ms;
    performance_summary.database_operations.push("create_table".to_string());
    results.push(step_result);

    // Test 3: Describe the table structure
    let step_result = test_describe_table();
    overall_success &= step_result.success;
    performance_summary.total_queries += 1;
    performance_summary.total_execution_time_ms += step_result.execution_time_ms;
    performance_summary.database_operations.push("describe_table".to_string());
    results.push(step_result);

    // Test 4: Insert test data
    let step_result = test_insert_data();
    overall_success &= step_result.success;
    performance_summary.total_queries += 1;
    performance_summary.total_execution_time_ms += step_result.execution_time_ms;
    performance_summary.database_operations.push("insert_data".to_string());
    results.push(step_result);

    // Test 5: Query data with enhanced functionality
    let step_result = test_query_enhanced();
    overall_success &= step_result.success;
    performance_summary.total_queries += 1;
    performance_summary.total_execution_time_ms += step_result.execution_time_ms;
    performance_summary.database_operations.push("query_enhanced".to_string());
    results.push(step_result);

    // Test 6: Test transactions
    let step_result = test_transactions();
    overall_success &= step_result.success;
    performance_summary.total_queries += 3; // begin, insert, commit
    performance_summary.total_execution_time_ms += step_result.execution_time_ms;
    performance_summary.database_operations.push("transactions".to_string());
    results.push(step_result);

    // Test 7: Test query explanation
    let step_result = test_query_explanation();
    overall_success &= step_result.success;
    performance_summary.total_queries += 1;
    performance_summary.total_execution_time_ms += step_result.execution_time_ms;
    performance_summary.database_operations.push("explain_query".to_string());
    results.push(step_result);

    // Test 8: Get database statistics
    let step_result = test_database_stats();
    overall_success &= step_result.success;
    performance_summary.total_queries += 1;
    performance_summary.total_execution_time_ms += step_result.execution_time_ms;
    performance_summary.database_operations.push("get_stats".to_string());
    results.push(step_result);

    // Test 9: Test transaction rollback
    let step_result = test_transaction_rollback();
    overall_success &= step_result.success;
    performance_summary.total_queries += 3; // begin, insert, rollback
    performance_summary.total_execution_time_ms += step_result.execution_time_ms;
    performance_summary.database_operations.push("transaction_rollback".to_string());
    results.push(step_result);

    // Test 10: Test querying multiple rows
    let step_result = test_query_multiple_rows();
    overall_success &= step_result.success;
    performance_summary.total_queries += 4; // 3 inserts + 1 query
    performance_summary.total_execution_time_ms += step_result.execution_time_ms;
    performance_summary.database_operations.push("query_multiple_rows".to_string());
    results.push(step_result);

    // Test 11: Test KV store functionality
    let step_result = test_kv_store();
    overall_success &= step_result.success;
    performance_summary.total_queries += 2; // store + read
    performance_summary.total_execution_time_ms += step_result.execution_time_ms;
    performance_summary.database_operations.push("kv_store".to_string());
    results.push(step_result);

    // Calculate final performance metrics
    if performance_summary.total_queries > 0 {
        performance_summary.average_query_time_ms = 
            performance_summary.total_execution_time_ms / performance_summary.total_queries as f64;
    }

    // Find fastest and slowest operations
    if let (Some(fastest), Some(slowest)) = find_fastest_slowest_operations(&results) {
        performance_summary.fastest_operation = Some(fastest);
        performance_summary.slowest_operation = Some(slowest);
    }

    let result = types::SqliteTestResult {
        test_name,
        success: overall_success,
        results,
        performance_summary,
        error_message: if overall_success { None } else { Some("Some tests failed".to_string()) },
    };

    Ok(result)
}

fn test_list_tables() -> types::TestStepResult {
    let start_time = std::time::Instant::now();
    
    match sqlite_list_tables(types::EmptyInput { _dummy: None }) {
        Ok(tables_json) => {
            let execution_time = start_time.elapsed().as_millis() as f64;
            
            // Parse the JSON to verify it's valid
            match serde_json::from_str::<Vec<String>>(&tables_json) {
                Ok(_tables) => types::TestStepResult {
                    step_name: "List Tables".to_string(),
                    success: true,
                    execution_time_ms: execution_time,
                    error_message: None,
                    details: Some(create_details_map(&[("tables_json", &tables_json)])),
                },
                Err(e) => types::TestStepResult {
                    step_name: "List Tables".to_string(),
                    success: false,
                    execution_time_ms: execution_time,
                    error_message: Some(format!("Failed to parse tables JSON: {}", e)),
                    details: None,
                },
            }
        }
        Err(e) => types::TestStepResult {
            step_name: "List Tables".to_string(),
            success: false,
            execution_time_ms: start_time.elapsed().as_millis() as f64,
            error_message: Some(format!("Failed to list tables: {}", e)),
            details: None,
        },
    }
}

fn test_create_table() -> types::TestStepResult {
    let start_time = std::time::Instant::now();
    
    let create_sql = "CREATE TABLE test_users (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        email TEXT UNIQUE,
        age INTEGER,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP
    )";

    let params = types::EnhancedSqlParams {
        sql: create_sql.to_string(),
        params: Vec::new(),
    };

    match sqlite_execute_enhanced(params) {
        Ok(result) => {
            let execution_time = start_time.elapsed().as_millis() as f64;
            types::TestStepResult {
                step_name: "Create Table".to_string(),
                success: true,
                execution_time_ms: execution_time,
                error_message: None,
                details: Some(create_details_map(&[
                    ("rows_affected", &result.metadata.rows_affected.to_string()),
                    ("sqlite_version", &result.metadata.sqlite_version),
                ])),
            }
        }
        Err(e) => types::TestStepResult {
            step_name: "Create Table".to_string(),
            success: false,
            execution_time_ms: start_time.elapsed().as_millis() as f64,
            error_message: Some(format!("Failed to create table: {}", e)),
            details: None,
        },
    }
}

fn test_describe_table() -> types::TestStepResult {
    let start_time = std::time::Instant::now();
    
    let input = types::TableNameInput {
        table_name: "test_users".to_string(),
    };

    match sqlite_describe_table(input) {
        Ok(table_info) => {
            let execution_time = start_time.elapsed().as_millis() as f64;
            types::TestStepResult {
                step_name: "Describe Table".to_string(),
                success: true,
                execution_time_ms: execution_time,
                error_message: None,
                details: Some(create_details_map(&[
                    ("table_name", &table_info.name),
                    ("column_count", &table_info.columns.len().to_string()),
                    ("index_count", &table_info.indexes.len().to_string()),
                ])),
            }
        }
        Err(e) => types::TestStepResult {
            step_name: "Describe Table".to_string(),
            success: false,
            execution_time_ms: start_time.elapsed().as_millis() as f64,
            error_message: Some(format!("Failed to describe table: {}", e)),
            details: None,
        },
    }
}

fn test_insert_data() -> types::TestStepResult {
    let start_time = std::time::Instant::now();
    
    let insert_sql = "INSERT INTO test_users (name, email, age) VALUES (?, ?, ?)";
    
    // Create typed parameters
    let mut name_value = serde_json::Map::new();
    name_value.insert("value".to_string(), serde_json::Value::String("John Doe".to_string()));
    
    let mut email_value = serde_json::Map::new();
    email_value.insert("value".to_string(), serde_json::Value::String("john@example.com".to_string()));
    
    let mut age_value = serde_json::Map::new();
    age_value.insert("value".to_string(), serde_json::Value::Number(serde_json::Number::from(30)));

    let params = types::EnhancedSqlParams {
        sql: insert_sql.to_string(),
        params: vec![
            types::TypedSqlParam {
                value: name_value,
                type_hint: Some(types::SqlTypeHint::Text),
            },
            types::TypedSqlParam {
                value: email_value,
                type_hint: Some(types::SqlTypeHint::Text),
            },
            types::TypedSqlParam {
                value: age_value,
                type_hint: Some(types::SqlTypeHint::Integer),
            },
        ],
    };

    match sqlite_execute_enhanced(params) {
        Ok(result) => {
            let execution_time = start_time.elapsed().as_millis() as f64;
            types::TestStepResult {
                step_name: "Insert Data".to_string(),
                success: true,
                execution_time_ms: execution_time,
                error_message: None,
                details: Some(create_details_map(&[
                    ("rows_affected", &result.metadata.rows_affected.to_string()),
                    ("last_insert_rowid", &result.metadata.last_insert_rowid.map_or("None".to_string(), |id| id.to_string())),
                ])),
            }
        }
        Err(e) => types::TestStepResult {
            step_name: "Insert Data".to_string(),
            success: false,
            execution_time_ms: start_time.elapsed().as_millis() as f64,
            error_message: Some(format!("Failed to insert data: {}", e)),
            details: None,
        },
    }
}

fn test_query_enhanced() -> types::TestStepResult {
    let start_time = std::time::Instant::now();
    
    let query_sql = "SELECT id, name, email, age FROM test_users WHERE age > ?";
    
    let mut age_param = serde_json::Map::new();
    age_param.insert("value".to_string(), serde_json::Value::Number(serde_json::Number::from(25)));

    let params = types::EnhancedSqlParams {
        sql: query_sql.to_string(),
        params: vec![
            types::TypedSqlParam {
                value: age_param,
                type_hint: Some(types::SqlTypeHint::Integer),
            },
        ],
    };

    match sqlite_query_enhanced(params) {
        Ok(result) => {
            let execution_time = start_time.elapsed().as_millis() as f64;
            types::TestStepResult {
                step_name: "Query Enhanced".to_string(),
                success: true,
                execution_time_ms: execution_time,
                error_message: None,
                details: Some(create_details_map(&[
                    ("rows_returned", &result.metadata.rows_returned.to_string()),
                    ("column_count", &result.columns.len().to_string()),
                    ("execution_time_ms", &result.metadata.execution_time_ms.to_string()),
                    ("data", format!("{}", serde_json::to_string(&result.data).unwrap()).as_str())
                ])),
            }
        }
        Err(e) => types::TestStepResult {
            step_name: "Query Enhanced".to_string(),
            success: false,
            execution_time_ms: start_time.elapsed().as_millis() as f64,
            error_message: Some(format!("Failed to query data: {}", e)),
            details: None,
        },
    }
}

fn test_transactions() -> types::TestStepResult {
    let start_time = std::time::Instant::now();
    
    // Begin transaction
    match sqlite_begin_transaction(types::EmptyInput { _dummy: None }) {
        Ok(tx_result) => {
            if !tx_result.success {
                return types::TestStepResult {
                    step_name: "Transactions".to_string(),
                    success: false,
                    execution_time_ms: start_time.elapsed().as_millis() as f64,
                    error_message: Some("Failed to begin transaction".to_string()),
                    details: None,
                };
            }

            // Insert data within transaction
            let insert_sql = "INSERT INTO test_users (name, email, age) VALUES (?, ?, ?)";
            
            let mut name_value = serde_json::Map::new();
            name_value.insert("value".to_string(), serde_json::Value::String("Jane Doe".to_string()));
            
            let mut email_value = serde_json::Map::new();
            email_value.insert("value".to_string(), serde_json::Value::String("jane@example.com".to_string()));
            
            let mut age_value = serde_json::Map::new();
            age_value.insert("value".to_string(), serde_json::Value::Number(serde_json::Number::from(28)));

            let params = types::EnhancedSqlParams {
                sql: insert_sql.to_string(),
                params: vec![
                    types::TypedSqlParam { value: name_value, type_hint: Some(types::SqlTypeHint::Text) },
                    types::TypedSqlParam { value: email_value, type_hint: Some(types::SqlTypeHint::Text) },
                    types::TypedSqlParam { value: age_value, type_hint: Some(types::SqlTypeHint::Integer) },
                ],
            };

            match sqlite_execute_enhanced(params) {
                Ok(_) => {
                    // Commit transaction
                    let commit_input = types::TransactionIdInput {
                        transaction_id: tx_result.transaction_id.clone(),
                    };
                    
                    match sqlite_commit_transaction(commit_input) {
                        Ok(commit_result) => {
                            let execution_time = start_time.elapsed().as_millis() as f64;
                            types::TestStepResult {
                                step_name: "Transactions".to_string(),
                                success: commit_result.success,
                                execution_time_ms: execution_time,
                                error_message: None,
                                details: Some(create_details_map(&[
                                    ("transaction_id", &tx_result.transaction_id),
                                    ("committed", &commit_result.success.to_string()),
                                ])),
                            }
                        }
                        Err(e) => types::TestStepResult {
                            step_name: "Transactions".to_string(),
                            success: false,
                            execution_time_ms: start_time.elapsed().as_millis() as f64,
                            error_message: Some(format!("Failed to commit transaction: {}", e)),
                            details: None,
                        },
                    }
                }
                Err(e) => {
                    // Rollback on error
                    let rollback_input = types::TransactionIdInput {
                        transaction_id: tx_result.transaction_id,
                    };
                    let _ = sqlite_rollback_transaction(rollback_input);
                    
                    types::TestStepResult {
                        step_name: "Transactions".to_string(),
                        success: false,
                        execution_time_ms: start_time.elapsed().as_millis() as f64,
                        error_message: Some(format!("Failed to insert in transaction: {}", e)),
                        details: None,
                    }
                }
            }
        }
        Err(e) => types::TestStepResult {
            step_name: "Transactions".to_string(),
            success: false,
            execution_time_ms: start_time.elapsed().as_millis() as f64,
            error_message: Some(format!("Failed to begin transaction: {}", e)),
            details: None,
        },
    }
}

fn test_query_explanation() -> types::TestStepResult {
    let start_time = std::time::Instant::now();
    
    let query_sql = "SELECT * FROM test_users WHERE age > 25 ORDER BY name";
    
    let params = types::EnhancedSqlParams {
        sql: query_sql.to_string(),
        params: Vec::new(),
    };

    match sqlite_explain_query(params) {
        Ok(explanation) => {
            let execution_time = start_time.elapsed().as_millis() as f64;
            types::TestStepResult {
                step_name: "Query Explanation".to_string(),
                success: true,
                execution_time_ms: execution_time,
                error_message: None,
                details: Some(create_details_map(&[
                    ("estimated_cost", &explanation.estimated_cost.to_string()),
                    ("estimated_rows", &explanation.estimated_rows.to_string()),
                    ("sqlite_version", &explanation.sqlite_version),
                ])),
            }
        }
        Err(e) => types::TestStepResult {
            step_name: "Query Explanation".to_string(),
            success: false,
            execution_time_ms: start_time.elapsed().as_millis() as f64,
            error_message: Some(format!("Failed to explain query: {}", e)),
            details: None,
        },
    }
}

fn test_database_stats() -> types::TestStepResult {
    let start_time = std::time::Instant::now();
    
    match sqlite_get_stats(types::EmptyInput { _dummy: None }) {
        Ok(stats) => {
            let execution_time = start_time.elapsed().as_millis() as f64;
            types::TestStepResult {
                step_name: "Database Stats".to_string(),
                success: true,
                execution_time_ms: execution_time,
                error_message: None,
                details: Some(create_details_map(&[
                    ("total_queries", &stats.total_queries.to_string()),
                    ("database_size_bytes", &stats.database_size_bytes.to_string()),
                    ("average_execution_time_ms", &stats.average_execution_time_ms.to_string()),
                    ("sqlite_version", &stats.sqlite_version),
                ])),
            }
        }
        Err(e) => types::TestStepResult {
            step_name: "Database Stats".to_string(),
            success: false,
            execution_time_ms: start_time.elapsed().as_millis() as f64,
            error_message: Some(format!("Failed to get database stats: {}", e)),
            details: None,
        },
    }
}

fn test_transaction_rollback() -> types::TestStepResult {
    let start_time = std::time::Instant::now();
    
    // Begin transaction
    match sqlite_begin_transaction(types::EmptyInput { _dummy: None }) {
        Ok(tx_result) => {
            if !tx_result.success {
                return types::TestStepResult {
                    step_name: "Transaction Rollback".to_string(),
                    success: false,
                    execution_time_ms: start_time.elapsed().as_millis() as f64,
                    error_message: Some("Failed to begin transaction".to_string()),
                    details: None,
                };
            }

            // Insert data within transaction (this will be rolled back)
            let insert_sql = "INSERT INTO test_users (name, email, age) VALUES (?, ?, ?)";
            
            let mut name_value = serde_json::Map::new();
            name_value.insert("value".to_string(), serde_json::Value::String("Rollback User".to_string()));
            
            let mut email_value = serde_json::Map::new();
            email_value.insert("value".to_string(), serde_json::Value::String("rollback@example.com".to_string()));
            
            let mut age_value = serde_json::Map::new();
            age_value.insert("value".to_string(), serde_json::Value::Number(serde_json::Number::from(99)));

            let params = types::EnhancedSqlParams {
                sql: insert_sql.to_string(),
                params: vec![
                    types::TypedSqlParam { value: name_value, type_hint: Some(types::SqlTypeHint::Text) },
                    types::TypedSqlParam { value: email_value, type_hint: Some(types::SqlTypeHint::Text) },
                    types::TypedSqlParam { value: age_value, type_hint: Some(types::SqlTypeHint::Integer) },
                ],
            };

            match sqlite_execute_enhanced(params) {
                Ok(_) => {
                    // Rollback transaction instead of committing
                    let rollback_input = types::TransactionIdInput {
                        transaction_id: tx_result.transaction_id.clone(),
                    };
                    
                    match sqlite_rollback_transaction(rollback_input) {
                        Ok(rollback_result) => {
                            let execution_time = start_time.elapsed().as_millis() as f64;
                            types::TestStepResult {
                                step_name: "Transaction Rollback".to_string(),
                                success: rollback_result.success,
                                execution_time_ms: execution_time,
                                error_message: None,
                                details: Some(create_details_map(&[
                                    ("transaction_id", &tx_result.transaction_id),
                                    ("rolled_back", &rollback_result.success.to_string()),
                                ])),
                            }
                        }
                        Err(e) => types::TestStepResult {
                            step_name: "Transaction Rollback".to_string(),
                            success: false,
                            execution_time_ms: start_time.elapsed().as_millis() as f64,
                            error_message: Some(format!("Failed to rollback transaction: {}", e)),
                            details: None,
                        },
                    }
                }
                Err(e) => types::TestStepResult {
                    step_name: "Transaction Rollback".to_string(),
                    success: false,
                    execution_time_ms: start_time.elapsed().as_millis() as f64,
                    error_message: Some(format!("Failed to insert in transaction: {}", e)),
                    details: None,
                },
            }
        }
        Err(e) => types::TestStepResult {
            step_name: "Transaction Rollback".to_string(),
            success: false,
            execution_time_ms: start_time.elapsed().as_millis() as f64,
            error_message: Some(format!("Failed to begin transaction: {}", e)),
            details: None,
        },
    }
}

fn test_kv_store() -> types::TestStepResult {
    let start_time = std::time::Instant::now();
    
    // Store a value
    let mut test_value = serde_json::Map::new();
    test_value.insert("message".to_string(), serde_json::Value::String("Hello from KV store!".to_string()));
    test_value.insert("timestamp".to_string(), serde_json::Value::Number(serde_json::Number::from(1699123456)));

    let store_input = types::KvStoreInput {
        table: "test_kv".to_string(),
        key: "test_key".to_string(),
        value: test_value.clone(),
    };

    match kv_store(store_input) {
        Ok(_) => {
            // Read the value back
            let read_input = types::KvReadInput {
                table: "test_kv".to_string(),
                key: "test_key".to_string(),
            };

            match kv_read(read_input) {
                Ok(Some(retrieved_value)) => {
                    let execution_time = start_time.elapsed().as_millis() as f64;
                    let values_match = retrieved_value == test_value;
                    
                    types::TestStepResult {
                        step_name: "KV Store".to_string(),
                        success: values_match,
                        execution_time_ms: execution_time,
                        error_message: if values_match { None } else { Some("Retrieved value doesn't match stored value".to_string()) },
                        details: Some(create_details_map(&[
                            ("stored_successfully", "true"),
                            ("retrieved_successfully", "true"),
                            ("values_match", &values_match.to_string()),
                        ])),
                    }
                }
                Ok(None) => types::TestStepResult {
                    step_name: "KV Store".to_string(),
                    success: false,
                    execution_time_ms: start_time.elapsed().as_millis() as f64,
                    error_message: Some("Value not found after storing".to_string()),
                    details: None,
                },
                Err(e) => types::TestStepResult {
                    step_name: "KV Store".to_string(),
                    success: false,
                    execution_time_ms: start_time.elapsed().as_millis() as f64,
                    error_message: Some(format!("Failed to read KV value: {}", e)),
                    details: None,
                },
            }
        }
        Err(e) => types::TestStepResult {
            step_name: "KV Store".to_string(),
            success: false,
            execution_time_ms: start_time.elapsed().as_millis() as f64,
            error_message: Some(format!("Failed to store KV value: {}", e)),
            details: None,
        },
    }
}

fn test_query_multiple_rows() -> types::TestStepResult {
    let start_time = std::time::Instant::now();
    
    // First, insert multiple test records
    let test_users = [
        ("Alice Smith", "alice@example.com", 25),
        ("Bob Johnson", "bob@example.com", 35),
        ("Carol Davis", "carol@example.com", 28),
    ];
    
    // Insert each user
    for (name, email, age) in &test_users {
        let insert_sql = "INSERT INTO test_users (name, email, age) VALUES (?, ?, ?)";
        
        let mut name_value = serde_json::Map::new();
        name_value.insert("value".to_string(), serde_json::Value::String(name.to_string()));
        
        let mut email_value = serde_json::Map::new();
        email_value.insert("value".to_string(), serde_json::Value::String(email.to_string()));
        
        let mut age_value = serde_json::Map::new();
        age_value.insert("value".to_string(), serde_json::Value::Number(serde_json::Number::from(*age)));

        let params = types::EnhancedSqlParams {
            sql: insert_sql.to_string(),
            params: vec![
                types::TypedSqlParam {
                    value: name_value,
                    type_hint: Some(types::SqlTypeHint::Text),
                },
                types::TypedSqlParam {
                    value: email_value,
                    type_hint: Some(types::SqlTypeHint::Text),
                },
                types::TypedSqlParam {
                    value: age_value,
                    type_hint: Some(types::SqlTypeHint::Integer),
                },
            ],
        };

        match sqlite_execute_enhanced(params) {
            Ok(_) => {}, // Continue with next insert
            Err(e) => {
                return types::TestStepResult {
                    step_name: "Query Multiple Rows".to_string(),
                    success: false,
                    execution_time_ms: start_time.elapsed().as_millis() as f64,
                    error_message: Some(format!("Failed to insert test data for {}: {}", name, e)),
                    details: None,
                };
            }
        }
    }
    
    // Now query for multiple rows - get all users with age >= 25
    let query_sql = "SELECT id, name, email, age FROM test_users WHERE age >= ? ORDER BY name";
    
    let mut age_param = serde_json::Map::new();
    age_param.insert("value".to_string(), serde_json::Value::Number(serde_json::Number::from(25)));

    let params = types::EnhancedSqlParams {
        sql: query_sql.to_string(),
        params: vec![
            types::TypedSqlParam {
                value: age_param,
                type_hint: Some(types::SqlTypeHint::Integer),
            },
        ],
    };

    match sqlite_query_enhanced(params) {
        Ok(result) => {
            let execution_time = start_time.elapsed().as_millis() as f64;
            
            // Verify we got multiple rows back
            let expected_min_rows = test_users.len(); // All test users should match age >= 25
            let actual_rows = result.metadata.rows_returned as usize;
            
            // Check if we got at least the expected number of rows (there might be more from previous tests)
            let success = actual_rows >= expected_min_rows;
            
            // Verify the data structure is correct
            let has_correct_columns = result.columns.len() == 4 &&
                result.columns.iter().any(|c| c.name == "id") &&
                result.columns.iter().any(|c| c.name == "name") &&
                result.columns.iter().any(|c| c.name == "email") &&
                result.columns.iter().any(|c| c.name == "age");
            
            let final_success = success && has_correct_columns;
            
            types::TestStepResult {
                step_name: "Query Multiple Rows".to_string(),
                success: final_success,
                execution_time_ms: execution_time,
                error_message: if final_success {
                    None
                } else {
                    Some(format!("Expected at least {} rows, got {}. Columns correct: {}",
                        expected_min_rows, actual_rows, has_correct_columns))
                },
                details: Some(create_details_map(&[
                    ("rows_returned", &result.metadata.rows_returned.to_string()),
                    ("expected_min_rows", &expected_min_rows.to_string()),
                    ("column_count", &result.columns.len().to_string()),
                    ("has_correct_columns", &has_correct_columns.to_string()),
                    ("execution_time_ms", &result.metadata.execution_time_ms.to_string()),
                ])),
            }
        }
        Err(e) => types::TestStepResult {
            step_name: "Query Multiple Rows".to_string(),
            success: false,
            execution_time_ms: start_time.elapsed().as_millis() as f64,
            error_message: Some(format!("Failed to query multiple rows: {}", e)),
            details: None,
        },
    }
}

fn create_details_map(pairs: &[(&str, &str)]) -> serde_json::Map<String, serde_json::Value> {
    let mut map = serde_json::Map::new();
    for (key, value) in pairs {
        map.insert(key.to_string(), serde_json::Value::String(value.to_string()));
    }
    map
}

fn find_fastest_slowest_operations(results: &[types::TestStepResult]) -> (Option<String>, Option<String>) {
    if results.is_empty() {
        return (None, None);
    }

    let mut fastest = &results[0];
    let mut slowest = &results[0];

    for result in results.iter() {
        if result.execution_time_ms < fastest.execution_time_ms {
            fastest = result;
        }
        if result.execution_time_ms > slowest.execution_time_ms {
            slowest = result;
        }
    }

    (Some(fastest.step_name.clone()), Some(slowest.step_name.clone()))
}
