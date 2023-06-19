use crate::models::QueryIdParam;
use crate::{app::DbPool, models::Product};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::Value;
use tracing::{info, warn};

use crate::services::ProductService;

pub struct ProductController;

impl ProductController {
    pub async fn get_product(
        State(pool): State<DbPool>,
        Query(QueryIdParam { id }): Query<QueryIdParam>,
    ) -> Result<impl IntoResponse, StatusCode> {
        let response = Json(ProductService::get_product(&pool, id).await.map_err(|e| {
            warn!("{e}");
            StatusCode::NOT_FOUND
        })?);
        Ok(response)
    }

    pub async fn get_all_products(
        State(pool): State<DbPool>,
    ) -> Result<impl IntoResponse, StatusCode> {
        let response = Json(ProductService::get_all_products(&pool).await.map_err(|e| {
            warn!("{e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?);
        Ok(response)
    }

    pub async fn create_product(
        State(pool): State<DbPool>,
        Json(product): Json<Product>,
    ) -> Result<impl IntoResponse, StatusCode> {
        info!("Received product: {:?}", product);

        let response = Json(
            ProductService::create_product(&pool, product)
                .await
                .map_err(|e| {
                    warn!("{e}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?,
        );
        Ok(response)
    }

    pub async fn update_product(
        State(pool): State<DbPool>,
        Query(QueryIdParam { id }): Query<QueryIdParam>,
        Json(product): Json<Product>,
    ) -> Result<impl IntoResponse, StatusCode> {
        if id != product.id {
            return Err(StatusCode::BAD_REQUEST);
        }

        let response = Json(
            ProductService::update_product(&pool, product)
                .await
                .map_err(|e| {
                    warn!("{e}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?,
        );
        Ok(response)
    }

    pub async fn partial_update_product(
        State(pool): State<DbPool>,
        Query(QueryIdParam { id }): Query<QueryIdParam>,
        Json(mut body): Json<Value>,
    ) -> Result<impl IntoResponse, StatusCode> {
        let body_map = body.as_object_mut().ok_or(StatusCode::BAD_REQUEST)?;
        body_map.remove("id");
        let mut product_with_id = ProductService::get_product(&pool, id).await.map_err(|e| {
            warn!("{e}");
            StatusCode::NOT_FOUND
        })?;

        info!(
            "Received body_map: {:?}\nTo update: {:?}",
            body_map, product_with_id
        );

        for (key, value) in body_map.iter_mut() {
            match key.as_str() {
                "name" => {
                    product_with_id.name =
                        value.as_str().ok_or(StatusCode::BAD_REQUEST)?.to_string()
                }
                "price" => {
                    product_with_id.price = value.as_i64().ok_or(StatusCode::BAD_REQUEST)? as i32
                }
                "available" => {
                    product_with_id.available = value.as_bool().ok_or(StatusCode::BAD_REQUEST)?
                }
                _ => return Err(StatusCode::BAD_REQUEST),
            }
        }

        let response = Json(
            ProductService::update_product(&pool, product_with_id)
                .await
                .map_err(|e| {
                    warn!("{e}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?,
        );
        Ok(response)
    }
}
