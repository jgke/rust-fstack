use futures::Future;
use futures::stream::Stream;
use gotham::error::Result as GothamResult;
use gotham::handler::{IntoHandlerError, HandlerError, NewHandler, Handler, IntoHandlerFuture, HandlerFuture};
use gotham::helpers::http::response::create_response;
use gotham::state::{FromState, State};
use hyper::{Body, StatusCode};
use std::panic::RefUnwindSafe;
use std::str::from_utf8;

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

    fn new_handler(&self) -> GothamResult<Self::Instance> {
        Ok(DBHandler { f: self.f })
    }
}

impl<F, R> Handler for DBHandler<F, R>
where
F: FnOnce(State, Connection) -> R + Send,
R: IntoHandlerFuture {
    fn handle(self, state: State) -> Box<HandlerFuture> {
        let conn = DB_CONNECTION.get().unwrap();
        (self.f)(state, Connection::new(Box::new(conn))).into_handler_future()
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

pub trait IntoHttpError {
    fn into_http_result(self, state: &State) -> Result<hyper::Response<Body>, hyper::Response<Body>>;
}

pub struct HttpResult(StatusCode, mime::Mime, Body);

impl IntoHttpError for Option<hyper::Response<Body>> {
    fn into_http_result(self, state: &State) -> Result<hyper::Response<Body>, hyper::Response<Body>> {
        self.ok_or_else(|| create_response(state, StatusCode::NOT_FOUND, mime::APPLICATION_JSON, Body::empty()))
    }
}

impl IntoHttpError for Result<hyper::Response<Body>, HttpResult> {
    fn into_http_result(self, state: &State) -> Result<hyper::Response<Body>, hyper::Response<Body>> {
        self.map_err(|e| {
            let HttpResult(status, mime, body) = e;
            create_response(state, status, mime, body)
        })
    }
}

impl From<bcrypt::BcryptError> for HttpResult {
    fn from(_: bcrypt::BcryptError) -> Self {
        HttpResult(StatusCode::BAD_REQUEST, mime::APPLICATION_JSON, Body::empty())
    }
}

impl From<StatusCode> for HttpResult {
    fn from(status: StatusCode) -> Self {
        HttpResult(status, mime::APPLICATION_JSON, Body::empty())
    }
}

impl From<serde_json::Error> for HttpResult {
    fn from(_: serde_json::Error) -> Self {
        HttpResult(StatusCode::BAD_REQUEST, mime::APPLICATION_JSON, Body::empty())
    }
}

impl From<hyper::http::header::ToStrError> for HttpResult {
    fn from(_: hyper::http::header::ToStrError) -> Self {
        HttpResult(StatusCode::BAD_REQUEST, mime::APPLICATION_JSON, Body::empty())
    }
}

impl From<frank_jwt::Error> for HttpResult {
    fn from(_: frank_jwt::Error) -> Self {
        HttpResult(StatusCode::BAD_REQUEST, mime::APPLICATION_JSON, Body::empty())
    }
}
impl From<std::num::TryFromIntError> for HttpResult {
    fn from(_: std::num::TryFromIntError) -> Self {
        HttpResult(StatusCode::BAD_REQUEST, mime::APPLICATION_JSON, Body::empty())
    }
}

impl From<std::option::NoneError> for HttpResult {
    fn from(_: std::option::NoneError) -> Self {
        HttpResult(StatusCode::NOT_FOUND, mime::APPLICATION_JSON, Body::empty())
    }
}

pub fn with_json<F, T, I>(mut state: State, action: F) -> Box<HandlerFuture>
where T: serde::de::DeserializeOwned + Send + Sync + 'static,
      F: FnOnce(&mut State, T) -> I + Send + Sync + 'static,
      I: IntoHttpError {
    let f = extract_json::<T>(&mut state)
        .then(|req| {
            let body = match req {
                Ok(req) => req,
                Err(_) => {
                    let resp = create_response(&state, StatusCode::BAD_REQUEST, mime::APPLICATION_JSON, Body::empty());
                    return Ok((state, resp))
                }
            };
            match action(&mut state, body).into_http_result(&state) {
                Ok(res) | Err(res) => Ok((state, res))
            }
        });
    Box::new(f)
}
