use actix_web::{HttpRequest, web};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    api::{error, messages, success},
    middlewares::get_extensions,
    modules::{
        call::{
            model::{
                CallHistoryQuery, CallHistoryResponse, InitiateCallRequest, InitiateCallResponse,
                RespondCallRequest,
            },
            repository_pg::{CallParticipantPgRepository, CallPgRepository},
            service::CallService,
        },
        user::{repository::UserRepository, repository_pg::UserRepositoryPg},
    },
    utils::{Claims, ValidatedJson},
};

pub type CallSvc = CallService<CallPgRepository, CallParticipantPgRepository>;

pub struct CallHandler {
    call_service: Arc<CallSvc>,
    user_repo: Arc<UserRepositoryPg>,
}

impl CallHandler {
    pub fn new(call_service: Arc<CallSvc>, user_repo: Arc<UserRepositoryPg>) -> Self {
        Self {
            call_service,
            user_repo,
        }
    }
}

fn current_user_id(req: &HttpRequest) -> Result<Uuid, error::Error> {
    Ok(get_extensions::<Claims>(req)?.sub)
}

fn path_call_id(call_id: web::Path<Uuid>) -> Uuid {
    *call_id
}

fn parse_history_cursor(cursor: Option<&str>) -> Result<Option<DateTime<Utc>>, error::Error> {
    cursor
        .map(DateTime::parse_from_rfc3339)
        .transpose()
        .map_err(|_| error::Error::bad_request_key(messages::i18n::Key::InvalidCursor))
        .map(|value| value.map(|dt| dt.with_timezone(&Utc)))
}

async fn load_call_user_profile(
    user_repo: &UserRepositoryPg,
    user_id: Uuid,
) -> Result<(String, Option<String>), error::Error> {
    let user = user_repo
        .find_by_id(&user_id)
        .await?
        .ok_or_else(|| error::Error::not_found_key(messages::i18n::Key::UserNotFound))?;

    Ok((user.display_name, user.avatar_url))
}

pub async fn initiate_call(
    call_handler: web::Data<Arc<CallHandler>>,
    ValidatedJson(body): ValidatedJson<InitiateCallRequest>,
    req: HttpRequest,
) -> Result<success::Success<InitiateCallResponse>, error::Error> {
    let user_id = current_user_id(&req)?;
    let (display_name, avatar_url) = load_call_user_profile(call_handler.user_repo.as_ref(), user_id).await?;

    let result = call_handler
        .call_service
        .initiate_call(user_id, body, display_name, avatar_url)
        .await?;

    Ok(success::Success::ok(Some(result)).message("Khởi tạo cuộc gọi thành công"))
}

pub async fn respond_call(
    call_handler: web::Data<Arc<CallHandler>>,
    call_id: web::Path<Uuid>,
    ValidatedJson(body): ValidatedJson<RespondCallRequest>,
    req: HttpRequest,
) -> Result<success::Success<()>, error::Error> {
    let user_id = current_user_id(&req)?;
    let call_id = path_call_id(call_id);

    call_handler
        .call_service
        .respond_call(user_id, call_id, body)
        .await?;

    Ok(success::Success::ok(None).message("Phản hồi cuộc gọi thành công"))
}

pub async fn cancel_call(
    call_handler: web::Data<Arc<CallHandler>>,
    call_id: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<success::Success<()>, error::Error> {
    let user_id = current_user_id(&req)?;
    let call_id = path_call_id(call_id);

    call_handler.call_service.cancel_call(user_id, call_id).await?;

    Ok(success::Success::ok(None).message("Hủy cuộc gọi thành công"))
}

pub async fn end_call(
    call_handler: web::Data<Arc<CallHandler>>,
    call_id: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<success::Success<()>, error::Error> {
    let user_id = current_user_id(&req)?;
    let call_id = path_call_id(call_id);

    call_handler.call_service.end_call(user_id, call_id).await?;

    Ok(success::Success::ok(None).message("Kết thúc cuộc gọi thành công"))
}

pub async fn get_call_history(
    call_handler: web::Data<Arc<CallHandler>>,
    query: web::Query<CallHistoryQuery>,
    req: HttpRequest,
) -> Result<success::Success<CallHistoryResponse>, error::Error> {
    let user_id = current_user_id(&req)?;
    let cursor = parse_history_cursor(query.cursor.as_deref())?;

    let data = call_handler
        .call_service
        .get_call_history(user_id, query.limit, cursor)
        .await?;

    Ok(success::Success::ok(Some(data)).message("Lấy lịch sử cuộc gọi thành công"))
}
