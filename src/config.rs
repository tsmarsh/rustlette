use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub graphlettes: Vec<Graphlette>,
    pub restlettes: Vec<Restlette>,
    pub port: u16,
    pub url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Graphlette {
    pub path: String,
    pub mongo: MongoConfig,
    pub schema: String,
    pub dtoConfig: DtoConfig,
}

#[derive(Debug, Deserialize)]
pub struct Restlette {
    pub path: String,
    pub mongo: MongoConfig,
    pub schema: String,
}

#[derive(Debug, Deserialize)]
pub struct MongoConfig {
    pub uri: String,
    pub collection: String,
    pub db: String,
    pub options: MongoOptions,
}

#[derive(Debug, Deserialize)]
pub struct MongoOptions {
    #[serde(rename = "directConnection")]
    pub direct_connection: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DtoConfig {
    pub singletons: Vec<DtoSingleton>,
    pub vectors: Vec<DtoVector>,
    pub resolvers: Vec<DtoResolver>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DtoSingleton {
    pub name: String,
    pub query: String,
    pub id: Option<String>, // Optional as it's not always present
}

#[derive(Debug, Deserialize, Clone)]
pub struct DtoVector {
    pub name: String,
    pub query: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DtoResolver {
    pub name: String,
    pub queryName: String,
    pub url: String,
    pub id: Option<String>, // Optional as it's not always present
}