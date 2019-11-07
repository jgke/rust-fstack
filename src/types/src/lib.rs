use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Account {
    pub id: u32,
    pub username: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Thread {
    pub id: u32,
    pub creator: u32,
    pub title: String,
    pub messages: Option<Vec<Message>>
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Message {
    pub id: u32,
    pub thread_id: u32,
    pub creator: u32,
    pub content: String,
}
