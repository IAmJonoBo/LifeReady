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
    Status200_OK {
        body: models::V1PeopleGet200Response,
        x_request_id: Option<uuid::Uuid>,
    },
    /// Error response
    Status400_ErrorResponse {
        body: models::V1PeopleGet400Response,
        x_request_id: Option<uuid::Uuid>,
    },
    /// Unauthorized
    Status401_Unauthorized {
        body: models::V1PeopleGet400Response,
        x_request_id: Option<uuid::Uuid>,
    },
    /// Forbidden
    Status403_Forbidden {
        body: models::V1PeopleGet400Response,
        x_request_id: Option<uuid::Uuid>,
    },
    /// Internal Server Error
    Status500_InternalServerError {
        body: models::V1PeopleGet400Response,
        x_request_id: Option<uuid::Uuid>,
    },
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum V1PeoplePostResponse {
    /// Created
    Status201_Created {
        body: models::Person,
        x_request_id: Option<uuid::Uuid>,
    },
    /// Error response
    Status400_ErrorResponse {
        body: models::V1PeopleGet400Response,
        x_request_id: Option<uuid::Uuid>,
    },
    /// Unauthorized
    Status401_Unauthorized {
        body: models::V1PeopleGet400Response,
        x_request_id: Option<uuid::Uuid>,
    },
    /// Forbidden
    Status403_Forbidden {
        body: models::V1PeopleGet400Response,
        x_request_id: Option<uuid::Uuid>,
    },
    /// Conflict
    Status409_Conflict {
        body: models::V1PeopleGet400Response,
        x_request_id: Option<uuid::Uuid>,
    },
    /// Unprocessable Entity
    Status422_UnprocessableEntity {
        body: models::V1PeopleGet400Response,
        x_request_id: Option<uuid::Uuid>,
    },
    /// Internal Server Error
    Status500_InternalServerError {
        body: models::V1PeopleGet400Response,
        x_request_id: Option<uuid::Uuid>,
    },
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
