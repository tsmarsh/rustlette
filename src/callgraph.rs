use std::error::Error;
use reqwest::Client;
use serde_json::Value;

pub async fn call_subgraph(url: &str, query: &str, query_name: &str) -> Result<Value, Box<dyn Error>> {
    let client = Client::new();
    let response = client
        .post(url)
        .json(&serde_json::json!({ "query": query }))
        .send()
        .await?
        .json::<Value>()
        .await?;

    if let Some(errors) = response.get("errors") {
        eprintln!("GraphQL errors: {:?}", errors);
    }

    Ok(response["data"][query_name].clone())
}