use actix_web::{
    Error, HttpMessage, HttpRequest,
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    http::header::{HeaderName, HeaderValue},
    middleware::Next,
};
use futures_util::{FutureExt, future::LocalBoxFuture};
use std::rc::Rc;
use std::time::Instant;
use uuid::Uuid;

use crate::{
    api::{error, messages},
    app_state::AppState,
    modules::user::schema::UserRole,
    observability::RequestContext,
    utils::Claims,
};

pub async fn request_context<B>(
    req: ServiceRequest,
    next: Next<B>,
) -> Result<ServiceResponse<B>, Error>
where
    B: MessageBody + 'static,
{
    let locale = crate::api::messages::i18n::detect_locale(
        req.headers()
            .get("Accept-Language")
            .and_then(|value| value.to_str().ok()),
    );

    let request_id = req
        .headers()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| Uuid::now_v7().to_string());

    if let Some(app_state) = req.app_data::<actix_web::web::Data<AppState>>() {
        app_state.metrics.inc_http_requests();
    }

    let method = req.method().to_string();
    let path = req.path().to_string();
    let locale_code = match locale {
        messages::i18n::Locale::Vi => "vi",
        messages::i18n::Locale::En => "en",
    };
    let user_agent = req
        .headers()
        .get("user-agent")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("unknown")
        .to_string();
    let client_ip = req
        .connection_info()
        .realip_remote_addr()
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| "unknown".to_string());
    let start = Instant::now();

    req.extensions_mut().insert(RequestContext {
        request_id: request_id.clone(),
    });

    let mut response = match next.call(req).await {
        Ok(response) => response,
        Err(err) => {
            let effective_err = if let Some(app_err) = err.as_error::<error::Error>()
                && let Some(localized_err) = app_err.localized_for_locale(locale)
            {
                localized_err.into()
            } else {
                err
            };

            let status = effective_err.as_response_error().status_code();
            let duration_ms = start.elapsed().as_millis() as u64;

            if let Some(app_err) = effective_err.as_error::<error::Error>() {
                if status.is_server_error() {
                    tracing::error!(
                        request_id = %request_id,
                        method = %method,
                        path = %path,
                        locale = %locale_code,
                        user_agent = %user_agent,
                        client_ip = %client_ip,
                        status = status.as_u16(),
                        code = app_err.code(),
                        duration_ms = duration_ms,
                        "HTTP request failed"
                    );
                } else {
                    tracing::warn!(
                        request_id = %request_id,
                        method = %method,
                        path = %path,
                        locale = %locale_code,
                        user_agent = %user_agent,
                        client_ip = %client_ip,
                        status = status.as_u16(),
                        code = app_err.code(),
                        duration_ms = duration_ms,
                        "HTTP request failed"
                    );
                }
            } else {
                tracing::error!(
                    request_id = %request_id,
                    method = %method,
                    path = %path,
                    locale = %locale_code,
                    user_agent = %user_agent,
                    client_ip = %client_ip,
                    status = status.as_u16(),
                    code = "internal_error",
                    duration_ms = duration_ms,
                    "HTTP request failed"
                );
            }

            return Err(effective_err);
        }
    };

    if let Ok(value) = HeaderValue::from_str(&request_id) {
        response
            .headers_mut()
            .insert(HeaderName::from_static("x-request-id"), value);
    }

    tracing::info!(
        request_id = %request_id,
        method = %method,
        path = %path,
        locale = %locale_code,
        user_agent = %user_agent,
        client_ip = %client_ip,
        status = response.status().as_u16(),
        duration_ms = start.elapsed().as_millis() as u64,
        "HTTP request"
    );

    Ok(response)
}

pub async fn authentication<B>(
    req: ServiceRequest,
    next: Next<B>,
) -> Result<ServiceResponse<B>, Error>
where
    B: MessageBody + 'static,
{
    if req.method() == actix_web::http::Method::OPTIONS {
        return next.call(req).await;
    }

    let auth = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());
    let token = match auth.and_then(|h| h.strip_prefix("Bearer ")) {
        Some(t) => t,
        None => {
            return Err(
                error::Error::unauthorized_key(messages::i18n::Key::TokenInvalidOrExpired).into(),
            );
        }
    };

    let app_state = req
        .app_data::<actix_web::web::Data<AppState>>()
        .ok_or_else(error::Error::internal_server_error)?;

    let claims = Claims::decode(token, app_state.config.jwt_secret.as_ref())
        .map_err(|_| error::Error::forbidden_key(messages::i18n::Key::TokenInvalidOrExpired))?;

    req.extensions_mut().insert(claims);

    next.call(req).await
}

pub fn get_extensions<T: Clone + 'static>(req: &HttpRequest) -> Result<T, error::Error> {
    let extensions = req.extensions();

    let claims = extensions
        .get::<T>()
        .ok_or_else(|| error::Error::unauthorized_key(messages::i18n::Key::AuthRequired))?
        .clone();

    Ok(claims)
}

pub fn authorization<B>(
    allowed_roles: Vec<UserRole>,
) -> impl Fn(
    ServiceRequest,
    Next<B>,
) -> LocalBoxFuture<'static, Result<ServiceResponse<B>, actix_web::Error>>
where
    B: MessageBody + 'static,
{
    let allowed_roles = Rc::new(allowed_roles);
    move |req: ServiceRequest, next: Next<B>| {
        let roles = allowed_roles.clone();
        async move {
            let role = get_extensions::<Claims>(req.request())?.role;

            if !roles.contains(&role) {
                return Err(error::Error::forbidden_key(messages::i18n::Key::AccessDenied).into());
            }
            next.call(req).await
        }
        .boxed_local()
    }
}
