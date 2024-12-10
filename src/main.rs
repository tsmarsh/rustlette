mod callgraph;
mod subgraph;

use graphql_parser::query::{parse_query, Document, Selection};

use std::error::Error;
use crate::callgraph::call_subgraph;
use crate::subgraph::process_context;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let query = r#"
    query {
        topQuery {
            subQuery {
                key
            }
        }
    }
    "#;

    let ast: Document<String> = parse_query(query)?;
    for def in &ast.definitions {
        match def {
            graphql_parser::query::Definition::Operation(op) => {
                match op {
                    graphql_parser::query::OperationDefinition::Query(query) => {
                        for selection in &query.selection_set.items {
                            if let Selection::Field(field) = selection {
                                let subquery = process_context("some_id", &field.selection_set.items, &field.name);
                                let result = call_subgraph("http://example.com/graphql", &subquery, &field.name).await?;
                                println!("Subquery result: {:?}", result);
                            }
                        }
                    }
                    _ => {
                        // Catch-all for any future OperationDefinition types
                        println!("Unknown operation type detected, skipping.");
                    }
                }
            }
            graphql_parser::query::Definition::Fragment(fragment) => {
                println!("Fragment detected: {:?}", fragment.name);
            }
        }
    }

    Ok(())
}