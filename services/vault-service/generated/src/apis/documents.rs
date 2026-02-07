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
pub enum V1DocumentsDocumentIdGetResponse {
    /// OK
    Status200_OK(models::Document),
    /// Error response
    Status0_ErrorResponse(models::V1DocumentsGetDefaultResponse),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum V1DocumentsDocumentIdVersionsPostResponse {
    /// Version committed
    Status201_VersionCommitted(models::DocumentVersion),
    /// Error response
    Status0_ErrorResponse(models::V1DocumentsGetDefaultResponse),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum V1DocumentsGetResponse {
    /// OK
    Status200_OK(models::DocumentList),
    /// Error response
    Status0_ErrorResponse(models::V1DocumentsGetDefaultResponse),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum V1DocumentsPostResponse {
    /// Init OK
    Status201_InitOK(models::DocumentInitResponse),
    /// Error response
    Status0_ErrorResponse(models::V1DocumentsGetDefaultResponse),
}

/// Documents
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Documents<E: std::fmt::Debug + Send + Sync + 'static = ()>:
    super::ErrorHandler<E>
{
    type Claims;

    /// Get document metadata.
    ///
    /// V1DocumentsDocumentIdGet - GET /vault/v1/documents/{document_id}
    async fn v1_documents_document_id_get(
        &self,

        method: &Method,
        host: &Host,
        cookies: &CookieJar,
        claims: &Self::Claims,
        path_params: &models::V1DocumentsDocumentIdGetPathParams,
    ) -> Result<V1DocumentsDocumentIdGetResponse, E>;

    /// Commit uploaded version (checksum required).
    ///
    /// V1DocumentsDocumentIdVersionsPost - POST /vault/v1/documents/{document_id}/versions
    async fn v1_documents_document_id_versions_post(
        &self,

        method: &Method,
        host: &Host,
        cookies: &CookieJar,
        claims: &Self::Claims,
        path_params: &models::V1DocumentsDocumentIdVersionsPostPathParams,
        body: &models::DocumentCommit,
    ) -> Result<V1DocumentsDocumentIdVersionsPostResponse, E>;

    /// List document metadata.
    ///
    /// V1DocumentsGet - GET /vault/v1/documents
    async fn v1_documents_get(
        &self,

        method: &Method,
        host: &Host,
        cookies: &CookieJar,
        claims: &Self::Claims,
        query_params: &models::V1DocumentsGetQueryParams,
    ) -> Result<V1DocumentsGetResponse, E>;

    /// Init document upload (server returns pre-signed upload target).
    ///
    /// V1DocumentsPost - POST /vault/v1/documents
    async fn v1_documents_post(
        &self,

        method: &Method,
        host: &Host,
        cookies: &CookieJar,
        claims: &Self::Claims,
        body: &models::DocumentInit,
    ) -> Result<V1DocumentsPostResponse, E>;
}
