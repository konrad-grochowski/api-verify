mod resources;
use async_trait::async_trait;
use cucumber::writer;
use cucumber::{given, then, when, World, WorldInit};
use reqwest;
use resources::request_handler;
use serde;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::env;
use std::fs;

#[given("I have some properties concerning a private API")]
fn setup_api_properties(world: &mut ApiWorld) {
    world.private_api_properties = Some(PrivateApiProperties {
        otp_secret: env::var("OTP_SECRET").expect("Missing environment variable: OTP_SECRET"),
        api_link: env::var("API_LINK").expect("Missing environment variable: API_LINK"),
        api_key: env::var("API_KEY").expect("Missing environment variable: API_KEY"),
        api_secret: env::var("API_SECRET").expect("Missing environment variable: API_SECRET"),
        open_orders_endpoint: env::var("OPEN_ORDERS_ENDPOINT")
            .expect("Missing environment variable: OPEN_ORDERS_ENDPOINT"),
    });
}

#[when("I request all open orders")]
async fn request_server_time(world: &mut ApiWorld) -> reqwest::Result<()> {
    let properties = world
        .private_api_properties
        .take()
        .expect("Api properties are empty");

    let result = request_handler::private_api_request(
        &properties.api_key,
        &properties.api_secret,
        &properties.otp_secret,
        &properties.api_link,
        &properties.open_orders_endpoint,
    )
    .await;
    world.raw_api_response = Some(result?);
    Ok(())
}

#[then("the open orders list is presented to me")]
async fn verify_open_orders(world: &mut ApiWorld) -> reqwest::Result<()> {
    let raw_api_response = world
        .raw_api_response
        .take()
        .expect("World should contain api response at this point");

    let json_response: serde_json::Value = raw_api_response.json().await?;
    println!("List of open orders:");
    if let Some(content) = json_response["result"]["open"].as_object() {
        for (key, value) in content {
            println!("{:?}: {:?}", key, value);
        }
    }

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct PrivateApiProperties {
    otp_secret: String,
    api_link: String,
    api_key: String,
    api_secret: String,
    open_orders_endpoint: String,
}

#[derive(Debug, WorldInit)]
pub struct ApiWorld {
    private_api_properties: Option<PrivateApiProperties>,
    raw_api_response: Option<reqwest::Response>,
}

#[async_trait(?Send)]
impl World for ApiWorld {
    type Error = Infallible;

    async fn new() -> Result<Self, Infallible> {
        Ok(Self {
            private_api_properties: None,
            raw_api_response: None,
        })
    }
}

#[tokio::main]
async fn main() {
    let file = fs::File::create("/results/private.xml").unwrap();
    ApiWorld::cucumber()
        .with_writer(writer::JUnit::new(file, 0))
        .run("features/private.feature")
        .await;
}
