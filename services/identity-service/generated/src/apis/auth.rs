use async_trait::async_trait;
use axum::extract::*;
use axum_extra::extract::CookieJar;
use bytes::Bytes;
use headers::Host;
use http::Method;
use serde::{Deserialize, Serialize};

use crate::{models, types::*};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum V1AuthLoginPostResponse {
    /// Challenge created
    Status200_ChallengeCreated(models::LoginChallenge),
    /// Error response
    Status0_ErrorResponse(models::V1AuthLoginPostDefaultResponse),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum V1AuthMfaVerifyPostResponse {
    /// Session issued
    Status200_SessionIssued(models::Session),
    /// Error response
    Status0_ErrorResponse(models::V1AuthLoginPostDefaultResponse),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum V1MeGetResponse {
    /// Current principal profile
    Status200_CurrentPrincipalProfile(models::Me),
    /// Error response
    Status0_ErrorResponse(models::V1AuthLoginPostDefaultResponse),
}

/// Auth
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Auth<E: std::fmt::Debug + Send + Sync + 'static = ()>: super::ErrorHandler<E> {
    type Claims;

    /// Start login (passwordless or password).
    ///
    /// V1AuthLoginPost - POST /identity/v1/auth/login
    async fn v1_auth_login_post(
        &self,

        method: &Method,
        host: &Host,
        cookies: &CookieJar,
        body: &models::LoginRequest,
    ) -> Result<V1AuthLoginPostResponse, E>;

    /// Verify MFA challenge.
    ///
    /// V1AuthMfaVerifyPost - POST /identity/v1/auth/mfa/verify
    async fn v1_auth_mfa_verify_post(
        &self,

        method: &Method,
        host: &Host,
        cookies: &CookieJar,
        body: &models::MfaVerifyRequest,
    ) -> Result<V1AuthMfaVerifyPostResponse, E>;

    /// Get current user.
    ///
    /// V1MeGet - GET /identity/v1/me
    async fn v1_me_get(
        &self,

        method: &Method,
        host: &Host,
        cookies: &CookieJar,
        claims: &Self::Claims,
    ) -> Result<V1MeGetResponse, E>;
}
