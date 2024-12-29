use std::collections::HashMap;
use serde_json::Value;
use crate::config::DtoResolver;

pub struct DTOFactory {
    resolvers: Vec<DtoResolver>,
}

impl DTOFactory {
    pub fn new(resolvers: Vec<DtoResolver>) -> Self {
        DTOFactory { resolvers }
    }

    /// Process a single object
    pub fn fill_one(&self, data: &Value) -> Value {
        let mut result = Value::Object(Default::default());

        if let Some(obj) = data.as_object() {
            for (key, value) in obj {
                if let Some(resolver) = self.resolvers.get(key) {
                    result[key] = resolver(value);
                } else {
                    result[key] = value.clone(); // Default to raw value
                }
            }
        }

        result
    }

    /// Process multiple objects
    pub fn fill_many(&self, data_list: &[Value]) -> Vec<Value> {
        data_list.iter().map(|data| self.fill_one(data)).collect()
    }
}