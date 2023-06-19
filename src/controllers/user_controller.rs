use crate::{
    app::DbPool,
    models::{AuthError, RequestUser, TokenResponse, Roles, Claims},
    services::user_service::*, setup::KEYS,
};
use argon2::{password_hash::PasswordHash, PasswordVerifier};
use axum::{extract::State, Json};
use chrono::Utc;
use color_eyre::Result;
use jsonwebtoken::{encode, Header};

static HOUR_IN_SECONDS: usize = 3600;

pub struct UserController;

impl UserController {
    pub async fn authorize(
        State(pool): State<DbPool>,
        Json(user): Json<RequestUser>,
    ) -> Result<Json<TokenResponse>, AuthError> {
        if user.name.is_empty() || user.password.is_empty() {
            return Err(AuthError::MissingCredentials);
        }

        let claims = Self::get_claims(pool, &user).await?;
        let token = Self::create_token(&claims).await?;
        Ok(Json(token))
    }

    pub async fn create_user(
        State(pool): State<DbPool>,
        Json(user): Json<RequestUser>,
    ) -> Result<Json<TokenResponse>, AuthError> {
        let role;
        if user.name.contains("customer") {
            role = Roles::Customer;
        } else if user.name.contains("admin") {
            role = Roles::Admin;
        } else {
            return Err(AuthError::WrongCredentials);
        }

        let user = UserService::create_user(&pool, user, role)
            .await
            .map_err(|_| AuthError::WrongCredentials)?;

        let claims = Claims::new(
            user.name,
            user.role,
            Utc::now().timestamp() as usize + HOUR_IN_SECONDS / 12,
        );

        let token = Self::create_token(&claims).await?;
        Ok(Json(token))
    }

    async fn get_claims(pool: DbPool, requested_user: &RequestUser) -> Result<Claims, AuthError> {
        let user = UserService::get_user(&pool, &requested_user.name)
            .await
            .map_err(|_| AuthError::WrongCredentials)?;

        let argon2 = get_argon2_instance().map_err(|_| AuthError::TokenCreation)?;
        let parsed_hash =
            PasswordHash::new(&user.passwd_hash).map_err(|_| AuthError::TokenCreation)?;

        let verified = argon2.verify_password(requested_user.password.as_bytes(), &parsed_hash);
        match verified {
            Ok(_) => {
                let claims = Claims::new(
                    user.name,
                    user.role,
                    Utc::now().timestamp() as usize + HOUR_IN_SECONDS / 12,
                );
                Ok(claims)
            }
            Err(_) => Err(AuthError::WrongCredentials),
        }
    }

    async fn create_token(claims: &Claims) -> Result<TokenResponse, AuthError> {
        Ok(TokenResponse::new(
            encode(&Header::default(), &claims, &KEYS.encoding)
                .map_err(|_| AuthError::TokenCreation)?,
        ))
    }
}
