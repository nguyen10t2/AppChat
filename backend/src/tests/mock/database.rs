use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use uuid::Uuid;

#[derive(Clone, Default)]
pub struct MockDatabase {
    pool: Option<sqlx::PgPool>,
    users: Arc<Mutex<HashMap<Uuid, serde_json::Value>>>,
    conversations: Arc<Mutex<HashMap<Uuid, serde_json::Value>>>,
    calls: Arc<Mutex<HashMap<Uuid, serde_json::Value>>>,
}

impl MockDatabase {
    #[must_use]
    pub fn new() -> Self {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy("postgres://postgres:postgres@localhost/postgres")
            .ok();

        Self {
            pool,
            users: Arc::new(Mutex::new(HashMap::new())),
            conversations: Arc::new(Mutex::new(HashMap::new())),
            calls: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn pool(&self) -> sqlx::PgPool {
        self.pool.clone().unwrap_or_else(|| {
            sqlx::postgres::PgPoolOptions::new()
                .max_connections(1)
                .connect_lazy("postgres://postgres:postgres@localhost/postgres")
                .expect("failed to create lazy postgres pool for mock database")
        })
    }

    pub fn put_user(&self, user_id: Uuid, value: serde_json::Value) {
        self.users
            .lock()
            .expect("mock database users mutex poisoned")
            .insert(user_id, value);
    }

    pub fn get_user(&self, user_id: &Uuid) -> Option<serde_json::Value> {
        self.users
            .lock()
            .expect("mock database users mutex poisoned")
            .get(user_id)
            .cloned()
    }

    pub fn put_conversation(&self, conversation_id: Uuid, value: serde_json::Value) {
        self.conversations
            .lock()
            .expect("mock database conversations mutex poisoned")
            .insert(conversation_id, value);
    }

    pub fn get_conversation(&self, conversation_id: &Uuid) -> Option<serde_json::Value> {
        self.conversations
            .lock()
            .expect("mock database conversations mutex poisoned")
            .get(conversation_id)
            .cloned()
    }

    pub fn put_call(&self, call_id: Uuid, value: serde_json::Value) {
        self.calls
            .lock()
            .expect("mock database calls mutex poisoned")
            .insert(call_id, value);
    }

    pub fn get_call(&self, call_id: &Uuid) -> Option<serde_json::Value> {
        self.calls
            .lock()
            .expect("mock database calls mutex poisoned")
            .get(call_id)
            .cloned()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use uuid::Uuid;

    use super::MockDatabase;

    #[tokio::test]
    async fn mock_database_exposes_lazy_pool() {
        let db = MockDatabase::new();
        let pool = db.pool();
        assert_eq!(pool.size(), 0);
    }

    #[tokio::test]
    async fn mock_database_stores_entities_in_memory() {
        let db = MockDatabase::new();
        let user_id = Uuid::now_v7();
        let conversation_id = Uuid::now_v7();
        let call_id = Uuid::now_v7();

        db.put_user(user_id, json!({"name": "alice"}));
        db.put_conversation(conversation_id, json!({"topic": "group"}));
        db.put_call(call_id, json!({"status": "initiated"}));

        assert_eq!(db.get_user(&user_id), Some(json!({"name": "alice"})));
        assert_eq!(db.get_conversation(&conversation_id), Some(json!({"topic": "group"})));
        assert_eq!(db.get_call(&call_id), Some(json!({"status": "initiated"})));
    }
}
