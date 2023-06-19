use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq, Default)]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub price: i32,
    pub available: bool,
}
