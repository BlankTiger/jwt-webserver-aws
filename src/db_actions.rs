use async_trait::async_trait;
use color_eyre::Result;
use sqlx::postgres::PgPool;
use std::env;

use crate::services::{CustomerService, OrderService, ProductService, UserService};

pub async fn get_pool() -> Result<PgPool> {
    let database_url = env::var("DATABASE_URL")?;
    Ok(PgPool::connect(&database_url).await?)
}

#[async_trait]
pub trait MockFillable {
    async fn fill_with_mocked_data(&self) -> Result<()>;
}

#[async_trait]
pub trait Clearable {
    async fn clear(&self) -> Result<()>;
}

pub struct DbMockData {
    pub product_service: ProductService,
    pub order_service: OrderService,
    pub customer_service: CustomerService,
    pub user_service: UserService,
}

impl Default for DbMockData {
    fn default() -> Self {
        Self::new()
    }
}

impl DbMockData {
    pub fn new() -> Self {
        DbMockData {
            product_service: ProductService {},
            order_service: OrderService {},
            customer_service: CustomerService {},
            user_service: UserService {},
        }
    }

    pub async fn fill(&self) -> Result<()> {
        self.customer_service.fill_with_mocked_data().await?;
        self.product_service.fill_with_mocked_data().await?;
        self.order_service.fill_with_mocked_data().await?;
        self.user_service.fill_with_mocked_data().await?;
        Ok(())
    }

    pub async fn clear(&self) -> Result<()> {
        self.order_service.clear().await?;
        self.customer_service.clear().await?;
        self.product_service.clear().await?;
        self.user_service.clear().await?;
        Ok(())
    }
}
