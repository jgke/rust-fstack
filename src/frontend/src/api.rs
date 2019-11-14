#[cfg(debug_assertions)]
lazy_static! {
    pub static ref HOST: String = "http://localhost:80".to_string();
}

#[cfg(not(debug_assertions))]
lazy_static! {
    pub static ref HOST: String = "".to_string();
}

pub fn login() -> String {
    format!("{}/login", *HOST)
}

pub fn create_account() -> String {
    format!("{}/account", *HOST)
}

pub fn all_threads() -> String {
    format!("{}/thread", *HOST)
}

pub fn thread(thread_id: i32) -> String {
    format!("{}/thread/{}", *HOST, thread_id)
}

pub fn new_thread() -> String {
    format!("{}/thread", *HOST)
}

pub fn new_message(thread_id: i32) -> String {
    format!("{}/thread/{}", *HOST, thread_id)
}
