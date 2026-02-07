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
pub enum V1RolesGrantsPostResponse {
    /// Created
    Status201_Created(models::RoleGrant),
    /// Error response
    Status0_ErrorResponse(models::V1PeopleGetDefaultResponse),
}

/// Roles
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Roles<E: std::fmt::Debug + Send + Sync + 'static = ()>: super::ErrorHandler<E> {
    type Claims;

    /// Invite & grant role scope (staged access policy enforced server-side).
    ///
    /// V1RolesGrantsPost - POST /estate/v1/roles/grants
    async fn v1_roles_grants_post(
        &self,

        method: &Method,
        host: &Host,
        cookies: &CookieJar,
        claims: &Self::Claims,
        body: &models::RoleGrantCreate,
    ) -> Result<V1RolesGrantsPostResponse, E>;
}
