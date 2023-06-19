use crate::models::QueryIdParam;
use crate::{app::DbPool, models::*};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::NaiveDateTime;
use serde_json::Value;
use tracing::{info, warn};

use crate::services::OrderService;

pub struct OrderController;

impl OrderController {
    pub async fn get_order(
        State(pool): State<DbPool>,
        Query(QueryIdParam { id }): Query<QueryIdParam>,
    ) -> Result<impl IntoResponse, StatusCode> {
        let response = Json(OrderService::get_order(&pool, id).await.map_err(|e| {
            warn!("{e}");
            StatusCode::NOT_FOUND
        })?);
        Ok(response)
    }

    pub async fn get_all_orders(
        State(pool): State<DbPool>,
    ) -> Result<impl IntoResponse, StatusCode> {
        let response = Json(OrderService::get_all_orders(&pool).await.map_err(|e| {
            warn!("{e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?);
        Ok(response)
    }

    pub async fn create_order(
        State(pool): State<DbPool>,
        Json(order): Json<OrderWithProducts>,
    ) -> Result<impl IntoResponse, StatusCode> {
        info!("Received order: {:?}", order);

        let response = Json(
            OrderService::create_order(&pool, order)
                .await
                .map_err(|e| {
                    warn!("{e}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?,
        );
        Ok(response)
    }

    pub async fn update_order(
        State(pool): State<DbPool>,
        Query(QueryIdParam { id }): Query<QueryIdParam>,
        Json(order): Json<OrderWithProducts>,
    ) -> Result<impl IntoResponse, StatusCode> {
        if id != order.id {
            return Err(StatusCode::BAD_REQUEST);
        }

        let response = Json(
            OrderService::update_order(&pool, order)
                .await
                .map_err(|e| {
                    warn!("{e}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?,
        );

        Ok(response)
    }

    pub async fn partial_update_order(
        State(pool): State<DbPool>,
        Query(QueryIdParam { id }): Query<QueryIdParam>,
        Json(mut body): Json<Value>,
    ) -> Result<impl IntoResponse, StatusCode> {
        let body_map = body.as_object_mut().ok_or(StatusCode::BAD_REQUEST)?;
        body_map.remove("id");

        let mut order_with_products = OrderService::get_order_with_products(&pool, id)
            .await
            .map_err(|e| {
                warn!("{e}");
                StatusCode::NOT_FOUND
            })?;

        info!(
            "Received body_map: {:?}\nTo update: {:?}",
            body_map, order_with_products
        );

        for (key, value) in body_map.iter_mut() {
            match key.as_str() {
                "customer_id" => {
                    order_with_products.customer_id =
                        value.as_i64().ok_or(StatusCode::BAD_REQUEST)? as i32
                }
                "status" => {
                    order_with_products.status =
                        value.as_str().ok_or(StatusCode::BAD_REQUEST)?.to_string()
                }
                "created_at" => {
                    order_with_products.created_at = NaiveDateTime::parse_from_str(
                        value.as_str().ok_or(StatusCode::BAD_REQUEST)?,
                        "%Y-%m-%d %H:%M:%S",
                    )
                    .map_err(|e| {
                        warn!("{e}");
                        StatusCode::BAD_REQUEST
                    })?
                }
                "products" => {
                    order_with_products.products = value
                        .as_object()
                        .ok_or(StatusCode::BAD_REQUEST)?
                        .iter()
                        .map(|(key, value)| -> (i32, i32) {
                            (
                                key.parse::<i32>()
                                    .map_err(|e| {
                                        warn!("{e}");
                                        StatusCode::BAD_REQUEST
                                    })
                                    .unwrap(),
                                value.as_i64().ok_or(StatusCode::BAD_REQUEST).unwrap() as i32,
                            )
                        })
                        .collect()
                }
                _ => return Err(StatusCode::BAD_REQUEST),
            }
        }

        let response = Json(
            OrderService::update_order(&pool, order_with_products)
                .await
                .map_err(|e| {
                    warn!("{e}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?,
        );

        Ok(response)
    }
}
