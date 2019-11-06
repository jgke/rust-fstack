use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct Person {
    pub name: String
}
