use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use deadpool_redis::{CreatePoolError, PoolError, redis::RedisError};
use std::borrow::Cow;

use crate::api::messages;
use crate::ENV;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Yêu cầu không hợp lệ: {0}")]
    BadRequest(Cow<'static, str>),
    #[error("Không được cấp quyền: {0}")]
    Unauthorized(Cow<'static, str>),
    #[error("Bị từ chối truy cập: {0}")]
    Forbidden(Cow<'static, str>),
    #[error("Không tìm thấy dữ liệu: {0}")]
    NotFound(Cow<'static, str>),
    #[error("Xung đột dữ liệu: {0}")]
    Conflict(Cow<'static, str>),
    #[error("Lỗi hệ thống nội bộ")]
    InternalServer,
}

#[derive(serde::Serialize)]
pub struct ErrorBody {
    pub code: &'static str,
    pub message: Cow<'static, str>,
}

fn error_code(error: &Error) -> &'static str {
    match error {
        Error::BadRequest(_) => "bad_request",
        Error::Unauthorized(_) => "unauthorized",
        Error::Forbidden(_) => "forbidden",
        Error::NotFound(_) => "not_found",
        Error::Conflict(_) => "conflict",
        Error::InternalServer => "internal_error",
    }
}

impl Error {
    pub fn bad_request(msg: impl Into<Cow<'static, str>>) -> Self {
        Self::BadRequest(msg.into())
    }

    pub fn unauthorized(msg: impl Into<Cow<'static, str>>) -> Self {
        Self::Unauthorized(msg.into())
    }

    pub fn forbidden(msg: impl Into<Cow<'static, str>>) -> Self {
        Self::Forbidden(msg.into())
    }

    pub fn not_found(msg: impl Into<Cow<'static, str>>) -> Self {
        Self::NotFound(msg.into())
    }

    pub fn conflict(msg: impl Into<Cow<'static, str>>) -> Self {
        Self::Conflict(msg.into())
    }

    #[allow(dead_code)]
    pub fn internal_server_error() -> Self {
        Self::InternalServer
    }
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match *self {
            Error::BadRequest(_) => StatusCode::BAD_REQUEST,
            Error::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Error::Forbidden(_) => StatusCode::FORBIDDEN,
            Error::NotFound(_) => StatusCode::NOT_FOUND,
            Error::Conflict(_) => StatusCode::CONFLICT,
            Error::InternalServer => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let header = ("Access-Control-Allow-Origin", ENV.frontend_url.as_str());
        let mut res = HttpResponse::build(self.status_code());

        res.insert_header(header);
        res.insert_header(("Access-Control-Allow-Credentials", "true"));

        match self {
            // Has Message
            Error::NotFound(msg)
            | Error::Conflict(msg)
            | Error::Unauthorized(msg)
            | Error::BadRequest(msg)
            | Error::Forbidden(msg) => res.json(ErrorBody {
                code: error_code(self),
                message: msg.clone(),
            }),
            // No Message
            Error::InternalServer => res.json(ErrorBody {
                code: error_code(self),
                message: messages::error::INTERNAL_SERVER.into(),
            }),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum SystemError {
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
    // Custom Errors
    #[error("Bad Request: {0}")]
    BadRequest(Cow<'static, str>),
    #[error("Unauthorized: {0}")]
    Unauthorized(Cow<'static, str>),
    #[error("Forbidden: {0}")]
    Forbidden(Cow<'static, str>),
    #[error("Database Not Found: {0}")]
    NotFound(Cow<'static, str>),
    #[error("Database Conflict: {0:?}")]
    Conflict(Option<DbErrorMeta>),
    #[error("Internal System Error: {0}")]
    InternalError(Cow<'static, str>),
}

fn conflict_message(meta: &Option<DbErrorMeta>) -> Cow<'static, str> {
    let Some(m) = meta else {
        return "Dữ liệu đã tồn tại".into();
    };

    let Some(constraint) = &m.constraint else {
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

#[derive(Debug)]
pub struct DbErrorMeta {
    pub code: Option<String>,
    pub constraint: Option<String>,
    pub message: String,
}

impl From<SystemError> for Error {
    fn from(value: SystemError) -> Self {
        match value {
            SystemError::BadRequest(msg) => Error::BadRequest(msg),
            SystemError::Unauthorized(msg) => Error::Unauthorized(msg),
            SystemError::Forbidden(msg) => Error::Forbidden(msg),
            SystemError::NotFound(msg) => Error::NotFound(msg),
            SystemError::Conflict(meta) => Error::Conflict(conflict_message(&meta)),
            _ => {
                tracing::error!("Internal Server Error: {:?}", value);
                Error::InternalServer
            }
        }
    }
}

impl From<sqlx::Error> for SystemError {
    fn from(err: sqlx::Error) -> Self {
        tracing::error!("{:?}", err);
        if let sqlx::Error::Database(db_err) = &err {
            match db_err.code().as_deref() {
                Some("23505") => {
                    return SystemError::Conflict(Some(DbErrorMeta {
                        code: db_err.code().map(|s| s.to_string()),
                        constraint: db_err.constraint().map(|s| s.to_string()),
                        message: db_err.message().to_string(),
                    }));
                }
                Some("42P01") => {
                    return SystemError::NotFound("Không tìm thấy dữ liệu".into());
                }
                _ => {
                    tracing::error!("Unhandled DB error: {:?}", db_err);
                    return SystemError::DatabaseError(db_err.message().to_string().into());
                }
            }
        }
        SystemError::InternalError(err.to_string().into())
    }
}

impl SystemError {
    pub fn bad_request(msg: impl Into<Cow<'static, str>>) -> Self {
        Self::BadRequest(msg.into())
    }

    pub fn not_found(msg: impl Into<Cow<'static, str>>) -> Self {
        Self::NotFound(msg.into())
    }

    pub fn unauthorized(msg: impl Into<Cow<'static, str>>) -> Self {
        Self::Unauthorized(msg.into())
    }

    pub fn forbidden(msg: impl Into<Cow<'static, str>>) -> Self {
        Self::Forbidden(msg.into())
    }

    pub fn internal_error(msg: impl Into<Cow<'static, str>>) -> Self {
        Self::InternalError(msg.into())
    }
}
