use mongodb::Database;
use crate::models::message_model::Message;
use mongodb::error::Result as MongoResult;

pub struct MessageService {
    pub db: Database,
}
impl MessageService {
    fn collection(&self) -> mongodb::Collection<Message> {
        self.db.collection::<Message>("messages")
    }

    pub async fn init_indexes(&self) -> MongoResult<()> {
        self.collection()
            .create_index(
                mongodb::IndexModel::builder()
                    .keys(mongodb::bson::doc! { "coversation_id": 1, "created_at": -1 })
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