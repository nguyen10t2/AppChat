use crate::models::conversation_model::Conversation;
use crate::models::message_model::Message;
use crate::models::conversation_model::LastMessage;
use mongodb::bson::oid::ObjectId;

pub async fn update_conversation_after_create_message(
    conversation: &mut Conversation,
    message: &Message,
    sender_id: &ObjectId,
) {
    conversation.seen_by.clear();
    conversation.last_message_at = message.created_at;
    conversation.last_message = Some(LastMessage {
        _id: message.id.clone().unwrap(),
        content: message.content.clone(),
        sender_id: Some(sender_id.clone()),
        created_at: message.created_at,
    });

    conversation.participant_ids.iter().for_each(|p| {
        let member_id = p.user_id;
        let is_sender = &member_id == sender_id;
        let count = conversation.unread_counts.entry(member_id).or_default();
        if !is_sender {
            *count += 1;
        } else {
            *count = 0;
        }
    });
}