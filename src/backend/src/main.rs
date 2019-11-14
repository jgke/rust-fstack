#![feature(proc_macro_hygiene, decl_macro, type_alias_impl_trait, try_trait)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate gotham_derive;
#[macro_use]
extern crate serde_json;

mod auth;
mod db;
mod db_traits;
#[macro_use]
mod handler_utils;
mod router;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state = router::S::new();
    let addr = "127.0.0.1:7878";
    db::DB_CONNECTION.get()?; // Apply migrations
    println!("Listening for requests at http://{}", addr);
    Ok(gotham::start(addr, router::router(state)))
}
