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
pub enum V1InstructionsPostResponse {
    /// Created
    Status201_Created(models::Instruction),
    /// Error response
    Status0_ErrorResponse(models::V1PeopleGetDefaultResponse),
}

/// Instructions
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Instructions<E: std::fmt::Debug + Send + Sync + 'static = ()>:
    super::ErrorHandler<E>
{
    type Claims;

    /// Create instruction.
    ///
    /// V1InstructionsPost - POST /estate/v1/instructions
    async fn v1_instructions_post(
        &self,

        method: &Method,
        host: &Host,
        cookies: &CookieJar,
        claims: &Self::Claims,
        body: &models::InstructionCreate,
    ) -> Result<V1InstructionsPostResponse, E>;
}
