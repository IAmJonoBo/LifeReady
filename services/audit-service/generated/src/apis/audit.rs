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
pub enum V1AuditEventsPostResponse {
    /// Appended
    Status201_Appended(models::AuditEvent),
    /// Error response
    Status0_ErrorResponse(models::V1AuditEventsPostDefaultResponse),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum V1AuditExportGetResponse {
    /// Export ready
    Status200_ExportReady(models::AuditExport),
    /// Error response
    Status0_ErrorResponse(models::V1AuditEventsPostDefaultResponse),
}

/// Audit
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Audit<E: std::fmt::Debug + Send + Sync + 'static = ()>: super::ErrorHandler<E> {
    type Claims;

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
