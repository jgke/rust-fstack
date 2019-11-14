use serde::{Deserialize, Serialize};

#[cfg(not(cargo_web))]
#[macro_use]
extern crate gotham_derive;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(cargo_web), derive(StateData, StaticResponseExtender))]
pub struct Token {
    pub token: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(cargo_web), derive(StateData, StaticResponseExtender))]
pub struct CreateAccount {
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Account {
    pub id: i32,
    pub username: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(cargo_web), derive(StateData, StaticResponseExtender))]
pub struct Login {
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(cargo_web), derive(StateData, StaticResponseExtender))]
pub struct CreateThread {
    pub title: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Thread {
    pub id: i32,
    pub creator: String,
    pub title: String,
    pub messages: Option<Vec<Message>>,
    pub latest_message: Option<Message>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(cargo_web), derive(StateData, StaticResponseExtender))]
pub struct CreateMessage {
    pub content: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Message {
    pub id: i32,
    pub creator: String,
    pub content: String,
}

