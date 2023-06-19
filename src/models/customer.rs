use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Default)]
pub struct Customer {
    pub id: i32,
    pub name: String,
    pub address: String,
}
