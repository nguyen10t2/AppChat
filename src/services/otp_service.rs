use mongodb::Database;
use mongodb::bson::{DateTime as BsonDateTime, doc};

pub struct OtpService {
    pub db: Database,
}

impl OtpService {
    pub fn new(db: Database) -> Self {
        OtpService { db }
    }

    fn collection(&self) -> mongodb::Collection<crate::models::otp_model::Otp> {
        self.db.collection::<crate::models::otp_model::Otp>("otps")
    }

    pub async fn init_indexes(&self) -> mongodb::error::Result<()> {
        self.collection()
            .create_index(
                mongodb::IndexModel::builder()
                    .keys(mongodb::bson::doc! { "email": 1, "created_at": -1 })
                    .options(
                        mongodb::options::IndexOptions::builder()
                            .unique(false)
                            .build(),
                    )
                    .build(),
            )
            .await?;

        Ok(())
    }

    pub async fn create_otp(
        &self,
        email: &str,
        code: &str,
        expires_at: BsonDateTime,
    ) -> mongodb::error::Result<crate::models::otp_model::Otp> {
        let now = BsonDateTime::from_system_time(chrono::Utc::now().into());
        let mut otp = crate::models::otp_model::Otp {
            id: None,
            email: email.to_string(),
            code: code.to_string(),
            expires_at,
            is_used: false,
            created_at: now,
            updated_at: now,
        };
        let insert_result = self.collection().insert_one(&otp).await?;
        otp.id = Some(
            insert_result
                .inserted_id
                .as_object_id()
                .ok_or_else(|| mongodb::error::Error::custom("Lỗi khi lấy ID đã chèn cho OTP"))?,
        );
        Ok(otp)
    }

    pub async fn get_otp_record(
        &self,
        email: &str,
    ) -> mongodb::error::Result<Option<crate::models::otp_model::Otp>> {
        let filter = doc! { 
            "email": email,
            "is_used": false,
        };
        let mut cursor = self.collection()
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
}
