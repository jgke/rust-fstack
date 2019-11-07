use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Person {
    pub name: String
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Thread {
    pub id: u64,
    pub name: String,
}
