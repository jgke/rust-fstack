use futures::Future;
use futures::stream::Stream;
use gotham::error::Result;
use gotham::handler::{IntoHandlerError, HandlerError, NewHandler, Handler, IntoHandlerFuture, HandlerFuture};
use gotham::state::{FromState, State};
use hyper::{Body, StatusCode};
use std::panic::RefUnwindSafe;
use std::str::from_utf8;
use std::sync::Arc;

use crate::db::{DB_CONNECTION, Connection};

#[derive(Copy, Clone, Debug)]
pub struct DBHandlerI<F, R>
where F: FnOnce(State, Connection) -> R + Send,
      R: IntoHandlerFuture {
    f: F
}

#[derive(Debug)]
pub struct DBHandler<F, R>
where F: FnOnce(State, Connection) -> R + Send,
      R: IntoHandlerFuture {
    f: F
}

impl<F, R> NewHandler for DBHandlerI<F, R>
where F: FnOnce(State, Connection) -> R + Copy + Send + Sync + RefUnwindSafe,
      R: IntoHandlerFuture {
    type Instance = DBHandler<F, R>;

    fn new_handler(&self) -> Result<Self::Instance> {
        Ok(DBHandler { f: self.f })
    }
}

impl<F, R> Handler for DBHandler<F, R>
where
F: FnOnce(State, Connection) -> R + Send,
R: IntoHandlerFuture {
    fn handle(self, state: State) -> Box<HandlerFuture> {
        let conn = DB_CONNECTION.take().get().unwrap();
        (self.f)(state, Connection::new(Arc::new(conn))).into_handler_future()
    }
}

pub fn r<F, R>(f: F) -> DBHandlerI<F, R>
where F: FnOnce(State, Connection) -> R + Send,
      R: IntoHandlerFuture {
    DBHandlerI { f }
}


fn bad_request<E>(e: E) -> HandlerError
where
    E: std::error::Error + Send + 'static,
{
    e.into_handler_error().with_status(StatusCode::BAD_REQUEST)
}

pub fn extract_json<T>(state: &mut State) -> impl Future<Item = T, Error = HandlerError>
where
    T: serde::de::DeserializeOwned,
{
    Body::take_from(state)
        .concat2()
        .map_err(bad_request)
        .and_then(|body| {
            let b = body.to_vec();
            from_utf8(&b)
                .map_err(bad_request)
                .and_then(|s| serde_json::from_str::<T>(s).map_err(bad_request))
        })
}
