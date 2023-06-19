use crate::db_actions::{get_pool, Clearable, MockFillable};
use chrono::Local;
use color_eyre::Result;
use sqlx::{PgPool, QueryBuilder};
use std::collections::HashMap;
use tracing::info;

use super::PG_LIMIT;

use super::customer_service::CustomerService;
use super::product_service::ProductService;
use crate::models::*;
use async_trait::async_trait;

pub struct OrderService;

macro_rules! create_orders {
    ($a: expr, $b: expr) => {
        OrderService::create_orders($a, $b, false)
    };
    ($a: expr, $b: expr, true) => {
        OrderService::create_orders($a, $b, true)
    };
}

#[async_trait]
impl MockFillable for OrderService {
    async fn fill_with_mocked_data(&self) -> Result<()> {
        let pool = get_pool().await?;
        let mut customer_orders = HashMap::new();
        let products_in_db = ProductService::get_all_products(&pool).await?;
        let customers_in_db = CustomerService::get_all_customers(&pool).await?;

        let new_order = Order {
            id: 1,
            customer_id: customers_in_db[0].id,
            status: "In progress".to_string(),
            created_at: Local::now().naive_local(),
        };
        let mut products_in_order = HashMap::new();
        products_in_order.insert(&products_in_db[0], 1);
        products_in_order.insert(&products_in_db[1], 2);
        customer_orders.insert(new_order, products_in_order);

        let new_order = Order {
            id: 2,
            customer_id: customers_in_db[0].id,
            status: "In progress".to_string(),
            created_at: Local::now().naive_local(),
        };
        let mut products_in_order = HashMap::new();
        products_in_order.insert(&products_in_db[0], 6);
        products_in_order.insert(&products_in_db[1], 2);
        customer_orders.insert(new_order, products_in_order);

        let new_order = Order {
            id: 3,
            customer_id: customers_in_db[1].id,
            status: "New".to_string(),
            created_at: Local::now().naive_local(),
        };
        let mut products_in_order = HashMap::new();
        products_in_order.insert(&products_in_db[0], 3);
        products_in_order.insert(&products_in_db[1], 4);
        customer_orders.insert(new_order, products_in_order);
        create_orders!(&pool, &customer_orders, true).await?;
        // OrderService::create_orders(&pool, &customer_orders).await?;
        Ok(())
    }
}

#[async_trait]
impl Clearable for OrderService {
    async fn clear(&self) -> Result<()> {
        let pool = get_pool().await?;
        sqlx::query!("delete from products_in_orders")
            .execute(&pool)
            .await?;
        sqlx::query!("delete from orders").execute(&pool).await?;
        Ok(())
    }
}

impl OrderService {
    pub async fn create_orders(
        pool: &PgPool,
        customer_orders: &HashMap<Order, HashMap<&Product, i32>>,
        with_id: bool,
    ) -> Result<()> {
        for (new_order, products_in_order) in customer_orders.iter() {
            let curr_order_row: (i32,) = match with_id {
                true => {
                    sqlx::query_as(
                    "insert into orders (id, customer_id, status, created_at) values ($1, $2, $3, $4) returning id",
                )
                    .bind(new_order.id)
                    .bind(new_order.customer_id)
                    .bind(&new_order.status)
                    .bind(new_order.created_at)
                    .fetch_one(pool)
                    .await?
                }
                false => {
                    sqlx::query_as(
                    "insert into orders (customer_id, status, created_at) values ($1, $2, $3) returning id",
                )
                    .bind(new_order.customer_id)
                    .bind(&new_order.status)
                    .bind(new_order.created_at)
                    .fetch_one(pool)
                    .await?
                }
            };
            let curr_order_id = curr_order_row.0;

            let products_in_order: Vec<ProductInOrder> = products_in_order
                .iter()
                .map(|(product, amount)| ProductInOrder {
                    order_id: curr_order_id,
                    product_id: product.id,
                    quantity: *amount,
                })
                .collect();

            let mut query_builder = QueryBuilder::new(
                "insert into products_in_orders (order_id, product_id, quantity) ",
            );
            query_builder.push_values(
                products_in_order.into_iter().take(PG_LIMIT as usize / 3),
                |mut builder, product_in_order| {
                    builder
                        .push_bind(product_in_order.order_id)
                        .push_bind(product_in_order.product_id)
                        .push_bind(product_in_order.quantity);
                },
            );

            info!("Executing group insert query: {}", query_builder.sql());
            let query = query_builder.build();
            query.execute(pool).await?;
        }

        Ok(())
    }

    pub async fn get_order(pool: &PgPool, order_id: i32) -> Result<Order> {
        let order = sqlx::query_as!(Order, "select * from orders where id = $1", order_id)
            .fetch_one(pool)
            .await?;

        Ok(order)
    }

    pub async fn get_order_with_products(
        pool: &PgPool,
        order_id: i32,
    ) -> Result<OrderWithProducts> {
        let order = sqlx::query_as!(Order, "select * from orders where id = $1", order_id)
            .fetch_one(pool)
            .await?;

        let products_in_order = sqlx::query_as!(
            ProductInOrder,
            "select * from products_in_orders where order_id = $1",
            order_id
        )
        .fetch_all(pool)
        .await?;

        let mut products = HashMap::new();
        for product in products_in_order {
            products.insert(product.product_id, product.quantity);
        }

        Ok(OrderWithProducts {
            id: order.id,
            customer_id: order.customer_id,
            status: order.status,
            created_at: order.created_at,
            products,
        })
    }

    pub async fn get_all_orders(pool: &PgPool) -> Result<Vec<Order>> {
        let mut all_orders = Vec::new();
        let all_customers = sqlx::query_as!(Customer, "select * from customers")
            .fetch_all(pool)
            .await?;

        for customer in all_customers {
            let mut customer_orders = sqlx::query_as!(
                Order,
                "select * from orders where customer_id = $1",
                customer.id
            )
            .fetch_all(pool)
            .await?;

            all_orders.append(&mut customer_orders);
        }

        Ok(all_orders)
    }

    pub async fn create_order(pool: &PgPool, new_order: OrderWithProducts) -> Result<i32> {
        let curr_order_row: (i32,) = sqlx::query_as(
            "insert into orders (customer_id, status, created_at) values ($1, $2, $3) returning id",
        )
        .bind(new_order.customer_id)
        .bind(&new_order.status)
        .bind(Local::now().naive_local())
        .fetch_one(pool)
        .await?;
        let curr_order_id = curr_order_row.0;

        let products_in_order: Vec<ProductInOrder> = new_order
            .products
            .iter()
            .map(|(product, amount)| ProductInOrder {
                order_id: curr_order_id,
                product_id: *product,
                quantity: *amount,
            })
            .collect();

        let mut query_builder =
            QueryBuilder::new("insert into products_in_orders (order_id, product_id, quantity) ");
        query_builder.push_values(
            products_in_order.into_iter().take(PG_LIMIT as usize / 3),
            |mut builder, product_in_order| {
                builder
                    .push_bind(product_in_order.order_id)
                    .push_bind(product_in_order.product_id)
                    .push_bind(product_in_order.quantity);
            },
        );

        info!("Executing group insert query: {}", query_builder.sql());
        let query = query_builder.build();
        query.execute(pool).await?;

        Ok(curr_order_id)
    }

    pub async fn update_order(pool: &PgPool, order: OrderWithProducts) -> Result<()> {
        sqlx::query!(
            "update orders set customer_id = $1, status = $2, created_at = $3 where id = $4",
            order.customer_id,
            order.status,
            order.created_at,
            order.id
        )
        .execute(pool)
        .await?;

        sqlx::query!(
            "delete from products_in_orders where order_id = $1",
            order.id
        )
        .execute(pool)
        .await?;

        let products_in_order: Vec<ProductInOrder> = order
            .products
            .iter()
            .map(|(product, amount)| ProductInOrder {
                order_id: order.id,
                product_id: *product,
                quantity: *amount,
            })
            .collect();

        let mut query_builder =
            QueryBuilder::new("insert into products_in_orders (order_id, product_id, quantity) ");
        query_builder.push_values(
            products_in_order.into_iter().take(PG_LIMIT as usize / 3),
            |mut builder, product_in_order| {
                builder
                    .push_bind(product_in_order.order_id)
                    .push_bind(product_in_order.product_id)
                    .push_bind(product_in_order.quantity);
            },
        );

        info!("Executing group insert query: {}", query_builder.sql());
        let query = query_builder.build();
        query.execute(pool).await?;

        Ok(())
    }
}
