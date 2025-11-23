use mongodb::Database;
use mongodb::bson::oid::ObjectId;
use mongodb::bson::doc;
use mongodb::error::Result as MongoResult;
use futures::stream::TryStreamExt;

use crate::models::friend_model::{Friend, PopulatedFriendShip};

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

    pub async fn find_one(&self, user_id_a: &ObjectId, user_id_b: &ObjectId) -> MongoResult<Option<Friend>> {
        self.collection()
            .find_one(
                mongodb::bson::doc! {
                    "$or": [
                        { "user_a_id": user_id_a, "user_b_id": user_id_b },
                        { "user_a_id": user_id_b, "user_b_id": user_id_a },
                    ]
                }
            )
            .await
    }

    pub async fn create(&self, friend: &Friend) -> MongoResult<()> {
        self.collection().insert_one(friend).await?;
        Ok(())
    }

    pub async fn find_friends_of_user(&self, user_id: &ObjectId) -> MongoResult<Vec<Friend>> {
        let mut cursor = self.collection()
            .find(
                mongodb::bson::doc! {
                    "$or": [
                        { "user_a_id": user_id },
                        { "user_b_id": user_id },
                    ]
                }
            )
            .await?;

        let mut friends = Vec::new();
        while let Some(friend) = cursor.try_next().await? {
            friends.push(friend);
        }
        Ok(friends)
    }

    pub async fn get_friendships(
        &self,
        user_id: &ObjectId,
    ) -> MongoResult<Vec<PopulatedFriendShip>> {
        let pipeline = vec! [
            doc! { "$match": {
                "$or": {
                    "user_a_id": user_id,
                    "user_b_id": user_id,
                }
            }},

            doc! { "$lookup": {
                "from": "users",
                "localField": "user_a_id",
                "foreignField": "_id",
                "as": "user_a"
            }},
            doc! { "$unwind": "$user_a" },

            doc! { "$lookup": {
                "from": "users",
                "localField": "user_b_id",
                "foreignField": "_id",
                "as": "user_b"
            }},
            doc! { "$unwind": "$user_b" },

            doc! { "$project": {
                "_id": 1,
                "user_a": {
                    "_id": "$user_a._id",
                    "fullname": "$user_a.fullname",
                    "avatar_url": "$user_a.avatar_url",
                },
                "user_b": {
                    "_id": "$user_b._id",
                    "fullname": "$user_b.fullname",
                    "avatar_url": "$user_b.avatar_url",
                },
            }},
        ];

        let mut cursor = self.collection()
            .aggregate(pipeline)
            .with_type::<PopulatedFriendShip>()
            .await?;

        let mut friendships = Vec::new();
        while let Some(friendship) = cursor.try_next().await? {
            friendships.push(friendship);
        }
        Ok(friendships)
    }
}