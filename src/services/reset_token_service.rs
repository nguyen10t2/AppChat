use mongodb::{Collection, Database, bson::doc};
use mongodb::bson::DateTime as BsonDateTime;


use crate::models::reset_token::ResetToken;

pub struct ResetTokenService {
    pub db: Database,
}

#[allow(dead_code)]
impl ResetTokenService {
    fn collection(&self) -> Collection<ResetToken> {
        self.db.collection::<ResetToken>("reset_tokens")
    }

    pub async fn init_indexes(&self) -> mongodb::error::Result<()> {
        self.collection()
            .create_index(
                mongodb::IndexModel::builder()
                    .keys(mongodb::bson::doc! { "email": 1 })
                    .options(
                        mongodb::options::IndexOptions::builder()
                            .unique(true)
                            .build(),
                    )
                    .build(),
            )
            .await?;

        Ok(())
    }

    pub async fn create_reset_token(
        &self,
        email: &str,
        token: &str,
        expires_at: mongodb::bson::DateTime,
    ) -> mongodb::error::Result<ResetToken> {
        let now = mongodb::bson::DateTime::from_system_time(chrono::Utc::now().into());
        let mut reset_token = ResetToken {
            id: None,
            email: email.to_string(),
            token: token.to_string(),
            expires_at,
            created_at: now,
            updated_at: now,
        };
        let insert_result = self.collection().insert_one(&reset_token).await?;
        reset_token.id = Some(insert_result.inserted_id.as_object_id().ok_or_else(|| {
            mongodb::error::Error::custom("Lỗi khi lấy ID đã chèn cho ResetToken")
        })?);
        Ok(reset_token)
    }

    pub async fn delete_one(&self, email: &str) -> mongodb::error::Result<()> {
        //  expires_at < NOW() - INTERVAL '1 day'
        let fillter = doc! {
            "$or": [
                { "email": email },
                { "expires_at": doc! { "$lt": BsonDateTime::from_system_time(chrono::Utc::now().into()) } }
            ]
        };
        self.collection().delete_one(fillter).await?;
        Ok(())
    }
}
