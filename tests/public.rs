use async_trait::async_trait;
use cucumber::{given, then, when, writer, World, WorldInit};
use itertools::Itertools;
use jsonschema::{Draft, JSONSchema};
use reqwest;
use serde_json;
use std::convert::Infallible;
use std::env;
use std::fs;

// Custom world struct for shared state
#[derive(Debug, WorldInit)]
pub struct ApiWorld {
    api_link: Option<String>,
    raw_api_response: Option<reqwest::Response>,
}

#[async_trait(?Send)]
impl World for ApiWorld {
    type Error = Infallible;

    async fn new() -> Result<Self, Infallible> {
        Ok(Self {
            api_link: None,
            raw_api_response: None,
        })
    }
}

#[given(regex = r"I have link to a public api endpoint returning (server time|asset pair info)")]
fn get_link_to_api(world: &mut ApiWorld, endpoint_type: String) {
    let endpoint_env_var = match endpoint_type.as_str() {
        "server time" => "SERVER_TIME_ENDPOINT",
        "asset pair info" => "ASSET_PAIR_ENDPOINT",
        _ => unreachable!(),
    };
    let endpoint = env::var(endpoint_env_var)
        .expect(format!("Missing secret value: {}", endpoint_env_var).as_str());
    let api_link = env::var("API_LINK").expect("Missing secret value: API_LINK");

    let full_link = [api_link, endpoint].concat();
    world.api_link = Some(full_link.into());
}

#[when(regex = r"I request (server time|asset pair info)")]
async fn request_server_time(world: &mut ApiWorld) -> reqwest::Result<()> {
    let raw_api_response = reqwest::get(world.api_link.as_ref().unwrap()).await?;
    world.raw_api_response = Some(raw_api_response);
    Ok(())
}

#[then(regex = r"the (server time|asset pair info) format is correct")]
async fn verify_response(world: &mut ApiWorld, endpoint_type: String) -> reqwest::Result<()> {
    let raw_api_response = world
        .raw_api_response
        .take()
        .expect("World should contain api response at this point");

    let json_response: serde_json::Value = raw_api_response.json().await?;

    let schema_file = match endpoint_type.as_str() {
        "server time" => "server_time_schema.json",
        "asset pair info" => "asset_pair_schema.json",
        _ => unreachable!(),
    };
    let schema: serde_json::Value = serde_json::from_str(
        fs::read_to_string(format!("./schemas/{}", schema_file))
            .expect("Something went wrong reading the file")
            .as_str(),
    )
    .expect("Schema secret should be possible to parse to json");

    let compiled = JSONSchema::options()
        .with_draft(Draft::Draft7)
        .compile(&schema)
        .expect("Schema should be valid");
    let result = compiled.validate(&json_response);
    match result {
        Ok(_) => Ok(()),
        Err(errors) => {
            let joined_errors = errors.map(|err| format!("{}", err)).join("\n, ");
            panic!("The following errors occured: {}", joined_errors)
        }
    }
}

#[tokio::main]
async fn main() {
    let file = fs::File::create("/results/public.xml").unwrap();
    ApiWorld::cucumber()
        .with_writer(writer::JUnit::new(file, 0))
        .run("features/public.feature")
        .await;
}
