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
pub enum V1PeopleGetResponse {
    /// OK
    Status200_OK(models::V1PeopleGet200Response),
    /// Error response
    Status0_ErrorResponse(models::V1PeopleGetDefaultResponse),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum V1PeoplePostResponse {
    /// Created
    Status201_Created(models::Person),
    /// Error response
    Status0_ErrorResponse(models::V1PeopleGetDefaultResponse),
}

/// People
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait People<E: std::fmt::Debug + Send + Sync + 'static = ()>: super::ErrorHandler<E> {
    type Claims;

    /// List people.
    ///
    /// V1PeopleGet - GET /estate/v1/people
    async fn v1_people_get(
        &self,

        method: &Method,
        host: &Host,
        cookies: &CookieJar,
        claims: &Self::Claims,
        query_params: &models::V1PeopleGetQueryParams,
    ) -> Result<V1PeopleGetResponse, E>;

    /// Create person (contact/role target).
    ///
    /// V1PeoplePost - POST /estate/v1/people
    async fn v1_people_post(
        &self,

        method: &Method,
        host: &Host,
        cookies: &CookieJar,
        claims: &Self::Claims,
        body: &models::PersonCreate,
    ) -> Result<V1PeoplePostResponse, E>;
}
