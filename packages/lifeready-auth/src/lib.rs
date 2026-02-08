use axum::{
    body::Body,
    extract::FromRequestParts,
    http::{header, HeaderMap, HeaderValue, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{Duration, TimeZone, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tower::{Layer, Service};
use uuid::Uuid;

pub const REQUEST_ID_HEADER: &str = "x-request-id";

type AxumRequest = Request<Body>;
type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

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
    #[serde(default)]
    pub scopes: Vec<String>,
    pub exp: usize,
    pub iat: usize,
    pub jti: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<String>,
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
            scopes: vec![access_level_scope(access_level).to_string()],
            exp,
            iat,
            jti: Uuid::new_v4().to_string(),
            iss: None,
            aud: None,
            email,
        }
    }
}

#[derive(Clone)]
pub struct AuthConfig {
    encoding_key: Arc<EncodingKey>,
    decoding_key: Arc<DecodingKey>,
    issuer: Option<String>,
    audience: Option<String>,
    leeway_seconds: u64,
}

impl AuthConfig {
    pub fn new(secret: impl Into<String>) -> Self {
        let secret = secret.into();
        let encoding_key = Arc::new(EncodingKey::from_secret(secret.as_bytes()));
        let decoding_key = Arc::new(DecodingKey::from_secret(secret.as_bytes()));
        Self {
            encoding_key,
            decoding_key,
            issuer: None,
            audience: None,
            leeway_seconds: 30,
        }
    }

    pub fn from_env() -> Self {
        let secret = match std::env::var("JWT_SECRET") {
            Ok(secret) => secret,
            Err(_) => {
                tracing::warn!("JWT_SECRET not set; using dev-only fallback secret");
                "dev-only-secret-change-me".to_string()
            }
        };

        let mut config = Self::new(secret);

        if let Ok(issuer) = std::env::var("JWT_ISSUER") {
            if !issuer.trim().is_empty() {
                config = config.with_issuer(issuer);
            }
        }

        if let Ok(audience) = std::env::var("JWT_AUDIENCE") {
            if !audience.trim().is_empty() {
                config = config.with_audience(audience);
            }
        }

        config
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
        let mut claims = claims.clone();
        if claims.iss.is_none() {
            claims.iss = self.issuer.clone();
        }
        if claims.aud.is_none() {
            claims.aud = self.audience.clone();
        }
        let mut header = Header::new(Algorithm::HS256);
        header.typ = Some("JWT".into());
        encode(&header, &claims, self.encoding_key.as_ref())
            .map_err(|error| AuthError::invalid(format!("token encode failed: {error}")))
    }

    pub fn decode_token(&self, token: &str) -> Result<Claims, AuthError> {
        let validation = self.validation();
        let data = decode::<Claims>(token, self.decoding_key.as_ref(), &validation)
            .map_err(|error| AuthError::unauthorized(format!("token decode failed: {error}")))?;

        Ok(data.claims)
    }

    fn validation(&self) -> Validation {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.leeway = self.leeway_seconds;

        if let Some(issuer) = &self.issuer {
            validation.set_issuer(&[issuer]);
        }
        if let Some(audience) = &self.audience {
            validation.set_audience(&[audience]);
        }

        validation
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
pub struct RequestContext {
    pub request_id: RequestId,
    pub principal_id: String,
    pub roles: Vec<Role>,
    pub allowed_tiers: Vec<SensitivityTier>,
    pub scopes: Vec<String>,
    pub expires_at: chrono::DateTime<Utc>,
    pub email: Option<String>,
}

impl RequestContext {
    fn from_claims(request_id: RequestId, claims: &Claims) -> Self {
        let scopes = if claims.scopes.is_empty() {
            vec![access_level_scope(claims.access_level).to_string()]
        } else {
            claims.scopes.clone()
        };

        let expires_at = Utc
            .timestamp_opt(claims.exp as i64, 0)
            .single()
            .unwrap_or_else(Utc::now);

        Self {
            request_id,
            principal_id: claims.sub.clone(),
            roles: vec![claims.role],
            allowed_tiers: claims.tiers.clone(),
            scopes,
            expires_at,
            email: claims.email.clone(),
        }
    }
}

pub fn ctx<B>(req: &Request<B>) -> Option<&RequestContext> {
    req.extensions().get::<RequestContext>()
}

impl<S> FromRequestParts<S> for RequestContext
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<RequestContext>()
            .cloned()
            .ok_or_else(|| AuthError::unauthorized("missing auth context"))
    }
}

#[derive(Debug, Clone)]
pub enum AuthError {
    Unauthorized { detail: String },
    Forbidden { detail: String },
    Invalid { detail: String },
}

impl AuthError {
    pub fn unauthorized(detail: impl Into<String>) -> Self {
        Self::Unauthorized {
            detail: detail.into(),
        }
    }

    pub fn forbidden(detail: impl Into<String>) -> Self {
        Self::Forbidden {
            detail: detail.into(),
        }
    }

    pub fn invalid(detail: impl Into<String>) -> Self {
        Self::Invalid {
            detail: detail.into(),
        }
    }

    pub fn into_response(self, request_id: Option<RequestId>) -> Response {
        let (status, title, r#type, detail) = match self {
            AuthError::Unauthorized { detail } => (
                StatusCode::UNAUTHORIZED,
                "Unauthorized",
                "https://errors.lifeready.local/auth/unauthorized",
                Some(detail),
            ),
            AuthError::Forbidden { detail } => (
                StatusCode::FORBIDDEN,
                "Forbidden",
                "https://errors.lifeready.local/auth/forbidden",
                Some(detail),
            ),
            AuthError::Invalid { detail } => (
                StatusCode::BAD_REQUEST,
                "Invalid request",
                "https://errors.lifeready.local/request/invalid",
                Some(detail),
            ),
        };

        problem_response(status, r#type, title, detail, request_id.map(|id| id.0))
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

fn access_level_scope(level: AccessLevel) -> &'static str {
    match level {
        AccessLevel::ReadOnlyPacks => "read:packs",
        AccessLevel::ReadOnlyAll => "read:all",
        AccessLevel::LimitedWrite => "write:limited",
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

#[derive(Clone)]
pub struct AuthLayer {
    config: Arc<AuthConfig>,
    allowlist: Arc<Vec<String>>,
}

impl AuthLayer {
    pub fn new(config: Arc<AuthConfig>) -> Self {
        Self {
            config,
            allowlist: Arc::new(Vec::new()),
        }
    }

    pub fn with_allowlist(
        mut self,
        allowlist: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.allowlist = Arc::new(allowlist.into_iter().map(Into::into).collect());
        self
    }
}

impl<S> Layer<S> for AuthLayer {
    type Service = AuthService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthService {
            inner,
            config: self.config.clone(),
            allowlist: self.allowlist.clone(),
        }
    }
}

#[derive(Clone)]
pub struct AuthService<S> {
    inner: S,
    config: Arc<AuthConfig>,
    allowlist: Arc<Vec<String>>,
}

impl<S, B> Service<Request<B>> for AuthService<S>
where
    S: Service<Request<B>, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
    B: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<B>) -> Self::Future {
        let config = self.config.clone();
        let allowlist = self.allowlist.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            let path = req.uri().path();
            if path == "/healthz" || allowlist.iter().any(|allowed| allowed == path) {
                return inner.call(req).await;
            }

            let request_id = req
                .extensions()
                .get::<RequestId>()
                .copied()
                .unwrap_or_else(|| RequestId(Uuid::new_v4()));
            req.extensions_mut().insert(request_id);

            let token = match bearer_token(req.headers()) {
                Ok(token) => token,
                Err(error) => return Ok(error.into_response(Some(request_id))),
            };

            let claims = match config.decode_token(token) {
                Ok(claims) => claims,
                Err(error) => return Ok(error.into_response(Some(request_id))),
            };

            let ctx = RequestContext::from_claims(request_id, &claims);
            req.extensions_mut().insert(claims);
            req.extensions_mut().insert(ctx);

            inner.call(req).await
        })
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

    let request_id = req
        .extensions()
        .get::<RequestId>()
        .copied()
        .unwrap_or_else(|| RequestId(Uuid::new_v4()));
    req.extensions_mut().insert(request_id);
    let token = match bearer_token(req.headers()) {
        Ok(token) => token,
        Err(error) => return error.into_response(Some(request_id)),
    };

    let claims = match state.config.decode_token(token) {
        Ok(claims) => claims,
        Err(error) => return error.into_response(Some(request_id)),
    };

    let ctx = RequestContext::from_claims(request_id, &claims);
    req.extensions_mut().insert(claims);
    req.extensions_mut().insert(ctx);
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
    problem_response(
        StatusCode::NOT_FOUND,
        "https://errors.lifeready.local/request/not-found",
        "Not found",
        Some(detail.into()),
        request_id.map(|id| id.0),
    )
}

pub fn conflict(request_id: Option<RequestId>, detail: impl Into<String>) -> Response {
    problem_response(
        StatusCode::CONFLICT,
        "https://errors.lifeready.local/request/conflict",
        "Conflict",
        Some(detail.into()),
        request_id.map(|id| id.0),
    )
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

    #[test]
    fn decode_token_roundtrip() {
        let config = AuthConfig::new("test-secret");
        let claims = Claims::new(
            "user",
            Role::Principal,
            vec![SensitivityTier::Green],
            AccessLevel::ReadOnlyAll,
            None,
            60,
        );

        let token = config.issue_token(&claims).expect("token");
        let decoded = config.decode_token(&token).expect("decoded");

        assert_eq!(decoded.sub, claims.sub);
        assert_eq!(decoded.role, claims.role);
        assert_eq!(decoded.access_level, claims.access_level);
    }

    #[test]
    fn decode_token_rejects_expired() {
        let config = AuthConfig::new("test-secret");
        let claims = Claims::new(
            "user",
            Role::Principal,
            vec![SensitivityTier::Green],
            AccessLevel::ReadOnlyAll,
            None,
            -120,
        );
        let token = config.issue_token(&claims).expect("token");

        let err = config.decode_token(&token).expect_err("expired");
        match err {
            AuthError::Unauthorized { .. } => {}
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
