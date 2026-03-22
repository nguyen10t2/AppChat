use actix_web::{HttpRequest, delete, get, patch, post, web};
use uuid::Uuid;

use crate::{
    api::{error, messages, success},
    middlewares::get_extensions,
    modules::{
        conversation::{
            model::{
                AddMemberRequest, ConversationDetail, MessageQueryRequest, NewConversation,
                UpdateGroupRequest,
            },
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

fn current_user_id(req: &HttpRequest) -> Result<Uuid, error::Error> {
    Ok(get_extensions::<Claims>(req)?.sub)
}

fn path_uuid(path: web::Path<Uuid>) -> Uuid {
    *path
}

async fn ensure_conversation_member(
    conversation_svc: &ConversationSvc,
    conversation_id: Uuid,
    user_id: Uuid,
) -> Result<(), error::Error> {
    let (_, is_member) = conversation_svc
        .get_conversation_and_check_membership(conversation_id, user_id)
        .await?;

    if !is_member {
        return Err(error::Error::forbidden_key(
            messages::i18n::Key::NotConversationMember,
        ));
    }

    Ok(())
}

async fn ensure_can_create_conversation_membership(
    friend_svc: &FriendSvc,
    creator_id: Uuid,
    member_ids: &[Uuid],
) -> Result<(), error::Error> {
    for &member_id in member_ids {
        if member_id != creator_id
            && !friend_svc
                .is_friend(creator_id, member_id)
                .await
                .unwrap_or(false)
        {
            return Err(error::Error::forbidden_key(
                messages::i18n::Key::NotFriendsWithAllMembers,
            ));
        }
    }

    Ok(())
}

/// Lấy danh sách toàn bộ các cuộc trò chuyện của User
#[get("")]
pub async fn get_conversations(
    conversation_svc: web::Data<ConversationSvc>,
    req: HttpRequest,
) -> Result<success::Success<Vec<ConversationDetail>>, error::Error> {
    let user_id = current_user_id(&req)?;

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
    let user_id = current_user_id(&req)?;
    let conversation_id = path_uuid(conversation_id);

    ensure_conversation_member(conversation_svc.get_ref(), conversation_id, user_id).await?;

    let (messages, cursor) = conversation_svc
        .get_message(conversation_id, query.limit, query.cursor.clone())
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
    let user_id = current_user_id(&req)?;

    ensure_can_create_conversation_membership(friend_svc.get_ref(), user_id, &body.member_ids)
        .await?;

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
    let user_id = current_user_id(&req)?;
    let conversation_id = path_uuid(conversation_id);

    conversation_svc.mark_as_seen(conversation_id, user_id).await?;

    Ok(success::Success::ok(Some("Đã đánh dấu đã xem".to_string()))
        .message("Đánh dấu tin nhắn đã xem thành công"))
}

/// Cập nhật thông tin nhóm
#[patch("/{conversation_id}/group")]
pub async fn update_group(
    conversation_svc: web::Data<ConversationSvc>,
    conversation_id: web::Path<Uuid>,
    ValidatedJson(body): ValidatedJson<UpdateGroupRequest>,
    req: HttpRequest,
) -> Result<success::Success<()>, error::Error> {
    let user_id = current_user_id(&req)?;
    let conversation_id = path_uuid(conversation_id);

    conversation_svc
        .update_group_info(conversation_id, user_id, body.name, body.avatar_url)
        .await?;

    Ok(success::Success::ok(None).message("Cập nhật thông tin nhóm thành công"))
}

/// Thêm thành viên vào nhóm
#[post("/{conversation_id}/members")]
pub async fn add_member(
    conversation_svc: web::Data<ConversationSvc>,
    friend_svc: web::Data<FriendSvc>,
    conversation_id: web::Path<Uuid>,
    ValidatedJson(body): ValidatedJson<AddMemberRequest>,
    req: HttpRequest,
) -> Result<success::Success<()>, error::Error> {
    let user_id = current_user_id(&req)?;
    let conversation_id = path_uuid(conversation_id);

    let is_friend = friend_svc
        .is_friend(user_id, body.user_id)
        .await
        .unwrap_or(false);

    conversation_svc
        .add_member(conversation_id, user_id, body.user_id, is_friend)
        .await?;

    Ok(success::Success::ok(None).message("Thêm thành viên vào nhóm thành công"))
}

/// Xóa thành viên hoặc rời khỏi nhóm
#[delete("/{conversation_id}/members/{target_user_id}")]
pub async fn remove_member(
    conversation_svc: web::Data<ConversationSvc>,
    path: web::Path<(Uuid, Uuid)>,
    req: HttpRequest,
) -> Result<success::Success<()>, error::Error> {
    let user_id = current_user_id(&req)?;
    let (conversation_id, target_user_id) = path.into_inner();

    conversation_svc
        .remove_member(conversation_id, user_id, target_user_id)
        .await?;

    Ok(success::Success::ok(None).message("Thực hiện hành động thành công"))
}
