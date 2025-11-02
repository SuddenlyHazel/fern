from typing import Optional, List  # noqa: F401
from datetime import datetime  # noqa: F401
import extism  # noqa: F401 # pyright: ignore
import time
import json


from pdk_types import (
    ColumnInfo,
    DatabaseStats,
    EmptyInput,
    EnhancedSqlParams,
    EnhancedSqlResult,
    IndexInfo,
    KvReadInput,
    KvStoreInput,
    PerformanceSummary,
    QueryExplanation,
    QueryMetadata,
    SqliteTestResult,
    TableInfo,
    TableNameInput,
    TestStepResult,
    TransactionIdInput,
    TransactionResult,
    TypedSqlParam,
)  # noqa: F401


from pdk_imports import (
    kv_read,
    kv_store,
    sqlite_begin_transaction,
    sqlite_commit_transaction,
    sqlite_describe_table,
    sqlite_execute_enhanced,
    sqlite_explain_query,
    sqlite_get_stats,
    sqlite_list_tables,
    sqlite_query_enhanced,
    sqlite_rollback_transaction,
)  # noqa: F401


def create_details_map(pairs: List[tuple[str, str]]) -> dict:
    """Create a details dictionary from key-value pairs"""
    return {key: value for key, value in pairs}


def find_fastest_slowest_operations(results: List[TestStepResult]) -> tuple[Optional[str], Optional[str]]:
    """Find the fastest and slowest operations from test results"""
    if not results:
        return (None, None)
    
    fastest = results[0]
    slowest = results[0]
    
    for result in results:
        if result.executionTimeMs < fastest.executionTimeMs:
            fastest = result
        if result.executionTimeMs > slowest.executionTimeMs:
            slowest = result
    
    return (fastest.stepName, slowest.stepName)


def test_list_tables() -> TestStepResult:
    """Test listing tables in the database"""
    start_time = time.time()
    
    try:
        tables_json = sqlite_list_tables(EmptyInput())
        execution_time = (time.time() - start_time) * 1000
        
        # Parse the JSON to verify it's valid
        try:
            tables = json.loads(tables_json)
            return TestStepResult(
                stepName="List Tables",
                success=True,
                executionTimeMs=execution_time,
                errorMessage=None,
                details=create_details_map([("tables_json", tables_json)])
            )
        except json.JSONDecodeError as e:
            return TestStepResult(
                stepName="List Tables",
                success=False,
                executionTimeMs=execution_time,
                errorMessage=f"Failed to parse tables JSON: {e}",
                details=None
            )
    except Exception as e:
        execution_time = (time.time() - start_time) * 1000
        return TestStepResult(
            stepName="List Tables",
            success=False,
            executionTimeMs=execution_time,
            errorMessage=f"Failed to list tables: {e}",
            details=None
        )


def test_create_table() -> TestStepResult:
    """Test creating a table"""
    start_time = time.time()
    
    create_sql = """CREATE TABLE test_users (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        email TEXT UNIQUE,
        age INTEGER,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP
    )"""
    
    params = EnhancedSqlParams(
        sql=create_sql,
        params=[]
    )
    
    try:
        result = sqlite_execute_enhanced(params)
        execution_time = (time.time() - start_time) * 1000
        return TestStepResult(
            stepName="Create Table",
            success=True,
            executionTimeMs=execution_time,
            errorMessage=None,
            details=create_details_map([
                ("rows_affected", str(result.metadata.rowsAffected)),
                ("sqlite_version", result.metadata.sqliteVersion)
            ])
        )
    except Exception as e:
        execution_time = (time.time() - start_time) * 1000
        return TestStepResult(
            stepName="Create Table",
            success=False,
            executionTimeMs=execution_time,
            errorMessage=f"Failed to create table: {e}",
            details=None
        )


def test_describe_table() -> TestStepResult:
    """Test describing table structure"""
    start_time = time.time()
    
    input_data = TableNameInput(tableName="test_users")
    
    try:
        table_info = sqlite_describe_table(input_data)
        execution_time = (time.time() - start_time) * 1000
        return TestStepResult(
            stepName="Describe Table",
            success=True,
            executionTimeMs=execution_time,
            errorMessage=None,
            details=create_details_map([
                ("table_name", table_info.name),
                ("column_count", str(len(table_info.columns))),
                ("index_count", str(len(table_info.indexes)))
            ])
        )
    except Exception as e:
        execution_time = (time.time() - start_time) * 1000
        return TestStepResult(
            stepName="Describe Table",
            success=False,
            executionTimeMs=execution_time,
            errorMessage=f"Failed to describe table: {e}",
            details=None
        )


def test_insert_data() -> TestStepResult:
    """Test inserting data into the table"""
    start_time = time.time()
    
    insert_sql = "INSERT INTO test_users (name, email, age) VALUES (?, ?, ?)"
    
    # Create typed parameters
    name_value = {"value": "John Doe"}
    email_value = {"value": "john@example.com"}
    age_value = {"value": 30}
    
    params = EnhancedSqlParams(
        sql=insert_sql,
        params=[
            TypedSqlParam(value=name_value, typeHint="TEXT"),
            TypedSqlParam(value=email_value, typeHint="TEXT"),
            TypedSqlParam(value=age_value, typeHint="INTEGER")
        ]
    )
    
    try:
        result = sqlite_execute_enhanced(params)
        execution_time = (time.time() - start_time) * 1000
        return TestStepResult(
            stepName="Insert Data",
            success=True,
            executionTimeMs=execution_time,
            errorMessage=None,
            details=create_details_map([
                ("rows_affected", str(result.metadata.rowsAffected)),
                ("last_insert_rowid", str(result.metadata.lastInsertRowid) if result.metadata.lastInsertRowid else "None")
            ])
        )
    except Exception as e:
        execution_time = (time.time() - start_time) * 1000
        return TestStepResult(
            stepName="Insert Data",
            success=False,
            executionTimeMs=execution_time,
            errorMessage=f"Failed to insert data: {e}",
            details=None
        )


def test_query_enhanced() -> TestStepResult:
    """Test querying data with enhanced functionality"""
    start_time = time.time()
    
    query_sql = "SELECT id, name, email, age FROM test_users WHERE age > ?"
    age_param = {"value": 25}
    
    params = EnhancedSqlParams(
        sql=query_sql,
        params=[
            TypedSqlParam(value=age_param, typeHint="INTEGER")
        ]
    )
    
    try:
        result = sqlite_query_enhanced(params)
        execution_time = (time.time() - start_time) * 1000
        return TestStepResult(
            stepName="Query Enhanced",
            success=True,
            executionTimeMs=execution_time,
            errorMessage=None,
            details=create_details_map([
                ("rows_returned", str(result.metadata.rowsReturned)),
                ("column_count", str(len(result.columns))),
                ("execution_time_ms", str(result.metadata.executionTimeMs))
            ])
        )
    except Exception as e:
        execution_time = (time.time() - start_time) * 1000
        return TestStepResult(
            stepName="Query Enhanced",
            success=False,
            executionTimeMs=execution_time,
            errorMessage=f"Failed to query data: {e}",
            details=None
        )


def test_transactions() -> TestStepResult:
    """Test database transactions"""
    start_time = time.time()
    
    try:
        # Begin transaction
        tx_result = sqlite_begin_transaction(EmptyInput())
        if not tx_result.success:
            return TestStepResult(
                stepName="Transactions",
                success=False,
                executionTimeMs=(time.time() - start_time) * 1000,
                errorMessage="Failed to begin transaction",
                details=None
            )
        
        # Insert data within transaction
        insert_sql = "INSERT INTO test_users (name, email, age) VALUES (?, ?, ?)"
        
        name_value = {"value": "Jane Doe"}
        email_value = {"value": "jane@example.com"}
        age_value = {"value": 28}
        
        params = EnhancedSqlParams(
            sql=insert_sql,
            params=[
                TypedSqlParam(value=name_value, typeHint="TEXT"),
                TypedSqlParam(value=email_value, typeHint="TEXT"),
                TypedSqlParam(value=age_value, typeHint="INTEGER")
            ]
        )
        
        try:
            sqlite_execute_enhanced(params)
            
            # Commit transaction
            commit_input = TransactionIdInput(transactionId=tx_result.transactionId)
            commit_result = sqlite_commit_transaction(commit_input)
            
            execution_time = (time.time() - start_time) * 1000
            return TestStepResult(
                stepName="Transactions",
                success=commit_result.success,
                executionTimeMs=execution_time,
                errorMessage=None,
                details=create_details_map([
                    ("transaction_id", tx_result.transactionId),
                    ("committed", str(commit_result.success))
                ])
            )
        except Exception as e:
            # Rollback on error
            rollback_input = TransactionIdInput(transactionId=tx_result.transactionId)
            sqlite_rollback_transaction(rollback_input)
            
            return TestStepResult(
                stepName="Transactions",
                success=False,
                executionTimeMs=(time.time() - start_time) * 1000,
                errorMessage=f"Failed to insert in transaction: {e}",
                details=None
            )
    except Exception as e:
        return TestStepResult(
            stepName="Transactions",
            success=False,
            executionTimeMs=(time.time() - start_time) * 1000,
            errorMessage=f"Failed to begin transaction: {e}",
            details=None
        )


def test_query_explanation() -> TestStepResult:
    """Test query explanation functionality"""
    start_time = time.time()
    
    query_sql = "SELECT * FROM test_users WHERE age > 25 ORDER BY name"
    
    params = EnhancedSqlParams(
        sql=query_sql,
        params=[]
    )
    
    try:
        explanation = sqlite_explain_query(params)
        execution_time = (time.time() - start_time) * 1000
        return TestStepResult(
            stepName="Query Explanation",
            success=True,
            executionTimeMs=execution_time,
            errorMessage=None,
            details=create_details_map([
                ("estimated_cost", str(explanation.estimatedCost)),
                ("estimated_rows", str(explanation.estimatedRows)),
                ("sqlite_version", explanation.sqliteVersion)
            ])
        )
    except Exception as e:
        execution_time = (time.time() - start_time) * 1000
        return TestStepResult(
            stepName="Query Explanation",
            success=False,
            executionTimeMs=execution_time,
            errorMessage=f"Failed to explain query: {e}",
            details=None
        )


def test_database_stats() -> TestStepResult:
    """Test getting database statistics"""
    start_time = time.time()
    
    try:
        stats = sqlite_get_stats(EmptyInput())
        execution_time = (time.time() - start_time) * 1000
        return TestStepResult(
            stepName="Database Stats",
            success=True,
            executionTimeMs=execution_time,
            errorMessage=None,
            details=create_details_map([
                ("total_queries", str(stats.totalQueries)),
                ("database_size_bytes", str(stats.databaseSizeBytes)),
                ("average_execution_time_ms", str(stats.averageExecutionTimeMs)),
                ("sqlite_version", stats.sqliteVersion)
            ])
        )
    except Exception as e:
        execution_time = (time.time() - start_time) * 1000
        return TestStepResult(
            stepName="Database Stats",
            success=False,
            executionTimeMs=execution_time,
            errorMessage=f"Failed to get database stats: {e}",
            details=None
        )


def test_transaction_rollback() -> TestStepResult:
    """Test transaction rollback functionality"""
    start_time = time.time()
    
    try:
        # Begin transaction
        tx_result = sqlite_begin_transaction(EmptyInput())
        if not tx_result.success:
            return TestStepResult(
                stepName="Transaction Rollback",
                success=False,
                executionTimeMs=(time.time() - start_time) * 1000,
                errorMessage="Failed to begin transaction",
                details=None
            )
        
        # Insert data within transaction (this will be rolled back)
        insert_sql = "INSERT INTO test_users (name, email, age) VALUES (?, ?, ?)"
        
        name_value = {"value": "Rollback User"}
        email_value = {"value": "rollback@example.com"}
        age_value = {"value": 99}
        
        params = EnhancedSqlParams(
            sql=insert_sql,
            params=[
                TypedSqlParam(value=name_value, typeHint="TEXT"),
                TypedSqlParam(value=email_value, typeHint="TEXT"),
                TypedSqlParam(value=age_value, typeHint="INTEGER")
            ]
        )
        
        try:
            sqlite_execute_enhanced(params)
            
            # Rollback transaction instead of committing
            rollback_input = TransactionIdInput(transactionId=tx_result.transactionId)
            rollback_result = sqlite_rollback_transaction(rollback_input)
            
            execution_time = (time.time() - start_time) * 1000
            return TestStepResult(
                stepName="Transaction Rollback",
                success=rollback_result.success,
                executionTimeMs=execution_time,
                errorMessage=None,
                details=create_details_map([
                    ("transaction_id", tx_result.transactionId),
                    ("rolled_back", str(rollback_result.success))
                ])
            )
        except Exception as e:
            return TestStepResult(
                stepName="Transaction Rollback",
                success=False,
                executionTimeMs=(time.time() - start_time) * 1000,
                errorMessage=f"Failed to insert in transaction: {e}",
                details=None
            )
    except Exception as e:
        return TestStepResult(
            stepName="Transaction Rollback",
            success=False,
            executionTimeMs=(time.time() - start_time) * 1000,
            errorMessage=f"Failed to begin transaction: {e}",
            details=None
        )


def test_kv_store() -> TestStepResult:
    """Test key-value store functionality"""
    start_time = time.time()
    
    # Store a value
    test_value = {
        "message": "Hello from KV store!",
        "timestamp": 1699123456
    }
    
    store_input = KvStoreInput(
        table="test_kv",
        key="test_key",
        value=test_value
    )
    
    try:
        kv_store(store_input)
        
        # Read the value back
        read_input = KvReadInput(
            table="test_kv",
            key="test_key"
        )
        
        # Work around the generated code issue by calling the raw function directly
        try:
            from pdk_imports import _kv_read
            result_json = _kv_read(read_input.to_json())
            if result_json and result_json.strip():
                retrieved_value = json.loads(result_json)
            else:
                retrieved_value = None
        except Exception:
            # Fallback to the generated wrapper if available
            try:
                retrieved_value = kv_read(read_input)
            except Exception:
                retrieved_value = None
        
        execution_time = (time.time() - start_time) * 1000
        
        if retrieved_value is not None:
            values_match = retrieved_value == test_value
            
            return TestStepResult(
                stepName="KV Store",
                success=values_match,
                executionTimeMs=execution_time,
                errorMessage=None if values_match else "Retrieved value doesn't match stored value",
                details=create_details_map([
                    ("stored_successfully", "true"),
                    ("retrieved_successfully", "true"),
                    ("values_match", str(values_match))
                ])
            )
        else:
            return TestStepResult(
                stepName="KV Store",
                success=False,
                executionTimeMs=execution_time,
                errorMessage="Value not found after storing",
                details=None
            )
    except Exception as e:
        execution_time = (time.time() - start_time) * 1000
        return TestStepResult(
            stepName="KV Store",
            success=False,
            executionTimeMs=execution_time,
            errorMessage=f"Failed to store/read KV value: {e}",
            details=None
        )


# Test entry point that demonstrates all SQLite functionality
def test_enhanced_sql(input: str) -> SqliteTestResult:
    """Main test function that runs all SQLite functionality tests"""
    results = []
    performance_summary = PerformanceSummary(
        totalQueries=0,
        totalExecutionTimeMs=0.0,
        averageQueryTimeMs=0.0,
        databaseOperations=[],
        fastestOperation=None,
        slowestOperation=None
    )
    
    test_name = "Comprehensive SQLite Test Suite"
    overall_success = True
    
    # Test 1: List tables (should be empty initially)
    step_result = test_list_tables()
    overall_success &= step_result.success
    performance_summary.totalQueries += 1
    performance_summary.totalExecutionTimeMs += step_result.executionTimeMs
    performance_summary.databaseOperations.append("list_tables")
    results.append(step_result)
    
    # Test 2: Create a test table
    step_result = test_create_table()
    overall_success &= step_result.success
    performance_summary.totalQueries += 1
    performance_summary.totalExecutionTimeMs += step_result.executionTimeMs
    performance_summary.databaseOperations.append("create_table")
    results.append(step_result)
    
    # Test 3: Describe the table structure
    step_result = test_describe_table()
    overall_success &= step_result.success
    performance_summary.totalQueries += 1
    performance_summary.totalExecutionTimeMs += step_result.executionTimeMs
    performance_summary.databaseOperations.append("describe_table")
    results.append(step_result)
    
    # Test 4: Insert test data
    step_result = test_insert_data()
    overall_success &= step_result.success
    performance_summary.totalQueries += 1
    performance_summary.totalExecutionTimeMs += step_result.executionTimeMs
    performance_summary.databaseOperations.append("insert_data")
    results.append(step_result)
    
    # Test 5: Query data with enhanced functionality
    step_result = test_query_enhanced()
    overall_success &= step_result.success
    performance_summary.totalQueries += 1
    performance_summary.totalExecutionTimeMs += step_result.executionTimeMs
    performance_summary.databaseOperations.append("query_enhanced")
    results.append(step_result)
    
    # Test 6: Test transactions
    step_result = test_transactions()
    overall_success &= step_result.success
    performance_summary.totalQueries += 3  # begin, insert, commit
    performance_summary.totalExecutionTimeMs += step_result.executionTimeMs
    performance_summary.databaseOperations.append("transactions")
    results.append(step_result)
    
    # Test 7: Test query explanation
    step_result = test_query_explanation()
    overall_success &= step_result.success
    performance_summary.totalQueries += 1
    performance_summary.totalExecutionTimeMs += step_result.executionTimeMs
    performance_summary.databaseOperations.append("explain_query")
    results.append(step_result)
    
    # Test 8: Get database statistics
    step_result = test_database_stats()
    overall_success &= step_result.success
    performance_summary.totalQueries += 1
    performance_summary.totalExecutionTimeMs += step_result.executionTimeMs
    performance_summary.databaseOperations.append("get_stats")
    results.append(step_result)
    
    # Test 9: Test transaction rollback
    step_result = test_transaction_rollback()
    overall_success &= step_result.success
    performance_summary.totalQueries += 3  # begin, insert, rollback
    performance_summary.totalExecutionTimeMs += step_result.executionTimeMs
    performance_summary.databaseOperations.append("transaction_rollback")
    results.append(step_result)
    
    # Test 10: Test KV store functionality
    step_result = test_kv_store()
    overall_success &= step_result.success
    performance_summary.totalQueries += 2  # store + read
    performance_summary.totalExecutionTimeMs += step_result.executionTimeMs
    performance_summary.databaseOperations.append("kv_store")
    results.append(step_result)
    
    # Calculate final performance metrics
    if performance_summary.totalQueries > 0:
        performance_summary.averageQueryTimeMs = (
            performance_summary.totalExecutionTimeMs / performance_summary.totalQueries
        )
    
    # Find fastest and slowest operations
    fastest_op, slowest_op = find_fastest_slowest_operations(results)
    performance_summary.fastestOperation = fastest_op
    performance_summary.slowestOperation = slowest_op
    
    result = SqliteTestResult(
        testName=test_name,
        success=overall_success,
        results=results,
        performanceSummary=performance_summary,
        errorMessage=None if overall_success else "Some tests failed"
    )
    
    return result
