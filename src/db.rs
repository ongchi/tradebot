use anyhow::Result;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;

pub type DbPool = Pool<SqliteConnectionManager>;
pub type DbConn = PooledConnection<SqliteConnectionManager>;

pub fn get_pool(uri: Option<String>) -> Result<DbPool> {
    let manager = SqliteConnectionManager::file(uri.unwrap());
    let pool = r2d2::Pool::new(manager).unwrap();

    let conn = pool.get().unwrap();

    rusqlite::vtab::array::load_module(&conn)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS trades (
                    symbol  TEXT NOT NULL,
                    mts     DATETIME NOT NULL,
                    amount  REAL NOT NULL,
                    rate    REAL NOT NULL,
                    period  INTEGER NOT NULL
                )",
        params![],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS credits (
                    id              INTEGER PRIMARY KEY,
                    symbol          TEXT NOT NULL,
                    amount          REAL NOT NULL,
                    rate            REAL NOT NULL,
                    period          INTEGER NOT NULL,
                    opening         DATETIME NOT NULL,
                    last_payout     DATETIME NOT NULL,
                    position_pair   TEXT NOT NULL
                )",
        params![],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS provided (
                    id              INTEGER PRIMARY KEY,
                    symbol          TEXT NOT NULL,
                    \"create\"      DATETIME NOT NULL,
                    \"update\"      DATETIME NOT NULL,
                    amount          REAL NOT NULL,
                    rate            REAL NOT NULL,
                    period          INTEGER NOT NULL,
                    position_pair   TEXT NOT NULL
                )",
        params![],
    )?;

    Ok(pool)
}
