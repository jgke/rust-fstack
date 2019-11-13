use bcrypt::{DEFAULT_COST, hash, verify};
use futures::Future;
use gotham::handler::{IntoHandlerError, HandlerFuture};
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
use crate::handler_utils::{r, extract_json};

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
    let token = auth::encrypt(json!({"sub": id})).unwrap();
    Token { token }
}

pub fn new_account(mut state: State, connection: db::Connection) -> Box<HandlerFuture> {
    with_json!(CreateAccount, state,
               |account| {
                   let hashed = hash(account.password, DEFAULT_COST - 2).unwrap();
                   db::create_account(connection, &account.username, &hashed)
               },
               |res| {
                   match res {
                       Some(id) => {
                           let body = serde_json::to_string(&get_token(id)).unwrap();
                           create_response(&state, StatusCode::CREATED, mime::APPLICATION_JSON, body.to_string())
                       }
                       None => create_response(&state, StatusCode::CONFLICT, mime::APPLICATION_JSON, Body::empty()),
                   }
               })
}

pub fn login(mut state: State, connection: db::Connection) -> Box<HandlerFuture> {
    with_json!(Login, state,
               |account| {
                   connection.transaction(|tx| -> Result<Option<i32>, ()>  {
                       let (id, password) = db::get_password(&tx, &account.username).ok_or(())?;
                       let valid = verify(&account.password, &password).map_err(|_| ())?;
                       db::update_last_logged_in(&tx, &account.username);
                       tx.commit().map_err(|_| ())?;
                       if valid {
                           Ok(Some(id))
                       } else {
                           Ok(None)
                       }
                   }).ok()?
               },
               |account_id| {
                   match account_id {
                       Some(id) => {
                           let body = serde_json::to_string(&get_token(id)).unwrap();
                           create_response(&state, StatusCode::OK, mime::APPLICATION_JSON, body.to_string())
                       }
                       None => create_response(&state, StatusCode::NOT_FOUND, mime::APPLICATION_JSON, Body::empty())
                   }
               })
}

pub fn get_account(state: State, connection: db::Connection) -> (State, String) {
    let account = {
        let id = AccountId::borrow_from(&state).id;
        db::get_account(connection, id)
    };
    (state, serde_json::to_string(&account).unwrap())
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
    let headers = HeaderMap::borrow_from(&state);
    let id = headers.get("token")
        .and_then(|h| h.to_str().ok())
        .and_then(|t| auth::decrypt(t).ok())
        .and_then(|t| t.1["sub"].as_i64())
        .and_then(|id| id.try_into().ok());

    if let Some(id) = id {
        with_json!(CreateThread, state,
                   move |thread: CreateThread| db::create_thread(connection, id, &thread.title),
                   |_| create_response(&state, StatusCode::CREATED, mime::APPLICATION_JSON, Body::empty()))
    } else {
        Box::new(futures::lazy(|| {
            let resp = create_response(&state, StatusCode::UNAUTHORIZED, mime::APPLICATION_JSON, Body::empty());
            Ok((state, resp))
        }))
    }
}


pub fn create_message(mut state: State, connection: db::Connection) -> Box<HandlerFuture> {
    let thread_id = ThreadId::borrow_from(&state).id;
    let headers = HeaderMap::borrow_from(&state);
    let id = headers.get("token")
        .and_then(|h| h.to_str().ok())
        .and_then(|t| auth::decrypt(t).ok())
        .and_then(|t| t.1["sub"].as_i64())
        .and_then(|id| id.try_into().ok());

    if let Some(id) = id {
        with_json!(CreateMessage, state,
                   move |msg| db::create_message(connection, id, thread_id, &msg.content),
                   |_| create_response(&state, StatusCode::CREATED, mime::APPLICATION_JSON, Body::empty()))
    } else {
        Box::new(futures::lazy(|| {
            let resp = create_response(&state, StatusCode::UNAUTHORIZED, mime::APPLICATION_JSON, Body::empty());
            Ok((state, resp))
        }))
    }
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
