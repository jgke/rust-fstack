#![feature(proc_macro_hygiene, decl_macro)]

extern crate gotham;
#[macro_use]
extern crate gotham_derive;
extern crate hyper;
extern crate postgres;

use std::sync::{Arc, Mutex};

use postgres::{Connection, TlsMode};

use gotham::middleware::state::StateMiddleware;
use gotham::pipeline::single::single_pipeline;
use gotham::pipeline::single_middleware;
use gotham::router::builder::*;
use gotham::router::Router;
use gotham::state::{FromState, State};

use serde::Serialize;

#[derive(Clone, StateData)]
struct S {
    connection: Arc<Mutex<Connection>>
}

impl S {
    fn new(conn: Connection) -> Self {
        let connection = Arc::new(Mutex::new(conn));
        S { connection }
    }
}

#[derive(Debug, Serialize)]
struct Person {
    name: String
}

pub fn say_hello(state: State) -> (State, String) {
    let res_vec: Vec<Person> = {
        let s = S::borrow_from(&state);
        s.connection.lock().unwrap().query("SELECT name FROM person", &[]).unwrap()
            .into_iter()
            .map(|row| Person { name: row.get(0) })
            .collect()
    };
    (state, serde_json::to_string(&res_vec).unwrap())
}

fn router(state: S) -> Router {
    // create our state middleware to share the counter
    let middleware = StateMiddleware::new(state);

    // create a middleware pipeline from our middleware
    let pipeline = single_middleware(middleware);

    // construct a basic chain from our pipeline
    let (chain, pipelines) = single_pipeline(pipeline);

    // build a router with the chain & pipeline
    build_router(chain, pipelines, |route| {
        route.get("/").to(say_hello);
    })
}

fn get_db_connection() -> Result<Connection, postgres::Error> {
    Connection::connect("postgres://postgres:password@localhost:5432/postgres", TlsMode::None)
}

pub fn start() {
    match get_db_connection() {
        Ok(connection) => {
            let state = S::new(connection);
            let addr = "127.0.0.1:7878";
            println!("Listening for requests at http://{}", addr);
            gotham::start(addr, router(state))
        }
        Err(s) => { panic!("{}", s) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gotham::test::TestServer;
    use hyper::StatusCode;

    #[test]
    fn receive_hello_world_response() {
        let s = S::new(get_db_connection().unwrap());
        let test_server = TestServer::new(router(s)).unwrap();
        let response = test_server
            .client()
            .get("http://localhost")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
