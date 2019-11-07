use postgres::GenericConnection;
use types::{Account, Thread};

pub use crate::db_traits::{DBConnectionInstance, Connection, Transaction};
use crate::db_traits::{IntoGenericConnection, get_db_connection};

lazy_static! {
    pub(crate) static ref DB_CONNECTION: DBConnectionInstance = get_db_connection().unwrap();
}

pub fn create_account<T: IntoGenericConnection>(db: T, username: &str, password: &str) {
    let conn = db.into_generic_connection();
    conn.query("INSERT INTO account (username, password) VALUES ($1, $2)", &[&username, &password]).unwrap();
}

pub fn get_account<T: IntoGenericConnection>(db: T, id: u32) -> Option<Account> {
    let conn = db.into_generic_connection();
    conn.query("SELECT id, username FROM account WHERE id=$1", &[&id])
        .unwrap()
        .into_iter()
        .map(|row| Account { id: row.get(0), username: row.get(1) })
        .next()
}

pub fn create_thread<T: IntoGenericConnection>(db: T, title: &str) {
    let conn = db.into_generic_connection();
    conn.query("INSERT INTO thread (title, creator) VALUES ($1, $2)", &[&title, &1]).unwrap();
}

pub fn get_threads<T: IntoGenericConnection>(db: T) -> Vec<Thread> {
    let conn = db.into_generic_connection();
    conn.query("SELECT id, creator, title FROM thread", &[])
        .unwrap()
        .into_iter()
        .map(|row| Thread {
            id: row.get(0),
            creator: row.get(1),
            title: row.get(2),
            messages: None,
        })
        .collect()
}
