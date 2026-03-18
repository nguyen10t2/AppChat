use actix_web::{HttpRequest, delete, patch, post, web};
use uuid::Uuid;

use crate::{
    api::{error, success},
    middlewares::get_extensions,
    modules::{
        conversation::{
            handle::ConversationSvc,
            repository_pg::{
                ConversationPgRepository, LastMessagePgRepository, ParticipantPgRepository,
            },
            schema::ConversationEntity,
        },
        friend::handle::FriendSvc,
        message::{
            model::{
                EditMessageRequest, SendDirectMessage, SendDirectMessagePayload,
                SendGroupMessage,
            },
            repository_pg::MessageRepositoryPg,
            schema::MessageEntity,
            service::MessageService,
        },
    },
    utils::{Claims, ValidatedJson},
};

type MessageSvc = MessageService<
    MessageRepositoryPg,
    ConversationPgRepository,
    ParticipantPgRepository,
    LastMessagePgRepository,
>;

/// Gửi tin nhắn cá nhân
#[post("/")]
pub async fn send_direct_message(
    message_service: web::Data<MessageSvc>,
    friend_svc: web::Data<FriendSvc>,
    body: web::Json<SendDirectMessage>,
    req: HttpRequest,
) -> Result<success::Success<MessageEntity>, error::Error> {
    let user_id = get_extensions::<Claims>(&req)?.sub;
    let recipient_id = body
        .recipient_id
        .ok_or(error::Error::bad_request("Cần có ID người nhận"))?;

    if !friend_svc
        .is_friend(user_id, recipient_id)
        .await
        .unwrap_or(false)
    {
        return Err(error::Error::forbidden(
            "Bạn không phải bạn bè với người nhận",
        ));
    }

    let message = message_service
        .send_direct_message_payload(
            user_id,
            recipient_id,
            SendDirectMessagePayload {
                conversation_id: body.conversation_id,
                content: body.content.clone(),
                message_type: body._type.clone(),
                file_url: body.file_url.clone(),
                reply_to_id: body.reply_to_id,
            },
        )
        .await?;

    Ok(success::Success::ok(Some(message)).message("Gửi tin nhắn cá nhân thành công"))
}

/// Gửi tin nhắn nhóm
#[post("/")]
pub async fn send_group_message(
    message_service: web::Data<MessageSvc>,
    conversation_svc: web::Data<ConversationSvc>,
    body: web::Json<SendGroupMessage>,
    req: HttpRequest,
) -> Result<success::Success<MessageEntity>, error::Error> {
    let user_id = get_extensions::<Claims>(&req)?.sub;

    let (_, is_member) = conversation_svc
        .get_conversation_and_check_membership(body.conversation_id, user_id)
        .await?;

    if !is_member {
        return Err(error::Error::forbidden(
            "Bạn không phải thành viên của cuộc trò chuyện này",
        ));
    }

    let message = message_service
        .send_group_message_payload(
            user_id,
            body.conversation_id,
            body.content.clone(),
            body._type.clone(),
            body.file_url.clone(),
            body.reply_to_id,
        )
        .await?;

    Ok(success::Success::ok(Some(message)).message("Gửi tin nhắn nhóm thành công"))
}

/// Xóa tin nhắn cục bộ/Server
#[delete("/{message_id}")]
pub async fn delete_message(
    message_service: web::Data<MessageSvc>,
    message_id: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<success::Success<()>, error::Error> {
    let user_id = get_extensions::<Claims>(&req)?.sub;
    message_service.delete_message(*message_id, user_id).await?;
    Ok(success::Success::no_content())
}

/// Chỉnh sửa nội dung tin nhắn
#[patch("/{message_id}")]
pub async fn edit_message(
    message_service: web::Data<MessageSvc>,
    message_id: web::Path<Uuid>,
    ValidatedJson(body): ValidatedJson<EditMessageRequest>,
    req: HttpRequest,
) -> Result<success::Success<MessageEntity>, error::Error> {
    let user_id = get_extensions::<Claims>(&req)?.sub;

    let message = message_service
        .edit_message(*message_id, user_id, body.content)
        .await?;
    Ok(success::Success::ok(Some(message)).message("Chỉnh sửa tin nhắn thành công"))
}
