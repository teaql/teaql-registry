use axum::{
    body::Body,
    extract::{Request, State},
    http::{header, HeaderMap, Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::{collections::HashSet, net::SocketAddr, sync::Arc};
use teaql_registry_core::ServiceRuntime;

use crate::{
    api::AppState,
    security::{parse_basic_auth, verify_password, RbacChecker, TokenService},
    services::{SecurityService, TenantService},
};

/// Trusted identity, authorization and tenant state for exactly one request.
#[derive(Clone)]
pub struct RequestContext {
    pub runtime: Arc<ServiceRuntime>,
    pub tenant_id: u64,
    pub tenant_name: String,
    pub user_id: u64,
    pub username: String,
    pub client_ip: String,
    pub privileges: HashSet<String>,
    pub is_anonymous: bool,
}

impl RequestContext {
    pub fn has_privilege(&self, required: &str) -> bool {
        RbacChecker::new(self.privileges.clone()).has_privilege(required)
    }
}

/// Only trust proxy-provided addresses when deployment explicitly opts in.
pub fn extract_client_ip(headers: &HeaderMap, connect_info: Option<&SocketAddr>) -> String {
    let trust_proxy = std::env::var("TRUST_PROXY_HEADERS")
        .ok()
        .is_some_and(|value| value == "1" || value.eq_ignore_ascii_case("true"));
    if trust_proxy {
        if let Some(ip) = headers
            .get("x-forwarded-for")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.split(',').next())
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            return ip.to_string();
        }
        if let Some(ip) = headers
            .get("x-real-ip")
            .and_then(|value| value.to_str().ok())
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            return ip.to_string();
        }
    }
    connect_info
        .map(|address| address.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

pub async fn authenticate_request(
    State(state): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    let scope = match build_request_context(
        &state,
        request.method(),
        request.uri().path(),
        request.headers(),
    )
    .await
    {
        Ok(scope) => scope,
        Err(response) => return response,
    };
    request.extensions_mut().insert(Arc::new(scope));
    next.run(request).await
}

async fn build_request_context(
    state: &AppState,
    method: &Method,
    path: &str,
    headers: &HeaderMap,
) -> Result<RequestContext, Response> {
    let client_ip = extract_client_ip(headers, None);
    if !state.enforces_authentication() {
        let runtime = state
            .request_runtime(1, "Default Tenant")
            .await
            .map_err(internal_error)?;
        return Ok(RequestContext {
            runtime,
            tenant_id: 1,
            tenant_name: "Default Tenant".to_string(),
            user_id: 0,
            username: "test-harness".to_string(),
            client_ip,
            privileges: HashSet::from(["*".to_string()]),
            is_anonymous: false,
        });
    }

    let authorization = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok());
    let tenant_selector = headers
        .get("x-teaql-tenant")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty());

    let (tenant_id, tenant_name, basic_credentials) = if let Some(value) = authorization {
        if let Some((qualified_username, password)) = parse_basic_auth(value) {
            let (username, username_tenant) = split_qualified_username(&qualified_username);
            let selector = tenant_selector
                .or(username_tenant.as_deref())
                .unwrap_or("1");
            let tenant = resolve_tenant(&state.runtime, selector).await?;
            (tenant.0, tenant.1, Some((username, password)))
        } else if let Some(raw_token) = value.strip_prefix("Bearer ").map(str::trim) {
            let principal = TokenService::validate_token(
                raw_token,
                required_privilege(method, path).unwrap_or(""),
            )
            .ok_or_else(unauthorized)?;
            let tenant = resolve_tenant(&state.runtime, &principal.tenant_id.to_string()).await?;
            let runtime = state
                .request_runtime(tenant.0, &tenant.1)
                .await
                .map_err(internal_error)?;
            let scope = RequestContext {
                runtime,
                tenant_id: principal.tenant_id,
                tenant_name: tenant.1,
                user_id: principal.user_id,
                username: principal.username,
                client_ip,
                privileges: principal.scopes.into_iter().collect(),
                is_anonymous: false,
            };
            authorize(&scope, method, path)?;
            return Ok(scope);
        } else {
            return Err(unauthorized());
        }
    } else {
        let selector = tenant_selector.unwrap_or("1");
        let tenant = resolve_tenant(&state.runtime, selector).await?;
        (tenant.0, tenant.1, None)
    };

    let runtime = state
        .request_runtime(tenant_id, &tenant_name)
        .await
        .map_err(internal_error)?;
    let scope = if let Some((username, password)) = basic_credentials {
        let user =
            SecurityService::find_user_by_tenant_and_username(&runtime, tenant_id, &username)
                .await
                .map_err(internal_error)?
                .filter(|user| user.user_status_id() == 1001)
                .ok_or_else(unauthorized)?;
        let privileges = if verify_password(&password, &user.password_hash()) {
            SecurityService::permissions_for_user(&runtime, user.id())
                .await
                .map_err(internal_error)?
        } else {
            // Several native package clients send access tokens as the Basic
            // password. Bind that token to the selected tenant and username.
            let required = required_privilege(method, path).unwrap_or("");
            let principal = TokenService::validate_token(&password, required)
                .filter(|principal| {
                    principal.tenant_id == tenant_id
                        && principal.user_id == user.id()
                        && principal.username == username
                })
                .ok_or_else(unauthorized)?;
            principal.scopes.into_iter().collect()
        };
        RequestContext {
            runtime,
            tenant_id,
            tenant_name,
            user_id: user.id(),
            username,
            client_ip,
            privileges,
            is_anonymous: false,
        }
    } else if state.allows_anonymous_read() {
        RequestContext {
            runtime,
            tenant_id,
            tenant_name,
            user_id: 0,
            username: "anonymous".to_string(),
            client_ip,
            privileges: HashSet::from(["repository:read".to_string()]),
            is_anonymous: true,
        }
    } else {
        return Err(unauthorized());
    };
    authorize(&scope, method, path)?;
    Ok(scope)
}

#[allow(clippy::result_large_err)]
fn authorize(scope: &RequestContext, method: &Method, path: &str) -> Result<(), Response> {
    let Some(required) = required_privilege(method, path) else {
        return Ok(());
    };
    if scope.has_privilege(required) {
        Ok(())
    } else if scope.is_anonymous {
        Err(unauthorized())
    } else {
        Err((StatusCode::FORBIDDEN, "insufficient registry permission").into_response())
    }
}

fn required_privilege(method: &Method, path: &str) -> Option<&'static str> {
    if matches!(
        path,
        "/" | "/ui" | "/console" | "/help" | "/metrics" | "/v2" | "/v2/"
    ) || path.starts_with("/assets/")
        || path.ends_with("/status")
        || path.ends_with("/status/writable")
    {
        return None;
    }
    if path.starts_with("/service/rest/v1/tenants") {
        return Some("platform:admin");
    }
    if path.starts_with("/service/rest/") {
        return Some("registry:admin");
    }
    if method == Method::GET || method == Method::HEAD {
        Some("repository:read")
    } else {
        Some("repository:write")
    }
}

fn split_qualified_username(value: &str) -> (String, Option<String>) {
    if let Some((username, tenant)) = value.rsplit_once('@') {
        return (username.to_string(), Some(tenant.to_string()));
    }
    if let Some((tenant, username)) = value.split_once('\\') {
        return (username.to_string(), Some(tenant.to_string()));
    }
    (value.to_string(), None)
}

async fn resolve_tenant(
    runtime: &ServiceRuntime,
    selector: &str,
) -> Result<(u64, String), Response> {
    let tenant = if let Ok(id) = selector.parse::<u64>() {
        TenantService::get_tenant(runtime, id).await
    } else {
        TenantService::find_tenant_by_code(runtime, selector).await
    }
    .map_err(internal_error)?
    .ok_or_else(|| (StatusCode::UNAUTHORIZED, "unknown tenant").into_response())?;
    Ok((tenant.id(), tenant.name().to_string()))
}

fn unauthorized() -> Response {
    let mut response = (
        StatusCode::UNAUTHORIZED,
        "valid registry credentials required",
    )
        .into_response();
    response.headers_mut().insert(
        header::WWW_AUTHENTICATE,
        "Basic realm=\"TeaQL Registry\""
            .parse()
            .expect("static header is valid"),
    );
    response
}

fn internal_error(error: impl std::fmt::Display) -> Response {
    tracing::error!(error = %error, "request security context failed");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "registry security context failed",
    )
        .into_response()
}
