use core::hash;

use mongodb::bson::DateTime as BsonDateTime;
use rand::Rng;
use tokio::task;

use crate::libs::hash::hash_password;

pub fn generate_otp() -> String {
    let mut rng = rand::rng();
    (0..6)
        .map(|_| rng.random_range(0..10).to_string())
        .collect()
}

pub struct OtpCode {
    pub plain_otp: String,
    pub hashed_otp: String,
    pub expires_at: BsonDateTime,
}

const OTP_TTL: i64 = 10 * 60;

impl OtpCode {
    pub async fn new() -> Self {
        let plain_otp = generate_otp();
        let hashed_otp = {
            let otp_clone = plain_otp.clone();
            task::spawn_blocking(move || hash_password(&otp_clone).unwrap())
                .await
                .unwrap()
        };
        let expires_at = BsonDateTime::from_system_time(
            (chrono::Utc::now() + chrono::Duration::seconds(OTP_TTL)).into(),
        );
        OtpCode {
            plain_otp,
            hashed_otp,
            expires_at,
        }
    }
}