use async_trait::async_trait;
use axum::{
    extract::FromRequestParts,
    headers::{authorization::Bearer, Authorization},
    http::request::Parts,
    RequestPartsExt, TypedHeader,
};
use jsonwebtoken::{decode, Validation};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

use super::Roles;
use crate::{models::AuthError, setup::KEYS};

#[derive(Serialize, Deserialize, Debug)]
pub struct Claims {
    pub name: String,
    pub role: Roles,
    pub exp: usize,
}

impl Display for Claims {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "User:\n- name: {},\n- role: {:?}", self.name, self.role)
    }
}

impl Claims {
    pub fn new(name: String, role: Roles, exp: usize) -> Self {
        Self { name, role, exp }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::InvalidToken)?;
        // Decode the user data
        let token_data = decode::<Claims>(bearer.token(), &KEYS.decoding, &Validation::default())
            .map_err(|_| AuthError::InvalidToken)?;

        Ok(token_data.claims)
    }
}
