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
pub fn check_xss_map<T>(v: &std::collections::HashMap<String, T>) -> std::result::Result<(), validator::ValidationError> {
    if v.keys().any(|k| ammonia::is_html(k)) {
        std::result::Result::Err(validator::ValidationError::new("xss detected"))
    } else {
        std::result::Result::Ok(())
    }
}




    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
    #[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
    pub struct V1AuditExportGetQueryParams {
                #[serde(rename = "case_id")]
                    #[serde(skip_serializing_if="Option::is_none")]
                    pub case_id: Option<uuid::Uuid>,
    }



#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct AuditAppend {
    #[serde(rename = "actor_principal_id")]
    pub actor_principal_id: uuid::Uuid,

    #[serde(rename = "action")]
    #[validate(
            length(max = 120),
          custom(function = "check_xss_string"),
    )]
    pub action: String,

    #[serde(rename = "tier")]
          #[validate(nested)]
    pub tier: models::SensitivityTier,

    #[serde(rename = "case_id")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub case_id: Option<uuid::Uuid>,

    #[serde(rename = "payload")]
          #[validate(custom(function = "check_xss_map"))]
    pub payload: std::collections::HashMap<String, crate::types::Object>,

}



impl AuditAppend {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(actor_principal_id: uuid::Uuid, action: String, tier: models::SensitivityTier, payload: std::collections::HashMap<String, crate::types::Object>, ) -> AuditAppend {
        AuditAppend {
 actor_principal_id,
 action,
 tier,
 case_id: None,
 payload,
        }
    }
}

/// Converts the AuditAppend value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for AuditAppend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            // Skipping actor_principal_id in query parameter serialization


            Some("action".to_string()),
            Some(self.action.to_string()),

            // Skipping tier in query parameter serialization

            // Skipping case_id in query parameter serialization

            // Skipping payload in query parameter serialization
            // Skipping payload in query parameter serialization

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a AuditAppend value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for AuditAppend {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub actor_principal_id: Vec<uuid::Uuid>,
            pub action: Vec<String>,
            pub tier: Vec<models::SensitivityTier>,
            pub case_id: Vec<uuid::Uuid>,
            pub payload: Vec<std::collections::HashMap<String, crate::types::Object>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing AuditAppend".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "actor_principal_id" => intermediate_rep.actor_principal_id.push(<uuid::Uuid as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "action" => intermediate_rep.action.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "tier" => intermediate_rep.tier.push(<models::SensitivityTier as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "case_id" => intermediate_rep.case_id.push(<uuid::Uuid as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    "payload" => return std::result::Result::Err("Parsing a container in this style is not supported in AuditAppend".to_string()),
                    _ => return std::result::Result::Err("Unexpected key while parsing AuditAppend".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(AuditAppend {
            actor_principal_id: intermediate_rep.actor_principal_id.into_iter().next().ok_or_else(|| "actor_principal_id missing in AuditAppend".to_string())?,
            action: intermediate_rep.action.into_iter().next().ok_or_else(|| "action missing in AuditAppend".to_string())?,
            tier: intermediate_rep.tier.into_iter().next().ok_or_else(|| "tier missing in AuditAppend".to_string())?,
            case_id: intermediate_rep.case_id.into_iter().next(),
            payload: intermediate_rep.payload.into_iter().next().ok_or_else(|| "payload missing in AuditAppend".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<AuditAppend> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<AuditAppend>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<AuditAppend>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for AuditAppend - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<AuditAppend> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <AuditAppend as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into AuditAppend - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}



#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct AuditEvent {
    #[serde(rename = "event_id")]
    pub event_id: uuid::Uuid,

    #[serde(rename = "created_at")]
    pub created_at: chrono::DateTime::<chrono::Utc>,

    #[serde(rename = "prev_hash")]
    #[validate(
            regex(path = *RE_AUDITEVENT_PREV_HASH),
          custom(function = "check_xss_string"),
    )]
    pub prev_hash: String,

    #[serde(rename = "event_hash")]
    #[validate(
            regex(path = *RE_AUDITEVENT_EVENT_HASH),
          custom(function = "check_xss_string"),
    )]
    pub event_hash: String,

}


lazy_static::lazy_static! {
    static ref RE_AUDITEVENT_PREV_HASH: regex::Regex = regex::Regex::new("^[a-f0-9]{64}$").unwrap();
}
lazy_static::lazy_static! {
    static ref RE_AUDITEVENT_EVENT_HASH: regex::Regex = regex::Regex::new("^[a-f0-9]{64}$").unwrap();
}

impl AuditEvent {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(event_id: uuid::Uuid, created_at: chrono::DateTime::<chrono::Utc>, prev_hash: String, event_hash: String, ) -> AuditEvent {
        AuditEvent {
 event_id,
 created_at,
 prev_hash,
 event_hash,
        }
    }
}

/// Converts the AuditEvent value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for AuditEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            // Skipping event_id in query parameter serialization

            // Skipping created_at in query parameter serialization


            Some("prev_hash".to_string()),
            Some(self.prev_hash.to_string()),


            Some("event_hash".to_string()),
            Some(self.event_hash.to_string()),

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a AuditEvent value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for AuditEvent {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub event_id: Vec<uuid::Uuid>,
            pub created_at: Vec<chrono::DateTime::<chrono::Utc>>,
            pub prev_hash: Vec<String>,
            pub event_hash: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing AuditEvent".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "event_id" => intermediate_rep.event_id.push(<uuid::Uuid as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "created_at" => intermediate_rep.created_at.push(<chrono::DateTime::<chrono::Utc> as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "prev_hash" => intermediate_rep.prev_hash.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "event_hash" => intermediate_rep.event_hash.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing AuditEvent".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(AuditEvent {
            event_id: intermediate_rep.event_id.into_iter().next().ok_or_else(|| "event_id missing in AuditEvent".to_string())?,
            created_at: intermediate_rep.created_at.into_iter().next().ok_or_else(|| "created_at missing in AuditEvent".to_string())?,
            prev_hash: intermediate_rep.prev_hash.into_iter().next().ok_or_else(|| "prev_hash missing in AuditEvent".to_string())?,
            event_hash: intermediate_rep.event_hash.into_iter().next().ok_or_else(|| "event_hash missing in AuditEvent".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<AuditEvent> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<AuditEvent>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<AuditEvent>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for AuditEvent - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<AuditEvent> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <AuditEvent as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into AuditEvent - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}



#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct AuditExport {
    #[serde(rename = "download_url")]
          #[validate(custom(function = "check_xss_string"))]
    pub download_url: String,

    #[serde(rename = "expires_at")]
    pub expires_at: chrono::DateTime::<chrono::Utc>,

    #[serde(rename = "head_hash")]
    #[validate(
            regex(path = *RE_AUDITEXPORT_HEAD_HASH),
          custom(function = "check_xss_string"),
    )]
    pub head_hash: String,

    #[serde(rename = "events_sha256")]
    #[validate(
            regex(path = *RE_AUDITEXPORT_EVENTS_SHA256),
          custom(function = "check_xss_string"),
    )]
    pub events_sha256: String,

}


lazy_static::lazy_static! {
    static ref RE_AUDITEXPORT_HEAD_HASH: regex::Regex = regex::Regex::new("^[a-f0-9]{64}$").unwrap();
}
lazy_static::lazy_static! {
    static ref RE_AUDITEXPORT_EVENTS_SHA256: regex::Regex = regex::Regex::new("^[a-f0-9]{64}$").unwrap();
}

impl AuditExport {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(download_url: String, expires_at: chrono::DateTime::<chrono::Utc>, head_hash: String, events_sha256: String, ) -> AuditExport {
        AuditExport {
 download_url,
 expires_at,
 head_hash,
 events_sha256,
        }
    }
}

/// Converts the AuditExport value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for AuditExport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![

            Some("download_url".to_string()),
            Some(self.download_url.to_string()),

            // Skipping expires_at in query parameter serialization


            Some("head_hash".to_string()),
            Some(self.head_hash.to_string()),


            Some("events_sha256".to_string()),
            Some(self.events_sha256.to_string()),

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a AuditExport value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for AuditExport {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub download_url: Vec<String>,
            pub expires_at: Vec<chrono::DateTime::<chrono::Utc>>,
            pub head_hash: Vec<String>,
            pub events_sha256: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing AuditExport".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "download_url" => intermediate_rep.download_url.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "expires_at" => intermediate_rep.expires_at.push(<chrono::DateTime::<chrono::Utc> as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "head_hash" => intermediate_rep.head_hash.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "events_sha256" => intermediate_rep.events_sha256.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing AuditExport".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(AuditExport {
            download_url: intermediate_rep.download_url.into_iter().next().ok_or_else(|| "download_url missing in AuditExport".to_string())?,
            expires_at: intermediate_rep.expires_at.into_iter().next().ok_or_else(|| "expires_at missing in AuditExport".to_string())?,
            head_hash: intermediate_rep.head_hash.into_iter().next().ok_or_else(|| "head_hash missing in AuditExport".to_string())?,
            events_sha256: intermediate_rep.events_sha256.into_iter().next().ok_or_else(|| "events_sha256 missing in AuditExport".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<AuditExport> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<AuditExport>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<AuditExport>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for AuditExport - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<AuditExport> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <AuditExport as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into AuditExport - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}



#[derive(Debug, Clone, PartialEq, PartialOrd,  serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct IsoDateTime(pub chrono::DateTime::<chrono::Utc>);

impl validator::Validate for IsoDateTime {
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {

        std::result::Result::Ok(())
    }
}

impl std::convert::From<chrono::DateTime::<chrono::Utc>> for IsoDateTime {
    fn from(x: chrono::DateTime::<chrono::Utc>) -> Self {
        IsoDateTime(x)
    }
}

impl std::convert::From<IsoDateTime> for chrono::DateTime::<chrono::Utc> {
    fn from(x: IsoDateTime) -> Self {
        x.0
    }
}

impl std::ops::Deref for IsoDateTime {
    type Target = chrono::DateTime::<chrono::Utc>;
    fn deref(&self) -> &chrono::DateTime::<chrono::Utc> {
        &self.0
    }
}

impl std::ops::DerefMut for IsoDateTime {
    fn deref_mut(&mut self) -> &mut chrono::DateTime::<chrono::Utc> {
        &mut self.0
    }
}



#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ReadyzGet200Response {
    /// Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "status")]
          #[validate(custom(function = "check_xss_string"))]
    pub status: String,

}



impl ReadyzGet200Response {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(status: String, ) -> ReadyzGet200Response {
        ReadyzGet200Response {
 status,
        }
    }
}

/// Converts the ReadyzGet200Response value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for ReadyzGet200Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![

            Some("status".to_string()),
            Some(self.status.to_string()),

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ReadyzGet200Response value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ReadyzGet200Response {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub status: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing ReadyzGet200Response".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "status" => intermediate_rep.status.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing ReadyzGet200Response".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ReadyzGet200Response {
            status: intermediate_rep.status.into_iter().next().ok_or_else(|| "status missing in ReadyzGet200Response".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ReadyzGet200Response> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<ReadyzGet200Response>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<ReadyzGet200Response>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for ReadyzGet200Response - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<ReadyzGet200Response> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <ReadyzGet200Response as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into ReadyzGet200Response - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}



/// Enumeration of values.
/// Since this enum's variants do not hold data, we can easily define them as `#[repr(C)]`
/// which helps with FFI.
#[allow(non_camel_case_types, clippy::large_enum_variant)]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk_enum_derive::LabelledGenericEnum))]
pub enum SensitivityTier {
    #[serde(rename = "green")]
    Green,
    #[serde(rename = "amber")]
    Amber,
    #[serde(rename = "red")]
    Red,
}

impl validator::Validate for SensitivityTier
{
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {
        std::result::Result::Ok(())
    }
}

impl std::fmt::Display for SensitivityTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            SensitivityTier::Green => write!(f, "green"),
            SensitivityTier::Amber => write!(f, "amber"),
            SensitivityTier::Red => write!(f, "red"),
        }
    }
}

impl std::str::FromStr for SensitivityTier {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "green" => std::result::Result::Ok(SensitivityTier::Green),
            "amber" => std::result::Result::Ok(SensitivityTier::Amber),
            "red" => std::result::Result::Ok(SensitivityTier::Red),
            _ => std::result::Result::Err(format!(r#"Value not valid: {s}"#)),
        }
    }
}


#[derive(Debug, Clone, PartialEq, PartialOrd,  serde::Serialize, serde::Deserialize)]
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
pub struct V1AuditEventsPost400Response {
    #[serde(rename = "type")]
          #[validate(custom(function = "check_xss_string"))]
    pub r_type: String,

    #[serde(rename = "title")]
    #[validate(
            length(max = 200),
          custom(function = "check_xss_string"),
    )]
    pub title: String,

    #[serde(rename = "status")]
    #[validate(
            range(min = 100u16, max = 599u16),
    )]
    pub status: u16,

    #[serde(rename = "detail")]
    #[validate(
            length(max = 2000),
          custom(function = "check_xss_string"),
    )]
    #[serde(skip_serializing_if="Option::is_none")]
    pub detail: Option<String>,

    #[serde(rename = "instance")]
          #[validate(custom(function = "check_xss_string"))]
    #[serde(skip_serializing_if="Option::is_none")]
    pub instance: Option<String>,

    #[serde(rename = "request_id")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub request_id: Option<uuid::Uuid>,

    #[serde(rename = "errors")]
          #[validate(custom(function = "check_xss_map"))]
    #[serde(skip_serializing_if="Option::is_none")]
    pub r_errors: Option<std::collections::HashMap<String, Vec<String>>>,

}



impl V1AuditEventsPost400Response {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(r_type: String, title: String, status: u16, ) -> V1AuditEventsPost400Response {
        V1AuditEventsPost400Response {
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

/// Converts the V1AuditEventsPost400Response value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for V1AuditEventsPost400Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![

            Some("type".to_string()),
            Some(self.r_type.to_string()),


            Some("title".to_string()),
            Some(self.title.to_string()),


            Some("status".to_string()),
            Some(self.status.to_string()),


            self.detail.as_ref().map(|detail| {
                [
                    "detail".to_string(),
                    detail.to_string(),
                ].join(",")
            }),


            self.instance.as_ref().map(|instance| {
                [
                    "instance".to_string(),
                    instance.to_string(),
                ].join(",")
            }),

            // Skipping request_id in query parameter serialization

            // Skipping errors in query parameter serialization
            // Skipping errors in query parameter serialization

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a V1AuditEventsPost400Response value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for V1AuditEventsPost400Response {
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
                None => return std::result::Result::Err("Missing value while parsing V1AuditEventsPost400Response".to_string())
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
                    "errors" => return std::result::Result::Err("Parsing a container in this style is not supported in V1AuditEventsPost400Response".to_string()),
                    _ => return std::result::Result::Err("Unexpected key while parsing V1AuditEventsPost400Response".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(V1AuditEventsPost400Response {
            r_type: intermediate_rep.r_type.into_iter().next().ok_or_else(|| "type missing in V1AuditEventsPost400Response".to_string())?,
            title: intermediate_rep.title.into_iter().next().ok_or_else(|| "title missing in V1AuditEventsPost400Response".to_string())?,
            status: intermediate_rep.status.into_iter().next().ok_or_else(|| "status missing in V1AuditEventsPost400Response".to_string())?,
            detail: intermediate_rep.detail.into_iter().next(),
            instance: intermediate_rep.instance.into_iter().next(),
            request_id: intermediate_rep.request_id.into_iter().next(),
            r_errors: intermediate_rep.r_errors.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<V1AuditEventsPost400Response> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<V1AuditEventsPost400Response>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<V1AuditEventsPost400Response>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for V1AuditEventsPost400Response - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<V1AuditEventsPost400Response> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <V1AuditEventsPost400Response as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into V1AuditEventsPost400Response - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}


