use std::sync::Arc;

use crate::auth::auth::{Authorizer, Request};
use crate::config::{Config, DtoConfig, DtoResolver, Graphlette, Restlette};
use crate::graph::dto::DTOFactory;
use bson::DateTime;
use handlebars::Handlebars;
use mongodb::{bson::doc, Database};
use serde_json::Value;

pub struct Context {
    pub dto_factory: DTOFactory,
    pub root: Root,
}

impl Context {
    pub fn new(
        db: Arc<mongodb::Database>,
        authorizer: Arc<dyn Authorizer>,
        config: Graphlette,
    ) -> Self {
        let dto_factory = DTOFactory::new(config.dtoConfig);
        let root = Root::new(db, Arc::clone(&dto_factory), authorizer, config);
        Context { dto_factory, root }
    }
}

pub struct Root {
    pub singletons: Vec<DtoResolver>,
    pub vectors: Vec<DtoResolver>,
}

impl Root {
    pub fn new(
        db: Arc<mongodb::Database>,
        dto_factory: Arc<DTOFactory>,
        authorizer: Arc<dyn Authorizer>,
        config: DtoConfig,
    ) -> Self {
        let singletons = config
            .singletons
            .iter()
            .map(|s| {
                singleton(
                    db.clone(),
                    dto_factory.clone(),
                    authorizer.clone(),
                    &s.query,
                )
            })
            .collect();

        let vectors = config
            .vectors
            .iter()
            .map(|v| {
                vector(
                    db.clone(),
                    dto_factory.clone(),
                    authorizer.clone(),
                    &v.query,
                )
            })
            .collect();

        Root {
            singletons,
            vectors,
        }
    }
}

pub fn process_query_template(parameters: &Value, query_template: &str) -> Result<Value, String> {
    let handlebars = Handlebars::new();
    let query = handlebars
        .render_template(query_template, parameters)
        .map_err(|e| format!("Template rendering failed: {}", e))?;

    serde_json::from_str(&query).map_err(|e| format!("Failed to parse query JSON: {}", e))
}

pub fn vector(
    db: Arc<Database>,
    dto_factory: Arc<DTOFactory>,
    authorizer: Arc<dyn Authorizer>,
    query_template: &str,
) -> DtoResolver {
    Box::new(move |args: &Value, context: Option<&Context>| {
        let query = process_query_template(args, query_template).unwrap();
        let timestamp = get_timestamp(args);
        let time_filter = doc! { "createdAt": { "$lt": timestamp } };

        let mut query = doc!(query.as_object().unwrap().clone());
        query.insert("createdAt".to_string(), time_filter);

        // MongoDB query example
        let results = db.collection("your_collection").aggregate(vec![
            doc! { "$match": query },
            doc! { "$sort": { "createdAt": -1 } },
            doc! { "$group": { "_id": "$id", "doc": { "$first": "$$ROOT" } } },
            doc! { "$replaceRoot": { "newRoot": "$doc" } },
        ]);

        let results: Vec<_> = results.filter(|r| {
            context
                .map(|ctx| authorizer.is_authorized(&ctx, r))
                .unwrap_or(true)
        });

        dto_factory.fill_many(results)
    })
}
pub fn singleton(
    db: Arc<Database>,
    collection_name: &str,
    query_template: &str,
    authorizer: Arc<dyn Authorizer>,
) -> impl Fn(Value, Option<&Request>) -> Option<Value> + Send + Sync {
    move |args: Value, req: Option<&Request>| {
        // Render the query template with arguments
        let query = match process_query_template(&args, query_template) {
            Ok(q) => q,
            Err(err) => {
                eprintln!("Failed to render query template: {}", err);
                return None;
            }
        };

        // Parse the query as BSON for MongoDB
        let bson_query = match serde_json::to_bson(&query) {
            Ok(bson) => bson,
            Err(err) => {
                eprintln!("Failed to convert query to BSON: {}", err);
                return None;
            }
        };

        // Fetch the document from the MongoDB collection
        let collection = db.collection::<Value>(collection_name);
        let result = match collection.find_one(bson_query.as_document()) {
            Ok(Some(doc)) => doc,
            Ok(None) => {
                eprintln!("No document found for query: {:?}", query);
                return None;
            }
            Err(err) => {
                eprintln!("MongoDB query error: {}", err);
                return None;
            }
        };

        // Apply authorization if a request context is provided
        if let Some(req) = req {
            if !authorizer.is_authorized(req, &result) {
                eprintln!("Authorization failed for document: {:?}", result);
                return None;
            }
        }

        Some(result)
    }
}

pub fn get_timestamp(args: &Value) -> DateTime {
    args.get("at")
        .and_then(|at| at.as_i64())
        .map(DateTime::from_millis)
        .unwrap_or_else(|| DateTime::now())
}
