use postgres::GenericConnection;
use types::{Account, Thread, Message};

pub use crate::db_traits::{DBConnectionPool, Connection, Transaction};
use crate::db_traits::{IntoGenericConnection as IGC, get_db_connection};

lazy_static! {
    pub(crate) static ref DB_CONNECTION: DBConnectionPool = get_db_connection().unwrap();
}

pub fn create_account<T: IGC>(db: T, username: &str, password: &str) -> Option<i32> {
    let conn = db.into_generic_connection();
    conn.query("INSERT INTO account (username, password, last_logged_in) \
               VALUES ($1, $2, $3) \
               RETURNING id", &[&username, &password, &chrono::Utc::now()]).ok()?
        .into_iter()
        .next()
        .map(|row| row.get(0))
}

pub fn get_password<T: IGC>(db: T, username: &str) -> Option<(i32, String)> {
    let conn = db.into_generic_connection();
    conn.query("SELECT id, password FROM account WHERE username=$1",
               &[&username]).unwrap()
        .into_iter()
        .next()
        .map(|row| (row.get(0), row.get(1)))
}

pub fn update_last_logged_in<T: IGC>(db: T, username: &str) {
    let conn = db.into_generic_connection();
    conn.query("UPDATE account SET last_logged_in=$2 WHERE username=$1",
               &[&username, &chrono::Utc::now()]).unwrap();
}

pub fn get_account<T: IGC>(db: T, id: i32) -> Option<Account> {
    let conn = db.into_generic_connection();
    conn.query("SELECT id, username FROM account WHERE id=$1", &[&id]).unwrap()
        .into_iter()
        .map(|row| Account { id: row.get(0), username: row.get(1) })
        .next()
}

pub fn create_thread<T: IGC>(db: T, account_id: i32, title: &str) {
    let conn = db.into_generic_connection();
    conn.query("INSERT INTO thread (title, creator) VALUES ($1, $2)", &[&title, &account_id]).unwrap();
}

pub fn get_threads<T: IGC>(db: T) -> Vec<Thread> {
    let conn = db.into_generic_connection();
    conn.query("SELECT t.id, a.username, t.title \
               FROM thread t \
               LEFT JOIN account a ON t.creator = a.id", &[])
        .unwrap()
        .into_iter()
        .map(|row| Thread {
            id: row.get(0),
            creator: row.get(1),
            title: row.get(2),
            messages: None,
            latest_message: None,
        })
        .collect()
}

pub fn get_thread<T: IGC>(db: T, id: i32) -> Option<Thread> {
    let conn = db.into_generic_connection();
    let result = conn.query(
        "SELECT t.id, a1.username, title, m.id, a2.username, m.content \
         FROM thread t \
         LEFT JOIN message m ON m.thread_id = t.id \
         LEFT JOIN account a1 ON t.creator = a1.id \
         LEFT JOIN account a2 ON m.creator = a2.id \
         WHERE t.id=$1", &[&id])
        .unwrap();

    let thread_row = result.iter().next()?;

    Some(Thread {
        id: thread_row.get(0), creator: thread_row.get(1),
        title: thread_row.get(2),
        latest_message: None,
        messages: Some(result
                       .into_iter()
                       .filter(|row| row.get::<usize, Option<i32>>(3).is_some())
                       .map(|row| Message {
                           id: row.get(3),
                           creator: row.get(4),
                           content: row.get(5),
                       })
                       .collect()),
    })
}

pub fn create_message<T: IGC>(db: T, account_id: i32, thread_id: i32, message: &str) {
    let conn = db.into_generic_connection();
    conn.query("INSERT INTO message (thread_id, content, creator) VALUES ($1, $2, $3)",
               &[&thread_id, &message, &account_id]).unwrap();
}
