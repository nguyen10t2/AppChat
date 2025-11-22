pub fn generate_reset_token() -> String {
    use rand::Rng;
    let mut rng = rand::rng();
    let token: String = (0..32)
        .map(|_| {
            let idx = rng.random_range(0..62);
            let c = match idx {
                0..=9 => (b'0' + idx as u8) as char,
                10..=35 => (b'a' + (idx - 10) as u8) as char,
                36..=61 => (b'A' + (idx - 36) as u8) as char,
                _ => unreachable!(),
            };
            c
        })
        .collect();
    token
}

const RESET_TOKEN_TTL_MINUTES: i64 = 30;

pub struct ResetToken {
    pub hashed_token: String,
    pub expires_at: mongodb::bson::DateTime,
}

impl ResetToken {
    pub async fn new() -> Self {
        use crate::libs::hash::hash_password;
        let hashed_token = {
            let token = generate_reset_token();
            tokio::task::spawn_blocking(move || hash_password(&token).unwrap())
                .await
                .unwrap()
        };
        let expires_at = mongodb::bson::DateTime::from_system_time(
            (chrono::Utc::now() + chrono::Duration::minutes(RESET_TOKEN_TTL_MINUTES)).into(),
        );
        ResetToken {
            hashed_token,
            expires_at,
        }
    }
}