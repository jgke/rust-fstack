#![feature(proc_macro_hygiene, decl_macro, type_alias_impl_trait)]

#[macro_use]
extern crate lazy_static;
extern crate gotham;
#[macro_use]
extern crate gotham_derive;
extern crate hyper;
extern crate postgres;

mod db;

use gotham::error::*;
use gotham::handler::{NewHandler, Handler, IntoHandlerFuture, HandlerFuture};
use gotham::middleware::state::StateMiddleware;
use gotham::pipeline::single::single_pipeline;
use gotham::pipeline::new_pipeline;
use gotham::router::builder::*;
use gotham::router::Router;
use gotham::state::State;
use std::panic::RefUnwindSafe;

use db::DBConnectionInstance;

#[derive(Clone, Debug, StateData)]
struct S {
}

impl S {
    fn new() -> Self {
        S { }
    }
}

lazy_static! {
    static ref DB_CONNECTION: DBConnectionInstance = db::get_db_connection().unwrap();
}

pub fn say_hello(state: State, connection: db::Connection) -> (State, String) {
    let result = {
        connection.transaction(|tx| {
            db::get_person(tx, "Foo Bar")
                .ok_or(())
        })
        //db::get_person(connection, "Foo Bar");
        //db::get_person(connection, "Foo Bar")
    };
    (state, serde_json::to_string(&result).unwrap())
}

#[derive(Copy, Clone, Debug)]
struct DBHandlerI<F, R>
where F: FnOnce(State, db::Connection) -> R + Send,
      R: IntoHandlerFuture {
    f: F
}

#[derive(Debug)]
struct DBHandler<F, R>
where F: FnOnce(State, db::Connection) -> R + Send,
      R: IntoHandlerFuture {
    f: F
}

fn r<F, R>(f: F) -> DBHandlerI<F, R>
where F: FnOnce(State, db::Connection) -> R + Send,
      R: IntoHandlerFuture {
    DBHandlerI { f }
}

impl<F, R> NewHandler for DBHandlerI<F, R>
where F: FnOnce(State, db::Connection) -> R + Copy + Send + Sync + RefUnwindSafe,
      R: IntoHandlerFuture {
    type Instance = DBHandler<F, R>;

    fn new_handler(&self) -> Result<Self::Instance> {
        Ok(DBHandler { f: self.f })
    }
}

impl<F, R> Handler for DBHandler<F, R>
where
F: FnOnce(State, db::Connection) -> R + Send,
R: IntoHandlerFuture {
    fn handle(self, state: State) -> Box<HandlerFuture> {
        let conn = DB_CONNECTION.take();
        (self.f)(state, db::Connection::new(&conn)).into_handler_future()
    }
}

fn router(state: S) -> Router {
    let middleware = StateMiddleware::new(state);
    let pipeline = new_pipeline().add(middleware).build();
    let (chain, pipelines) = single_pipeline(pipeline);

    // build a router with the chain & pipeline
    build_router(chain, pipelines, |route| {
        route.get("/").to_new_handler(r(say_hello));
    })
}

pub fn start() {
    let state = S::new();
    let addr = "127.0.0.1:7878";
    println!("Listening for requests at http://{}", addr);
    gotham::start(addr, router(state))
}

#[cfg(test)]
mod tests {
    use super::*;
    use gotham::test::TestServer;
    use hyper::StatusCode;

    #[test]
    fn receive_hello_world_response() {
        let s = S::new(db::get_db_connection().unwrap());
        let test_server = TestServer::new(router(s)).unwrap();
        let response = test_server
            .client()
            .get("http://localhost")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
