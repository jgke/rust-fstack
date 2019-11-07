use postgres::GenericConnection;
use r2d2_postgres::{TlsMode, PostgresConnectionManager};
use std::sync::Arc;
use std::collections::HashSet;

#[derive(Clone, Debug)]
pub struct DBConnectionInstance {
    connection: r2d2::Pool<r2d2_postgres::PostgresConnectionManager>,
}

impl DBConnectionInstance {
    pub fn new(connection: r2d2::Pool<r2d2_postgres::PostgresConnectionManager>) -> DBConnectionInstance {
        DBConnectionInstance { connection }
    }
    pub fn take(&self) -> r2d2::Pool<r2d2_postgres::PostgresConnectionManager> {
        self.connection.clone()
    }
}

#[derive(Debug)]
pub struct Connection {
    connection: Arc<r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>>
}

unsafe impl Sync for Connection {}
unsafe impl Send for Connection {}

#[derive(Debug)]
pub struct Transaction<'a> {
    tx: Arc<postgres::transaction::Transaction<'a>>
}

unsafe impl Sync for Transaction<'_> {}
unsafe impl Send for Transaction<'_> {}

impl Connection {
    pub fn new(connection: Arc<r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>>) -> Connection {
        Connection { connection }
    }
    pub fn transaction<F, R, E>(self, callback: F) -> Result<R, E>
    where F: FnOnce(Transaction) -> Result<R, E> {
        let tx = self.connection.transaction().unwrap();
        let res = callback(Transaction { tx: Arc::new(tx) })?;
        Ok(res)
    }
}

impl Transaction<'_> {
    pub fn commit(self) -> Result<(), postgres::error::Error> {
        Arc::try_unwrap(self.tx).unwrap().commit()
    }
}

const MIGRATIONS: &[(&str, &str)] = &[
    ("initial", include_str!("../migrations/initial.sql"))
];

pub fn run_migrations(connection: r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>) -> Result<(), postgres::Error> {
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

    dbg!(&applied_migrations, &migrations);

    for migration in migrations {
        println!("Applying migration {}", migration.0);
        tx.query("INSERT INTO _migration (name) VALUES ($1)", &[&migration.0])?;
        tx.batch_execute(migration.1)?;
    }

    tx.commit()?;

    Ok(())
}

pub fn get_db_connection() -> Result<DBConnectionInstance, Box<dyn std::error::Error>> {
    let manager = PostgresConnectionManager::new(
        "postgres://postgres:password@localhost:6314/postgres",
        TlsMode::None).unwrap();
    let pool = r2d2::Pool::new(manager).unwrap();

    run_migrations(pool.get()?).unwrap();

    Ok(DBConnectionInstance::new(pool))
}

pub trait IntoGenericConnection {
    type G: GenericConnection;
    fn into_generic_connection(&self) -> &Self::G;
}

impl IntoGenericConnection for Connection {
    type G = postgres::Connection;

    fn into_generic_connection(&self) -> &Self::G {
        &self.connection
    }
}

impl<'a> IntoGenericConnection for &'a Transaction<'a> {
    type G = postgres::transaction::Transaction<'a>;

    fn into_generic_connection(&self) -> &Self::G {
        &self.tx
    }
}
