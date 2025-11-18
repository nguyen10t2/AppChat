use mongodb::action::Update;
use mongodb::bson::DateTime as BsonDateTime;
use mongodb::error::Result as MongoResult;
use mongodb::options::UpdateOptions;
use mongodb::{Collection, Database, bson::doc};

use crate::models::reset_token_model::ResetToken;

pub struct ResetTokenService {
    pub db: Database,
}

#[allow(dead_code)]
impl ResetTokenService {
    fn collection(&self) -> Collection<ResetToken> {
        self.db.collection::<ResetToken>("reset_tokens")
    }

    pub async fn init_indexes(&self) -> MongoResult<()> {
        let email_index = mongodb::IndexModel::builder()
            .keys(doc! { "email": 1 })
            .options(
                mongodb::options::IndexOptions::builder()
                    .unique(true)
                    .build(),
            )
            .build();
        let ttl_index = mongodb::IndexModel::builder()
            .keys(doc! { "expires_at": 1 })
            .options(
                mongodb::options::IndexOptions::builder()
                    .expire_after(std::time::Duration::from_secs(0))
                    .build(),
            )
            .build();

        self.collection().create_indexes([email_index, ttl_index]).await?;
        Ok(())
    }

    pub async fn create_reset_token(
        &self,
        email: &str,
        token: &str,
        expires_at: BsonDateTime,
    ) -> MongoResult<()> {
        let now = BsonDateTime::from_system_time(chrono::Utc::now().into());
        self.collection()
            .update_one(
                doc! { "email": email },
                doc! {
                    "$set": {
                        "token": token,
                        "expires_at": expires_at,
                        "updated_at": now,
                    },
                    "$setOnInsert": {
                        "created_at": now,
                    },
                }
            )
            .with_options(UpdateOptions::builder().upsert(true).build())
            .await?;
        Ok(())
    }

    pub async fn delete_by_email_or_expired(&self, email: &str) -> MongoResult<u64> {
        //  expires_at < NOW() - INTERVAL '1 day'
        let fillter = doc! {
            "$or": [
                { "email": email },
                { "expires_at": doc! { "$lt": BsonDateTime::from_system_time(chrono::Utc::now().into()) } }
            ]
        };
        let result = self.collection().delete_many(fillter).await?;
        Ok(result.deleted_count)
    }

    pub async fn delete_by_email(&self, email: &str) -> MongoResult<bool> {
        let result = self
            .collection()
            .delete_one(doc! { "email": email })
            .await?;
        Ok(result.deleted_count > 0)
    }

    pub async fn delete_expired(&self) -> MongoResult<u64> {
        let filter = doc! {
            "expires_at": { "$lt": BsonDateTime::from_system_time(chrono::Utc::now().into()) }
        };
        let result = self.collection().delete_many(filter).await?;
        Ok(result.deleted_count)
    }

    pub async fn find_one(&self, email: &str) -> MongoResult<Option<ResetToken>> {
        let filter = doc! { "email": email };
        let reset_token = self.collection().find_one(filter).await?;
        Ok(reset_token)
    }
}
