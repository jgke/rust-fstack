use gotham::middleware::state::StateMiddleware;
use gotham::pipeline::single::single_pipeline;
use gotham::pipeline::new_pipeline;
use gotham::router::builder::*;
use gotham::router::Router;
use gotham::state::State;

use crate::handler::r;
use crate::db;

#[derive(Clone, Debug, StateData)]
pub struct S { }
impl S { pub fn new() -> Self { S { } } }

pub fn say_hello(state: State, connection: db::Connection) -> (State, String) {
    let result = db::get_person(connection, "Foo Bar");
    (state, serde_json::to_string(&result).unwrap())
}


pub fn router(state: S) -> Router {
    let middleware = StateMiddleware::new(state);
    let pipeline = new_pipeline().add(middleware).build();
    let (chain, pipelines) = single_pipeline(pipeline);

    // build a router with the chain & pipeline
    build_router(chain, pipelines, |route| {
        route.get("/").to_new_handler(r(say_hello));
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
            .get("http://localhost")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
