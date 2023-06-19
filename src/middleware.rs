use std::fmt::Debug;

use crate::models::{Claims, Roles};
use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use tracing::warn;

pub async fn middleware_require_any_auth<B>(
    _claims: Claims,
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    Ok(next.run(request).await)
}

pub async fn middleware_require_customer_role<B>(
    claims: Claims,
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode>
where
    B: Debug,
{
    if claims.role != Roles::Customer {
        let (head, body) = request.into_parts();
        warn!(
            "{} is not customer\nrequest head: {:?}\nrequest body: {:?}",
            claims, head, body
        );
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(next.run(request).await)
}

pub async fn middleware_require_admin_role<B>(
    claims: Claims,
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode>
where
    B: Debug,
{
    if claims.role != Roles::Admin {
        let (head, body) = request.into_parts();
        warn!(
            "{} is not admin\nrequest head: {:?}\nrequest body: {:?}",
            claims, head, body
        );
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(next.run(request).await)
}
