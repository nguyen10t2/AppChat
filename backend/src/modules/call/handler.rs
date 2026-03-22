use actix_web::{HttpRequest, web};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    api::{error, success},
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

pub async fn initiate_call(
    call_handler: web::Data<Arc<CallHandler>>,
    ValidatedJson(body): ValidatedJson<InitiateCallRequest>,
    req: HttpRequest,
) -> Result<success::Success<InitiateCallResponse>, error::Error> {
    let user_id = get_extensions::<Claims>(&req)?.sub;

    let user = call_handler
        .user_repo
        .find_by_id(&user_id)
        .await?
        .ok_or_else(|| error::Error::not_found("Không tìm thấy người dùng"))?;

    let result = call_handler
        .call_service
        .initiate_call(user_id, body, user.display_name, user.avatar_url)
        .await?;

    Ok(success::Success::ok(Some(result)).message("Khởi tạo cuộc gọi thành công"))
}

pub async fn respond_call(
    call_handler: web::Data<Arc<CallHandler>>,
    call_id: web::Path<Uuid>,
    ValidatedJson(body): ValidatedJson<RespondCallRequest>,
    req: HttpRequest,
) -> Result<success::Success<()>, error::Error> {
    let user_id = get_extensions::<Claims>(&req)?.sub;

    call_handler
        .call_service
        .respond_call(user_id, *call_id, body)
        .await?;

    Ok(success::Success::ok(None).message("Phản hồi cuộc gọi thành công"))
}

pub async fn cancel_call(
    call_handler: web::Data<Arc<CallHandler>>,
    call_id: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<success::Success<()>, error::Error> {
    let user_id = get_extensions::<Claims>(&req)?.sub;

    call_handler.call_service.cancel_call(user_id, *call_id).await?;

    Ok(success::Success::ok(None).message("Hủy cuộc gọi thành công"))
}

pub async fn end_call(
    call_handler: web::Data<Arc<CallHandler>>,
    call_id: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<success::Success<()>, error::Error> {
    let user_id = get_extensions::<Claims>(&req)?.sub;

    call_handler.call_service.end_call(user_id, *call_id).await?;

    Ok(success::Success::ok(None).message("Kết thúc cuộc gọi thành công"))
}

pub async fn get_call_history(
    call_handler: web::Data<Arc<CallHandler>>,
    query: web::Query<CallHistoryQuery>,
    req: HttpRequest,
) -> Result<success::Success<CallHistoryResponse>, error::Error> {
    let user_id = get_extensions::<Claims>(&req)?.sub;

    let cursor = query
        .cursor
        .as_deref()
        .map(DateTime::parse_from_rfc3339)
        .transpose()
        .map_err(|_| error::Error::bad_request("Cursor không hợp lệ"))?
        .map(|dt| dt.with_timezone(&Utc));

    let data = call_handler
        .call_service
        .get_call_history(user_id, query.limit, cursor)
        .await?;

    Ok(success::Success::ok(Some(data)).message("Lấy lịch sử cuộc gọi thành công"))
}
