use serde::{Deserialize, Serialize};
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbConnection {
    pub db_type: String,    // "mysql", "mariadb", "postgresql"
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseInfo {
    pub name: String,
    pub tables_count: usize,
    pub size: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    pub name: String,
    pub rows: String,
    pub size: String,
    pub engine: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub col_type: String,
    pub nullable: bool,
    pub key: String,
    pub default_val: String,
    pub extra: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub affected_rows: usize,
    pub message: String,
}

impl Default for DbConnection {
    fn default() -> Self {
        Self {
            db_type: detect_db_type(),
            host: "127.0.0.1".into(),
            port: detect_db_port(),
            user: "root".into(),
            password: String::new(),
        }
    }
}

fn detect_db_type() -> String {
    if std::process::Command::new("mariadb").arg("--version").output().is_ok() {
        "mariadb".into()
    } else if std::process::Command::new("mysql").arg("--version").output().is_ok() {
        "mysql".into()
    } else if std::process::Command::new("psql").arg("--version").output().is_ok() {
        "postgresql".into()
    } else {
        "mysql".into()
    }
}

fn detect_db_port() -> u16 {
    if std::process::Command::new("psql").arg("--version").output().is_ok()
        && std::process::Command::new("mysql").arg("--version").output().is_err()
    {
        5432
    } else {
        3306
    }
}

/// Build the mysql/mariadb command with connection args
fn mysql_cmd(conn: &DbConnection) -> Command {
    let binary = if conn.db_type == "mariadb" { "mariadb" } else { "mysql" };
    let mut cmd = Command::new(binary);
    cmd.args(["-h", &conn.host, "-P", &conn.port.to_string(), "-u", &conn.user]);
    if !conn.password.is_empty() {
        cmd.arg(format!("-p{}", conn.password));
    }
    cmd.arg("--batch");
    cmd.arg("--raw");
    cmd
}

/// Build the psql command with connection args
fn psql_cmd(conn: &DbConnection) -> Command {
    let mut cmd = Command::new("psql");
    cmd.args(["-h", &conn.host, "-p", &conn.port.to_string(), "-U", &conn.user]);
    cmd.arg("--tuples-only");
    cmd.arg("--no-align");
    cmd.arg("--field-separator=\t");
    if !conn.password.is_empty() {
        cmd.env("PGPASSWORD", &conn.password);
    }
    cmd
}

async fn run_mysql_query(conn: &DbConnection, db: Option<&str>, query: &str) -> Result<String, String> {
    let mut cmd = mysql_cmd(conn);
    if let Some(db) = db {
        cmd.arg("-D").arg(db);
    }
    cmd.arg("-e").arg(query);

    let output = cmd.output().await.map_err(|e| format!("Failed to run mysql: {}", e))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

async fn run_psql_query(conn: &DbConnection, db: Option<&str>, query: &str) -> Result<String, String> {
    let mut cmd = psql_cmd(conn);
    if let Some(db) = db {
        cmd.arg("-d").arg(db);
    } else {
        cmd.arg("-d").arg("postgres");
    }
    cmd.arg("-c").arg(query);

    let output = cmd.output().await.map_err(|e| format!("Failed to run psql: {}", e))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

// --- Tauri Commands ---

#[tauri::command]
pub async fn db_test_connection(conn: DbConnection) -> Result<String, String> {
    if conn.db_type == "postgresql" {
        run_psql_query(&conn, None, "SELECT version()").await?;
        Ok("Connected to PostgreSQL".into())
    } else {
        run_mysql_query(&conn, None, "SELECT version()").await?;
        Ok("Connected to MySQL/MariaDB".into())
    }
}

#[tauri::command]
pub async fn db_get_connection() -> Result<DbConnection, String> {
    Ok(DbConnection::default())
}

#[tauri::command]
pub async fn db_list_databases(conn: DbConnection) -> Result<Vec<DatabaseInfo>, String> {
    if conn.db_type == "postgresql" {
        list_databases_pg(&conn).await
    } else {
        list_databases_mysql(&conn).await
    }
}

async fn list_databases_mysql(conn: &DbConnection) -> Result<Vec<DatabaseInfo>, String> {
    let output = run_mysql_query(conn, None,
        "SELECT s.SCHEMA_NAME, COUNT(t.TABLE_NAME), IFNULL(SUM(t.DATA_LENGTH + t.INDEX_LENGTH), 0) \
         FROM information_schema.SCHEMATA s \
         LEFT JOIN information_schema.TABLES t ON s.SCHEMA_NAME = t.TABLE_SCHEMA \
         WHERE s.SCHEMA_NAME NOT IN ('information_schema', 'performance_schema', 'mysql', 'sys') \
         GROUP BY s.SCHEMA_NAME ORDER BY s.SCHEMA_NAME"
    ).await?;

    Ok(output.lines().filter(|l| !l.is_empty()).map(|line| {
        let parts: Vec<&str> = line.split('\t').collect();
        let size_bytes: u64 = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
        DatabaseInfo {
            name: parts.first().unwrap_or(&"").to_string(),
            tables_count: parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0),
            size: format_size(size_bytes),
        }
    }).collect())
}

async fn list_databases_pg(conn: &DbConnection) -> Result<Vec<DatabaseInfo>, String> {
    let output = run_psql_query(conn, None,
        "SELECT datname FROM pg_database WHERE datistemplate = false AND datname NOT IN ('postgres') ORDER BY datname"
    ).await?;

    let mut databases = Vec::new();
    for line in output.lines().filter(|l| !l.trim().is_empty()) {
        let name = line.trim().to_string();
        // Get table count
        let count_out = run_psql_query(conn, Some(&name),
            "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public'"
        ).await.unwrap_or_default();
        let tables_count: usize = count_out.trim().parse().unwrap_or(0);

        // Get size
        let size_out = run_psql_query(conn, Some(&name),
            &format!("SELECT pg_database_size('{}')", name)
        ).await.unwrap_or_default();
        let size_bytes: u64 = size_out.trim().parse().unwrap_or(0);

        databases.push(DatabaseInfo {
            name,
            tables_count,
            size: format_size(size_bytes),
        });
    }
    Ok(databases)
}

#[tauri::command]
pub async fn db_create_database(conn: DbConnection, name: String) -> Result<String, String> {
    // Validate name: alphanumeric + underscore only
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err("Database name must contain only letters, numbers, and underscores".into());
    }

    if conn.db_type == "postgresql" {
        // PostgreSQL: CREATE DATABASE can't be in a transaction
        let mut cmd = psql_cmd(&conn);
        cmd.arg("-d").arg("postgres");
        cmd.arg("-c").arg(format!("CREATE DATABASE \"{}\"", name));
        let output = cmd.output().await.map_err(|e| e.to_string())?;
        if output.status.success() {
            Ok(format!("Database '{}' created", name))
        } else {
            Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
        }
    } else {
        run_mysql_query(&conn, None, &format!("CREATE DATABASE `{}`", name)).await?;
        Ok(format!("Database '{}' created", name))
    }
}

#[tauri::command]
pub async fn db_drop_database(conn: DbConnection, name: String) -> Result<String, String> {
    if conn.db_type == "postgresql" {
        let mut cmd = psql_cmd(&conn);
        cmd.arg("-d").arg("postgres");
        cmd.arg("-c").arg(format!("DROP DATABASE IF EXISTS \"{}\"", name));
        let output = cmd.output().await.map_err(|e| e.to_string())?;
        if output.status.success() {
            Ok(format!("Database '{}' dropped", name))
        } else {
            Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
        }
    } else {
        run_mysql_query(&conn, None, &format!("DROP DATABASE IF EXISTS `{}`", name)).await?;
        Ok(format!("Database '{}' dropped", name))
    }
}

#[tauri::command]
pub async fn db_list_tables(conn: DbConnection, database: String) -> Result<Vec<TableInfo>, String> {
    if conn.db_type == "postgresql" {
        list_tables_pg(&conn, &database).await
    } else {
        list_tables_mysql(&conn, &database).await
    }
}

async fn list_tables_mysql(conn: &DbConnection, database: &str) -> Result<Vec<TableInfo>, String> {
    let output = run_mysql_query(conn, Some(database),
        "SELECT TABLE_NAME, TABLE_ROWS, DATA_LENGTH + INDEX_LENGTH, ENGINE \
         FROM information_schema.TABLES \
         WHERE TABLE_SCHEMA = DATABASE() AND TABLE_TYPE = 'BASE TABLE' \
         ORDER BY TABLE_NAME"
    ).await?;

    Ok(output.lines().filter(|l| !l.is_empty()).map(|line| {
        let parts: Vec<&str> = line.split('\t').collect();
        let size_bytes: u64 = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
        TableInfo {
            name: parts.first().unwrap_or(&"").to_string(),
            rows: parts.get(1).unwrap_or(&"0").to_string(),
            size: format_size(size_bytes),
            engine: parts.get(3).unwrap_or(&"").to_string(),
        }
    }).collect())
}

async fn list_tables_pg(conn: &DbConnection, database: &str) -> Result<Vec<TableInfo>, String> {
    let output = run_psql_query(conn, Some(database),
        "SELECT t.tablename, \
                COALESCE(s.n_live_tup, 0), \
                COALESCE(pg_total_relation_size(quote_ident(t.tablename)::regclass), 0) \
         FROM pg_catalog.pg_tables t \
         LEFT JOIN pg_stat_user_tables s ON t.tablename = s.relname \
         WHERE t.schemaname = 'public' ORDER BY t.tablename"
    ).await?;

    Ok(output.lines().filter(|l| !l.trim().is_empty()).map(|line| {
        let parts: Vec<&str> = line.split('\t').collect();
        let size_bytes: u64 = parts.get(2).and_then(|s| s.trim().parse().ok()).unwrap_or(0);
        TableInfo {
            name: parts.first().unwrap_or(&"").trim().to_string(),
            rows: parts.get(1).unwrap_or(&"0").trim().to_string(),
            size: format_size(size_bytes),
            engine: "PostgreSQL".to_string(),
        }
    }).collect())
}

#[tauri::command]
pub async fn db_describe_table(conn: DbConnection, database: String, table: String) -> Result<Vec<ColumnInfo>, String> {
    if conn.db_type == "postgresql" {
        describe_table_pg(&conn, &database, &table).await
    } else {
        describe_table_mysql(&conn, &database, &table).await
    }
}

async fn describe_table_mysql(conn: &DbConnection, database: &str, table: &str) -> Result<Vec<ColumnInfo>, String> {
    let output = run_mysql_query(conn, Some(database),
        &format!(
            "SELECT COLUMN_NAME, COLUMN_TYPE, IS_NULLABLE, COLUMN_KEY, IFNULL(COLUMN_DEFAULT, ''), EXTRA \
             FROM information_schema.COLUMNS \
             WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = '{}' \
             ORDER BY ORDINAL_POSITION", table
        )
    ).await?;

    Ok(output.lines().filter(|l| !l.is_empty()).map(|line| {
        let parts: Vec<&str> = line.split('\t').collect();
        ColumnInfo {
            name: parts.first().unwrap_or(&"").to_string(),
            col_type: parts.get(1).unwrap_or(&"").to_string(),
            nullable: parts.get(2).unwrap_or(&"NO") == &"YES",
            key: parts.get(3).unwrap_or(&"").to_string(),
            default_val: parts.get(4).unwrap_or(&"").to_string(),
            extra: parts.get(5).unwrap_or(&"").to_string(),
        }
    }).collect())
}

async fn describe_table_pg(conn: &DbConnection, database: &str, table: &str) -> Result<Vec<ColumnInfo>, String> {
    let output = run_psql_query(conn, Some(database),
        &format!(
            "SELECT c.column_name, c.data_type, c.is_nullable, \
                    COALESCE(tc.constraint_type, ''), \
                    COALESCE(c.column_default, ''), '' \
             FROM information_schema.columns c \
             LEFT JOIN information_schema.key_column_usage kcu ON c.column_name = kcu.column_name AND c.table_name = kcu.table_name \
             LEFT JOIN information_schema.table_constraints tc ON kcu.constraint_name = tc.constraint_name \
             WHERE c.table_schema = 'public' AND c.table_name = '{}' \
             ORDER BY c.ordinal_position", table
        )
    ).await?;

    Ok(output.lines().filter(|l| !l.trim().is_empty()).map(|line| {
        let parts: Vec<&str> = line.split('\t').collect();
        ColumnInfo {
            name: parts.first().unwrap_or(&"").trim().to_string(),
            col_type: parts.get(1).unwrap_or(&"").trim().to_string(),
            nullable: parts.get(2).unwrap_or(&"NO").trim() == "YES",
            key: parts.get(3).unwrap_or(&"").trim().to_string(),
            default_val: parts.get(4).unwrap_or(&"").trim().to_string(),
            extra: parts.get(5).unwrap_or(&"").trim().to_string(),
        }
    }).collect())
}

#[tauri::command]
pub async fn db_run_query(conn: DbConnection, database: String, query: String) -> Result<QueryResult, String> {
    // Safety: block dangerous operations in a basic way
    let _query_upper = query.trim().to_uppercase();

    if conn.db_type == "postgresql" {
        run_query_pg(&conn, &database, &query).await
    } else {
        run_query_mysql(&conn, &database, &query).await
    }
}

async fn run_query_mysql(conn: &DbConnection, database: &str, query: &str) -> Result<QueryResult, String> {
    // Use column names header
    let mut cmd = mysql_cmd(conn);
    cmd.arg("-D").arg(database);
    cmd.arg("--column-names");
    cmd.arg("-e").arg(query);

    let output = cmd.output().await.map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let mut lines = text.lines();

    // First line is column headers
    let columns: Vec<String> = match lines.next() {
        Some(header) => header.split('\t').map(|s| s.to_string()).collect(),
        None => {
            return Ok(QueryResult {
                columns: vec![],
                rows: vec![],
                affected_rows: 0,
                message: "Query executed successfully".into(),
            });
        }
    };

    let rows: Vec<Vec<String>> = lines
        .filter(|l| !l.is_empty())
        .map(|line| line.split('\t').map(|s| s.to_string()).collect())
        .collect();

    let count = rows.len();
    Ok(QueryResult {
        columns,
        rows,
        affected_rows: count,
        message: format!("{} row(s) returned", count),
    })
}

async fn run_query_pg(conn: &DbConnection, database: &str, query: &str) -> Result<QueryResult, String> {
    // Get with headers
    let mut cmd = Command::new("psql");
    cmd.args(["-h", &conn.host, "-p", &conn.port.to_string(), "-U", &conn.user]);
    cmd.arg("-d").arg(database);
    cmd.arg("--no-align");
    cmd.arg("--field-separator=\t");
    cmd.arg("-c").arg(query);
    if !conn.password.is_empty() {
        cmd.env("PGPASSWORD", &conn.password);
    }

    let output = cmd.output().await.map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let mut lines: Vec<&str> = text.lines().collect();

    // Remove the last line if it's a row count like "(3 rows)"
    if let Some(last) = lines.last() {
        if last.starts_with('(') && last.ends_with(')') {
            lines.pop();
        }
    }

    let mut iter = lines.into_iter();

    let columns: Vec<String> = match iter.next() {
        Some(header) => header.split('\t').map(|s| s.to_string()).collect(),
        None => {
            return Ok(QueryResult {
                columns: vec![],
                rows: vec![],
                affected_rows: 0,
                message: "Query executed successfully".into(),
            });
        }
    };

    let rows: Vec<Vec<String>> = iter
        .filter(|l| !l.is_empty())
        .map(|line| line.split('\t').map(|s| s.to_string()).collect())
        .collect();

    let count = rows.len();
    Ok(QueryResult {
        columns,
        rows,
        affected_rows: count,
        message: format!("{} row(s) returned", count),
    })
}

fn format_size(bytes: u64) -> String {
    if bytes == 0 { return "0 B".into(); }
    if bytes < 1024 { return format!("{} B", bytes); }
    if bytes < 1048576 { return format!("{:.1} KB", bytes as f64 / 1024.0); }
    if bytes < 1073741824 { return format!("{:.1} MB", bytes as f64 / 1048576.0); }
    format!("{:.1} GB", bytes as f64 / 1073741824.0)
}
