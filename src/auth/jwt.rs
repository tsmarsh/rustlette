use std::collections::HashMap;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::Deserialize;
use serde_json::Value;
use crate::auth::auth::{Authorizer, Request};

pub struct JWTSubAuthorizer;

#[derive(Debug, Deserialize)]
struct Claims {
    sub: Option<String>,
}

impl Authorizer for JWTSubAuthorizer {
    fn get_credential(&self, req: &Request) -> Option<String> {
        if let Some(auth_header) = req.get_header("authorization") {
            if auth_header.starts_with("Bearer ") {
                let token = &auth_header[7..];
                let mut validation = Validation::new(Algorithm::HS256);
                validation.insecure_disable_signature_validation();

                if let Ok(decoded) = decode::<Claims>(
                    token,
                    &DecodingKey::from_secret(&[]), // No actual key needed
                    &validation,      // No validation
                ) {
                    return decoded.claims.sub;
                }
            }
        }
        None
    }

    fn secure_data(&self, req: &Request, mut data: Value) -> Value {
        if let Some(subscriber) = self.get_credential(req) {
            data["authorized_readers"] = Value::Array(vec![Value::String(subscriber)]);
        }
        data
    }

    fn authorize_response(&self, _req: &Request, res: Value) -> Value {
        res
    }

    fn is_authorized(&self, req: &Request, data: &Value) -> bool {
        if let Some(subscriber) = self.get_credential(req) {
            if let Some(authorized_readers) = data.get("authorized_readers") {
                if let Some(readers) = authorized_readers.as_array() {
                    return readers.iter().any(|reader| reader == &Value::String(subscriber.clone()));
                }
            }
        }
        true
    }

    fn secure_read(&self, req: &Request, query: &mut HashMap<String, Value>) {
        if let Some(subscriber) = self.get_credential(req) {
            query.insert(
                "authorized_readers".to_string(),
                Value::Object([("$in".to_string(), Value::Array(vec![Value::String(subscriber)]))]
                    .iter()
                    .cloned()
                    .collect()),
            );
        }
    }
}