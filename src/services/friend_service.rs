use mongodb::Database;
use mongodb::bson::oid::ObjectId;
use mongodb::error::Result as MongoResult;
use mongodb::bson::DateTime as BsonDateTime;

pub struct FriendService {
    pub db: Database,
}

impl FriendService {
    fn collection(&self) -> mongodb::Collection<crate::models::friend_model::Friend> {
        self.db
            .collection::<crate::models::friend_model::Friend>("friends")
    }

    pub async fn init_indexes(&self) -> MongoResult<()> {
        self.collection()
            .create_index(
                mongodb::IndexModel::builder()
                    .keys(mongodb::bson::doc! { "user_a_id": 1, "user_b_id": 1 })
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

    pub async fn find_one(&self, user_id_a: ObjectId, user_id_b: ObjectId) -> MongoResult<bool> {
        
    }
}