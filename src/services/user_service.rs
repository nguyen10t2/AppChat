use crate::libs::hash::hash_password;
use crate::models::user_model::{User, UserPreview};
use chrono::Utc;
use futures::stream::TryStreamExt;
use mongodb::bson::from_document;
use mongodb::bson::oid::ObjectId as Oid;
use mongodb::bson::{DateTime as BsonDateTime, doc, Document};
use mongodb::error::Result as MongoResult;
use mongodb::{Collection, Database};

#[derive(Clone)]
pub struct UserService {
    pub db: Database,
}

#[allow(dead_code)]
impl UserService {
    fn collection(&self) -> Collection<User> {
        self.db.collection::<User>("users")
    }

    pub async fn init_indexes(&self) -> MongoResult<()> {
        self.collection()
            .create_index(
                mongodb::IndexModel::builder()
                    .keys(doc! { "email": 1 })
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

    pub async fn create_user(
        &self,
        fullname: &str,
        email: &str,
        password: &str,
    ) -> MongoResult<User> {
        let now = BsonDateTime::from_system_time(Utc::now().into());
        let hashed_password = hash_password(password)
            .map_err(|e| mongodb::error::Error::custom(format!("Lỗi khi băm mật khẩu: {}", e)))?;
        let mut user = User {
            id: None,
            fullname: fullname.to_string(),
            email: email.to_string(),
            password: hashed_password,
            avatar_id: None,
            avatar_url: None,
            bio: None,
            phone: None,
            is_active: false,
            created_at: now,
            updated_at: now,
        };

        let insert_result = self.collection().insert_one(&user).await?;
        user.id = Some(
            insert_result
                .inserted_id
                .as_object_id()
                .ok_or_else(|| mongodb::error::Error::custom("Invalid inserted _id"))?,
        );

        Ok(user)
    }

    pub async fn is_exists(&self, email: &str) -> MongoResult<bool> {
        let user = self.collection().find_one(doc! { "email": email }).await?;
        Ok(user.is_some())
    }

    pub async fn find_by_email(&self, email: &str) -> MongoResult<Option<User>> {
        self.collection()
            .find_one(doc! { "email": email, "is_active": true })
            .await
    }

    pub async fn find_by_id(&self, id: &Oid) -> MongoResult<Option<User>> {
        self.collection()
            .find_one(doc! { "_id": id, "is_active": true })
            .await
    }

    pub async fn update_user(&self, email: &str, new_password: &str) -> MongoResult<Option<User>> {
        let hashed_password = hash_password(new_password)
            .map_err(|e| mongodb::error::Error::custom(format!("Lỗi khi băm mật khẩu: {}", e)))?;

        self.collection()
            .find_one_and_update(
                doc! { "email": email },
                doc! {
                    "$set": {
                        "password": hashed_password,
                        "updated_at": BsonDateTime::from_system_time(Utc::now().into())
                    }
                },
            )
            .await
    }

    pub async fn delete_user(&self, id: &Oid) -> MongoResult<bool> {
        let result = self.collection().delete_one(doc! { "_id": id }).await?;
        Ok(result.deleted_count > 0)
    }

    pub async fn activate_user(&self, email: &str) -> MongoResult<bool> {
        let result = self
            .collection()
            .update_one(
                doc! { "email": email },
                doc! {
                    "$set": {
                        "is_active": true,
                        "updated_at": BsonDateTime::from_system_time(Utc::now().into())
                    }
                },
            )
            .await?;
        Ok(result.modified_count > 0)
    }

    pub async fn find_by_id_preview(
        &self,
        id: &Oid,
    ) -> MongoResult<Option<UserPreview>> {
        let filter = doc! { "_id": id, "is_active": true };
        let projection = doc! {
            "_id": 1,
            "fullname": 1,
            "avatar_url": 1,
        };
        self.db.collection::<UserPreview>("users")
            .find_one(filter)
            .projection(projection)
            .await
    }
}