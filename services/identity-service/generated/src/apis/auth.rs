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
pub enum ReadyzGetResponse {
    /// Service and critical dependencies are ready to serve traffic
    Status200_ServiceAndCriticalDependenciesAreReadyToServeTraffic
    {
        body: models::ReadyzGet200Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Service is reachable but critical dependencies are not ready
    Status503_ServiceIsReachableButCriticalDependenciesAreNotReady
    {
        body: models::ReadyzGet200Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum V1AuthLoginPostResponse {
    /// Challenge created
    Status200_ChallengeCreated
    {
        body: models::LoginChallenge,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Error response
    Status400_ErrorResponse
    {
        body: models::V1AuthLoginPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Unauthorized
    Status401_Unauthorized
    {
        body: models::V1AuthLoginPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Forbidden
    Status403_Forbidden
    {
        body: models::V1AuthLoginPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Unprocessable Entity
    Status422_UnprocessableEntity
    {
        body: models::V1AuthLoginPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Internal Server Error
    Status500_InternalServerError
    {
        body: models::V1AuthLoginPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum V1AuthMfaVerifyPostResponse {
    /// Session issued
    Status200_SessionIssued
    {
        body: models::Session,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Error response
    Status400_ErrorResponse
    {
        body: models::V1AuthLoginPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Unauthorized
    Status401_Unauthorized
    {
        body: models::V1AuthLoginPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Forbidden
    Status403_Forbidden
    {
        body: models::V1AuthLoginPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Unprocessable Entity
    Status422_UnprocessableEntity
    {
        body: models::V1AuthLoginPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Internal Server Error
    Status500_InternalServerError
    {
        body: models::V1AuthLoginPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum V1MeGetResponse {
    /// Current principal profile
    Status200_CurrentPrincipalProfile
    {
        body: models::Me,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Unauthorized
    Status401_Unauthorized
    {
        body: models::V1AuthLoginPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Forbidden
    Status403_Forbidden
    {
        body: models::V1AuthLoginPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Internal Server Error
    Status500_InternalServerError
    {
        body: models::V1AuthLoginPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
}




/// Auth
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Auth<E: std::fmt::Debug + Send + Sync + 'static = ()>: super::ErrorHandler<E> {
    type Claims;

    /// Readiness check (dependencies available).
    ///
    /// ReadyzGet - GET /identity/readyz
    async fn readyz_get(
    &self,
    
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
        claims: &Self::Claims,
    ) -> Result<ReadyzGetResponse, E>;

    /// Start login (passwordless or password).
    ///
    /// V1AuthLoginPost - POST /identity/v1/auth/login
    async fn v1_auth_login_post(
    &self,
    
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
        claims: &Self::Claims,
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
        claims: &Self::Claims,
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
