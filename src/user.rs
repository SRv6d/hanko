use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct User {
    pub name: String,
    pub principals: Vec<String>,
    pub sources: Vec<String>,
}
