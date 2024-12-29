use async_graphql::*;
use mongodb::{bson::doc, options::ClientOptions, Client};
use serde_json::json;
use tokio;
use mockito::{Mock, Matcher};

#[tokio::test]
async fn test_simple_root() {
    let mongo_uri = "mongodb://localhost:27017";
    let client = Client::with_uri_str(mongo_uri).await.unwrap();
    let db = client.database("test_db");
    let collection = db.collection("simple");

    collection
        .insert_one(
            doc! {
                "id": "test_id",
                "payload": { "foo": "bar", "eggs": 6 },
                "createdAt": bson::DateTime::now(),
            }
        )
        .await
        .unwrap();

    #[derive(SimpleObject)]
    struct Test {
        id: String,
        foo: String,
        eggs: i32,
    }

    struct QueryRoot;

    #[Object]
    impl QueryRoot {
        async fn get_by_id(&self, ctx: &Context<'_>, id: String) -> Test {
            let db = ctx.data::<mongodb::Database>().unwrap();
            let collection = db.collection("simple");

            let doc = collection
                .find_one(doc! { "id": id })
                .await
                .unwrap()
                .unwrap();

            let id = doc.get_str("id").unwrap().to_string();
            let payload = doc.get_document("payload").unwrap();
            let foo = payload.get_str("foo").unwrap().to_string();
            let eggs = payload.get_i32("eggs").unwrap();

            Test { id, foo, eggs }
        }
    }

    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .data(db.clone())
        .finish();

    let query = r#"
        {
            getById(id: "test_id") {
                eggs
            }
        }
    "#;

    let response = schema.execute(query).await;

    assert!(response.errors.is_empty(), "{:?}", response.errors);
    let data = response.data.into_json().unwrap();
    assert_eq!(data["getById"]["eggs"], 6);
}
//
// #[tokio::test]
// async fn test_mocked_dependency() {
//     let _mock = Mock::mock("POST", "/")
//         .match_body(Matcher::Regex(".*".to_string()))
//         .with_status(200)
//         .with_body(json!({
//             "data": { "getById": { "name": "mega" } }
//         })
//             .to_string())
//         .create();
//
//     #[derive(SimpleObject)]
//     struct Coop {
//         name: String,
//     }
//
//     #[derive(SimpleObject)]
//     struct Test {
//         id: String,
//         name: String,
//         eggs: i32,
//         coop: Coop,
//     }
//
//     struct QueryRoot;
//
//     #[Object]
//     impl QueryRoot {
//         async fn get_by_id(&self, ctx: &Context<'_>, id: String) -> Test {
//             let db = ctx.data::<mongodb::Database>().unwrap();
//             let collection = db.collection("simple");
//
//             let doc = collection
//                 .find_one(doc! { "id": id })
//                 .await?
//                 .unwrap();
//
//             let id = doc.get_str("id").unwrap().to_string();
//             let payload = doc.get_document("payload").unwrap();
//             let name = payload.get_str("name").unwrap().to_string();
//             let eggs = payload.get_i32("eggs").unwrap();
//             let coop_id = payload.get_str("coop_id").unwrap();
//
//             // Mocked HTTP call
//             let client = reqwest::Client::new();
//             let coop_response: serde_json::Value = client
//                 .post(mockito::server_url())
//                 .json(&json!({ "id": coop_id }))
//                 .send()
//                 .await
//                 .unwrap()
//                 .json()
//                 .await
//                 .unwrap();
//
//             let coop_name = coop_response["data"]["getById"]["name"]
//                 .as_str()
//                 .unwrap()
//                 .to_string();
//
//             Test {
//                 id,
//                 name,
//                 eggs,
//                 coop: Coop { name: coop_name },
//             }
//         }
//     }
//
//     let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
//         .finish();
//
//     let query = r#"
//         {
//             getById(id: "chuck") {
//                 id
//                 name
//                 coop {
//                     name
//                 }
//             }
//         }
//     "#;
//
//     let response = schema.execute(query).await;
//
//     assert!(response.errors.is_empty(), "{:?}", response.errors);
//     let data = response.data.into_json().unwrap();
//     assert_eq!(data["getById"]["coop"]["name"], "mega");
// }