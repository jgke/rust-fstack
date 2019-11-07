#![feature(proc_macro_hygiene, decl_macro, type_alias_impl_trait)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate gotham_derive;

mod db;
mod db_traits;
#[macro_use]
mod handler_utils;
mod router;

pub fn main() {
    let state = router::S::new();
    let addr = "127.0.0.1:7878";
    println!("Listening for requests at http://{}", addr);
    gotham::start(addr, router::router(state))
}
