use std::sync::{Arc, Mutex, MutexGuard};

use postgres::{TlsMode, GenericConnection};
use serde::Serialize;

#[derive(Clone, Debug)]
pub struct DBConnectionInstance {
    connection: Arc<Mutex<postgres::Connection>>,
}

impl DBConnectionInstance {
    pub fn new(connection: postgres::Connection) -> DBConnectionInstance {
        DBConnectionInstance { connection: Arc::new(Mutex::new(connection)) }
    }
    pub fn take(&self) -> MutexGuard<'_, postgres::Connection> {
        self.connection.lock().unwrap()
    }
}

#[derive(Debug)]
pub struct Connection<'a> {
    connection: &'a postgres::Connection
}

#[derive(Debug)]
pub struct Transaction<'a> {
    tx: &'a postgres::transaction::Transaction<'a>
}

impl Connection<'_> {
    pub fn new(connection: &postgres::Connection) -> Connection {
        Connection { connection }
    }
    pub fn transaction<F, R, E>(self, callback: F) -> Result<R, E>
    where F: FnOnce(&Transaction) -> Result<R, E> {
        let tx = self.connection.transaction().unwrap();
        let res = callback(&Transaction { tx: &tx })?;
        tx.commit().unwrap();
        Ok(res)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Person {
    name: String
}

pub fn get_db_connection() -> Result<DBConnectionInstance, postgres::Error> {
    postgres::Connection::connect("postgres://postgres:password@localhost:6314/postgres", TlsMode::None)
        .map(DBConnectionInstance::new)
}

pub trait IntoGenericConnection {
    type G: GenericConnection;
    fn into_generic_connection(&self) -> &Self::G;
}

impl IntoGenericConnection for Connection<'_> {
    type G = postgres::Connection;

    fn into_generic_connection(&self) -> &Self::G {
        self.connection
    }
}

impl<'a> IntoGenericConnection for &'a Transaction<'a> {
    type G = postgres::transaction::Transaction<'a>;

    fn into_generic_connection(&self) -> &Self::G {
        self.tx
    }
}

pub fn get_person<T: IntoGenericConnection>(db: T, name: &str) -> Option<Person> {
    let conn = db.into_generic_connection();
    let res = conn
        .query("SELECT name FROM person WHERE name=$1 LIMIT 1", &[&name]).unwrap();
    res .into_iter()
        .map(|row| Person { name: row.get(0) })
        .next()
}
