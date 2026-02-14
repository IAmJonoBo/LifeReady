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
pub enum V1AuditEventsPostResponse {
    /// Appended
    Status201_Appended
    {
        body: models::AuditEvent,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Error response
    Status400_ErrorResponse
    {
        body: models::V1AuditEventsPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Unauthorized
    Status401_Unauthorized
    {
        body: models::V1AuditEventsPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Forbidden
    Status403_Forbidden
    {
        body: models::V1AuditEventsPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Conflict
    Status409_Conflict
    {
        body: models::V1AuditEventsPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Unprocessable Entity
    Status422_UnprocessableEntity
    {
        body: models::V1AuditEventsPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Internal Server Error
    Status500_InternalServerError
    {
        body: models::V1AuditEventsPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum V1AuditExportGetResponse {
    /// Export ready
    Status200_ExportReady
    {
        body: models::AuditExport,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Error response
    Status400_ErrorResponse
    {
        body: models::V1AuditEventsPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Unauthorized
    Status401_Unauthorized
    {
        body: models::V1AuditEventsPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Forbidden
    Status403_Forbidden
    {
        body: models::V1AuditEventsPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Not Found
    Status404_NotFound
    {
        body: models::V1AuditEventsPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Internal Server Error
    Status500_InternalServerError
    {
        body: models::V1AuditEventsPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
}




/// Audit
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Audit<E: std::fmt::Debug + Send + Sync + 'static = ()>: super::ErrorHandler<E> {
    type Claims;

    /// Readiness check (dependencies available).
    ///
    /// ReadyzGet - GET /audit/readyz
    async fn readyz_get(
    &self,
    
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
        claims: &Self::Claims,
    ) -> Result<ReadyzGetResponse, E>;

    /// Append audit event (internal services).
    ///
    /// V1AuditEventsPost - POST /audit/v1/audit/events
    async fn v1_audit_events_post(
    &self,
    
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
        claims: &Self::Claims,
            body: &models::AuditAppend,
    ) -> Result<V1AuditEventsPostResponse, E>;

    /// Export audit proof bundle (hash head + verification inputs).
    ///
    /// V1AuditExportGet - GET /audit/v1/audit/export
    async fn v1_audit_export_get(
    &self,
    
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
        claims: &Self::Claims,
      query_params: &models::V1AuditExportGetQueryParams,
    ) -> Result<V1AuditExportGetResponse, E>;
}
