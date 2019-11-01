#![feature(proc_macro_hygiene, decl_macro, type_alias_impl_trait)]

#[macro_use]
extern crate lazy_static;
extern crate gotham;
#[macro_use]
extern crate gotham_derive;
extern crate hyper;

extern crate r2d2;
extern crate r2d2_postgres;
extern crate postgres;

mod db;
mod db_traits;
mod handler;
mod router;

pub fn start() {
    let state = router::S::new();
    let addr = "127.0.0.1:7878";
    println!("Listening for requests at http://{}", addr);
    gotham::start(addr, router::router(state))
}
