use mongodb::Database;
use crate::models::conversation_model::Conversation;
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
        let insert_result = self.collection().insert_one(conversation).await?;
        Ok(insert_result
            .inserted_id
            .as_object_id()
            .expect("Failed to get inserted_id as ObjectId"))
    }

    pub async fn find_conversation_by_id(&self, conversation_id: &mongodb::bson::oid::ObjectId) -> MongoResult<Option<Conversation>> {
        self.collection()
            .find_one(
                mongodb::bson::doc! { "_id": conversation_id },
            )
            .await
    }
}