use r2d2_postgres::{TlsMode, PostgresConnectionManager};
use std::collections::HashSet;

pub type DBConnectionPool = r2d2::Pool<r2d2_postgres::PostgresConnectionManager>;
pub type DBConnection = r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>;

#[derive(Debug)]
pub struct Connection(Box<DBConnection>);

#[derive(Debug)]
pub struct Transaction<'a>(Box<postgres::transaction::Transaction<'a>>);

pub trait IntoGenericConnection {
    type G: postgres::GenericConnection;
    fn into_generic_connection(&self) -> &Self::G;
}

impl IntoGenericConnection for Connection {
    type G = postgres::Connection;

    fn into_generic_connection(&self) -> &Self::G {
        &self.0
    }
}

impl<'a> IntoGenericConnection for &'a Transaction<'a> {
    type G = postgres::transaction::Transaction<'a>;

    fn into_generic_connection(&self) -> &Self::G {
        &self.0
    }
}

impl Connection {
    pub fn new(connection: Box<DBConnection>) -> Connection {
        Connection(connection)
    }
    pub fn transaction<F, R, E>(self, callback: F) -> Result<R, E>
    where F: FnOnce(Transaction) -> Result<R, E> {
        let tx = self.0.transaction().unwrap();
        let res = callback(Transaction(Box::new(tx)))?;
        Ok(res)
    }
}

// postgres::Connection isn't thread safe, but we only access it 
// from a single thread at a time, so this is safe
unsafe impl Sync for Connection {}
unsafe impl Send for Connection {}
unsafe impl Sync for Transaction<'_> {}
unsafe impl Send for Transaction<'_> {}

impl Transaction<'_> {
    pub fn commit(self) -> Result<(), postgres::error::Error> {
        self.0.commit()
    }
}

pub fn get_db_connection() -> Result<DBConnectionPool, Box<dyn std::error::Error>> {
    let manager = PostgresConnectionManager::new(
        "postgres://postgres:password@localhost:6314/postgres",
        TlsMode::None).unwrap();
    let pool = r2d2::Pool::new(manager).unwrap();

    run_migrations(pool.get()?).unwrap();

    Ok(pool)
}

const MIGRATIONS: &[(&str, &str)] = &[
    ("initial", include_str!("../migrations/initial.sql"))
];

pub fn run_migrations(connection: DBConnection) -> Result<(), postgres::Error> {
    let tx = connection.transaction()?;

    tx.query("CREATE TABLE IF NOT EXISTS _migration (name TEXT UNIQUE)", &[])?;
    let applied_migrations: HashSet<String> = tx.query("SELECT name FROM _migration", &[])?
        .into_iter()
        .map(|row| row.get(0))
        .collect();

    let migrations: Vec<_> = MIGRATIONS
        .iter()
        .filter(|m| !applied_migrations.contains(m.0))
        .collect();

    if migrations.is_empty() {
        println!("All migrations are applied!");
    }

    for migration in migrations {
        println!("Applying migration {}", migration.0);
        tx.query("INSERT INTO _migration (name) VALUES ($1)", &[&migration.0])?;
        tx.batch_execute(migration.1)?;
    }

    tx.commit()?;

    Ok(())
}
