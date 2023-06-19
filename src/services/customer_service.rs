use super::PG_LIMIT;
use crate::db_actions::{get_pool, Clearable, MockFillable};
use color_eyre::Result;
use sqlx::{PgPool, QueryBuilder};
use tracing::info;

use crate::models::Customer;
use async_trait::async_trait;

pub struct CustomerService;

macro_rules! create_customers {
    ($a: expr, $b: expr) => {
        CustomerService::create_customers($a, $b, false)
    };
    ($a: expr, $b: expr, true) => {
        CustomerService::create_customers($a, $b, true)
    };
}

#[async_trait]
impl MockFillable for CustomerService {
    async fn fill_with_mocked_data(&self) -> Result<()> {
        let new_customers = [
            Customer {
                id: 1,
                name: "Customer 1".to_string(),
                address: "Address 1".to_string(),
            },
            Customer {
                id: 2,
                name: "Customer 2".to_string(),
                address: "Address 2".to_string(),
            },
        ];

        let pool = get_pool().await?;
        create_customers!(&pool, &new_customers, true).await?;
        Ok(())
    }
}

#[async_trait]
impl Clearable for CustomerService {
    async fn clear(&self) -> Result<()> {
        let pool = get_pool().await?;
        sqlx::query!("delete from customers").execute(&pool).await?;
        Ok(())
    }
}

impl CustomerService {
    pub async fn create_customer(pool: &PgPool, new_customer: Customer) -> Result<i32> {
        let new_customer_row: (i32,) =
            sqlx::query_as("insert into customers (name, address) values ($1, $2) returning id")
                .bind(new_customer.name)
                .bind(new_customer.address)
                .fetch_one(pool)
                .await?;

        Ok(new_customer_row.0)
    }

    pub async fn create_customers(pool: &PgPool, new_customers: &[Customer], with_id: bool) -> Result<()> {
        let mut query_builder = match with_id {
            true => {
                let mut query_builder = QueryBuilder::new("insert into customers (id, name, address) ");
                query_builder.push_values(
                    new_customers.iter().take(PG_LIMIT as usize / 3),
                    |mut builder, customer| {
                        builder
                            .push_bind(customer.id)
                            .push_bind(&customer.name)
                            .push_bind(&customer.address);
                    },
                );
                query_builder
            }
            false => {
                let mut query_builder = QueryBuilder::new("insert into customers (name, address) ");
                query_builder.push_values(
                    new_customers.iter().take(PG_LIMIT as usize / 2),
                    |mut builder, customer| {
                        builder
                            .push_bind(&customer.name)
                            .push_bind(&customer.address);
                    },
                );
                query_builder
            }
        };

        info!("Executing group insert query: {}", query_builder.sql());
        let query = query_builder.build();
        query.execute(pool).await?;

        Ok(())
    }

    pub async fn get_customer(pool: &PgPool, id: i32) -> Result<Customer> {
        Ok(
            sqlx::query_as!(Customer, "select * from customers where id = $1", id)
                .fetch_one(pool)
                .await?,
        )
    }

    pub async fn get_all_customers(pool: &PgPool) -> Result<Vec<Customer>> {
        info!("Returning all customers from database");
        Ok(sqlx::query_as!(Customer, "select * from customers")
            .fetch_all(pool)
            .await?)
    }

    pub async fn update_customer(pool: &PgPool, updated_customer: Customer) -> Result<()> {
        sqlx::query!(
            "update customers set name = $1, address = $2 where id = $3",
            updated_customer.name,
            updated_customer.address,
            updated_customer.id
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
