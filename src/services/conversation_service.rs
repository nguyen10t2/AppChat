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
}