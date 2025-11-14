use mongodb::{Client, Database};
use std::env;

pub async fn get_database() -> Database {
    let mongo_uri = env::var("MONGODB_URI").expect("MONGODB_URI must be set");
    let db_name = env::var("DB_NAME").expect("DB_NAME must be set");

    let client = Client::with_uri_str(&mongo_uri)
        .await
        .expect("Failed to initialize MongoDB client");

    client.database(&db_name)
}