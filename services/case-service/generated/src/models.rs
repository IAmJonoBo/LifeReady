#![allow(unused_qualifications)]

use http::HeaderValue;
use validator::Validate;

#[cfg(feature = "server")]
use crate::header;
use crate::{models, types::*};

#[allow(dead_code)]
fn from_validation_error(e: validator::ValidationError) -> validator::ValidationErrors {
    let mut errs = validator::ValidationErrors::new();
    errs.add("na", e);
    errs
}

#[allow(dead_code)]
pub fn check_xss_string(v: &str) -> std::result::Result<(), validator::ValidationError> {
    if ammonia::is_html(v) {
        std::result::Result::Err(validator::ValidationError::new("xss detected"))
    } else {
        std::result::Result::Ok(())
    }
}

#[allow(dead_code)]
pub fn check_xss_vec_string(v: &[String]) -> std::result::Result<(), validator::ValidationError> {
    if v.iter().any(|i| ammonia::is_html(i)) {
        std::result::Result::Err(validator::ValidationError::new("xss detected"))
    } else {
        std::result::Result::Ok(())
    }
}

#[allow(dead_code)]
pub fn check_xss_map_string(
    v: &std::collections::HashMap<String, String>,
) -> std::result::Result<(), validator::ValidationError> {
    if v.keys().any(|k| ammonia::is_html(k)) || v.values().any(|v| ammonia::is_html(v)) {
        std::result::Result::Err(validator::ValidationError::new("xss detected"))
    } else {
        std::result::Result::Ok(())
    }
}

#[allow(dead_code)]
pub fn check_xss_map_nested<T>(
    v: &std::collections::HashMap<String, T>,
) -> std::result::Result<(), validator::ValidationError>
where
    T: validator::Validate,
{
    if v.keys().any(|k| ammonia::is_html(k)) || v.values().any(|v| v.validate().is_err()) {
        std::result::Result::Err(validator::ValidationError::new("xss detected"))
    } else {
        std::result::Result::Ok(())
    }
}

#[allow(dead_code)]
pub fn check_xss_map<T>(
    v: &std::collections::HashMap<String, T>,
) -> std::result::Result<(), validator::ValidationError> {
    if v.keys().any(|k| ammonia::is_html(k)) {
        std::result::Result::Err(validator::ValidationError::new("xss detected"))
    } else {
        std::result::Result::Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct V1CasesCaseIdEvidenceSlotNamePutPathParams {
    pub case_id: uuid::Uuid,
    #[validate(length(max = 120))]
    pub slot_name: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct V1CasesCaseIdExportPostPathParams {
    pub case_id: uuid::Uuid,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Case {
    #[serde(rename = "case_id")]
    pub case_id: uuid::Uuid,

    #[serde(rename = "case_type")]
    #[validate(nested)]
    pub case_type: models::CaseType,

    #[serde(rename = "status")]
    #[validate(nested)]
    pub status: models::CaseStatus,

    #[serde(rename = "created_at")]
    pub created_at: chrono::DateTime<chrono::Utc>,

    #[serde(rename = "blocked_reasons")]
    #[validate(length(max = 20), custom(function = "check_xss_vec_string"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_reasons: Option<Vec<String>>,
}

impl Case {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(
        case_id: uuid::Uuid,
        case_type: models::CaseType,
        status: models::CaseStatus,
        created_at: chrono::DateTime<chrono::Utc>,
    ) -> Case {
        Case {
            case_id,
            case_type,
            status,
            created_at,
            blocked_reasons: None,
        }
    }
}

/// Converts the Case value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for Case {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            // Skipping case_id in query parameter serialization

            // Skipping case_type in query parameter serialization

            // Skipping status in query parameter serialization

            // Skipping created_at in query parameter serialization
            self.blocked_reasons.as_ref().map(|blocked_reasons| {
                [
                    "blocked_reasons".to_string(),
                    blocked_reasons
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(","),
                ]
                .join(",")
            }),
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Case value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Case {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub case_id: Vec<uuid::Uuid>,
            pub case_type: Vec<models::CaseType>,
            pub status: Vec<models::CaseStatus>,
            pub created_at: Vec<chrono::DateTime<chrono::Utc>>,
            pub blocked_reasons: Vec<Vec<String>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err("Missing value while parsing Case".to_string())
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "case_id" => intermediate_rep.case_id.push(
                        <uuid::Uuid as std::str::FromStr>::from_str(val)
                            .map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "case_type" => intermediate_rep.case_type.push(
                        <models::CaseType as std::str::FromStr>::from_str(val)
                            .map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "status" => intermediate_rep.status.push(
                        <models::CaseStatus as std::str::FromStr>::from_str(val)
                            .map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "created_at" => intermediate_rep.created_at.push(
                        <chrono::DateTime<chrono::Utc> as std::str::FromStr>::from_str(val)
                            .map_err(|x| x.to_string())?,
                    ),
                    "blocked_reasons" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in Case"
                                .to_string(),
                        )
                    }
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing Case".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Case {
            case_id: intermediate_rep
                .case_id
                .into_iter()
                .next()
                .ok_or_else(|| "case_id missing in Case".to_string())?,
            case_type: intermediate_rep
                .case_type
                .into_iter()
                .next()
                .ok_or_else(|| "case_type missing in Case".to_string())?,
            status: intermediate_rep
                .status
                .into_iter()
                .next()
                .ok_or_else(|| "status missing in Case".to_string())?,
            created_at: intermediate_rep
                .created_at
                .into_iter()
                .next()
                .ok_or_else(|| "created_at missing in Case".to_string())?,
            blocked_reasons: intermediate_rep.blocked_reasons.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Case> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<Case>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<Case>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                r#"Invalid header value for Case - value: {hdr_value} is invalid {e}"#
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<Case> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => match <Case as std::str::FromStr>::from_str(value) {
                std::result::Result::Ok(value) => {
                    std::result::Result::Ok(header::IntoHeaderValue(value))
                }
                std::result::Result::Err(err) => std::result::Result::Err(format!(
                    r#"Unable to convert header value '{value}' into Case - {err}"#
                )),
            },
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                r#"Unable to convert header: {hdr_value:?} to string: {e}"#
            )),
        }
    }
}

/// Enumeration of values.
/// Since this enum's variants do not hold data, we can easily define them as `#[repr(C)]`
/// which helps with FFI.
#[allow(non_camel_case_types, clippy::large_enum_variant)]
#[repr(C)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[cfg_attr(feature = "conversion", derive(frunk_enum_derive::LabelledGenericEnum))]
pub enum CaseStatus {
    #[serde(rename = "draft")]
    Draft,
    #[serde(rename = "ready")]
    Ready,
    #[serde(rename = "blocked")]
    Blocked,
    #[serde(rename = "exported")]
    Exported,
    #[serde(rename = "closed")]
    Closed,
    #[serde(rename = "revoked")]
    Revoked,
}

impl validator::Validate for CaseStatus {
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {
        std::result::Result::Ok(())
    }
}

impl std::fmt::Display for CaseStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            CaseStatus::Draft => write!(f, "draft"),
            CaseStatus::Ready => write!(f, "ready"),
            CaseStatus::Blocked => write!(f, "blocked"),
            CaseStatus::Exported => write!(f, "exported"),
            CaseStatus::Closed => write!(f, "closed"),
            CaseStatus::Revoked => write!(f, "revoked"),
        }
    }
}

impl std::str::FromStr for CaseStatus {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "draft" => std::result::Result::Ok(CaseStatus::Draft),
            "ready" => std::result::Result::Ok(CaseStatus::Ready),
            "blocked" => std::result::Result::Ok(CaseStatus::Blocked),
            "exported" => std::result::Result::Ok(CaseStatus::Exported),
            "closed" => std::result::Result::Ok(CaseStatus::Closed),
            "revoked" => std::result::Result::Ok(CaseStatus::Revoked),
            _ => std::result::Result::Err(format!(r#"Value not valid: {s}"#)),
        }
    }
}

/// Enumeration of values.
/// Since this enum's variants do not hold data, we can easily define them as `#[repr(C)]`
/// which helps with FFI.
#[allow(non_camel_case_types, clippy::large_enum_variant)]
#[repr(C)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[cfg_attr(feature = "conversion", derive(frunk_enum_derive::LabelledGenericEnum))]
pub enum CaseType {
    #[serde(rename = "emergency_pack")]
    EmergencyPack,
    #[serde(rename = "mhca39")]
    Mhca39,
    #[serde(rename = "death_readiness")]
    DeathReadiness,
}

impl validator::Validate for CaseType {
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {
        std::result::Result::Ok(())
    }
}

impl std::fmt::Display for CaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            CaseType::EmergencyPack => write!(f, "emergency_pack"),
            CaseType::Mhca39 => write!(f, "mhca39"),
            CaseType::DeathReadiness => write!(f, "death_readiness"),
        }
    }
}

impl std::str::FromStr for CaseType {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "emergency_pack" => std::result::Result::Ok(CaseType::EmergencyPack),
            "mhca39" => std::result::Result::Ok(CaseType::Mhca39),
            "death_readiness" => std::result::Result::Ok(CaseType::DeathReadiness),
            _ => std::result::Result::Err(format!(r#"Value not valid: {s}"#)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct EmergencyPackRequest {
    #[serde(rename = "directive_document_ids")]
    #[validate(length(min = 1, max = 50), nested)]
    pub directive_document_ids: Vec<models::Uuid>,

    #[serde(rename = "emergency_contacts")]
    #[validate(length(min = 1, max = 10), nested)]
    pub emergency_contacts: Vec<models::EmergencyPackRequestEmergencyContactsInner>,
}

impl EmergencyPackRequest {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(
        directive_document_ids: Vec<models::Uuid>,
        emergency_contacts: Vec<models::EmergencyPackRequestEmergencyContactsInner>,
    ) -> EmergencyPackRequest {
        EmergencyPackRequest {
            directive_document_ids,
            emergency_contacts,
        }
    }
}

/// Converts the EmergencyPackRequest value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for EmergencyPackRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            // Skipping directive_document_ids in query parameter serialization

            // Skipping emergency_contacts in query parameter serialization

        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a EmergencyPackRequest value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for EmergencyPackRequest {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub directive_document_ids: Vec<Vec<models::Uuid>>,
            pub emergency_contacts: Vec<Vec<models::EmergencyPackRequestEmergencyContactsInner>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing EmergencyPackRequest".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    "directive_document_ids" => return std::result::Result::Err("Parsing a container in this style is not supported in EmergencyPackRequest".to_string()),
                    "emergency_contacts" => return std::result::Result::Err("Parsing a container in this style is not supported in EmergencyPackRequest".to_string()),
                    _ => return std::result::Result::Err("Unexpected key while parsing EmergencyPackRequest".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(EmergencyPackRequest {
            directive_document_ids: intermediate_rep
                .directive_document_ids
                .into_iter()
                .next()
                .ok_or_else(|| {
                    "directive_document_ids missing in EmergencyPackRequest".to_string()
                })?,
            emergency_contacts: intermediate_rep
                .emergency_contacts
                .into_iter()
                .next()
                .ok_or_else(|| "emergency_contacts missing in EmergencyPackRequest".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<EmergencyPackRequest> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<EmergencyPackRequest>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<EmergencyPackRequest>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                r#"Invalid header value for EmergencyPackRequest - value: {hdr_value} is invalid {e}"#
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<EmergencyPackRequest> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <EmergencyPackRequest as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        r#"Unable to convert header value '{value}' into EmergencyPackRequest - {err}"#
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                r#"Unable to convert header: {hdr_value:?} to string: {e}"#
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct EmergencyPackRequestEmergencyContactsInner {
    #[serde(rename = "name")]
    #[validate(length(max = 120), custom(function = "check_xss_string"))]
    pub name: String,

    #[serde(rename = "phone_e164")]
    #[validate(length(max = 20), custom(function = "check_xss_string"))]
    pub phone_e164: String,

    #[serde(rename = "relationship")]
    #[validate(length(max = 80), custom(function = "check_xss_string"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relationship: Option<String>,
}

impl EmergencyPackRequestEmergencyContactsInner {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(name: String, phone_e164: String) -> EmergencyPackRequestEmergencyContactsInner {
        EmergencyPackRequestEmergencyContactsInner {
            name,
            phone_e164,
            relationship: None,
        }
    }
}

/// Converts the EmergencyPackRequestEmergencyContactsInner value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for EmergencyPackRequestEmergencyContactsInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            Some("name".to_string()),
            Some(self.name.to_string()),
            Some("phone_e164".to_string()),
            Some(self.phone_e164.to_string()),
            self.relationship.as_ref().map(|relationship| {
                ["relationship".to_string(), relationship.to_string()].join(",")
            }),
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a EmergencyPackRequestEmergencyContactsInner value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for EmergencyPackRequestEmergencyContactsInner {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub name: Vec<String>,
            pub phone_e164: Vec<String>,
            pub relationship: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val =
                match string_iter.next() {
                    Some(x) => x,
                    None => return std::result::Result::Err(
                        "Missing value while parsing EmergencyPackRequestEmergencyContactsInner"
                            .to_string(),
                    ),
                };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "name" => intermediate_rep.name.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "phone_e164" => intermediate_rep.phone_e164.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "relationship" => intermediate_rep.relationship.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    _ => return std::result::Result::Err(
                        "Unexpected key while parsing EmergencyPackRequestEmergencyContactsInner"
                            .to_string(),
                    ),
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(EmergencyPackRequestEmergencyContactsInner {
            name: intermediate_rep.name.into_iter().next().ok_or_else(|| {
                "name missing in EmergencyPackRequestEmergencyContactsInner".to_string()
            })?,
            phone_e164: intermediate_rep
                .phone_e164
                .into_iter()
                .next()
                .ok_or_else(|| {
                    "phone_e164 missing in EmergencyPackRequestEmergencyContactsInner".to_string()
                })?,
            relationship: intermediate_rep.relationship.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<EmergencyPackRequestEmergencyContactsInner> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<EmergencyPackRequestEmergencyContactsInner>>
    for HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<EmergencyPackRequestEmergencyContactsInner>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                r#"Invalid header value for EmergencyPackRequestEmergencyContactsInner - value: {hdr_value} is invalid {e}"#
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue>
    for header::IntoHeaderValue<EmergencyPackRequestEmergencyContactsInner>
{
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <EmergencyPackRequestEmergencyContactsInner as std::str::FromStr>::from_str(
                    value,
                ) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        r#"Unable to convert header value '{value}' into EmergencyPackRequestEmergencyContactsInner - {err}"#
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                r#"Unable to convert header: {hdr_value:?} to string: {e}"#
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct EvidenceAttach {
    #[serde(rename = "document_id")]
    pub document_id: uuid::Uuid,
}

impl EvidenceAttach {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(document_id: uuid::Uuid) -> EvidenceAttach {
        EvidenceAttach { document_id }
    }
}

/// Converts the EvidenceAttach value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for EvidenceAttach {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            // Skipping document_id in query parameter serialization

        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a EvidenceAttach value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for EvidenceAttach {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub document_id: Vec<uuid::Uuid>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing EvidenceAttach".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "document_id" => intermediate_rep.document_id.push(
                        <uuid::Uuid as std::str::FromStr>::from_str(val)
                            .map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing EvidenceAttach".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(EvidenceAttach {
            document_id: intermediate_rep
                .document_id
                .into_iter()
                .next()
                .ok_or_else(|| "document_id missing in EvidenceAttach".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<EvidenceAttach> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<EvidenceAttach>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<EvidenceAttach>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                r#"Invalid header value for EvidenceAttach - value: {hdr_value} is invalid {e}"#
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<EvidenceAttach> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <EvidenceAttach as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        r#"Unable to convert header value '{value}' into EvidenceAttach - {err}"#
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                r#"Unable to convert header: {hdr_value:?} to string: {e}"#
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct EvidenceSlot {
    #[serde(rename = "slot_name")]
    #[validate(custom(function = "check_xss_string"))]
    pub slot_name: String,

    #[serde(rename = "document_id")]
    pub document_id: uuid::Uuid,

    #[serde(rename = "added_at")]
    pub added_at: chrono::DateTime<chrono::Utc>,
}

impl EvidenceSlot {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(
        slot_name: String,
        document_id: uuid::Uuid,
        added_at: chrono::DateTime<chrono::Utc>,
    ) -> EvidenceSlot {
        EvidenceSlot {
            slot_name,
            document_id,
            added_at,
        }
    }
}

/// Converts the EvidenceSlot value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for EvidenceSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            Some("slot_name".to_string()),
            Some(self.slot_name.to_string()),
            // Skipping document_id in query parameter serialization

            // Skipping added_at in query parameter serialization
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a EvidenceSlot value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for EvidenceSlot {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub slot_name: Vec<String>,
            pub document_id: Vec<uuid::Uuid>,
            pub added_at: Vec<chrono::DateTime<chrono::Utc>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing EvidenceSlot".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "slot_name" => intermediate_rep.slot_name.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "document_id" => intermediate_rep.document_id.push(
                        <uuid::Uuid as std::str::FromStr>::from_str(val)
                            .map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "added_at" => intermediate_rep.added_at.push(
                        <chrono::DateTime<chrono::Utc> as std::str::FromStr>::from_str(val)
                            .map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing EvidenceSlot".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(EvidenceSlot {
            slot_name: intermediate_rep
                .slot_name
                .into_iter()
                .next()
                .ok_or_else(|| "slot_name missing in EvidenceSlot".to_string())?,
            document_id: intermediate_rep
                .document_id
                .into_iter()
                .next()
                .ok_or_else(|| "document_id missing in EvidenceSlot".to_string())?,
            added_at: intermediate_rep
                .added_at
                .into_iter()
                .next()
                .ok_or_else(|| "added_at missing in EvidenceSlot".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<EvidenceSlot> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<EvidenceSlot>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<EvidenceSlot>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                r#"Invalid header value for EvidenceSlot - value: {hdr_value} is invalid {e}"#
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<EvidenceSlot> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <EvidenceSlot as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        r#"Unable to convert header value '{value}' into EvidenceSlot - {err}"#
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                r#"Unable to convert header: {hdr_value:?} to string: {e}"#
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ExportResponse {
    #[serde(rename = "download_url")]
    #[validate(custom(function = "check_xss_string"))]
    pub download_url: String,

    #[serde(rename = "expires_at")]
    pub expires_at: chrono::DateTime<chrono::Utc>,

    #[serde(rename = "manifest_sha256")]
    #[validate(
            regex(path = *RE_EXPORTRESPONSE_MANIFEST_SHA256),
          custom(function = "check_xss_string"),
    )]
    pub manifest_sha256: String,
}

lazy_static::lazy_static! {
    static ref RE_EXPORTRESPONSE_MANIFEST_SHA256: regex::Regex = regex::Regex::new("^[a-f0-9]{64}$").unwrap();
}

impl ExportResponse {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(
        download_url: String,
        expires_at: chrono::DateTime<chrono::Utc>,
        manifest_sha256: String,
    ) -> ExportResponse {
        ExportResponse {
            download_url,
            expires_at,
            manifest_sha256,
        }
    }
}

/// Converts the ExportResponse value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for ExportResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            Some("download_url".to_string()),
            Some(self.download_url.to_string()),
            // Skipping expires_at in query parameter serialization
            Some("manifest_sha256".to_string()),
            Some(self.manifest_sha256.to_string()),
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ExportResponse value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ExportResponse {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub download_url: Vec<String>,
            pub expires_at: Vec<chrono::DateTime<chrono::Utc>>,
            pub manifest_sha256: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing ExportResponse".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "download_url" => intermediate_rep.download_url.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "expires_at" => intermediate_rep.expires_at.push(
                        <chrono::DateTime<chrono::Utc> as std::str::FromStr>::from_str(val)
                            .map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "manifest_sha256" => intermediate_rep.manifest_sha256.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing ExportResponse".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ExportResponse {
            download_url: intermediate_rep
                .download_url
                .into_iter()
                .next()
                .ok_or_else(|| "download_url missing in ExportResponse".to_string())?,
            expires_at: intermediate_rep
                .expires_at
                .into_iter()
                .next()
                .ok_or_else(|| "expires_at missing in ExportResponse".to_string())?,
            manifest_sha256: intermediate_rep
                .manifest_sha256
                .into_iter()
                .next()
                .ok_or_else(|| "manifest_sha256 missing in ExportResponse".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ExportResponse> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<ExportResponse>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<ExportResponse>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                r#"Invalid header value for ExportResponse - value: {hdr_value} is invalid {e}"#
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<ExportResponse> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <ExportResponse as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        r#"Unable to convert header value '{value}' into ExportResponse - {err}"#
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                r#"Unable to convert header: {hdr_value:?} to string: {e}"#
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct IsoDateTime(pub chrono::DateTime<chrono::Utc>);

impl validator::Validate for IsoDateTime {
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {
        std::result::Result::Ok(())
    }
}

impl std::convert::From<chrono::DateTime<chrono::Utc>> for IsoDateTime {
    fn from(x: chrono::DateTime<chrono::Utc>) -> Self {
        IsoDateTime(x)
    }
}

impl std::convert::From<IsoDateTime> for chrono::DateTime<chrono::Utc> {
    fn from(x: IsoDateTime) -> Self {
        x.0
    }
}

impl std::ops::Deref for IsoDateTime {
    type Target = chrono::DateTime<chrono::Utc>;
    fn deref(&self) -> &chrono::DateTime<chrono::Utc> {
        &self.0
    }
}

impl std::ops::DerefMut for IsoDateTime {
    fn deref_mut(&mut self) -> &mut chrono::DateTime<chrono::Utc> {
        &mut self.0
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Mhca39Create {
    #[serde(rename = "subject_person_id")]
    pub subject_person_id: uuid::Uuid,

    #[serde(rename = "applicant_person_id")]
    pub applicant_person_id: uuid::Uuid,

    #[serde(rename = "relationship_to_subject")]
    #[validate(length(max = 80), custom(function = "check_xss_string"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relationship_to_subject: Option<String>,

    #[serde(rename = "notes")]
    #[validate(length(max = 4000), custom(function = "check_xss_string"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,

    #[serde(rename = "required_evidence_slots")]
    #[validate(length(max = 20), custom(function = "check_xss_vec_string"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_evidence_slots: Option<Vec<String>>,
}

impl Mhca39Create {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(subject_person_id: uuid::Uuid, applicant_person_id: uuid::Uuid) -> Mhca39Create {
        Mhca39Create {
            subject_person_id,
            applicant_person_id,
            relationship_to_subject: None,
            notes: None,
            required_evidence_slots: None,
        }
    }
}

/// Converts the Mhca39Create value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for Mhca39Create {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            // Skipping subject_person_id in query parameter serialization

            // Skipping applicant_person_id in query parameter serialization
            self.relationship_to_subject
                .as_ref()
                .map(|relationship_to_subject| {
                    [
                        "relationship_to_subject".to_string(),
                        relationship_to_subject.to_string(),
                    ]
                    .join(",")
                }),
            self.notes
                .as_ref()
                .map(|notes| ["notes".to_string(), notes.to_string()].join(",")),
            self.required_evidence_slots
                .as_ref()
                .map(|required_evidence_slots| {
                    [
                        "required_evidence_slots".to_string(),
                        required_evidence_slots
                            .iter()
                            .map(|x| x.to_string())
                            .collect::<Vec<_>>()
                            .join(","),
                    ]
                    .join(",")
                }),
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Mhca39Create value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Mhca39Create {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub subject_person_id: Vec<uuid::Uuid>,
            pub applicant_person_id: Vec<uuid::Uuid>,
            pub relationship_to_subject: Vec<String>,
            pub notes: Vec<String>,
            pub required_evidence_slots: Vec<Vec<String>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing Mhca39Create".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "subject_person_id" => intermediate_rep.subject_person_id.push(
                        <uuid::Uuid as std::str::FromStr>::from_str(val)
                            .map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "applicant_person_id" => intermediate_rep.applicant_person_id.push(
                        <uuid::Uuid as std::str::FromStr>::from_str(val)
                            .map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "relationship_to_subject" => intermediate_rep.relationship_to_subject.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "notes" => intermediate_rep.notes.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    "required_evidence_slots" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in Mhca39Create"
                                .to_string(),
                        )
                    }
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing Mhca39Create".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Mhca39Create {
            subject_person_id: intermediate_rep
                .subject_person_id
                .into_iter()
                .next()
                .ok_or_else(|| "subject_person_id missing in Mhca39Create".to_string())?,
            applicant_person_id: intermediate_rep
                .applicant_person_id
                .into_iter()
                .next()
                .ok_or_else(|| "applicant_person_id missing in Mhca39Create".to_string())?,
            relationship_to_subject: intermediate_rep.relationship_to_subject.into_iter().next(),
            notes: intermediate_rep.notes.into_iter().next(),
            required_evidence_slots: intermediate_rep.required_evidence_slots.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Mhca39Create> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<Mhca39Create>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<Mhca39Create>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                r#"Invalid header value for Mhca39Create - value: {hdr_value} is invalid {e}"#
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<Mhca39Create> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <Mhca39Create as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        r#"Unable to convert header value '{value}' into Mhca39Create - {err}"#
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                r#"Unable to convert header: {hdr_value:?} to string: {e}"#
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Uuid(pub uuid::Uuid);

impl validator::Validate for Uuid {
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {
        std::result::Result::Ok(())
    }
}

impl std::convert::From<uuid::Uuid> for Uuid {
    fn from(x: uuid::Uuid) -> Self {
        Uuid(x)
    }
}

impl std::convert::From<Uuid> for uuid::Uuid {
    fn from(x: Uuid) -> Self {
        x.0
    }
}

impl std::ops::Deref for Uuid {
    type Target = uuid::Uuid;
    fn deref(&self) -> &uuid::Uuid {
        &self.0
    }
}

impl std::ops::DerefMut for Uuid {
    fn deref_mut(&mut self) -> &mut uuid::Uuid {
        &mut self.0
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct V1CasesEmergencyPackPost400Response {
    #[serde(rename = "type")]
    #[validate(custom(function = "check_xss_string"))]
    pub r_type: String,

    #[serde(rename = "title")]
    #[validate(length(max = 200), custom(function = "check_xss_string"))]
    pub title: String,

    #[serde(rename = "status")]
    #[validate(range(min = 100u16, max = 599u16))]
    pub status: u16,

    #[serde(rename = "detail")]
    #[validate(length(max = 2000), custom(function = "check_xss_string"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,

    #[serde(rename = "instance")]
    #[validate(custom(function = "check_xss_string"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,

    #[serde(rename = "request_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<uuid::Uuid>,

    #[serde(rename = "errors")]
    #[validate(custom(function = "check_xss_map"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r_errors: Option<std::collections::HashMap<String, Vec<String>>>,
}

impl V1CasesEmergencyPackPost400Response {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(r_type: String, title: String, status: u16) -> V1CasesEmergencyPackPost400Response {
        V1CasesEmergencyPackPost400Response {
            r_type,
            title,
            status,
            detail: None,
            instance: None,
            request_id: None,
            r_errors: None,
        }
    }
}

/// Converts the V1CasesEmergencyPackPost400Response value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for V1CasesEmergencyPackPost400Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            Some("type".to_string()),
            Some(self.r_type.to_string()),
            Some("title".to_string()),
            Some(self.title.to_string()),
            Some("status".to_string()),
            Some(self.status.to_string()),
            self.detail
                .as_ref()
                .map(|detail| ["detail".to_string(), detail.to_string()].join(",")),
            self.instance
                .as_ref()
                .map(|instance| ["instance".to_string(), instance.to_string()].join(",")),
            // Skipping request_id in query parameter serialization

            // Skipping errors in query parameter serialization
            // Skipping errors in query parameter serialization
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a V1CasesEmergencyPackPost400Response value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for V1CasesEmergencyPackPost400Response {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub r_type: Vec<String>,
            pub title: Vec<String>,
            pub status: Vec<u16>,
            pub detail: Vec<String>,
            pub instance: Vec<String>,
            pub request_id: Vec<uuid::Uuid>,
            pub r_errors: Vec<std::collections::HashMap<String, Vec<String>>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing V1CasesEmergencyPackPost400Response"
                            .to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "type" => intermediate_rep.r_type.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "title" => intermediate_rep.title.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "status" => intermediate_rep.status.push(<u16 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "detail" => intermediate_rep.detail.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "instance" => intermediate_rep.instance.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "request_id" => intermediate_rep.request_id.push(<uuid::Uuid as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    "errors" => return std::result::Result::Err("Parsing a container in this style is not supported in V1CasesEmergencyPackPost400Response".to_string()),
                    _ => return std::result::Result::Err("Unexpected key while parsing V1CasesEmergencyPackPost400Response".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(V1CasesEmergencyPackPost400Response {
            r_type: intermediate_rep
                .r_type
                .into_iter()
                .next()
                .ok_or_else(|| "type missing in V1CasesEmergencyPackPost400Response".to_string())?,
            title: intermediate_rep.title.into_iter().next().ok_or_else(|| {
                "title missing in V1CasesEmergencyPackPost400Response".to_string()
            })?,
            status: intermediate_rep.status.into_iter().next().ok_or_else(|| {
                "status missing in V1CasesEmergencyPackPost400Response".to_string()
            })?,
            detail: intermediate_rep.detail.into_iter().next(),
            instance: intermediate_rep.instance.into_iter().next(),
            request_id: intermediate_rep.request_id.into_iter().next(),
            r_errors: intermediate_rep.r_errors.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<V1CasesEmergencyPackPost400Response> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<V1CasesEmergencyPackPost400Response>>
    for HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<V1CasesEmergencyPackPost400Response>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                r#"Invalid header value for V1CasesEmergencyPackPost400Response - value: {hdr_value} is invalid {e}"#
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue>
    for header::IntoHeaderValue<V1CasesEmergencyPackPost400Response>
{
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <V1CasesEmergencyPackPost400Response as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        r#"Unable to convert header value '{value}' into V1CasesEmergencyPackPost400Response - {err}"#
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                r#"Unable to convert header: {hdr_value:?} to string: {e}"#
            )),
        }
    }
}
