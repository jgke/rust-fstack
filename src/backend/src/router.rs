use futures::Future;
use gotham::handler::{IntoHandlerError, HandlerFuture};
use gotham::helpers::http::response::create_response;
use gotham::middleware::state::StateMiddleware;
use gotham::pipeline::single::single_pipeline;
use gotham::pipeline::new_pipeline;
use gotham::router::builder::*;
use gotham::router::Router;
use gotham::state::{State, FromState};
use hyper::{Body, StatusCode};
use serde::Deserialize;

use crate::handler_utils::{r, extract_json};
use crate::db;

#[derive(Clone, Debug, StateData)]
pub struct S { }
impl S { pub fn new() -> Self { S { } } }

#[derive(Deserialize, StateData, StaticResponseExtender)]
struct NewAccount {
    username: String,
    password: String,
}

#[derive(Deserialize, StateData, StaticResponseExtender)]
struct CreateThread {
    title: String,
}

#[derive(Deserialize, StateData, StaticResponseExtender)]
struct ThreadId {
    id: i32,
}

pub fn new_account(mut state: State, connection: db::Connection) -> Box<HandlerFuture> {
    let f = extract_json::<NewAccount>(&mut state)
        .map(move |account| {
            db::create_account(connection, &account.username, &account.password);
        })
        .map_err(|e| e.into_handler_error().with_status(StatusCode::BAD_REQUEST))
        .then(|result| match result {
            Ok(_) => {
                let res = create_response(&state, StatusCode::CREATED, mime::APPLICATION_JSON, Body::empty());
                Ok((state, res))
            },
            Err(r) => Err((state, r)),
        });
    Box::new(f)
}

pub fn get_threads(state: State, connection: db::Connection) -> (State, String) {
    (state, serde_json::to_string(&db::get_threads(connection)).unwrap())
}

pub fn get_thread(state: State, connection: db::Connection) -> (State, String) {
    let thread = {
        let id = ThreadId::borrow_from(&state).id;
        db::get_thread(connection, id)
    };
    (state, serde_json::to_string(&thread).unwrap())
}

pub fn create_thread(mut state: State, connection: db::Connection) -> Box<HandlerFuture> {
    let f = extract_json::<CreateThread>(&mut state)
        .map(move |thread| {
            db::create_thread(connection, &thread.title);
        })
        .map_err(|e| e.into_handler_error().with_status(StatusCode::BAD_REQUEST))
        .then(|result| match result {
            Ok(_) => {
                let res = create_response(&state, StatusCode::CREATED, mime::APPLICATION_JSON, Body::empty());
                Ok((state, res))
            },
            Err(r) => Err((state, r)),
        });
    Box::new(f)
}

pub fn router(state: S) -> Router {
    let middleware = StateMiddleware::new(state);
    let pipeline = new_pipeline().add(middleware).build();
    let (chain, pipelines) = single_pipeline(pipeline);

    // build a router with the chain & pipeline
    build_router(chain, pipelines, |route| {
        route.post("/account").to_new_handler(r(new_account));
        route.get("/thread").to_new_handler(r(get_threads));
        route.get("/thread/:id")
            .with_path_extractor::<ThreadId>()
            .to_new_handler(r(get_thread));
        route.post("/thread").to_new_handler(r(create_thread));
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gotham::test::TestServer;
    use hyper::StatusCode;
    use uuid::Uuid;

    #[test]
    fn receive_hello_world_response() {
        let s = S::new();
        let test_server = TestServer::new(router(s)).unwrap();
        let response = test_server
            .client()
            .post("http://localhost/account",
                  format!("{{\"username\": \"{}\", \"password\": \"{}\"}}", Uuid::new_v4(), Uuid::new_v4()),
                  mime::APPLICATION_JSON)
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }
}
