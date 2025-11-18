use mongodb::Database;
use mongodb::bson::{DateTime as BsonDateTime, doc};
use mongodb::error::Result as MongoResult;

use crate::models::otp_model::Otp;

pub struct OtpService {
    pub db: Database,
}

#[allow(dead_code)]
impl OtpService {
    pub fn new(db: Database) -> Self {
        OtpService { db }
    }

    fn collection(&self) -> mongodb::Collection<Otp> {
        self.db.collection::<Otp>("otps")
    }

    pub async fn init_indexes(&self) -> MongoResult<()> {
        let email_created_index = mongodb::IndexModel::builder()
            .keys(doc! { "email": 1, "created_at": -1 })
            .build();

        let ttl_index = mongodb::IndexModel::builder()
            .keys(doc! { "expires_at": 1 })
            .options(
                mongodb::options::IndexOptions::builder()
                    .expire_after(std::time::Duration::from_secs(0))
                    .build(),
            )
            .build();

        self.collection()
            .create_indexes([email_created_index, ttl_index])
            .await?;
        Ok(())
    }

    pub async fn create_otp(
        &self,
        email: &str,
        code: &str,
        expires_at: BsonDateTime,
    ) -> MongoResult<()> {
        let now = BsonDateTime::from_system_time(chrono::Utc::now().into());
        // if conflict, update otp, expires_at, updated_at, is_used = false
        self.collection()
            .update_one(
                doc! { "email": email },
                doc! {
                    "$set": {
                        "code": code,
                        "expires_at": expires_at,
                        "is_used": false,
                        "updated_at": now,
                    },
                    "$setOnInsert": {
                        "created_at": now,
                    },
                },
            )
            .with_options(
                mongodb::options::UpdateOptions::builder()
                    .upsert(true)
                    .build(),
            )
            .await?;
        Ok(())
    }
    
    pub async fn get_otp_record(&self, email: &str) -> MongoResult<Option<Otp>> {
        let filter = doc! {
            "email": email,
            "is_used": false,
        };
        let mut cursor = self
            .collection()
            .find(filter)
            .sort(doc! { "created_at": -1 })
            .limit(1)
            .await?;

        if cursor.advance().await? {
            Ok(Some(cursor.deserialize_current()?))
        } else {
            Ok(None)
        }
    }

    pub async fn get_last_otp(&self, email: &str) -> MongoResult<Option<Otp>> {
        let filter = doc! {
            "email": email,
        };
        let mut cursor = self
            .collection()
            .find(filter)
            .sort(doc! { "created_at": -1 })
            .limit(1)
            .await?;

        if cursor.advance().await? {
            Ok(Some(cursor.deserialize_current()?))
        } else {
            Ok(None)
        }
    }

    pub async fn updated_otp(&self, email: &str) -> MongoResult<bool> {
        let filter = doc! {
            "email": email,
            "is_used": false,
        };
        let update = doc! {
            "$set": {
                "is_used": true,
                "updated_at": BsonDateTime::from_system_time(chrono::Utc::now().into()),
            }
        };
        let result = self.collection().update_one(filter, update).await?;
        Ok(result.modified_count > 0)
    }

    pub async fn resend_count(&self, email: &str) -> MongoResult<u64> {
        let filter = doc! {
            "email": email,
            "is_used": false,
        };
        let count = self.collection().count_documents(filter).await?;
        Ok(count)
    }

    pub async fn delete_otp(&self) -> MongoResult<()> {
        let filter = doc! {
            "expires_at": { "$lt": BsonDateTime::from_system_time(chrono::Utc::now().into()) }
        };
        self.collection().delete_many(filter).await?;
        Ok(())
    }
}
