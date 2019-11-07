use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Account {
    pub id: i32,
    pub username: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Thread {
    pub id: i32,
    pub creator: i32,
    pub title: String,
    pub messages: Option<Vec<Message>>
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Message {
    pub id: i32,
    pub creator: i32,
    pub content: String,
}
