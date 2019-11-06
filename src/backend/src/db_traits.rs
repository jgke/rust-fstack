use postgres::GenericConnection;
use r2d2_postgres::{TlsMode, PostgresConnectionManager};
use std::sync::Arc;

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
    where F: FnOnce(&Transaction) -> Result<R, E> {
        let tx = self.connection.transaction().unwrap();
        let res = callback(&Transaction { tx: Arc::new(tx) })?;
        Ok(res)
    }
}

pub fn get_db_connection() -> Result<DBConnectionInstance, postgres::Error> {
    let manager = PostgresConnectionManager::new(
        "postgres://postgres:password@localhost:6314/postgres",
        TlsMode::None).unwrap();
    let pool = r2d2::Pool::new(manager).unwrap();
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
