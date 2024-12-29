use reqwest::Client;
use serde_json::Value;

pub trait QueryForwarder {
    fn forward(&self, query: &str, query_name: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>>;
}

pub struct HttpForwarder {
    pub url: String,
}

impl QueryForwarder for HttpForwarder {
    fn forward(&self, query: &str, query_name: &str) -> Result<Value, Box<dyn std::error::Error>> {
        let client = Client::new();
        let response: Value = client
            .post(&self.url)
            .json(&serde_json::json!({ "query": query }))
            .send()?
            .json()?;

        if let Some(errors) = response.get("errors") {
            eprintln!("GraphQL errors: {:?}", errors);
        }

        Ok(response["data"][query_name].clone())
    }
}