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
struct Name {
    name: String
}

pub fn get_all_persons(state: State, connection: db::Connection) -> (State, String) {
    let result = db::get_persons(connection);
    (state, serde_json::to_string(&result).unwrap())
}

pub fn add_new(mut state: State, connection: db::Connection) -> Box<HandlerFuture> {
    let f = extract_json::<Name>(&mut state)
        .map(move |name| {
            db::add_person(connection, &name.name);
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

pub fn delete_if_found(mut state: State, connection: db::Connection) -> (State, hyper::Response<Body>) {
    let query = Name::take_from(&mut state);
    let result = connection.transaction(|tx| {
        db::get_person(&tx, &query.name)
            .map(|_p| {
                db::delete_person(&tx, &query.name);
                tx.commit().unwrap();
            })
            .ok_or(())
    });
    let response = match result {
        Ok(_) => create_response(&state, StatusCode::OK, mime::APPLICATION_JSON, Body::empty()),
        Err(_) => create_response(&state, StatusCode::NOT_FOUND, mime::APPLICATION_JSON, Body::empty()),
    };
    (state, response)
}

pub fn router(state: S) -> Router {
    let middleware = StateMiddleware::new(state);
    let pipeline = new_pipeline().add(middleware).build();
    let (chain, pipelines) = single_pipeline(pipeline);

    // build a router with the chain & pipeline
    build_router(chain, pipelines, |route| {
        route.get("/").to_new_handler(r(get_all_persons));
        route.post("/").to_new_handler(r(add_new));
        route.delete("/")
            .with_query_string_extractor::<Name>()
            .to_new_handler(r(delete_if_found));
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gotham::test::TestServer;
    use hyper::StatusCode;

    #[test]
    fn receive_hello_world_response() {
        let s = S::new();
        let test_server = TestServer::new(router(s)).unwrap();
        let response = test_server
            .client()
            .get("http://localhost?name=Foo%20Bar")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
