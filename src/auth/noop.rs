use std::collections::HashMap;
use serde_json::Value;
use crate::auth::auth::{Authorizer, Request};

pub struct NoOpAuthorizer;

impl Authorizer for NoOpAuthorizer {
    fn get_credential(&self, _req: &Request) -> Option<String> {
        None
    }

    fn secure_data(&self, _req: &Request, data: Value) -> Value {
        data
    }

    fn authorize_response(&self, _req: &Request, res: Value) -> Value {
        res
    }

    fn is_authorized(&self, _req: &Request, _data: &Value) -> bool {
        true
    }

    fn secure_read(&self, _req: &Request, _query: &mut HashMap<String, Value>) {}
}