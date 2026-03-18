use actix_web::{HttpRequest, get, post, web};
use uuid::Uuid;

use crate::{
    api::{error, success},
    middlewares::get_extensions,
    modules::{
        conversation::{
            model::{ConversationDetail, MessageQueryRequest, NewConversation},
            repository_pg::{ConversationPgRepository, ParticipantPgRepository},
            service::ConversationService,
        },
        friend::handle::FriendSvc,
        message::{model::GetMessageResponse, repository_pg::MessageRepositoryPg},
    },
    utils::{Claims, ValidatedJson, ValidatedQuery},
};

pub type ConversationSvc =
    ConversationService<ConversationPgRepository, ParticipantPgRepository, MessageRepositoryPg>;

/// Lấy danh sách toàn bộ các cuộc trò chuyện của User
#[get("")]
pub async fn get_conversations(
    conversation_svc: web::Data<ConversationSvc>,
    req: HttpRequest,
) -> Result<success::Success<Vec<ConversationDetail>>, error::Error> {
    let user_id = get_extensions::<Claims>(&req)?.sub;

    let conversations = conversation_svc.get_by_user_id(user_id).await?;

    Ok(success::Success::ok(Some(conversations))
        .message("Lấy danh sách cuộc trò chuyện thành công"))
}

/// Lấy danh sách tin nhắn trong một cuộc trò chuyện cụ thể (có phân trang cursor)
#[get("/{conversation_id}/messages")]
pub async fn get_messages(
    conversation_svc: web::Data<ConversationSvc>,
    conversation_id: web::Path<Uuid>,
    req: HttpRequest,
    ValidatedQuery(query): ValidatedQuery<MessageQueryRequest>,
) -> Result<success::Success<GetMessageResponse>, error::Error> {
    let user_id = get_extensions::<Claims>(&req)?.sub;
    let (_, is_member) = conversation_svc
        .get_conversation_and_check_membership(*conversation_id, user_id)
        .await?;

    if !is_member {
        return Err(error::Error::forbidden(
            "Bạn không phải thành viên của cuộc trò chuyện này",
        ));
    }

    let (messages, cursor) = conversation_svc
        .get_message(*conversation_id, query.limit, query.cursor.clone())
        .await?;
    Ok(
        success::Success::ok(Some(GetMessageResponse { messages, cursor }))
            .message("Lấy danh sách tin nhắn thành công"),
    )
}

/// Tạo cuộc trò chuyện mới (Direct hoặc Group)
#[post("")]
pub async fn create_conversation(
    conversation_svc: web::Data<ConversationSvc>,
    friend_svc: web::Data<FriendSvc>,
    ValidatedJson(body): ValidatedJson<NewConversation>,
    req: HttpRequest,
) -> Result<success::Success<Option<ConversationDetail>>, error::Error> {
    let user_id = get_extensions::<Claims>(&req)?.sub;

    for &member_id in &body.member_ids {
        if member_id != user_id
            && !friend_svc
                .is_friend(user_id, member_id)
                .await
                .unwrap_or(false)
            {
                return Err(error::Error::forbidden(
                    "Bạn không phải bạn bè với tất cả các thành viên",
                ));
            }
    }

    let conversation = conversation_svc
        .create_conversation(body._type, body.name, body.member_ids, user_id)
        .await?;

    Ok(success::Success::ok(Some(conversation)).message("Tạo cuộc trò chuyện thành công"))
}

/// Đánh dấu đã xem toàn bộ tin nhắn trong một cuộc trò chuyện
#[post("/{conversation_id}/mark-as-seen")]
pub async fn mark_as_seen(
    conversation_svc: web::Data<ConversationSvc>,
    conversation_id: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<success::Success<String>, error::Error> {
    let user_id = get_extensions::<Claims>(&req)?.sub;

    conversation_svc
        .mark_as_seen(*conversation_id, user_id)
        .await?;

    Ok(success::Success::ok(Some("Đã đánh dấu đã xem".to_string()))
        .message("Đánh dấu tin nhắn đã xem thành công"))
}
