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
pub enum V1CasesCaseIdEvidenceSlotNamePutResponse {
    /// Updated
    Status200_Updated {
        body: models::EvidenceSlot,
        x_request_id: Option<uuid::Uuid>,
    },
    /// Error response
    Status400_ErrorResponse {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id: Option<uuid::Uuid>,
    },
    /// Unauthorized
    Status401_Unauthorized {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id: Option<uuid::Uuid>,
    },
    /// Forbidden
    Status403_Forbidden {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id: Option<uuid::Uuid>,
    },
    /// Not Found
    Status404_NotFound {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id: Option<uuid::Uuid>,
    },
    /// Conflict
    Status409_Conflict {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id: Option<uuid::Uuid>,
    },
    /// Unprocessable Entity
    Status422_UnprocessableEntity {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id: Option<uuid::Uuid>,
    },
    /// Internal Server Error
    Status500_InternalServerError {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id: Option<uuid::Uuid>,
    },
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum V1CasesCaseIdExportPostResponse {
    /// Export ready
    Status200_ExportReady {
        body: models::ExportResponse,
        x_request_id: Option<uuid::Uuid>,
    },
    /// Error response
    Status400_ErrorResponse {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id: Option<uuid::Uuid>,
    },
    /// Unauthorized
    Status401_Unauthorized {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id: Option<uuid::Uuid>,
    },
    /// Forbidden
    Status403_Forbidden {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id: Option<uuid::Uuid>,
    },
    /// Not Found
    Status404_NotFound {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id: Option<uuid::Uuid>,
    },
    /// Conflict
    Status409_Conflict {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id: Option<uuid::Uuid>,
    },
    /// Unprocessable Entity
    Status422_UnprocessableEntity {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id: Option<uuid::Uuid>,
    },
    /// Internal Server Error
    Status500_InternalServerError {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id: Option<uuid::Uuid>,
    },
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum V1CasesEmergencyPackPostResponse {
    /// Created
    Status201_Created {
        body: models::Case,
        x_request_id: Option<uuid::Uuid>,
    },
    /// Error response
    Status400_ErrorResponse {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id: Option<uuid::Uuid>,
    },
    /// Unauthorized
    Status401_Unauthorized {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id: Option<uuid::Uuid>,
    },
    /// Forbidden
    Status403_Forbidden {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id: Option<uuid::Uuid>,
    },
    /// Conflict
    Status409_Conflict {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id: Option<uuid::Uuid>,
    },
    /// Unprocessable Entity
    Status422_UnprocessableEntity {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id: Option<uuid::Uuid>,
    },
    /// Internal Server Error
    Status500_InternalServerError {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id: Option<uuid::Uuid>,
    },
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum V1CasesMhca39PostResponse {
    /// Created
    Status201_Created {
        body: models::Case,
        x_request_id: Option<uuid::Uuid>,
    },
    /// Error response
    Status400_ErrorResponse {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id: Option<uuid::Uuid>,
    },
    /// Unauthorized
    Status401_Unauthorized {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id: Option<uuid::Uuid>,
    },
    /// Forbidden
    Status403_Forbidden {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id: Option<uuid::Uuid>,
    },
    /// Conflict
    Status409_Conflict {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id: Option<uuid::Uuid>,
    },
    /// Unprocessable Entity
    Status422_UnprocessableEntity {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id: Option<uuid::Uuid>,
    },
    /// Internal Server Error
    Status500_InternalServerError {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id: Option<uuid::Uuid>,
    },
}

/// Cases
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Cases<E: std::fmt::Debug + Send + Sync + 'static = ()>: super::ErrorHandler<E> {
    type Claims;

    /// Attach evidence document to a slot.
    ///
    /// V1CasesCaseIdEvidenceSlotNamePut - PUT /case/v1/cases/{case_id}/evidence/{slot_name}
    async fn v1_cases_case_id_evidence_slot_name_put(
        &self,

        method: &Method,
        host: &Host,
        cookies: &CookieJar,
        claims: &Self::Claims,
        path_params: &models::V1CasesCaseIdEvidenceSlotNamePutPathParams,
        body: &models::EvidenceAttach,
    ) -> Result<V1CasesCaseIdEvidenceSlotNamePutResponse, E>;

    /// Export submission pack (blocked until requirements met).
    ///
    /// V1CasesCaseIdExportPost - POST /case/v1/cases/{case_id}/export
    async fn v1_cases_case_id_export_post(
        &self,

        method: &Method,
        host: &Host,
        cookies: &CookieJar,
        claims: &Self::Claims,
        path_params: &models::V1CasesCaseIdExportPostPathParams,
    ) -> Result<V1CasesCaseIdExportPostResponse, E>;

    /// Create or refresh Emergency Directive Pack.
    ///
    /// V1CasesEmergencyPackPost - POST /case/v1/cases/emergency-pack
    async fn v1_cases_emergency_pack_post(
        &self,

        method: &Method,
        host: &Host,
        cookies: &CookieJar,
        claims: &Self::Claims,
        body: &models::EmergencyPackRequest,
    ) -> Result<V1CasesEmergencyPackPostResponse, E>;

    /// Create MHCA 39 case (guided evidence pack; no incapacity claims).
    ///
    /// V1CasesMhca39Post - POST /case/v1/cases/mhca39
    async fn v1_cases_mhca39_post(
        &self,

        method: &Method,
        host: &Host,
        cookies: &CookieJar,
        claims: &Self::Claims,
        body: &models::Mhca39Create,
    ) -> Result<V1CasesMhca39PostResponse, E>;
}
