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
pub enum V1AssetsPostResponse {
    /// Created
    Status201_Created(models::Asset),
    /// Error response
    Status0_ErrorResponse(models::V1PeopleGetDefaultResponse),
}

/// Assets
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Assets<E: std::fmt::Debug + Send + Sync + 'static = ()>: super::ErrorHandler<E> {
    type Claims;

    /// Create asset.
    ///
    /// V1AssetsPost - POST /estate/v1/assets
    async fn v1_assets_post(
        &self,

        method: &Method,
        host: &Host,
        cookies: &CookieJar,
        claims: &Self::Claims,
        body: &models::AssetCreate,
    ) -> Result<V1AssetsPostResponse, E>;
}
