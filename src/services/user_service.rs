use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use async_trait::async_trait;

use color_eyre::{eyre::eyre, Result};
use sqlx::PgPool;
use tracing::info;

use crate::{
    db_actions::{get_pool, Clearable, MockFillable},
    models::{RequestUser, Roles, User},
    setup::PEPPER,
};

pub fn get_argon2_instance() -> Result<Argon2<'static>> {
    Argon2::new_with_secret(
        PEPPER.as_bytes(),
        Default::default(),
        Default::default(),
        Default::default(),
    )
    .map_err(color_eyre::Report::msg)
}

pub struct UserService;

#[async_trait]
impl MockFillable for UserService {
    async fn fill_with_mocked_data(&self) -> Result<()> {
        let customer = RequestUser {
            name: "example_customer".to_string(),
            password: "example_password".to_string(),
        };
        let admin = RequestUser {
            name: "example_admin".to_string(),
            password: "example_password".to_string(),
        };

        let pool = get_pool().await?;
        Self::create_user(&pool, customer, Roles::Customer).await?;
        Self::create_user(&pool, admin, Roles::Admin).await?;
        Ok(())
    }
}

#[async_trait]
impl Clearable for UserService {
    async fn clear(&self) -> Result<()> {
        let pool = get_pool().await?;
        sqlx::query!("delete from users").execute(&pool).await?;
        Ok(())
    }
}

impl UserService {
    pub async fn get_user(pool: &PgPool, name: &str) -> Result<User> {
        let user = sqlx::query_as!(
            User,
            r#"SELECT id, name, passwd_hash, role as "role: Roles" FROM users WHERE name = $1"#,
            name
        )
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    pub async fn create_user(pool: &PgPool, user: RequestUser, role: Roles) -> Result<User> {
        let argon2 = get_argon2_instance()?;
        let salt = SaltString::generate(&mut OsRng);
        let hash = argon2
            .hash_password(user.password.as_bytes(), &salt)
            .map_err(|e| eyre!(e))?;

        info!("Creating user: {}", user.name);
        let user = sqlx::query_as!(
            User,
            r#"INSERT INTO users (name, passwd_hash, role) VALUES ($1, $2, $3) RETURNING id, name, passwd_hash, role as "role: Roles""#,
            user.name,
            hash.to_string(),
            role as Roles
        )
        .fetch_one(pool)
        .await?;

        Ok(user)
    }
}
