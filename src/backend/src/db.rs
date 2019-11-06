use postgres::GenericConnection;
pub use types::Person;

pub use crate::db_traits::{DBConnectionInstance, Connection, Transaction};
use crate::db_traits::{IntoGenericConnection, get_db_connection};

lazy_static! {
    pub(crate) static ref DB_CONNECTION: DBConnectionInstance = get_db_connection().unwrap();
}

pub fn add_person<T: IntoGenericConnection>(db: T, name: &str) {
    let conn = db.into_generic_connection();
    conn.query("INSERT INTO person (name) VALUES ($1)", &[&name]).unwrap();
}

pub fn get_persons<T: IntoGenericConnection>(db: T) -> Vec<Person> {
    let conn = db.into_generic_connection();
    conn.query("SELECT name FROM person", &[])
        .unwrap()
        .into_iter()
        .map(|row| Person { name: row.get(0) })
        .collect()
}

pub fn get_person<T: IntoGenericConnection>(db: T, name: &str) -> Option<Person> {
    let conn = db.into_generic_connection();
    conn.query("SELECT name FROM person WHERE name=$1 LIMIT 1", &[&name])
        .unwrap()
        .into_iter()
        .map(|row| Person { name: row.get(0) })
        .next()
}

pub fn delete_person<T: IntoGenericConnection>(db: T, name: &str) {
    let conn = db.into_generic_connection();
    conn.query("DELETE FROM person WHERE name=$1", &[&name]).unwrap();
}
