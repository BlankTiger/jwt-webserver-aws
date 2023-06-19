use crate::models::QueryIdParam;
use crate::{app::DbPool, models::Customer};
use axum::extract::Query;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde_json::Value;
use tracing::{info, warn};

use crate::services::CustomerService;

pub struct CustomerController;

impl CustomerController {
    pub async fn get_customer(
        State(pool): State<DbPool>,
        Query(QueryIdParam { id }): Query<QueryIdParam>,
    ) -> Result<impl IntoResponse, StatusCode> {
        let response = Json(
            CustomerService::get_customer(&pool, id)
                .await
                .map_err(|e| {
                    warn!("{e}");
                    StatusCode::NOT_FOUND
                })?,
        );
        Ok(response)
    }

    pub async fn get_all_customers(
        State(pool): State<DbPool>,
    ) -> Result<impl IntoResponse, StatusCode> {
        let response = Json(
            CustomerService::get_all_customers(&pool)
                .await
                .map_err(|e| {
                    warn!("{e}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?,
        );
        Ok(response)
    }

    pub async fn create_customer(
        State(pool): State<DbPool>,
        Json(customer): Json<Customer>,
    ) -> Result<impl IntoResponse, StatusCode> {
        let response = Json(
            CustomerService::create_customer(&pool, customer)
                .await
                .map_err(|e| {
                    warn!("{e}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?,
        );
        Ok(response)
    }

    pub async fn update_customer(
        State(pool): State<DbPool>,
        Query(QueryIdParam { id }): Query<QueryIdParam>,
        Json(customer): Json<Customer>,
    ) -> Result<impl IntoResponse, StatusCode> {
        if id != customer.id {
            return Err(StatusCode::BAD_REQUEST);
        }

        let response = Json(
            CustomerService::update_customer(&pool, customer)
                .await
                .map_err(|e| {
                    warn!("{e}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?,
        );
        Ok(response)
    }

    pub async fn partial_update_customer(
        State(pool): State<DbPool>,
        Query(QueryIdParam { id }): Query<QueryIdParam>,
        Json(mut body): Json<Value>,
    ) -> Result<impl IntoResponse, StatusCode> {
        let body_map = body.as_object_mut().ok_or(StatusCode::BAD_REQUEST)?;
        body_map.remove("id");
        let mut customer_with_id = CustomerService::get_customer(&pool, id)
            .await
            .map_err(|e| {
                warn!("{e}");
                StatusCode::NOT_FOUND
            })?;

        info!(
            "Received body_map: {:?}\nTo update: {:?}",
            body_map, customer_with_id
        );

        for (key, value) in body_map.iter_mut() {
            match key.as_str() {
                "name" => {
                    customer_with_id.name =
                        value.as_str().ok_or(StatusCode::BAD_REQUEST)?.to_string()
                }
                "address" => {
                    customer_with_id.address =
                        value.as_str().ok_or(StatusCode::BAD_REQUEST)?.to_string()
                }
                _ => return Err(StatusCode::BAD_REQUEST),
            }
        }

        let response = Json(
            CustomerService::update_customer(&pool, customer_with_id)
                .await
                .map_err(|e| {
                    warn!("{e}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?,
        );
        Ok(response)
    }
}
