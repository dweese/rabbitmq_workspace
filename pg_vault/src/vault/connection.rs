// pg_vault/src/vault/connection.rs

//! High-level database connection with security and convenience features

use crate::vault::{SessionInfo, VaultError, VaultResult};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio_postgres::{Client, Row, Statement, Error as PgError};

/// Connection configuration
#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    /// Query timeout in seconds
    pub query_timeout: u64,
    /// Enable query logging
    pub log_queries: bool,
    /// Maximum query result size in MB
    pub max_result_size_mb: usize,
    /// Connection pool settings
    pub pool_settings: PoolConfig,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            query_timeout: 30,
            log_queries: false,
            max_result_size_mb: 100,
            pool_settings: PoolConfig::default(),
        }
    }
}

/// Pool configuration
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum number of connections in pool
    pub max_connections: usize,
    /// Minimum number of connections to maintain
    pub min_connections: usize,
    /// Connection idle timeout
    pub idle_timeout: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 10,
            min_connections: 1,
            idle_timeout: Duration::from_secs(600), // 10 minutes
        }
    }
}

/// Query execution parameters
#[derive(Debug)]
pub struct QueryParams<'a> {
    /// SQL query string
    pub sql: &'a str,
    /// Query parameters
    pub params: &'a [&'a (dyn tokio_postgres::types::ToSql + Sync)],
    /// Query timeout override
    pub timeout: Option<Duration>,
}

/// Query result wrapper
#[derive(Debug)]
pub struct QueryResult {
    /// Result rows
    pub rows: Vec<Row>,
    /// Number of rows affected (for INSERT/UPDATE/DELETE)
    pub rows_affected: u64,
    /// Query execution time
    pub execution_time: Duration,
    /// Query type
    pub query_type: QueryType,
}

/// Type of SQL query
#[derive(Debug, Clone, PartialEq)]
pub enum QueryType {
    Select,
    Insert,
    Update,
    Delete,
    Create,
    Drop,
    Alter,
    Other,
}

impl QueryType {
    /// Determine query type from SQL string
    pub fn from_sql(sql: &str) -> Self {
        let sql_trimmed = sql.trim().to_uppercase();
        
        if sql_trimmed.starts_with("SELECT") {
            QueryType::Select
        } else if sql_trimmed.starts_with("INSERT") {
            QueryType::Insert
        } else if sql_trimmed.starts_with("UPDATE") {
            QueryType::Update
        } else if sql_trimmed.starts_with("DELETE") {
            QueryType::Delete
        } else if sql_trimmed.starts_with("CREATE") {
            QueryType::Create
        } else if sql_trimmed.starts_with("DROP") {
            QueryType::Drop
        } else if sql_trimmed.starts_with("ALTER") {
            QueryType::Alter
        } else {
            QueryType::Other
        }
    }
}

/// Connection metrics for monitoring
#[derive(Debug, Clone, Default)]
pub struct ConnectionMetrics {
    /// Total queries executed
    pub total_queries: u64,
    /// Queries by type
    pub queries_by_type: HashMap<String, u64>,
    /// Total execution time
    pub total_execution_time: Duration,
    /// Average execution time
    pub avg_execution_time: Duration,
    /// Number of errors
    pub error_count: u64,
    /// Last query timestamp
    pub last_query_time: Option<SystemTime>,
}

impl ConnectionMetrics {
    fn update_query_stats(&mut self, query_type: &QueryType, execution_time: Duration) {
        self.total_queries += 1;
        self.total_execution_time += execution_time;
        self.avg_execution_time = self.total_execution_time / self.total_queries as u32;
        
        let type_key = format!("{query_type:?}");
        *self.queries_by_type.entry(type_key).or_insert(0) += 1;
        
        self.last_query_time = Some(SystemTime::now());
    }
    
    fn update_error_stats(&mut self) {
        self.error_count += 1;
    }
}

/// Database transaction wrapper
pub struct Transaction<'a> {
    /// The underlying transaction
    pub transaction: tokio_postgres::Transaction<'a>,
}

impl<'a> Transaction<'a> {
    /// Execute a query within the transaction
    pub async fn query(&self, sql: &str, params: &[&(dyn tokio_postgres::types::ToSql + Sync)]) -> Result<Vec<Row>, PgError> {
        self.transaction.query(sql, params).await
    }
    
    /// Execute a statement within the transaction
    pub async fn execute(&self, sql: &str, params: &[&(dyn tokio_postgres::types::ToSql + Sync)]) -> Result<u64, PgError> {
        self.transaction.execute(sql, params).await
    }
    
    /// Commit the transaction
    pub async fn commit(self) -> Result<(), PgError> {
        self.transaction.commit().await
    }
    
    /// Rollback the transaction
    pub async fn rollback(self) -> Result<(), PgError> {
        self.transaction.rollback().await
    }
}

/// High-level database connection with security and convenience features
pub struct Connection {
    /// The underlying database client
    client: Client,
    /// Session identifier
    session_id: String,
    /// Reference to sessions for activity tracking
    sessions: Arc<tokio::sync::Mutex<HashMap<String, SessionInfo>>>,
    /// Connection configuration
    config: ConnectionConfig,
    /// Query metrics
    metrics: Arc<tokio::sync::Mutex<ConnectionMetrics>>,
}

impl Connection {
    /// Create a new connection wrapper
    pub fn new(
        client: Client,
        session_id: String,
        sessions: Arc<tokio::sync::Mutex<HashMap<String, SessionInfo>>>,
        config: ConnectionConfig,
    ) -> Self {
        Self {
            client,
            session_id,
            sessions,
            config,
            metrics: Arc::new(tokio::sync::Mutex::new(ConnectionMetrics::default())),
        }
    }
    
    /// Execute a simple query
    pub async fn query(&self, sql: &str, params: &[&(dyn tokio_postgres::types::ToSql + Sync)]) -> VaultResult<QueryResult> {
        let start_time = SystemTime::now();
        
        // Update session activity
        self.update_session_activity().await;
        
        // Execute query with timeout
        let timeout_duration = Duration::from_secs(self.config.query_timeout);
        let result = tokio::time::timeout(timeout_duration, self.client.query(sql, params)).await;
        
        let execution_time = start_time.elapsed().unwrap_or(Duration::ZERO);
        let query_type = QueryType::from_sql(sql);
        
        match result {
            Ok(Ok(rows)) => {
                // Update metrics
                {
                    let mut metrics = self.metrics.lock().await;
                    metrics.update_query_stats(&query_type, execution_time);
                }
                
                Ok(QueryResult {
                    rows_affected: 0, // SELECT doesn't affect rows
                    rows,
                    execution_time,
                    query_type,
                })
            }
            Ok(Err(e)) => {
                // Update error metrics
                {
                    let mut metrics = self.metrics.lock().await;
                    metrics.update_error_stats();
                }
                Err(VaultError::Database(e))
            }
            Err(_) => {
                // Timeout occurred
                {
                    let mut metrics = self.metrics.lock().await;
                    metrics.update_error_stats();
                }
                Err(VaultError::Timeout(format!("Query timeout after {}s", self.config.query_timeout)))
            }
        }
    }
    
    /// Execute a statement (INSERT, UPDATE, DELETE)
    pub async fn execute(&self, sql: &str, params: &[&(dyn tokio_postgres::types::ToSql + Sync)]) -> VaultResult<QueryResult> {
        let start_time = SystemTime::now();
        
        // Update session activity
        self.update_session_activity().await;
        
        // Execute statement with timeout
        let timeout_duration = Duration::from_secs(self.config.query_timeout);
        let result = tokio::time::timeout(timeout_duration, self.client.execute(sql, params)).await;
        
        let execution_time = start_time.elapsed().unwrap_or(Duration::ZERO);
        let query_type = QueryType::from_sql(sql);
        
        match result {
            Ok(Ok(rows_affected)) => {
                // Update metrics
                {
                    let mut metrics = self.metrics.lock().await;
                    metrics.update_query_stats(&query_type, execution_time);
                }
                
                Ok(QueryResult {
                    rows: Vec::new(), // No rows returned for execute
                    rows_affected,
                    execution_time,
                    query_type,
                })
            }
            Ok(Err(e)) => {
                // Update error metrics
                {
                    let mut metrics = self.metrics.lock().await;
                    metrics.update_error_stats();
                }
                Err(VaultError::Database(e))
            }
            Err(_) => {
                // Timeout occurred
                {
                    let mut metrics = self.metrics.lock().await;
                    metrics.update_error_stats();
                }
                Err(VaultError::Timeout(format!("Execute timeout after {}s", self.config.query_timeout)))
            }
        }
    }
    
    /// Prepare a statement for repeated execution
    pub async fn prepare(&self, sql: &str) -> VaultResult<Statement> {
        self.client.prepare(sql).await.map_err(VaultError::Database)
    }
    
    /// Execute a prepared statement
    pub async fn query_prepared(&self, statement: &Statement, params: &[&(dyn tokio_postgres::types::ToSql + Sync)]) -> VaultResult<QueryResult> {
        let start_time = SystemTime::now();
        
        // Update session activity
        self.update_session_activity().await;
        
        // Get query type from the prepared statement's SQL
        // Note: tokio-postgres Statement doesn't expose the SQL directly
        // For now, we'll assume it's a SELECT query
        let query_type = QueryType::Select;
        
        // Execute prepared statement with timeout
        let timeout_duration = Duration::from_secs(self.config.query_timeout);
        let result = tokio::time::timeout(timeout_duration, self.client.query(statement, params)).await;
        
        let execution_time = start_time.elapsed().unwrap_or(Duration::ZERO);
        
        match result {
            Ok(Ok(rows)) => {
                // Update metrics
                {
                    let mut metrics = self.metrics.lock().await;
                    metrics.update_query_stats(&query_type, execution_time);
                }
                
                Ok(QueryResult {
                    rows_affected: 0,
                    rows,
                    execution_time,
                    query_type,
                })
            }
            Ok(Err(e)) => {
                // Update error metrics
                {
                    let mut metrics = self.metrics.lock().await;
                    metrics.update_error_stats();
                }
                Err(VaultError::Database(e))
            }
            Err(_) => {
                // Timeout occurred
                {
                    let mut metrics = self.metrics.lock().await;
                    metrics.update_error_stats();
                }
                Err(VaultError::Timeout(format!("Query prepared timeout after {}s", self.config.query_timeout)))
            }
        }
    }
    
    /// Execute a prepared statement (INSERT, UPDATE, DELETE)
    pub async fn execute_prepared(&self, statement: &Statement, params: &[&(dyn tokio_postgres::types::ToSql + Sync)]) -> VaultResult<QueryResult> {
        let start_time = SystemTime::now();
        
        // Update session activity
        self.update_session_activity().await;
        
        let query_type = QueryType::Other; // Can't determine from prepared statement
        
        // Execute prepared statement with timeout
        let timeout_duration = Duration::from_secs(self.config.query_timeout);
        let result = tokio::time::timeout(timeout_duration, self.client.execute(statement, params)).await;
        
        let execution_time = start_time.elapsed().unwrap_or(Duration::ZERO);
        
        match result {
            Ok(Ok(rows_affected)) => {
                // Update metrics
                {
                    let mut metrics = self.metrics.lock().await;
                    metrics.update_query_stats(&query_type, execution_time);
                }
                
                Ok(QueryResult {
                    rows: Vec::new(),
                    rows_affected,
                    execution_time,
                    query_type,
                })
            }
            Ok(Err(e)) => {
                // Update error metrics
                {
                    let mut metrics = self.metrics.lock().await;
                    metrics.update_error_stats();
                }
                Err(VaultError::Database(e))
            }
            Err(_) => {
                // Timeout occurred
                {
                    let mut metrics = self.metrics.lock().await;
                    metrics.update_error_stats();
                }
                Err(VaultError::Timeout(format!("Execute prepared timeout after {}s", self.config.query_timeout)))
            }
        }
    }
    
    /// Begin a database transaction
    pub async fn begin_transaction(&mut self) -> VaultResult<Transaction<'_>> {
        let transaction = self.client.transaction().await.map_err(VaultError::Database)?;
        Ok(Transaction { transaction })
    }
    
    /// Get connection metrics
    pub async fn get_metrics(&self) -> ConnectionMetrics {
        self.metrics.lock().await.clone()
    }
    
    /// Get session information
    pub async fn session_info(&self) -> Option<SessionInfo> {
        self.sessions.lock().await.get(&self.session_id).cloned()
    }
    
    /// Get session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }
    
    /// Check if the connection is still valid
    pub async fn is_valid(&self) -> bool {
        // Try to execute a simple query
        (self.client.query("SELECT 1", &[]).await).is_ok()
    }
    
    /// Get database version
    pub async fn database_version(&self) -> VaultResult<String> {
        let rows = self.client
            .query("SELECT version()", &[])
            .await
            .map_err(VaultError::Database)?;
        
        if let Some(row) = rows.first() {
            let version: String = row.get(0);
            Ok(version)
        } else {
        Err(VaultError::Configuration(
            "No version returned from database".to_string()
        ))
    }
}
    
    async fn update_session_activity(&self) {
        let mut sessions = self.sessions.lock().await;
        if let Some(session) = sessions.get_mut(&self.session_id) {
            session.update_activity();
        }
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        // Clean up session when connection is dropped
        let session_id = self.session_id.clone();
        let sessions = self.sessions.clone();
        
        tokio::spawn(async move {
            let mut sessions_guard = sessions.lock().await;
            sessions_guard.remove(&session_id);
        });
    }
}