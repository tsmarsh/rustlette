use graphql_parser::query::{parse_query, Document, Field, Selection};
use reqwest::Client;
use serde_json::Value;
use std::error::Error;

async fn call_subgraph(url: &str, query: &str, query_name: &str) -> Result<Value, Box<dyn Error>> {
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

fn process_selection_set(selection_set: &[Selection<String>]) -> String {
    selection_set
        .iter()
        .map(process_field_node)
        .collect::<Vec<_>>()
        .join(" ")
}

fn process_field_node(selection: &Selection<String>) -> String {
    match selection {
        Selection::Field(Field {
            name,
            selection_set,
            ..
        }) if !selection_set.items.is_empty() => format!(
            "{} {{ {} }}",
            name,
            process_selection_set(&selection_set.items)
        ),
        Selection::Field(Field { name, .. }) => name.clone(),
        _ => String::new(),
    }
}

fn process_context(id: &str, selection_set: &[Selection<String>], query_name: &str) -> String {
    let selections = process_selection_set(selection_set);
    format!("{{ {}(id: \"{}\") {{ {} }} }}", query_name, id, selections)
}

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