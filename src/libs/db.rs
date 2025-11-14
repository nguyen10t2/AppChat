use mongodb::{Client, Database};
use std::env;

pub async fn get_database() -> Database {
    let mongo_uri = env::var("MONGODB_URI").expect("MONGODB_URI chưa được thiết lập");
    let db_name = env::var("DB_NAME").expect("DB_NAME chưa được thiết lập");

    let client = Client::with_uri_str(&mongo_uri)
        .await
        .expect("Không thể kết nối đến MongoDB");

    client.database(&db_name)
}