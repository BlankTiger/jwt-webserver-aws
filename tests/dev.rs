use std::{collections::HashMap, fmt::Display};

use color_eyre::{eyre::eyre, Result};
use data::models::Customer;
use once_cell::sync::Lazy;
use reqwest::{header::AUTHORIZATION, Client};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize, Debug)]
struct AuthResponse {
    token: String,
    token_type: String,
}

impl Display for AuthResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\n- token: {}\n- token_type: {}",
            self.token, self.token_type
        )
    }
}

static URL: Lazy<String> = Lazy::new(|| {
    dotenvy::dotenv().ok();
    "http://".to_string() + &std::env::var("SERVER_ADDR").expect("URL must be set")
});

macro_rules! test_get_request_no_auth_endpoints {
    ($a: expr) => {
        let url = URL.to_string();
        let hc = httpc_test::new_client(&url)?;
        for route in $a {
            hc.do_get(route).await?.print().await?;
        }
    };
}

macro_rules! test_get_request_auth_endpoint {
    ($rc: expr, $route: expr, $token: expr) => {
        $rc.get(URL.to_string() + $route)
            .header(AUTHORIZATION, $token)
            .send()
            .await?
    };
}

macro_rules! test_admin_endpoint {
    ($req_builder: expr, $token: expr, $body: expr) => {
        $req_builder
            .header(AUTHORIZATION, $token)
            .json(&$body)
            .send()
            .await?
    };
}

#[tokio::test]
async fn test_no_auth_routes() -> Result<()> {
    let endpoints = [
        "/api/product/1",
        "/api/product/all",
        "/api/order/1",
        "/api/order/all",
    ];
    test_get_request_no_auth_endpoints!(endpoints);

    let rc = Client::new();
    let order = json!(
        {
            "id": 0,
            "customer_id": 1,
            "status": "Done",
            "created_at": "2023-04-25T08:41:23.104715",
            "products": {
                "1": 5,
                "2": 15
            }
        }
    );
    let response = rc
        .post(URL.to_string() + "/api/order")
        .json(&order)
        .send()
        .await?;
    dbg!(response.text().await?);
    Ok(())
}

#[tokio::test]
async fn test_customer_routes() -> Result<()> {
    let rc = Client::new();

    let mut credentials = HashMap::new();
    credentials.insert("name", "example_customer");
    credentials.insert("password", "example_password");
    let token = "Bearer ".to_string()
        + &rc
            .post(URL.to_string() + "/api/user/authorize")
            .json(&json!(credentials))
            .send()
            .await?
            .json::<AuthResponse>()
            .await?
            .token;

    let response = test_get_request_auth_endpoint!(rc, "/api/customer/1", &token);
    dbg!(response.json::<Customer>().await?);

    let response = test_get_request_auth_endpoint!(rc, "/api/customer/all", &token);
    dbg!(response.json::<Vec<Customer>>().await?);

    Ok(())
}

#[tokio::test]
async fn test_admin_routes_reject_non_authed() -> Result<()> {
    assert!(test_admin_product_routes("").await.is_err());
    assert!(test_admin_customer_routes("").await.is_err());
    assert!(test_admin_order_routes("").await.is_err());

    Ok(())
}

#[tokio::test]
async fn test_admin_routes_authed() -> Result<()> {
    let rc = Client::new();

    let mut credentials = HashMap::new();
    credentials.insert("name", "example_admin");
    credentials.insert("password", "example_password");
    let token = "Bearer ".to_string()
        + &rc
            .post(URL.to_string() + "/api/user/authorize")
            .json(&json!(credentials))
            .send()
            .await?
            .json::<AuthResponse>()
            .await?
            .token;

    test_admin_product_routes(&token).await?;
    test_admin_customer_routes(&token).await?;
    test_admin_order_routes(&token).await?;

    Ok(())
}

async fn test_admin_product_routes(token: &str) -> Result<()> {
    let rc = Client::new();

    let endpoint = "/api/admin/product";
    println!("\n========\nTesting: POST {endpoint}");
    let response = test_admin_endpoint!(
        rc.post(URL.to_string() + endpoint),
        token,
        json!(
            {
                "id": 0,
                "name": "New product",
                "price": 600,
                "available": true
            }
        )
    );
    if response.status() != 200 {
        return Err(eyre!("Failed to create customer"));
    }
    dbg!(&response);
    let product_id = &response.text().await?;
    let product =
        test_get_request_auth_endpoint!(rc, &("/api/product/".to_string() + product_id), token)
            .json::<data::models::Product>()
            .await?;
    dbg!(&product);

    let endpoint = "/api/admin/product/".to_string() + product_id;
    println!("\n========\nTesting: PUT {}", &endpoint);
    let response = test_admin_endpoint!(
        rc.put(URL.to_string() + &endpoint),
        token,
        json!(
            {
                "id": product.id,
                "name": "New product",
                "price": 600,
                "available": true
            }
        )
    );
    dbg!(&response);
    dbg!(&response.text().await?);

    let endpoint = "/api/admin/product/".to_string() + product_id;
    println!("\n========\nTesting: PATCH {}", &endpoint);
    let response = test_admin_endpoint!(
        rc.patch(URL.to_string() + &endpoint),
        token,
        json!(
            {
                "price": 1000
            }
        )
    );
    dbg!(&response);
    dbg!(&response.text().await?);

    Ok(())
}

async fn test_admin_customer_routes(token: &str) -> Result<()> {
    let rc = Client::new();

    let endpoint = "/api/admin/customer";
    println!("\n========\nTesting: POST {endpoint}");
    let response = test_admin_endpoint!(
        rc.post(URL.to_string() + endpoint),
        token,
        json!(
            {
                "id": 0,
                "name": "New customer",
                "address": "Some address"
            }
        )
    );
    if response.status() != 200 {
        return Err(eyre!("Failed to create customer"));
    }
    dbg!(&response);
    let customer_id = &1.to_string();

    let endpoint = "/api/admin/customer/".to_string() + customer_id;
    println!("\n========\nTesting: PUT {}", &endpoint);
    let response = test_admin_endpoint!(
        rc.put(URL.to_string() + &endpoint),
        token,
        json!(
            {
                "id": 1,
                "name": "Newer customer",
                "address": "Some address"
            }
        )
    );
    dbg!(&response);
    dbg!(&response.text().await?);

    let endpoint = "/api/admin/customer/".to_string() + customer_id;
    println!("\n========\nTesting: PATCH {}", &endpoint);
    let response = test_admin_endpoint!(
        rc.patch(URL.to_string() + &endpoint),
        token,
        json!(
            {
                "address": "Some other address"
            }
        )
    );
    dbg!(&response);
    dbg!(&response.text().await?);

    Ok(())
}

async fn test_admin_order_routes(token: &str) -> Result<()> {
    let rc = Client::new();

    let endpoint = "/api/order";
    println!("\n========\nTesting: POST {endpoint}");
    let response = test_admin_endpoint!(
        rc.post(URL.to_string() + endpoint),
        token,
        json!(
            {
                "id": 0,
                "customer_id": 1,
                "status": "Done",
                "created_at": "2023-04-25T08:41:23.104715",
                "products": {
                    "1": 5,
                    "2": 15
                }
            }
        )
    );
    dbg!(&response);
    let order_id = &response.text().await?;
    dbg!(&order_id);
    let order = test_get_request_auth_endpoint!(rc, &("/api/order/".to_string() + order_id), token)
        .json::<data::models::Order>()
        .await?;
    dbg!(&order);

    let endpoint = "/api/admin/order/".to_string() + order_id;
    println!("\n========\nTesting: PUT {}", &endpoint);
    let response = test_admin_endpoint!(
        rc.put(URL.to_string() + &endpoint),
        token,
        json!(
            {
                "id": order.id,
                "customer_id": 1,
                "status": "In progress",
                "created_at": "2023-04-25T08:41:23.104715",
                "products": {
                    "1": 5,
                    "2": 15
                }
            }
        )
    );
    if response.status() != 200 {
        return Err(eyre!("Failed to create customer"));
    }
    dbg!(&response);
    dbg!(&response.text().await?);

    let endpoint = "/api/admin/order/".to_string() + order_id;
    println!("\n========\nTesting: PATCH {}", &endpoint);
    let response = test_admin_endpoint!(
        rc.patch(URL.to_string() + &endpoint),
        token,
        json!(
            {
                "products": {
                    "1": 800,
                    "2": 15
                }
            }
        )
    );
    dbg!(&response);
    dbg!(&response.text().await?);

    Ok(())
}
