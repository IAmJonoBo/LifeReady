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
pub enum V1DocumentsDocumentIdDownloadGetResponse {
    /// Document bytes
    Status200_DocumentBytes
    {
        body: ByteArray,
        x_request_id:
        Option<
        uuid::Uuid
        >
        ,
        content_disposition:
        Option<
        String
        >
        ,
        x_document_sha256:
        Option<
        String
        >
    }
    ,
    /// Error response
    Status400_ErrorResponse
    {
        body: models::V1DocumentsGet400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Unauthorized
    Status401_Unauthorized
    {
        body: models::V1DocumentsGet400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Forbidden
    Status403_Forbidden
    {
        body: models::V1DocumentsGet400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Not Found
    Status404_NotFound
    {
        body: models::V1DocumentsGet400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Internal Server Error
    Status500_InternalServerError
    {
        body: models::V1DocumentsGet400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum V1DocumentsDocumentIdGetResponse {
    /// OK
    Status200_OK
    {
        body: models::Document,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Error response
    Status400_ErrorResponse
    {
        body: models::V1DocumentsGet400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Unauthorized
    Status401_Unauthorized
    {
        body: models::V1DocumentsGet400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Forbidden
    Status403_Forbidden
    {
        body: models::V1DocumentsGet400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Not Found
    Status404_NotFound
    {
        body: models::V1DocumentsGet400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Internal Server Error
    Status500_InternalServerError
    {
        body: models::V1DocumentsGet400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum V1DocumentsDocumentIdVersionsPostResponse {
    /// Version committed
    Status201_VersionCommitted
    {
        body: models::DocumentVersion,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Error response
    Status400_ErrorResponse
    {
        body: models::V1DocumentsGet400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Unauthorized
    Status401_Unauthorized
    {
        body: models::V1DocumentsGet400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Forbidden
    Status403_Forbidden
    {
        body: models::V1DocumentsGet400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Not Found
    Status404_NotFound
    {
        body: models::V1DocumentsGet400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Conflict
    Status409_Conflict
    {
        body: models::V1DocumentsGet400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Unprocessable Entity
    Status422_UnprocessableEntity
    {
        body: models::V1DocumentsGet400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Internal Server Error
    Status500_InternalServerError
    {
        body: models::V1DocumentsGet400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum V1DocumentsGetResponse {
    /// OK
    Status200_OK
    {
        body: models::DocumentList,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Error response
    Status400_ErrorResponse
    {
        body: models::V1DocumentsGet400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Unauthorized
    Status401_Unauthorized
    {
        body: models::V1DocumentsGet400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Forbidden
    Status403_Forbidden
    {
        body: models::V1DocumentsGet400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Internal Server Error
    Status500_InternalServerError
    {
        body: models::V1DocumentsGet400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum V1DocumentsPostResponse {
    /// Init OK
    Status201_InitOK
    {
        body: models::DocumentInitResponse,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Error response
    Status400_ErrorResponse
    {
        body: models::V1DocumentsGet400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Unauthorized
    Status401_Unauthorized
    {
        body: models::V1DocumentsGet400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Forbidden
    Status403_Forbidden
    {
        body: models::V1DocumentsGet400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Unprocessable Entity
    Status422_UnprocessableEntity
    {
        body: models::V1DocumentsGet400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Internal Server Error
    Status500_InternalServerError
    {
        body: models::V1DocumentsGet400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
}




/// Documents
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Documents<E: std::fmt::Debug + Send + Sync + 'static = ()>: super::ErrorHandler<E> {
    type Claims;

    /// Readiness check (dependencies available).
    ///
    /// ReadyzGet - GET /vault/readyz
    async fn readyz_get(
    &self,
    
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
        claims: &Self::Claims,
    ) -> Result<ReadyzGetResponse, E>;

    /// Download document content (re-verifies sha256 on read).
    ///
    /// V1DocumentsDocumentIdDownloadGet - GET /vault/v1/documents/{document_id}/download
    async fn v1_documents_document_id_download_get(
    &self,
    
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
        claims: &Self::Claims,
      path_params: &models::V1DocumentsDocumentIdDownloadGetPathParams,
      query_params: &models::V1DocumentsDocumentIdDownloadGetQueryParams,
    ) -> Result<V1DocumentsDocumentIdDownloadGetResponse, E>;

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
