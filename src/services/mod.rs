mod customer_service;
mod order_service;
mod product_service;
pub mod user_service;

pub use customer_service::CustomerService;
pub use order_service::OrderService;
pub use product_service::ProductService;
pub use user_service::UserService;

static PG_LIMIT: u16 = u16::MAX;
