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
pub struct V1DocumentsDocumentIdGetPathParams {
    pub document_id: uuid::Uuid,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct V1DocumentsDocumentIdVersionsPostPathParams {
    pub document_id: uuid::Uuid,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct V1DocumentsGetQueryParams {
    #[serde(rename = "limit")]
    #[validate(range(min = 1i32, max = 200i32))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Document {
    #[serde(rename = "document_id")]
    pub document_id: uuid::Uuid,

    #[serde(rename = "document_type")]
    #[validate(nested)]
    pub document_type: models::DocumentType,

    #[serde(rename = "title")]
    #[validate(custom(function = "check_xss_string"))]
    pub title: String,

    #[serde(rename = "sensitivity")]
    #[validate(nested)]
    pub sensitivity: models::SensitivityTier,

    #[serde(rename = "tags")]
    #[validate(length(max = 20), custom(function = "check_xss_vec_string"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,

    #[serde(rename = "created_at")]
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl Document {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(
        document_id: uuid::Uuid,
        document_type: models::DocumentType,
        title: String,
        sensitivity: models::SensitivityTier,
        created_at: chrono::DateTime<chrono::Utc>,
    ) -> Document {
        Document {
            document_id,
            document_type,
            title,
            sensitivity,
            tags: None,
            created_at,
        }
    }
}

/// Converts the Document value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for Document {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            // Skipping document_id in query parameter serialization

            // Skipping document_type in query parameter serialization
            Some("title".to_string()),
            Some(self.title.to_string()),
            // Skipping sensitivity in query parameter serialization
            self.tags.as_ref().map(|tags| {
                [
                    "tags".to_string(),
                    tags.iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(","),
                ]
                .join(",")
            }),
            // Skipping created_at in query parameter serialization
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Document value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Document {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub document_id: Vec<uuid::Uuid>,
            pub document_type: Vec<models::DocumentType>,
            pub title: Vec<String>,
            pub sensitivity: Vec<models::SensitivityTier>,
            pub tags: Vec<Vec<String>>,
            pub created_at: Vec<chrono::DateTime<chrono::Utc>>,
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
                        "Missing value while parsing Document".to_string(),
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
                    #[allow(clippy::redundant_clone)]
                    "document_type" => intermediate_rep.document_type.push(
                        <models::DocumentType as std::str::FromStr>::from_str(val)
                            .map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "title" => intermediate_rep.title.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "sensitivity" => intermediate_rep.sensitivity.push(
                        <models::SensitivityTier as std::str::FromStr>::from_str(val)
                            .map_err(|x| x.to_string())?,
                    ),
                    "tags" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in Document"
                                .to_string(),
                        )
                    }
                    #[allow(clippy::redundant_clone)]
                    "created_at" => intermediate_rep.created_at.push(
                        <chrono::DateTime<chrono::Utc> as std::str::FromStr>::from_str(val)
                            .map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing Document".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Document {
            document_id: intermediate_rep
                .document_id
                .into_iter()
                .next()
                .ok_or_else(|| "document_id missing in Document".to_string())?,
            document_type: intermediate_rep
                .document_type
                .into_iter()
                .next()
                .ok_or_else(|| "document_type missing in Document".to_string())?,
            title: intermediate_rep
                .title
                .into_iter()
                .next()
                .ok_or_else(|| "title missing in Document".to_string())?,
            sensitivity: intermediate_rep
                .sensitivity
                .into_iter()
                .next()
                .ok_or_else(|| "sensitivity missing in Document".to_string())?,
            tags: intermediate_rep.tags.into_iter().next(),
            created_at: intermediate_rep
                .created_at
                .into_iter()
                .next()
                .ok_or_else(|| "created_at missing in Document".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Document> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<Document>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<Document>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                r#"Invalid header value for Document - value: {hdr_value} is invalid {e}"#
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<Document> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <Document as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        r#"Unable to convert header value '{value}' into Document - {err}"#
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
pub struct DocumentCommit {
    #[serde(rename = "blob_ref")]
    #[validate(length(max = 512), custom(function = "check_xss_string"))]
    pub blob_ref: String,

    #[serde(rename = "sha256")]
    #[validate(
            regex(path = *RE_DOCUMENTCOMMIT_SHA256),
          custom(function = "check_xss_string"),
    )]
    pub sha256: String,

    #[serde(rename = "byte_size")]
    #[validate(range(min = 1u32, max = 52428800u32))]
    pub byte_size: u32,

    #[serde(rename = "mime_type")]
    #[validate(length(max = 80), custom(function = "check_xss_string"))]
    pub mime_type: String,
}

lazy_static::lazy_static! {
    static ref RE_DOCUMENTCOMMIT_SHA256: regex::Regex = regex::Regex::new("^[a-f0-9]{64}$").unwrap();
}

impl DocumentCommit {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(
        blob_ref: String,
        sha256: String,
        byte_size: u32,
        mime_type: String,
    ) -> DocumentCommit {
        DocumentCommit {
            blob_ref,
            sha256,
            byte_size,
            mime_type,
        }
    }
}

/// Converts the DocumentCommit value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for DocumentCommit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            Some("blob_ref".to_string()),
            Some(self.blob_ref.to_string()),
            Some("sha256".to_string()),
            Some(self.sha256.to_string()),
            Some("byte_size".to_string()),
            Some(self.byte_size.to_string()),
            Some("mime_type".to_string()),
            Some(self.mime_type.to_string()),
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a DocumentCommit value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for DocumentCommit {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub blob_ref: Vec<String>,
            pub sha256: Vec<String>,
            pub byte_size: Vec<u32>,
            pub mime_type: Vec<String>,
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
                        "Missing value while parsing DocumentCommit".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "blob_ref" => intermediate_rep.blob_ref.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "sha256" => intermediate_rep.sha256.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "byte_size" => intermediate_rep.byte_size.push(
                        <u32 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "mime_type" => intermediate_rep.mime_type.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing DocumentCommit".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(DocumentCommit {
            blob_ref: intermediate_rep
                .blob_ref
                .into_iter()
                .next()
                .ok_or_else(|| "blob_ref missing in DocumentCommit".to_string())?,
            sha256: intermediate_rep
                .sha256
                .into_iter()
                .next()
                .ok_or_else(|| "sha256 missing in DocumentCommit".to_string())?,
            byte_size: intermediate_rep
                .byte_size
                .into_iter()
                .next()
                .ok_or_else(|| "byte_size missing in DocumentCommit".to_string())?,
            mime_type: intermediate_rep
                .mime_type
                .into_iter()
                .next()
                .ok_or_else(|| "mime_type missing in DocumentCommit".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<DocumentCommit> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<DocumentCommit>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<DocumentCommit>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                r#"Invalid header value for DocumentCommit - value: {hdr_value} is invalid {e}"#
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<DocumentCommit> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <DocumentCommit as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        r#"Unable to convert header value '{value}' into DocumentCommit - {err}"#
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
pub struct DocumentInit {
    #[serde(rename = "document_type")]
    #[validate(nested)]
    pub document_type: models::DocumentType,

    #[serde(rename = "title")]
    #[validate(length(max = 160), custom(function = "check_xss_string"))]
    pub title: String,

    #[serde(rename = "sensitivity")]
    #[validate(nested)]
    pub sensitivity: models::SensitivityTier,

    #[serde(rename = "tags")]
    #[validate(length(max = 20), custom(function = "check_xss_vec_string"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

impl DocumentInit {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(
        document_type: models::DocumentType,
        title: String,
        sensitivity: models::SensitivityTier,
    ) -> DocumentInit {
        DocumentInit {
            document_type,
            title,
            sensitivity,
            tags: None,
        }
    }
}

/// Converts the DocumentInit value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for DocumentInit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            // Skipping document_type in query parameter serialization
            Some("title".to_string()),
            Some(self.title.to_string()),
            // Skipping sensitivity in query parameter serialization
            self.tags.as_ref().map(|tags| {
                [
                    "tags".to_string(),
                    tags.iter()
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

/// Converts Query Parameters representation (style=form, explode=false) to a DocumentInit value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for DocumentInit {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub document_type: Vec<models::DocumentType>,
            pub title: Vec<String>,
            pub sensitivity: Vec<models::SensitivityTier>,
            pub tags: Vec<Vec<String>>,
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
                        "Missing value while parsing DocumentInit".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "document_type" => intermediate_rep.document_type.push(
                        <models::DocumentType as std::str::FromStr>::from_str(val)
                            .map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "title" => intermediate_rep.title.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "sensitivity" => intermediate_rep.sensitivity.push(
                        <models::SensitivityTier as std::str::FromStr>::from_str(val)
                            .map_err(|x| x.to_string())?,
                    ),
                    "tags" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in DocumentInit"
                                .to_string(),
                        )
                    }
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing DocumentInit".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(DocumentInit {
            document_type: intermediate_rep
                .document_type
                .into_iter()
                .next()
                .ok_or_else(|| "document_type missing in DocumentInit".to_string())?,
            title: intermediate_rep
                .title
                .into_iter()
                .next()
                .ok_or_else(|| "title missing in DocumentInit".to_string())?,
            sensitivity: intermediate_rep
                .sensitivity
                .into_iter()
                .next()
                .ok_or_else(|| "sensitivity missing in DocumentInit".to_string())?,
            tags: intermediate_rep.tags.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<DocumentInit> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<DocumentInit>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<DocumentInit>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                r#"Invalid header value for DocumentInit - value: {hdr_value} is invalid {e}"#
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<DocumentInit> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <DocumentInit as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        r#"Unable to convert header value '{value}' into DocumentInit - {err}"#
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
pub struct DocumentInitResponse {
    #[serde(rename = "document_id")]
    pub document_id: uuid::Uuid,

    #[serde(rename = "upload_url")]
    #[validate(custom(function = "check_xss_string"))]
    pub upload_url: String,

    #[serde(rename = "upload_headers")]
    #[validate(custom(function = "check_xss_map_string"))]
    pub upload_headers: std::collections::HashMap<String, String>,
}

impl DocumentInitResponse {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(
        document_id: uuid::Uuid,
        upload_url: String,
        upload_headers: std::collections::HashMap<String, String>,
    ) -> DocumentInitResponse {
        DocumentInitResponse {
            document_id,
            upload_url,
            upload_headers,
        }
    }
}

/// Converts the DocumentInitResponse value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for DocumentInitResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            // Skipping document_id in query parameter serialization
            Some("upload_url".to_string()),
            Some(self.upload_url.to_string()),
            // Skipping upload_headers in query parameter serialization
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a DocumentInitResponse value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for DocumentInitResponse {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub document_id: Vec<uuid::Uuid>,
            pub upload_url: Vec<String>,
            pub upload_headers: Vec<std::collections::HashMap<String, String>>,
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
                        "Missing value while parsing DocumentInitResponse".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "document_id" => intermediate_rep.document_id.push(<uuid::Uuid as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "upload_url" => intermediate_rep.upload_url.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    "upload_headers" => return std::result::Result::Err("Parsing a container in this style is not supported in DocumentInitResponse".to_string()),
                    _ => return std::result::Result::Err("Unexpected key while parsing DocumentInitResponse".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(DocumentInitResponse {
            document_id: intermediate_rep
                .document_id
                .into_iter()
                .next()
                .ok_or_else(|| "document_id missing in DocumentInitResponse".to_string())?,
            upload_url: intermediate_rep
                .upload_url
                .into_iter()
                .next()
                .ok_or_else(|| "upload_url missing in DocumentInitResponse".to_string())?,
            upload_headers: intermediate_rep
                .upload_headers
                .into_iter()
                .next()
                .ok_or_else(|| "upload_headers missing in DocumentInitResponse".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<DocumentInitResponse> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<DocumentInitResponse>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<DocumentInitResponse>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                r#"Invalid header value for DocumentInitResponse - value: {hdr_value} is invalid {e}"#
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<DocumentInitResponse> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <DocumentInitResponse as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        r#"Unable to convert header value '{value}' into DocumentInitResponse - {err}"#
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
pub struct DocumentList {
    #[serde(rename = "items")]
    #[validate(length(max = 200), nested)]
    pub items: Vec<models::Document>,
}

impl DocumentList {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(items: Vec<models::Document>) -> DocumentList {
        DocumentList { items }
    }
}

/// Converts the DocumentList value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for DocumentList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            // Skipping items in query parameter serialization

        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a DocumentList value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for DocumentList {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub items: Vec<Vec<models::Document>>,
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
                        "Missing value while parsing DocumentList".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    "items" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in DocumentList"
                                .to_string(),
                        )
                    }
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing DocumentList".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(DocumentList {
            items: intermediate_rep
                .items
                .into_iter()
                .next()
                .ok_or_else(|| "items missing in DocumentList".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<DocumentList> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<DocumentList>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<DocumentList>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                r#"Invalid header value for DocumentList - value: {hdr_value} is invalid {e}"#
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<DocumentList> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <DocumentList as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        r#"Unable to convert header value '{value}' into DocumentList - {err}"#
                    )),
                }
            }
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
pub enum DocumentType {
    #[serde(rename = "id")]
    Id,
    #[serde(rename = "proof_of_address")]
    ProofOfAddress,
    #[serde(rename = "will")]
    Will,
    #[serde(rename = "advance_directive")]
    AdvanceDirective,
    #[serde(rename = "medical_letter")]
    MedicalLetter,
    #[serde(rename = "policy")]
    Policy,
    #[serde(rename = "statement")]
    Statement,
    #[serde(rename = "other")]
    Other,
}

impl validator::Validate for DocumentType {
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {
        std::result::Result::Ok(())
    }
}

impl std::fmt::Display for DocumentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            DocumentType::Id => write!(f, "id"),
            DocumentType::ProofOfAddress => write!(f, "proof_of_address"),
            DocumentType::Will => write!(f, "will"),
            DocumentType::AdvanceDirective => write!(f, "advance_directive"),
            DocumentType::MedicalLetter => write!(f, "medical_letter"),
            DocumentType::Policy => write!(f, "policy"),
            DocumentType::Statement => write!(f, "statement"),
            DocumentType::Other => write!(f, "other"),
        }
    }
}

impl std::str::FromStr for DocumentType {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "id" => std::result::Result::Ok(DocumentType::Id),
            "proof_of_address" => std::result::Result::Ok(DocumentType::ProofOfAddress),
            "will" => std::result::Result::Ok(DocumentType::Will),
            "advance_directive" => std::result::Result::Ok(DocumentType::AdvanceDirective),
            "medical_letter" => std::result::Result::Ok(DocumentType::MedicalLetter),
            "policy" => std::result::Result::Ok(DocumentType::Policy),
            "statement" => std::result::Result::Ok(DocumentType::Statement),
            "other" => std::result::Result::Ok(DocumentType::Other),
            _ => std::result::Result::Err(format!(r#"Value not valid: {s}"#)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct DocumentVersion {
    #[serde(rename = "document_id")]
    pub document_id: uuid::Uuid,

    #[serde(rename = "version_id")]
    pub version_id: uuid::Uuid,

    #[serde(rename = "sha256")]
    #[validate(
            regex(path = *RE_DOCUMENTVERSION_SHA256),
          custom(function = "check_xss_string"),
    )]
    pub sha256: String,

    #[serde(rename = "created_at")]
    pub created_at: chrono::DateTime<chrono::Utc>,
}

lazy_static::lazy_static! {
    static ref RE_DOCUMENTVERSION_SHA256: regex::Regex = regex::Regex::new("^[a-f0-9]{64}$").unwrap();
}

impl DocumentVersion {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(
        document_id: uuid::Uuid,
        version_id: uuid::Uuid,
        sha256: String,
        created_at: chrono::DateTime<chrono::Utc>,
    ) -> DocumentVersion {
        DocumentVersion {
            document_id,
            version_id,
            sha256,
            created_at,
        }
    }
}

/// Converts the DocumentVersion value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for DocumentVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            // Skipping document_id in query parameter serialization

            // Skipping version_id in query parameter serialization
            Some("sha256".to_string()),
            Some(self.sha256.to_string()),
            // Skipping created_at in query parameter serialization
        ];

        write!(
            f,
            "{}",
            params.into_iter().flatten().collect::<Vec<_>>().join(",")
        )
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a DocumentVersion value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for DocumentVersion {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub document_id: Vec<uuid::Uuid>,
            pub version_id: Vec<uuid::Uuid>,
            pub sha256: Vec<String>,
            pub created_at: Vec<chrono::DateTime<chrono::Utc>>,
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
                        "Missing value while parsing DocumentVersion".to_string(),
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
                    #[allow(clippy::redundant_clone)]
                    "version_id" => intermediate_rep.version_id.push(
                        <uuid::Uuid as std::str::FromStr>::from_str(val)
                            .map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "sha256" => intermediate_rep.sha256.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "created_at" => intermediate_rep.created_at.push(
                        <chrono::DateTime<chrono::Utc> as std::str::FromStr>::from_str(val)
                            .map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing DocumentVersion".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(DocumentVersion {
            document_id: intermediate_rep
                .document_id
                .into_iter()
                .next()
                .ok_or_else(|| "document_id missing in DocumentVersion".to_string())?,
            version_id: intermediate_rep
                .version_id
                .into_iter()
                .next()
                .ok_or_else(|| "version_id missing in DocumentVersion".to_string())?,
            sha256: intermediate_rep
                .sha256
                .into_iter()
                .next()
                .ok_or_else(|| "sha256 missing in DocumentVersion".to_string())?,
            created_at: intermediate_rep
                .created_at
                .into_iter()
                .next()
                .ok_or_else(|| "created_at missing in DocumentVersion".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<DocumentVersion> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<DocumentVersion>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<DocumentVersion>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                r#"Invalid header value for DocumentVersion - value: {hdr_value} is invalid {e}"#
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<DocumentVersion> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <DocumentVersion as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        r#"Unable to convert header value '{value}' into DocumentVersion - {err}"#
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

/// Enumeration of values.
/// Since this enum's variants do not hold data, we can easily define them as `#[repr(C)]`
/// which helps with FFI.
#[allow(non_camel_case_types, clippy::large_enum_variant)]
#[repr(C)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[cfg_attr(feature = "conversion", derive(frunk_enum_derive::LabelledGenericEnum))]
pub enum SensitivityTier {
    #[serde(rename = "green")]
    Green,
    #[serde(rename = "amber")]
    Amber,
    #[serde(rename = "red")]
    Red,
}

impl validator::Validate for SensitivityTier {
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

#[derive(Debug, Clone, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Sha256(pub String);

impl validator::Validate for Sha256 {
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {
        std::result::Result::Ok(())
    }
}

impl std::convert::From<String> for Sha256 {
    fn from(x: String) -> Self {
        Sha256(x)
    }
}

impl std::fmt::Display for Sha256 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for Sha256 {
    type Err = std::string::ParseError;
    fn from_str(x: &str) -> std::result::Result<Self, Self::Err> {
        std::result::Result::Ok(Sha256(x.to_string()))
    }
}

impl std::convert::From<Sha256> for String {
    fn from(x: Sha256) -> Self {
        x.0
    }
}

impl std::ops::Deref for Sha256 {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}

impl std::ops::DerefMut for Sha256 {
    fn deref_mut(&mut self) -> &mut String {
        &mut self.0
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
pub struct V1DocumentsGet400Response {
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

impl V1DocumentsGet400Response {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(r_type: String, title: String, status: u16) -> V1DocumentsGet400Response {
        V1DocumentsGet400Response {
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

/// Converts the V1DocumentsGet400Response value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for V1DocumentsGet400Response {
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

/// Converts Query Parameters representation (style=form, explode=false) to a V1DocumentsGet400Response value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for V1DocumentsGet400Response {
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
                        "Missing value while parsing V1DocumentsGet400Response".to_string(),
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
                    "errors" => return std::result::Result::Err("Parsing a container in this style is not supported in V1DocumentsGet400Response".to_string()),
                    _ => return std::result::Result::Err("Unexpected key while parsing V1DocumentsGet400Response".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(V1DocumentsGet400Response {
            r_type: intermediate_rep
                .r_type
                .into_iter()
                .next()
                .ok_or_else(|| "type missing in V1DocumentsGet400Response".to_string())?,
            title: intermediate_rep
                .title
                .into_iter()
                .next()
                .ok_or_else(|| "title missing in V1DocumentsGet400Response".to_string())?,
            status: intermediate_rep
                .status
                .into_iter()
                .next()
                .ok_or_else(|| "status missing in V1DocumentsGet400Response".to_string())?,
            detail: intermediate_rep.detail.into_iter().next(),
            instance: intermediate_rep.instance.into_iter().next(),
            request_id: intermediate_rep.request_id.into_iter().next(),
            r_errors: intermediate_rep.r_errors.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<V1DocumentsGet400Response> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<V1DocumentsGet400Response>> for HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<V1DocumentsGet400Response>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                r#"Invalid header value for V1DocumentsGet400Response - value: {hdr_value} is invalid {e}"#
            )),
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<V1DocumentsGet400Response> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <V1DocumentsGet400Response as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        r#"Unable to convert header value '{value}' into V1DocumentsGet400Response - {err}"#
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                r#"Unable to convert header: {hdr_value:?} to string: {e}"#
            )),
        }
    }
}
