use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use deadpool_redis::{CreatePoolError, PoolError, redis::RedisError};
use std::borrow::Cow;
use std::sync::LazyLock;

use crate::api::messages;

static INCLUDE_ERROR_BODY_META: LazyLock<bool> = LazyLock::new(|| {
    std::env::var("APP_ERROR_BODY_META")
        .map(|value| {
            let normalized = value.trim().to_ascii_lowercase();
            matches!(normalized.as_str(), "1" | "true" | "yes" | "on")
        })
        .unwrap_or(false)
});

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Yêu cầu không hợp lệ: {0}")]
    BadRequest(Cow<'static, str>),
    #[error("Yêu cầu không hợp lệ (key): {0:?}")]
    BadRequestKey(messages::i18n::Key),
    #[error("Không được cấp quyền: {0}")]
    Unauthorized(Cow<'static, str>),
    #[error("Không được cấp quyền (key): {0:?}")]
    UnauthorizedKey(messages::i18n::Key),
    #[error("Bị từ chối truy cập: {0}")]
    Forbidden(Cow<'static, str>),
    #[error("Bị từ chối truy cập (key): {0:?}")]
    ForbiddenKey(messages::i18n::Key),
    #[error("Không tìm thấy dữ liệu: {0}")]
    NotFound(Cow<'static, str>),
    #[error("Không tìm thấy dữ liệu (key): {0:?}")]
    NotFoundKey(messages::i18n::Key),
    #[error("Xung đột dữ liệu: {0}")]
    Conflict(Cow<'static, str>),
    #[error("Lỗi hệ thống nội bộ")]
    InternalServer,
    #[error("IO Error")]
    IOError(#[from] std::io::Error),
    // jwt errors
    #[error("JWT Error")]
    JwtError(#[from] jsonwebtoken::errors::Error),
    // argon2 errors
    #[error("Hash Error")]
    HashError(#[from] argon2::password_hash::Error),
    // sqlx errors
    #[error("Database Error : {0}")]
    DatabaseError(Cow<'static, str>),
    // serde errors
    #[error("JSON Serialization/Deserialization Error")]
    JsonError(#[from] serde_json::Error),
    // redis errors
    #[error(transparent)]
    PoolInit(#[from] CreatePoolError),
    #[error("Redis pool error: {0}")]
    PoolGet(#[from] PoolError),
    #[error("Redis error")]
    RedisError(#[from] RedisError),

    #[error("Internal System Error: {0}")]
    InternalError(Cow<'static, str>),
    #[error("Internal System Error (key): {0:?}")]
    InternalErrorKey(messages::i18n::Key),
}

fn localized_message_key(msg: &str) -> Option<messages::i18n::Key> {
    use messages::error as m;
    use messages::i18n::Key;

    match msg {
        m::INVALID_TOKEN => Some(Key::InvalidToken),
        m::TOKEN_INVALID_OR_EXPIRED => Some(Key::TokenInvalidOrExpired),
        m::AUTH_REQUIRED => Some(Key::AuthRequired),
        m::ACCESS_DENIED => Some(Key::AccessDenied),
        m::USER_NOT_FOUND => Some(Key::UserNotFound),
        m::USER_INFO_NOT_FOUND => Some(Key::UserInfoNotFound),
        m::UPDATE_EMPTY_PAYLOAD => Some(Key::UpdateEmptyPayload),
        m::FRIEND_RECEIVER_NOT_FOUND => Some(Key::FriendReceiverNotFound),
        m::FRIEND_REQUEST_NOT_FOUND => Some(Key::FriendRequestNotFound),
        m::FORBIDDEN_ACCEPT_FRIEND_REQUEST => Some(Key::ForbiddenAcceptFriendRequest),
        m::FORBIDDEN_DECLINE_FRIEND_REQUEST => Some(Key::ForbiddenDeclineFriendRequest),
        m::INVALID_CURSOR => Some(Key::InvalidCursor),
        m::INVALID_PAGINATION_CURSOR => Some(Key::InvalidPaginationCursor),
        m::MISSING_RECIPIENT_ID => Some(Key::MissingRecipientId),
        m::NOT_FRIEND_WITH_RECIPIENT => Some(Key::NotFriendWithRecipient),
        m::NOT_CONVERSATION_MEMBER => Some(Key::NotConversationMember),
        m::MISSING_FILE_ATTACHMENT => Some(Key::MissingFileAttachment),
        m::MISSING_FILE_NAME => Some(Key::MissingFileName),
        m::FILE_NOT_FOUND => Some(Key::FileNotFound),
        m::MESSAGE_NOT_FOUND => Some(Key::MessageNotFound),
        m::FILE_DELETE_FORBIDDEN => Some(Key::FileDeleteForbidden),
        m::CALL_NOT_FOUND => Some(Key::CallNotFound),
        m::CONVERSATION_NOT_FOUND => Some(Key::ConversationNotFound),
        m::CONVERSATION_MEMBER_REQUIRED => Some(Key::ConversationMemberRequired),
        m::GROUP_CREATOR_MISSING => Some(Key::GroupCreatorMissing),
        m::GROUP_DATA_ERROR => Some(Key::GroupDataError),
        m::GROUP_NOT_FOUND_OR_INVALID => Some(Key::GroupNotFoundOrInvalid),
        m::ADDED_USER_NOT_FOUND => Some(Key::AddedUserNotFound),
        m::CLOUDINARY_NOT_CONFIGURED => Some(Key::CloudinaryNotConfigured),
        m::SYSTEM_TIMESTAMP_UNAVAILABLE => Some(Key::SystemTimestampUnavailable),
        m::PASSWORD_HASH_FAILED => Some(Key::PasswordHashFailed),
        m::PASSWORD_VERIFY_FAILED => Some(Key::PasswordVerifyFailed),
        m::CONFIG_SECRET_KEY_MISSING => Some(Key::ConfigSecretKeyMissing),
        m::CONFIG_DATABASE_URL_MISSING => Some(Key::ConfigDatabaseUrlMissing),
        m::CONFIG_REDIS_URL_MISSING => Some(Key::ConfigRedisUrlMissing),
        m::MIGRATION_FILES_LOAD_FAILED => Some(Key::MigrationFilesLoadFailed),
        m::DATABASE_SCHEMA_INIT_FAILED => Some(Key::DatabaseSchemaInitFailed),
        m::ALREADY_FRIENDS => Some(Key::AlreadyFriends),
        m::INVALID_CREDENTIALS => Some(Key::InvalidCredentials),
        m::REPLY_TARGET_NOT_FOUND => Some(Key::ReplyTargetNotFound),
        _ => None,
    }
}

#[derive(serde::Serialize)]
pub struct ErrorBody {
    pub code: &'static str,
    pub message: Cow<'static, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<ErrorMeta>,
}

#[derive(serde::Serialize)]
pub struct ErrorMeta {
    pub retryable: bool,
}

fn conflict_message(constraint: Option<&str>) -> Cow<'static, str> {
    let Some(constraint) = constraint else {
        return "Dữ liệu đã tồn tại".into();
    };

    let field = constraint.split('_').nth(1).unwrap_or("Dữ liệu");

    let mut chars = field.chars();
    let field = match chars.next() {
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        None => "Dữ liệu".to_string(),
    };

    format!("{field} đã tồn tại trong hệ thống").into()
}

fn error_code(error: &AppError) -> &'static str {
    match error.status_code() {
        StatusCode::BAD_REQUEST => "bad_request",
        StatusCode::UNAUTHORIZED => "unauthorized",
        StatusCode::FORBIDDEN => "forbidden",
        StatusCode::NOT_FOUND => "not_found",
        StatusCode::CONFLICT => "conflict",
        _ => "internal_error",
    }
}

fn key_message(locale: messages::i18n::Locale, key: messages::i18n::Key) -> Cow<'static, str> {
    messages::i18n::t(locale, key).into()
}

fn build_error_body(error: &AppError, message: Cow<'static, str>) -> ErrorBody {
    let meta = if *INCLUDE_ERROR_BODY_META {
        Some(ErrorMeta {
            retryable: error.status_code().is_server_error(),
        })
    } else {
        None
    };

    ErrorBody {
        code: error_code(error),
        message,
        meta,
    }
}

impl AppError {
    pub fn code(&self) -> &'static str {
        error_code(self)
    }

    pub fn bad_request(msg: impl Into<Cow<'static, str>>) -> Self {
        Self::BadRequest(msg.into())
    }

    pub fn bad_request_key(key: messages::i18n::Key) -> Self {
        Self::BadRequestKey(key)
    }

    pub fn unauthorized(msg: impl Into<Cow<'static, str>>) -> Self {
        Self::Unauthorized(msg.into())
    }

    pub fn unauthorized_key(key: messages::i18n::Key) -> Self {
        Self::UnauthorizedKey(key)
    }

    pub fn forbidden(msg: impl Into<Cow<'static, str>>) -> Self {
        Self::Forbidden(msg.into())
    }

    pub fn forbidden_key(key: messages::i18n::Key) -> Self {
        Self::ForbiddenKey(key)
    }

    pub fn not_found(msg: impl Into<Cow<'static, str>>) -> Self {
        Self::NotFound(msg.into())
    }

    pub fn not_found_key(key: messages::i18n::Key) -> Self {
        Self::NotFoundKey(key)
    }

    pub fn conflict(msg: impl Into<Cow<'static, str>>) -> Self {
        Self::Conflict(msg.into())
    }

    #[allow(dead_code)]
    pub fn internal_server_error() -> Self {
        Self::InternalServer
    }

    pub fn internal_error(msg: impl Into<Cow<'static, str>>) -> Self {
        Self::InternalError(msg.into())
    }

    pub fn internal_error_key(key: messages::i18n::Key) -> Self {
        Self::InternalErrorKey(key)
    }

    pub fn localized_for_locale(&self, locale: messages::i18n::Locale) -> Option<Self> {
        use messages::i18n::t;

        let localized = match self {
            AppError::BadRequest(msg) => {
                if let Some(key) = localized_message_key(msg.as_ref()) {
                    AppError::BadRequest(t(locale, key).into())
                } else {
                    AppError::BadRequest(msg.clone())
                }
            }
            AppError::BadRequestKey(key) => AppError::BadRequest(t(locale, *key).into()),
            AppError::Unauthorized(msg) => {
                if let Some(key) = localized_message_key(msg.as_ref()) {
                    AppError::Unauthorized(t(locale, key).into())
                } else {
                    AppError::Unauthorized(msg.clone())
                }
            }
            AppError::UnauthorizedKey(key) => AppError::Unauthorized(t(locale, *key).into()),
            AppError::Forbidden(msg) => {
                if let Some(key) = localized_message_key(msg.as_ref()) {
                    AppError::Forbidden(t(locale, key).into())
                } else {
                    AppError::Forbidden(msg.clone())
                }
            }
            AppError::ForbiddenKey(key) => AppError::Forbidden(t(locale, *key).into()),
            AppError::NotFound(msg) => {
                if let Some(key) = localized_message_key(msg.as_ref()) {
                    AppError::NotFound(t(locale, key).into())
                } else {
                    AppError::NotFound(msg.clone())
                }
            }
            AppError::NotFoundKey(key) => AppError::NotFound(t(locale, *key).into()),
            AppError::Conflict(msg) => AppError::Conflict(msg.clone()),
            AppError::InternalErrorKey(key) => AppError::InternalError(t(locale, *key).into()),
            _ => return None,
        };

        Some(localized)
    }
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::BadRequest(_) | AppError::BadRequestKey(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized(_) | AppError::UnauthorizedKey(_) => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) | AppError::ForbiddenKey(_) => StatusCode::FORBIDDEN,
            AppError::NotFound(_) | AppError::NotFoundKey(_) => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let mut res = HttpResponse::build(self.status_code());

        res.insert_header(("Access-Control-Allow-Credentials", "true"));
        res.insert_header(("x-error-code", self.code()));

        match self {
            AppError::NotFound(msg)
            | AppError::Conflict(msg)
            | AppError::Unauthorized(msg)
            | AppError::BadRequest(msg)
            | AppError::Forbidden(msg) => res.json(build_error_body(self, msg.clone())),
            AppError::BadRequestKey(key) => res.json(build_error_body(
                self,
                key_message(messages::i18n::Locale::Vi, *key),
            )),
            AppError::UnauthorizedKey(key) => res.json(build_error_body(
                self,
                key_message(messages::i18n::Locale::Vi, *key),
            )),
            AppError::ForbiddenKey(key) => res.json(build_error_body(
                self,
                key_message(messages::i18n::Locale::Vi, *key),
            )),
            AppError::NotFoundKey(key) => res.json(build_error_body(
                self,
                key_message(messages::i18n::Locale::Vi, *key),
            )),
            AppError::InternalErrorKey(key) => res.json(build_error_body(
                self,
                key_message(messages::i18n::Locale::Vi, *key),
            )),
            _ => res.json(build_error_body(self, messages::error::INTERNAL_SERVER.into())),
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        tracing::error!("{:?}", err);
        if let sqlx::Error::Database(db_err) = &err {
            match db_err.code().as_deref() {
                Some("23505") => {
                    return AppError::Conflict(conflict_message(db_err.constraint()));
                }
                Some("42P01") => {
                    return AppError::NotFound("Không tìm thấy dữ liệu".into());
                }
                _ => {
                    tracing::error!("Unhandled DB error: {:?}", db_err);
                    return AppError::DatabaseError(db_err.message().to_string().into());
                }
            }
        }
        AppError::InternalError(err.to_string().into())
    }
}

pub type Error = AppError;
pub type SystemError = AppError;
