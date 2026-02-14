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
pub enum V1CasesCaseIdEvidenceSlotNamePutResponse {
    /// Updated
    Status200_Updated
    {
        body: models::EvidenceSlot,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Error response
    Status400_ErrorResponse
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Unauthorized
    Status401_Unauthorized
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Forbidden
    Status403_Forbidden
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Not Found
    Status404_NotFound
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Conflict
    Status409_Conflict
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Unprocessable Entity
    Status422_UnprocessableEntity
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Internal Server Error
    Status500_InternalServerError
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum V1CasesCaseIdExportPostResponse {
    /// Export ready
    Status200_ExportReady
    {
        body: models::ExportResponse,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Error response
    Status400_ErrorResponse
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Unauthorized
    Status401_Unauthorized
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Forbidden
    Status403_Forbidden
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Not Found
    Status404_NotFound
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Conflict
    Status409_Conflict
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Unprocessable Entity
    Status422_UnprocessableEntity
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Internal Server Error
    Status500_InternalServerError
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum V1CasesCaseIdLinkPostResponse {
    /// Link issued
    Status200_LinkIssued
    {
        body: models::LinkResponse,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Error response
    Status400_ErrorResponse
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Unauthorized
    Status401_Unauthorized
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Forbidden
    Status403_Forbidden
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Not Found
    Status404_NotFound
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Conflict
    Status409_Conflict
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Internal Server Error
    Status500_InternalServerError
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum V1CasesCaseIdPatchResponse {
    /// Updated
    Status200_Updated
    {
        body: models::Case,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Error response
    Status400_ErrorResponse
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Unauthorized
    Status401_Unauthorized
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Forbidden
    Status403_Forbidden
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Not Found
    Status404_NotFound
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Conflict
    Status409_Conflict
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Unprocessable Entity
    Status422_UnprocessableEntity
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Internal Server Error
    Status500_InternalServerError
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum V1CasesCaseIdRevokePostResponse {
    /// Revoked
    Status200_Revoked
    {
        body: models::RevokeResponse,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Error response
    Status400_ErrorResponse
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Unauthorized
    Status401_Unauthorized
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Forbidden
    Status403_Forbidden
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Not Found
    Status404_NotFound
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Conflict
    Status409_Conflict
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Internal Server Error
    Status500_InternalServerError
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum V1CasesCaseIdTransitionPostResponse {
    /// Transition successful
    Status200_TransitionSuccessful
    {
        body: models::TransitionResponse,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Error response
    Status400_ErrorResponse
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Unauthorized
    Status401_Unauthorized
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Forbidden
    Status403_Forbidden
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Not Found
    Status404_NotFound
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Conflict
    Status409_Conflict
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Internal Server Error
    Status500_InternalServerError
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum V1CasesDeathReadinessPostResponse {
    /// Created
    Status201_Created
    {
        body: models::Case,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Error response
    Status400_ErrorResponse
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Unauthorized
    Status401_Unauthorized
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Forbidden
    Status403_Forbidden
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Conflict
    Status409_Conflict
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Unprocessable Entity
    Status422_UnprocessableEntity
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Internal Server Error
    Status500_InternalServerError
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum V1CasesDeceasedEstateSaPostResponse {
    /// Created
    Status201_Created
    {
        body: models::Case,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Error response
    Status400_ErrorResponse
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Unauthorized
    Status401_Unauthorized
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Forbidden
    Status403_Forbidden
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Conflict
    Status409_Conflict
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Unprocessable Entity
    Status422_UnprocessableEntity
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Internal Server Error
    Status500_InternalServerError
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum V1CasesEmergencyPackPostResponse {
    /// Created
    Status201_Created
    {
        body: models::Case,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Error response
    Status400_ErrorResponse
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Unauthorized
    Status401_Unauthorized
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Forbidden
    Status403_Forbidden
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Conflict
    Status409_Conflict
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Unprocessable Entity
    Status422_UnprocessableEntity
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Internal Server Error
    Status500_InternalServerError
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum V1CasesMhca39PostResponse {
    /// Created
    Status201_Created
    {
        body: models::Case,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Error response
    Status400_ErrorResponse
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Unauthorized
    Status401_Unauthorized
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Forbidden
    Status403_Forbidden
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Conflict
    Status409_Conflict
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Unprocessable Entity
    Status422_UnprocessableEntity
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Internal Server Error
    Status500_InternalServerError
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum V1CasesPopiaIncidentPostResponse {
    /// Created
    Status201_Created
    {
        body: models::Case,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Error response
    Status400_ErrorResponse
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Unauthorized
    Status401_Unauthorized
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Forbidden
    Status403_Forbidden
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Conflict
    Status409_Conflict
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Unprocessable Entity
    Status422_UnprocessableEntity
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Internal Server Error
    Status500_InternalServerError
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum V1CasesWillPrepSaPostResponse {
    /// Created
    Status201_Created
    {
        body: models::Case,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Error response
    Status400_ErrorResponse
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Unauthorized
    Status401_Unauthorized
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Forbidden
    Status403_Forbidden
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Conflict
    Status409_Conflict
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Unprocessable Entity
    Status422_UnprocessableEntity
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
    ,
    /// Internal Server Error
    Status500_InternalServerError
    {
        body: models::V1CasesEmergencyPackPost400Response,
        x_request_id:
        Option<
        uuid::Uuid
        >
    }
}




/// Cases
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Cases<E: std::fmt::Debug + Send + Sync + 'static = ()>: super::ErrorHandler<E> {
    type Claims;

    /// Readiness check (dependencies available).
    ///
    /// ReadyzGet - GET /case/readyz
    async fn readyz_get(
    &self,
    
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
        claims: &Self::Claims,
    ) -> Result<ReadyzGetResponse, E>;

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

    /// Issue a time-limited share link for the case pack.
    ///
    /// V1CasesCaseIdLinkPost - POST /case/v1/cases/{case_id}/link
    async fn v1_cases_case_id_link_post(
    &self,
    
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
        claims: &Self::Claims,
      path_params: &models::V1CasesCaseIdLinkPostPathParams,
            body: &models::LinkRequest,
    ) -> Result<V1CasesCaseIdLinkPostResponse, E>;

    /// Update case metadata (append-only revision for incidents).
    ///
    /// V1CasesCaseIdPatch - PATCH /case/v1/cases/{case_id}
    async fn v1_cases_case_id_patch(
    &self,
    
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
        claims: &Self::Claims,
      path_params: &models::V1CasesCaseIdPatchPathParams,
            body: &models::CaseUpdate,
    ) -> Result<V1CasesCaseIdPatchResponse, E>;

    /// Revoke an issued share link immediately.
    ///
    /// V1CasesCaseIdRevokePost - POST /case/v1/cases/{case_id}/revoke
    async fn v1_cases_case_id_revoke_post(
    &self,
    
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
        claims: &Self::Claims,
      path_params: &models::V1CasesCaseIdRevokePostPathParams,
    ) -> Result<V1CasesCaseIdRevokePostResponse, E>;

    /// Advance case through workflow state machine.
    ///
    /// V1CasesCaseIdTransitionPost - POST /case/v1/cases/{case_id}/transition
    async fn v1_cases_case_id_transition_post(
    &self,
    
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
        claims: &Self::Claims,
      path_params: &models::V1CasesCaseIdTransitionPostPathParams,
            body: &models::TransitionRequest,
    ) -> Result<V1CasesCaseIdTransitionPostResponse, E>;

    /// Create Death Readiness case (executor nominee semantics).
    ///
    /// V1CasesDeathReadinessPost - POST /case/v1/cases/death-readiness
    async fn v1_cases_death_readiness_post(
    &self,
    
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
        claims: &Self::Claims,
            body: &models::DeathReadinessCreate,
    ) -> Result<V1CasesDeathReadinessPostResponse, E>;

    /// Create Deceased Estate Reporting case (SA).
    ///
    /// V1CasesDeceasedEstateSaPost - POST /case/v1/cases/deceased-estate-sa
    async fn v1_cases_deceased_estate_sa_post(
    &self,
    
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
        claims: &Self::Claims,
            body: &models::DeceasedEstateCreate,
    ) -> Result<V1CasesDeceasedEstateSaPostResponse, E>;

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

    /// Create POPIA security compromise incident.
    ///
    /// V1CasesPopiaIncidentPost - POST /case/v1/cases/popia-incident
    async fn v1_cases_popia_incident_post(
    &self,
    
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
        claims: &Self::Claims,
            body: &models::PopiaIncidentCreate,
    ) -> Result<V1CasesPopiaIncidentPostResponse, E>;

    /// Create Will Preparation case (SA witnessing workflow).
    ///
    /// V1CasesWillPrepSaPost - POST /case/v1/cases/will-prep-sa
    async fn v1_cases_will_prep_sa_post(
    &self,
    
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
        claims: &Self::Claims,
            body: &models::WillPrepCreate,
    ) -> Result<V1CasesWillPrepSaPostResponse, E>;
}
