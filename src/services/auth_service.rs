use jsonwebtoken::{
    DecodingKey, EncodingKey, Header, Validation, decode, encode, errors::Error as JwtError,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub user_id: String,
    pub email: String,
    pub exp: i64,
}

#[derive(Clone)]
pub struct AuthService {
    pub secret_key: String,
    pub access_token_ttl: i64,
}

impl AuthService {
    pub async fn create_access_token(
        &self,
        user_id: &str,
        email: &str,
    ) -> Result<String, JwtError> {
        let claims = Claims {
            user_id: user_id.to_string(),
            email: email.to_string(),
            exp: chrono::Utc::now().timestamp() + self.access_token_ttl,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret_key.as_ref()),
        )
    }

    pub async fn create_refresh_token(&self, user_id: &str, email: &str) -> Result<String, JwtError> {
        let claims = Claims {
            user_id: user_id.to_string(),
            email: email.to_string(),
            exp: chrono::Utc::now().timestamp() + (7 * 24 * 60 * 60),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret_key.as_ref()),
        )
    }

    pub async fn verify_token(&self, token: &str) -> Result<Claims, JwtError> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret_key.as_ref()),
            &Validation::default(),
        )
        .map(|data| data.claims)
    }
}
