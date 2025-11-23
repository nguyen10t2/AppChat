use mongodb::error::Result as MongoResult;
use mongodb::Database;
use mongodb::bson::oid::ObjectId;

use crate::models::friend_request_model::FriendRequest;

pub struct FriendRequestService {
    pub db: Database,
}

impl FriendRequestService {
    fn collection(&self) -> mongodb::Collection<FriendRequest> {
        self.db.collection::<FriendRequest>("friend_requests")
    }

    pub async fn init_indexes(&self) -> MongoResult<()> {
        let index_friend_model = mongodb::IndexModel::builder()
            .keys(mongodb::bson::doc! { "from": 1, "to": 1 })
            .options(
                mongodb::options::IndexOptions::builder()
                    .unique(true)
                    .build(),
            )
            .build();

        let from_index = mongodb::IndexModel::builder()
            .keys(mongodb::bson::doc! { "from": 1 })
            .options(
                mongodb::options::IndexOptions::builder()
                    .unique(false)
                    .build(),
            )
            .build();

        let to_index = mongodb::IndexModel::builder()
            .keys(mongodb::bson::doc! { "to": 1 })
            .options(
                mongodb::options::IndexOptions::builder()
                    .unique(false)
                    .build(),
            )
            .build();

        self.collection()
            .create_indexes([index_friend_model, from_index, to_index])
            .await?;

        Ok(())
    }

    pub async fn find_one(&self, from_user_id: &ObjectId, to_user_id: &ObjectId) -> MongoResult<Option<FriendRequest>> {
        self.collection()
            .find_one(
                mongodb::bson::doc! {
                    "$or": [
                        { "from": from_user_id, "to": to_user_id },
                        { "from": to_user_id, "to": from_user_id },
                    ]
                }
            )
            .await
    }

    pub async fn create(&self, friend_request: &FriendRequest) -> MongoResult<()> {
        self.collection()
            .insert_one(friend_request)
            .await?;
        Ok(())
    }

    pub async fn find_by_id_from_request(&self, request_id: &ObjectId) -> MongoResult<Option<FriendRequest>> {
        self.collection()
            .find_one(
                mongodb::bson::doc! {
                    "from": request_id,
                }
            )
            .await
    }

    pub async fn find_by_id_to_request(&self, request_id: &ObjectId) -> MongoResult<Option<FriendRequest>> {
        self.collection()
            .find_one(
                mongodb::bson::doc! {
                    "to": request_id,
                }
            )
            .await
    }

    pub async fn delete_by_id(&self, request_id: &ObjectId) -> MongoResult<()> {
        self.collection()
            .delete_one(
                mongodb::bson::doc! {
                    "from": request_id,
                }
            )
            .await?;
        Ok(())
    }
}