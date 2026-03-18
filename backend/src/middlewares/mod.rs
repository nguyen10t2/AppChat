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

use crate::{ENV, METRICS, api::error, modules::user::schema::UserRole, observability::RequestContext, utils::Claims};

pub async fn request_context<B>(
    req: ServiceRequest,
    next: Next<B>,
) -> Result<ServiceResponse<B>, Error>
where
    B: MessageBody + 'static,
{
    let request_id = req
        .headers()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| Uuid::now_v7().to_string());

    METRICS.inc_http_requests();

    let method = req.method().to_string();
    let path = req.path().to_string();
    let start = Instant::now();

    req.extensions_mut().insert(RequestContext {
        request_id: request_id.clone(),
    });

    let mut response = next.call(req).await?;

    if let Ok(value) = HeaderValue::from_str(&request_id) {
        response
            .headers_mut()
            .insert(HeaderName::from_static("x-request-id"), value);
    }

    tracing::info!(
        request_id = %request_id,
        method = %method,
        path = %path,
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
            return Err(error::Error::unauthorized("Token không hợp lệ hoặc đã hết hạn").into());
        }
    };

    let claims = Claims::decode(token, ENV.jwt_secret.as_ref())
        .map_err(|_| error::Error::forbidden("Token không hợp lệ hoặc đã hết hạn"))?;

    req.extensions_mut().insert(claims);

    next.call(req).await
}

pub fn get_extensions<T: Clone + 'static>(req: &HttpRequest) -> Result<T, error::Error> {
    let extensions = req.extensions();

    let claims = extensions
        .get::<T>()
        .ok_or_else(|| error::Error::unauthorized("Chưa được xác thực"))?
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
                return Err(error::Error::forbidden("Bạn không có quyền thực hiện thao tác này").into());
            }
            next.call(req).await
        }
        .boxed_local()
    }
}
