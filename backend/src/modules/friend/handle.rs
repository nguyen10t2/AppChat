use actix_web::{HttpRequest, delete, get, post, web};
use uuid::Uuid;

use crate::{
    api::{error, success},
    middlewares::get_extensions,
    modules::{
        friend::{
            model::{FriendRequestBody, FriendRequestResponse, FriendResponse},
            repository_pg::FriendRepositoryPg,
            schema::FriendRequestEntity,
            service::FriendService,
        },
        user::repository_pg::UserRepositoryPg,
    },
    utils::Claims,
};

pub type FriendSvc = FriendService<FriendRepositoryPg, UserRepositoryPg>;

fn current_user_id(req: &HttpRequest) -> Result<Uuid, error::Error> {
    Ok(get_extensions::<Claims>(req)?.sub)
}

fn path_id(path: web::Path<Uuid>) -> Uuid {
    *path
}

/// Gửi yêu cầu kết bạn cho người khác
#[post("/requests")]
pub async fn send_friend_request(
    friend_service: web::Data<FriendSvc>,
    body: web::Json<FriendRequestBody>,
    req: HttpRequest,
) -> Result<success::Success<FriendRequestEntity>, error::Error> {
    let sender_id = current_user_id(&req)?;
    let request = friend_service
        .send_friend_request(sender_id, body.recipient_id, body.message.clone())
        .await?;

    Ok(success::Success::created(Some(request)).message("Gửi yêu cầu kết bạn thành công"))
}

/// Chấp nhận một lời mời kết bạn đang chờ
#[post("/requests/{request_id}/accept")]
pub async fn accept_friend_request(
    friend_service: web::Data<FriendSvc>,
    request_id: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<success::Success<FriendResponse>, error::Error> {
    let receiver_id = current_user_id(&req)?;
    let request_id = path_id(request_id);
    let response = friend_service
        .accept_friend_request(receiver_id, request_id)
        .await?;

    Ok(success::Success::ok(Some(response)).message("Chấp nhận yêu cầu kết bạn thành công"))
}

/// Từ chối một lời mời kết bạn đang chờ
#[post("/requests/{request_id}/decline")]
pub async fn decline_friend_request(
    friend_service: web::Data<FriendSvc>,
    request_id: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<success::Success<()>, error::Error> {
    let receiver_id = current_user_id(&req)?;
    let request_id = path_id(request_id);
    friend_service
        .decline_friend_request(receiver_id, request_id)
        .await?;
    Ok(success::Success::no_content())
}

/// Lấy danh sách bạn bè của người dùng hiện tại
#[get("/")]
pub async fn list_friends(
    friend_service: web::Data<FriendSvc>,
    req: HttpRequest,
) -> Result<success::Success<Vec<FriendResponse>>, error::Error> {
    let user_id = current_user_id(&req)?;
    let friends = friend_service.get_friends(user_id).await?;

    Ok(success::Success::ok(Some(friends)).message("Lấy danh sách bạn bè thành công"))
}

/// Lấy danh sách các lời mời kết bạn (gửi và nhận)
#[get("/requests")]
pub async fn list_friend_requests(
    friend_service: web::Data<FriendSvc>,
    req: HttpRequest,
) -> Result<success::Success<Vec<FriendRequestResponse>>, error::Error> {
    let user_id = current_user_id(&req)?;
    let requests = friend_service.get_friend_requests(user_id).await?;

    Ok(success::Success::ok(Some(requests)).message("Lấy danh sách lời mời kết bạn thành công"))
}

/// Gỡ kết bạn với một người dùng khác
#[delete("/{friend_id}")]
pub async fn remove_friend(
    friend_service: web::Data<FriendSvc>,
    friend_id: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<success::Success<()>, error::Error> {
    let user_id = current_user_id(&req)?;
    let friend_id = path_id(friend_id);
    friend_service.remove_friend(user_id, friend_id).await?;
    Ok(success::Success::no_content())
}
