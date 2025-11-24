use mongodb::{Database, bson::{doc, oid::ObjectId}};
use crate::models::conversation_model::{Conversation, ConversationType};
use mongodb::error::Result as MongoResult;

pub struct ConversationService {
    pub db: Database,
}

impl ConversationService {
    fn collection(&self) -> mongodb::Collection<Conversation> {
        self.db.collection::<Conversation>("conversations")
    }

    pub async fn init_indexes(&self) -> MongoResult<()> {
        self.collection()
            .create_index(
                mongodb::IndexModel::builder()
                    .keys(mongodb::bson::doc! { "participant_ids.user_id": 1, "last_message_at": -1 })
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

    pub async fn create(&self, conversation: &Conversation) -> MongoResult<mongodb::bson::oid::ObjectId> {
        let result = self.collection().insert_one(conversation).await?;
        Ok(result
            .inserted_id
            .as_object_id()
            .expect("Failed to get inserted_id as ObjectId")
            .to_owned())
    }

    pub async fn find_conversation_by_id(&self, conversation_id: &mongodb::bson::oid::ObjectId) -> MongoResult<Option<Conversation>> {
        self.collection()
            .find_one(
                mongodb::bson::doc! { "_id": conversation_id },
            )
            .await
    }

    pub async fn update(&self, conversation: &Conversation) -> MongoResult<()> {
        self.collection()
            .replace_one(
                mongodb::bson::doc! { "_id": &conversation.id },
                conversation,
            )
            .await?;
        Ok(())
    }

    pub async fn find_with_participant(
        &self,
        user_id: &ObjectId,
        participant_id: &ObjectId,
    ) -> MongoResult<Option<Conversation>> {
        let fillter = doc! {
            "_type": ConversationType::Direct,
            "participant_ids.user_id": {
                "$all": [user_id, participant_id]
            }
        };
        self.collection().find_one(fillter).await
    }
}