use serde::Deserialize;

#[derive(Deserialize)]
pub struct QueryIdParam {
    pub id: i32,
}
