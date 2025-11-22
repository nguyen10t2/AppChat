use mongodb::error::Result as MongoResult;
use mongodb::Database;

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
            .keys(mongodb::bson::doc! { "from": 1, "to_user": 1 })
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
}