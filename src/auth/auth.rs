use serde_json::Value;
use std::collections::HashMap;
use serde::Deserialize;

pub trait Authorizer {
    fn get_credential(&self, req: &Request) -> Option<String>;
    fn secure_data(&self, req: &Request, data: Value) -> Value;
    fn authorize_response(&self, req: &Request, res: Value) -> Value;
    fn is_authorized(&self, req: &Request, data: &Value) -> bool;
    fn secure_read(&self, req: &Request, query: &mut HashMap<String, Value>);
}

pub struct Request {
    pub headers: HashMap<String, String>,
}

impl Request {
    pub fn new(headers: HashMap<String, String>) -> Self {
        Request { headers }
    }

    pub fn get_header(&self, name: &str) -> Option<&String> {
        self.headers.get(name)
    }
}

#[derive(Debug, Deserialize)]
struct Claims {
    sub: Option<String>,
}