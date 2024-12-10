use graphql_parser::query::{Field, Selection};


pub fn process_selection_set(selection_set: &[Selection<String>]) -> String {
    selection_set
        .iter()
        .map(process_field_node)
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn process_field_node(selection: &Selection<String>) -> String {
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

pub fn process_context(id: &str, selection_set: &[Selection<String>], query_name: &str) -> String {
    let selections = process_selection_set(selection_set);
    format!("{{ {}(id: \"{}\") {{ {} }} }}", query_name, id, selections)
}