use std::collections::HashMap;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq, Hash)]
pub struct Order {
    pub id: i32,
    pub customer_id: i32,
    pub status: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductInOrder {
    pub product_id: i32,
    pub order_id: i32,
    pub quantity: i32,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct OrderWithProducts {
    pub id: i32,
    pub customer_id: i32,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub products: HashMap<i32, i32>,
}
