use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenResponse {
    pub token: String,
    pub token_type: String,
}

impl TokenResponse {
    pub fn new(token: String) -> Self {
        TokenResponse {
            token,
            token_type: "Bearer".to_string(),
        }
    }
}
