use bcrypt::{DEFAULT_COST, hash, verify};
use gotham::handler::HandlerFuture;
use gotham::helpers::http::response::create_response;
use gotham::middleware::state::StateMiddleware;
use gotham::pipeline::single::single_pipeline;
use gotham::pipeline::new_pipeline;
use gotham::router::builder::*;
use gotham::router::Router;
use gotham::state::{State, FromState};
use hyper::{Body, StatusCode, HeaderMap};
use serde::Deserialize;
use std::convert::TryInto;
use types::*;

use crate::auth;
use crate::db;
use crate::handler_utils::{r, with_json};

#[derive(Clone, Debug, StateData)]
pub struct S { }
impl S { pub fn new() -> Self { S { } } }

#[derive(Deserialize, StateData, StaticResponseExtender)]
struct ThreadId {
    id: i32,
}

#[derive(Deserialize, StateData, StaticResponseExtender)]
struct AccountId {
    id: i32,
}

fn get_token(id: i32) -> Token {
    let token = auth::sign(json!({"sub": id})).unwrap();
    Token { token }
}

pub fn new_account(state: State, connection: db::Connection) -> Box<HandlerFuture> {
    with_json(state, |state, account: CreateAccount| {
        let hashed = hash(account.password, DEFAULT_COST - 2)?;
        let id = db::create_account(connection, &account.username, &hashed).ok_or(StatusCode::CONFLICT)?;
        let body = serde_json::to_string(&get_token(id))?;
        Ok(create_response(&state, StatusCode::CREATED, mime::APPLICATION_JSON, body.to_string()))
    })
}

pub fn login(state: State, connection: db::Connection) -> Box<HandlerFuture> {
    with_json(state, |state, account: Login| {
        connection.transaction(|tx| {
            let (id, password) = db::get_password(&tx, &account.username).ok_or(StatusCode::NOT_FOUND)?;
            let valid = verify(&account.password, &password)?;
            if valid {
                db::update_last_logged_in(&tx, &account.username);
                tx.commit().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                let body = serde_json::to_string(&get_token(id))?;
                Ok(create_response(&state, StatusCode::OK, mime::APPLICATION_JSON, body.to_string()))
            } else {
                Err(From::from(StatusCode::NOT_FOUND))
            }
        })
    })
}

pub fn get_account(state: State, connection: db::Connection) -> (State, String) {
    let id = AccountId::borrow_from(&state).id;
    let account = db::get_account(connection, id);
    (state, serde_json::to_string(&account).unwrap())
}

pub fn get_threads(state: State, connection: db::Connection) -> (State, String) {
    (state, serde_json::to_string(&db::get_threads(connection)).unwrap())
}

pub fn get_thread(state: State, connection: db::Connection) -> (State, String) {
    let id = ThreadId::borrow_from(&state).id;
    let thread = db::get_thread(connection, id);
    (state, serde_json::to_string(&thread).unwrap())
}

pub fn create_thread(state: State, connection: db::Connection) -> Box<HandlerFuture> {
    with_json(state, |state, thread: CreateThread| {
        let headers = HeaderMap::borrow_from(&state);
        let token = headers.get("token")?.to_str()?;
        let sub: i32 = auth::unsign(token)?.1["sub"].as_i64()?.try_into()?;

        db::create_thread(connection, sub, &thread.title);
        Ok(create_response(state, StatusCode::CREATED, mime::APPLICATION_JSON, Body::empty()))
    })
}


pub fn create_message(state: State, connection: db::Connection) -> Box<HandlerFuture> {
    with_json(state, |state, message: CreateMessage| {
        let thread_id = ThreadId::borrow_from(&state).id;
        let headers = HeaderMap::borrow_from(&state);
        let token = headers.get("token")?.to_str()?;
        let sub: i32 = auth::unsign(token)?.1["sub"].as_i64()?.try_into()?;
        db::create_message(connection, sub, thread_id, &message.content);
        Ok(create_response(&state, StatusCode::CREATED, mime::APPLICATION_JSON, Body::empty()))
    })
}

pub fn router(state: S) -> Router {
    let middleware = StateMiddleware::new(state);
    let pipeline = new_pipeline().add(middleware).build();
    let (chain, pipelines) = single_pipeline(pipeline);

    // build a router with the chain & pipeline
    build_router(chain, pipelines, |route| {
        route.post("/login").to_new_handler(r(login));
        route.post("/account").to_new_handler(r(new_account));
        route.get("/account/:id").to_new_handler(r(get_account));
        route.get("/thread").to_new_handler(r(get_threads));
        route.get("/thread/:id")
            .with_path_extractor::<ThreadId>()
            .to_new_handler(r(get_thread));
        route.post("/thread").to_new_handler(r(create_thread));
        route.post("/thread/:id")
            .with_path_extractor::<ThreadId>()
            .to_new_handler(r(create_message));
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
