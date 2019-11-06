use futures::{future, Future};
use gotham::handler::{IntoHandlerError, HandlerFuture};
use gotham::helpers::http::response::create_response;
use gotham::middleware::state::StateMiddleware;
use gotham::pipeline::single::single_pipeline;
use gotham::pipeline::new_pipeline;
use gotham::router::builder::*;
use gotham::router::Router;
use gotham::state::{State, FromState};
use hyper::StatusCode;
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

pub fn add_new(mut state: State, connection: db::Connection) -> Box<HandlerFuture> {
    let f = extract_json::<Name>(&mut state)
        .map(move |name| {
            db::add_person(connection, &name.name);
        })
        .map_err(|e| e.into_handler_error().with_status(StatusCode::BAD_REQUEST))
        .then(|result| match result {
            Ok(_) => {
                let res = create_response(&state, StatusCode::CREATED, mime::APPLICATION_JSON, "{}".to_string());
                Ok((state, res))
            },
            Err(r) => Err((state, r)),
        });
    Box::new(f)
}

pub fn find_by_name(mut state: State, connection: db::Connection) -> (State, String) {
    let result = {
        let query = Name::take_from(&mut state);
        db::get_person(connection, &query.name)
    };
    (state, serde_json::to_string(&result).unwrap())
}

pub fn router(state: S) -> Router {
    let middleware = StateMiddleware::new(state);
    let pipeline = new_pipeline().add(middleware).build();
    let (chain, pipelines) = single_pipeline(pipeline);

    // build a router with the chain & pipeline
    build_router(chain, pipelines, |route| {
        route.get("/")
            .with_query_string_extractor::<Name>()
            .to_new_handler(r(find_by_name));
        route.post("/").to_new_handler(r(add_new));
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
