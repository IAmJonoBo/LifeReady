use axum::{
    body::Body,
    http::{header, HeaderMap, HeaderValue, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

pub const REQUEST_ID_HEADER: &str = "x-request-id";

type AxumRequest = Request<Body>;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SensitivityTier {
    Green,
    Amber,
    Red,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Principal,
    Proxy,
    ExecutorNominee,
    EmergencyContact,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AccessLevel {
    ReadOnlyPacks,
    ReadOnlyAll,
    LimitedWrite,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Claims {
    pub sub: String,
    pub role: Role,
    pub tiers: Vec<SensitivityTier>,
    pub access_level: AccessLevel,
    pub exp: usize,
    pub iat: usize,
    pub jti: String,
    pub email: Option<String>,
}

impl Claims {
    pub fn new(
        sub: impl Into<String>,
        role: Role,
        tiers: Vec<SensitivityTier>,
        access_level: AccessLevel,
        email: Option<String>,
        ttl_seconds: i64,
    ) -> Self {
        let now = Utc::now();
        let exp = (now + Duration::seconds(ttl_seconds)).timestamp() as usize;
        let iat = now.timestamp() as usize;

        Self {
            sub: sub.into(),
            role,
            tiers,
            access_level,
            exp,
            iat,
            jti: Uuid::new_v4().to_string(),
            email,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuthConfig {
    secret: String,
    issuer: Option<String>,
    audience: Option<String>,
    leeway_seconds: u64,
}

impl AuthConfig {
    pub fn new(secret: impl Into<String>) -> Self {
        Self {
            secret: secret.into(),
            issuer: None,
            audience: None,
            leeway_seconds: 30,
        }
    }

    pub fn from_env() -> Self {
        match std::env::var("JWT_SECRET") {
            Ok(secret) => Self::new(secret),
            Err(_) => {
                tracing::warn!("JWT_SECRET not set; using dev-only fallback secret");
                Self::new("dev-only-secret-change-me")
            }
        }
    }

    pub fn with_issuer(mut self, issuer: impl Into<String>) -> Self {
        self.issuer = Some(issuer.into());
        self
    }

    pub fn with_audience(mut self, audience: impl Into<String>) -> Self {
        self.audience = Some(audience.into());
        self
    }

    pub fn issue_token(&self, claims: &Claims) -> Result<String, AuthError> {
        let mut header = Header::new(Algorithm::HS256);
        header.typ = Some("JWT".into());
        encode(
            &header,
            claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|error| AuthError::invalid(format!("token encode failed: {error}")))
    }

    pub fn decode_token(&self, token: &str) -> Result<Claims, AuthError> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.leeway = self.leeway_seconds;

        if let Some(issuer) = &self.issuer {
            validation.set_issuer(&[issuer]);
        }
        if let Some(audience) = &self.audience {
            validation.set_audience(&[audience]);
        }

        let data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &validation,
        )
        .map_err(|error| AuthError::unauthorized(format!("token decode failed: {error}")))?;

        Ok(data.claims)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RequestId(pub Uuid);

impl RequestId {
    pub fn from_headers(headers: &HeaderMap) -> Option<Self> {
        headers
            .get(REQUEST_ID_HEADER)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| Uuid::parse_str(value).ok())
            .map(RequestId)
    }
}

#[derive(Debug, Clone)]
pub struct RequiredAccess {
    pub min_tier: SensitivityTier,
    pub access_level: AccessLevel,
    pub allowed_roles: Option<Vec<Role>>,
}

#[derive(Debug, Clone)]
pub struct AuthError {
    status: StatusCode,
    title: String,
    detail: Option<String>,
    r#type: String,
}

impl AuthError {
    pub fn unauthorized(detail: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            title: "Unauthorized".into(),
            detail: Some(detail.into()),
            r#type: "https://errors.lifeready.local/auth/unauthorized".into(),
        }
    }

    pub fn forbidden(detail: impl Into<String>) -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            title: "Forbidden".into(),
            detail: Some(detail.into()),
            r#type: "https://errors.lifeready.local/auth/forbidden".into(),
        }
    }

    pub fn invalid(detail: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            title: "Invalid request".into(),
            detail: Some(detail.into()),
            r#type: "https://errors.lifeready.local/request/invalid".into(),
        }
    }

    pub fn into_response(self, request_id: Option<RequestId>) -> Response {
        problem_response(
            self.status,
            &self.r#type,
            &self.title,
            self.detail,
            request_id.map(|id| id.0),
        )
    }
}

#[derive(Debug, Serialize)]
struct ProblemDetails {
    #[serde(rename = "type")]
    r#type: String,
    title: String,
    status: u16,
    detail: Option<String>,
    instance: Option<String>,
    request_id: Option<Uuid>,
    errors: Option<std::collections::HashMap<String, Vec<String>>>,
}

fn problem_response(
    status: StatusCode,
    r#type: &str,
    title: &str,
    detail: Option<String>,
    request_id: Option<Uuid>,
) -> Response {
    let body = ProblemDetails {
        r#type: r#type.to_string(),
        title: title.to_string(),
        status: status.as_u16(),
        detail,
        instance: None,
        request_id,
        errors: None,
    };

    let mut response = (status, Json(body)).into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/problem+json"),
    );
    response
}

fn tier_rank(tier: SensitivityTier) -> u8 {
    match tier {
        SensitivityTier::Green => 0,
        SensitivityTier::Amber => 1,
        SensitivityTier::Red => 2,
    }
}

fn access_rank(level: AccessLevel) -> u8 {
    match level {
        AccessLevel::ReadOnlyPacks => 0,
        AccessLevel::ReadOnlyAll => 1,
        AccessLevel::LimitedWrite => 2,
    }
}

pub fn authorize(claims: &Claims, required: &RequiredAccess) -> Result<(), AuthError> {
    if let Some(roles) = &required.allowed_roles {
        if !roles.contains(&claims.role) {
            return Err(AuthError::forbidden("role not permitted"));
        }
    }

    let required_tier_rank = tier_rank(required.min_tier);
    let max_tier_rank = claims
        .tiers
        .iter()
        .map(|tier| tier_rank(*tier))
        .max()
        .unwrap_or(0);

    if max_tier_rank < required_tier_rank {
        return Err(AuthError::forbidden("insufficient tier access"));
    }

    if access_rank(claims.access_level) < access_rank(required.access_level) {
        return Err(AuthError::forbidden("insufficient access level"));
    }

    Ok(())
}

#[derive(Clone)]
pub struct AuthLayerState {
    config: AuthConfig,
    allowlist: Vec<String>,
}

impl AuthLayerState {
    pub fn new(config: AuthConfig, allowlist: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            config,
            allowlist: allowlist.into_iter().map(Into::into).collect(),
        }
    }
}

pub async fn request_id_middleware(mut req: AxumRequest, next: Next) -> Response {
    let request_id =
        RequestId::from_headers(req.headers()).unwrap_or_else(|| RequestId(Uuid::new_v4()));
    req.extensions_mut().insert(request_id);

    let mut response = next.run(req).await;
    response.headers_mut().insert(
        REQUEST_ID_HEADER,
        HeaderValue::from_str(&request_id.0.to_string())
            .unwrap_or_else(|_| HeaderValue::from_static("")),
    );
    response
}

pub async fn auth_middleware(
    axum::extract::State(state): axum::extract::State<AuthLayerState>,
    mut req: AxumRequest,
    next: Next,
) -> Response {
    let path = req.uri().path();
    if path == "/healthz" || state.allowlist.iter().any(|allowed| allowed == path) {
        return next.run(req).await;
    }

    let request_id = req.extensions().get::<RequestId>().copied();
    let token = match bearer_token(req.headers()) {
        Ok(token) => token,
        Err(error) => return error.into_response(request_id),
    };

    let claims = match state.config.decode_token(token) {
        Ok(claims) => claims,
        Err(error) => return error.into_response(request_id),
    };

    req.extensions_mut().insert(claims);
    next.run(req).await
}

fn bearer_token(headers: &HeaderMap) -> Result<&str, AuthError> {
    let value = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| AuthError::unauthorized("missing bearer token"))?;

    let value = value
        .strip_prefix("Bearer ")
        .ok_or_else(|| AuthError::unauthorized("invalid authorization scheme"))?;

    if value.trim().is_empty() {
        return Err(AuthError::unauthorized("empty bearer token"));
    }

    Ok(value)
}

pub fn access_denied(request_id: Option<RequestId>, detail: impl Into<String>) -> Response {
    AuthError::forbidden(detail).into_response(request_id)
}

pub fn invalid_request(request_id: Option<RequestId>, detail: impl Into<String>) -> Response {
    AuthError::invalid(detail).into_response(request_id)
}

pub fn not_found(request_id: Option<RequestId>, detail: impl Into<String>) -> Response {
    AuthError {
        status: StatusCode::NOT_FOUND,
        title: "Not found".into(),
        detail: Some(detail.into()),
        r#type: "https://errors.lifeready.local/request/not-found".into(),
    }
    .into_response(request_id)
}

pub fn conflict(request_id: Option<RequestId>, detail: impl Into<String>) -> Response {
    AuthError {
        status: StatusCode::CONFLICT,
        title: "Conflict".into(),
        detail: Some(detail.into()),
        r#type: "https://errors.lifeready.local/request/conflict".into(),
    }
    .into_response(request_id)
}

pub fn ok_response<T: Serialize>(payload: T) -> Response {
    Json(json!(payload)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authorize_rejects_missing_tier() {
        let claims = Claims::new(
            "user",
            Role::Principal,
            vec![SensitivityTier::Green],
            AccessLevel::LimitedWrite,
            None,
            60,
        );
        let required = RequiredAccess {
            min_tier: SensitivityTier::Amber,
            access_level: AccessLevel::ReadOnlyAll,
            allowed_roles: None,
        };
        let result = authorize(&claims, &required);
        assert!(result.is_err());
    }

    #[test]
    fn authorize_accepts_higher_access() {
        let claims = Claims::new(
            "user",
            Role::Principal,
            vec![SensitivityTier::Red],
            AccessLevel::LimitedWrite,
            None,
            60,
        );
        let required = RequiredAccess {
            min_tier: SensitivityTier::Amber,
            access_level: AccessLevel::ReadOnlyAll,
            allowed_roles: Some(vec![Role::Principal, Role::Proxy]),
        };
        let result = authorize(&claims, &required);
        assert!(result.is_ok());
    }
}
