use actix_web::{
    HttpRequest,
    cookie::{self, Cookie, time},
    delete, get, patch, post, web,
};
use uuid::Uuid;

use crate::modules::user::{model, service::UserService};
use crate::modules::websocket::presence::{PresenceInfo, PresenceService};
use crate::{ENV, middlewares::get_extensions};
use crate::{
    api::{error, success},
    utils::{ValidatedJson, ValidatedQuery},
};
use crate::{
    modules::user::{model::SignUpResponse, repository_pg::UserRepositoryPg},
    utils::Claims,
};

pub type UserSvc = UserService<UserRepositoryPg>;

/// Tiện ích lấy thông tin Profile của chính mình
#[get("/profile")]
pub async fn get_profile(
    user_service: web::Data<UserSvc>,
    req: HttpRequest,
) -> Result<success::Success<model::UserResponse>, error::Error> {
    let id = get_extensions::<Claims>(&req)?.sub;
    let user = user_service.get_by_id(id).await?;
    Ok(success::Success::ok(Some(user)).message("Lấy thông tin cá nhân thành công"))
}

/// Tiện ích lấy thông tin chi tiết một User khác qua ID
#[get("/{id:[0-9a-fA-F-]{36}}")]
pub async fn get_user(
    user_service: web::Data<UserSvc>,
    user_id: web::Path<Uuid>,
) -> Result<success::Success<model::UserResponse>, error::Error> {
    let user = user_service.get_by_id(user_id.into_inner()).await?;
    Ok(success::Success::ok(Some(user)).message("Lấy thông tin người dùng thành công"))
}

/// Tiện ích cập nhật thông tin cá nhân hiện tại
#[patch("/{id:[0-9a-fA-F-]{36}}")]
pub async fn update_user(
    user_service: web::Data<UserSvc>,
    user_id: web::Path<Uuid>,
    req: HttpRequest,
    ValidatedJson(user_data): ValidatedJson<model::UpdateUserModel>,
) -> Result<success::Success<()>, error::Error> {
    let auth_user_id = get_extensions::<Claims>(&req)?.sub;
    let target_id = user_id.into_inner();
    if auth_user_id != target_id {
        return Err(error::Error::forbidden(
            "Bạn chỉ có thể cập nhật thông tin của chính mình",
        ));
    }
    user_service.update(target_id, user_data).await?;
    Ok(success::Success::ok(None).message("Cập nhật thông tin thành công"))
}

/// Tiện ích xóa tài khoản cá nhân hiện tại
#[delete("/{id:[0-9a-fA-F-]{36}}")]
pub async fn delete_user(
    user_service: web::Data<UserSvc>,
    user_id: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<success::Success<()>, error::Error> {
    let auth_user_id = get_extensions::<Claims>(&req)?.sub;
    let target_id = user_id.into_inner();
    if auth_user_id != target_id {
        return Err(error::Error::forbidden(
            "Bạn chỉ có thể xóa tài khoản của chính mình",
        ));
    }
    user_service.delete(target_id).await?;
    Ok(success::Success::no_content())
}

/// Đăng ký tài khoản (Register)
#[post("/signup")]
pub async fn sign_up(
    user_service: web::Data<UserSvc>,
    ValidatedJson(user_data): ValidatedJson<model::SignUpModel>,
) -> Result<success::Success<SignUpResponse>, error::Error> {
    let user_id = user_service.sign_up(user_data).await?;
    Ok(
        success::Success::created(Some(SignUpResponse { id: user_id }))
            .message("Đăng ký tài khoản thành công"),
    )
}

/// Đăng nhập (Login)
#[post("/signin")]
pub async fn sign_in(
    user_service: web::Data<UserSvc>,
    ValidatedJson(user_data): ValidatedJson<model::SignInModel>,
) -> Result<success::Success<model::SignInResponse>, error::Error> {
    let (access_token, refresh_token) = user_service.sign_in(user_data).await?;
    let response = model::SignInResponse { access_token };
    let refresh_cookie = Cookie::build("refresh_token", refresh_token)
        .path("/")
        .http_only(true)
        .same_site(cookie::SameSite::Strict)
        .secure(ENV.cookie_secure)
        .max_age(time::Duration::seconds(ENV.refresh_token_expiration as i64))
        .finish();

    Ok(success::Success::ok(Some(response))
        .message("Đăng nhập thành công")
        .cookies(vec![refresh_cookie]))
}

/// Đăng xuất (Gỡ bỏ refresh token cookie)
#[get("/signout")]
pub async fn sign_out(
    user_service: web::Data<UserSvc>,
    req: HttpRequest,
) -> Result<success::Success<()>, error::Error> {
    let refresh_token = req.cookie("refresh_token").map(|c| c.value().to_string());
    user_service.sign_out(refresh_token).await?;
    let refresh_cookie = Cookie::build("refresh_token", "")
        .path("/")
        .http_only(true)
        .same_site(cookie::SameSite::Strict)
        .secure(ENV.cookie_secure)
        .max_age(time::Duration::seconds(0))
        .expires(time::OffsetDateTime::UNIX_EPOCH)
        .finish();

    Ok(success::Success::no_content().cookies(vec![refresh_cookie]))
}

/// Lấy Access Token mới thông qua Refresh Token cookie
#[post("/refresh")]
pub async fn refresh(
    user_service: web::Data<UserSvc>,
    req: HttpRequest,
) -> Result<success::Success<model::SignInResponse>, error::Error> {
    let refresh_token = req.cookie("refresh_token").map(|c| c.value().to_string());
    let (access_token, refresh_token) = user_service.refresh(refresh_token).await?;
    let response = model::SignInResponse { access_token };
    let refresh_cookie = Cookie::build("refresh_token", refresh_token)
        .path("/")
        .http_only(true)
        .same_site(cookie::SameSite::Strict)
        .secure(ENV.cookie_secure)
        .max_age(time::Duration::seconds(ENV.refresh_token_expiration as i64))
        .finish();
    Ok(success::Success::ok(Some(response))
        .message("Làm mới phiên truy cập thành công")
        .cookies(vec![refresh_cookie]))
}

/// Tính năng tìm kiếm Users
#[get("/search")]
pub async fn search_users(
    user_service: web::Data<UserSvc>,
    ValidatedQuery(query): ValidatedQuery<model::UserSearchQuery>,
) -> Result<success::Success<Vec<model::UserResponse>>, error::Error> {
    let users = user_service
        .search_users(&query.q, query.limit.unwrap_or(10))
        .await?;
    Ok(success::Success::ok(Some(users)).message("Tìm kiếm người dùng thành công"))
}

/// Batch query presence status cho nhiều users
///
/// POST /users/presence
/// Body: { "user_ids": ["uuid1", "uuid2", ...] }
///
/// Response: [{ "user_id": "...", "is_online": true, "last_seen": null }, ...]
#[post("/presence")]
pub async fn get_presence(
    presence_service: web::Data<PresenceService>,
    body: web::Json<model::PresenceQuery>,
) -> Result<success::Success<Vec<PresenceInfo>>, error::Error> {
    if body.user_ids.is_empty() {
        return Ok(success::Success::ok(Some(vec![])));
    }

    // Giới hạn số lượng users per request để tránh abuse
    if body.user_ids.len() > 200 {
        return Err(error::Error::bad_request(
            "Tối đa 200 user IDs cho mỗi lần gọi API",
        ));
    }

    let presences = presence_service
        .get_online_status_batch(&body.user_ids)
        .await?;
    Ok(success::Success::ok(Some(presences)))
}
