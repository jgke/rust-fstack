use postgres::GenericConnection;
use serde::Serialize;

pub use crate::db_traits::{DBConnectionInstance, Connection, Transaction};
use crate::db_traits::{IntoGenericConnection, get_db_connection};

lazy_static! {
    pub(crate) static ref DB_CONNECTION: DBConnectionInstance = get_db_connection().unwrap();
}

#[derive(Clone, Debug, Serialize)]
pub struct Person {
    name: String
}

pub fn get_person<T: IntoGenericConnection>(db: T, name: &str) -> Option<Person> {
    let conn = db.into_generic_connection();
    let res = conn
        .query("SELECT name FROM person WHERE name=$1 LIMIT 1", &[&name]).unwrap();
    res .into_iter()
        .map(|row| Person { name: row.get(0) })
        .next()
}
