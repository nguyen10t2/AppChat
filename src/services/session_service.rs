use mongodb::bson::oid::ObjectId;
use mongodb::bson::{DateTime as BsonDateTime, doc};
use mongodb::{Collection, Database, IndexModel};

use crate::models::session_model::Session;

pub struct SessionService {
    pub db: Database,
    pub refresh_token_ttl: i64,
}

impl SessionService {
    fn collection(&self) -> Collection<Session> {
        self.db.collection::<Session>("sessions")
    }

    pub async fn init_indexes(&self) -> mongodb::error::Result<()> {
        let ttl_index = IndexModel::builder()
            .keys(doc! { "expires_at": 1 })
            .options(
                mongodb::options::IndexOptions::builder()
                    .expire_after(std::time::Duration::from_secs(0))
                    .build(),
            )
            .build();

        let token_index = IndexModel::builder()
            .keys(doc! { "refresh_token": 1 })
            .options(
                mongodb::options::IndexOptions::builder()
                    .unique(true) // tuỳ bạn có cần unique không
                    .build(),
            )
            .build();
        
        self.collection().create_indexes([ttl_index, token_index]).await?;
        Ok(())
    }

    pub async fn create_session(
        &self,
        user_id: ObjectId,
        email: String,
        refresh_token: String,
    ) -> mongodb::error::Result<()> {
        let session = Session {
            user_id: Some(user_id),
            email,
            refresh_token,
            expires_at: BsonDateTime::from_system_time(
                chrono::Utc::now()
                    .checked_add_signed(chrono::Duration::seconds(self.refresh_token_ttl))
                    .unwrap()
                    .into(),
            ),
            created_at: BsonDateTime::from_system_time(chrono::Utc::now().into()),
            updated_at: BsonDateTime::from_system_time(chrono::Utc::now().into()),
        };

        self.collection().insert_one(&session).await?;
        Ok(())
    }

    pub async fn delete_session(&self, refresh_token: &str) -> mongodb::error::Result<()> {
        self.collection()
            .delete_one(doc! { "refresh_token": refresh_token })
            .await?;
        Ok(())
    }

    pub async fn find_one(&self, refresh_token: &str) -> mongodb::error::Result<Option<Session>> {
        self.collection()
            .find_one(doc! { "refresh_token": refresh_token })
            .await
    }

    pub async fn cleanup_expired(&self) -> mongodb::error::Result<()> {
        let filter = doc! {
            "expires_at": { "$lt": BsonDateTime::from_system_time(chrono::Utc::now().into()) }
        };
        self.collection().delete_many(filter).await?;
        Ok(())
    }
}
