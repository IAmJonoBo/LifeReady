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
    pub struct V1PeopleGetQueryParams {
                #[serde(rename = "limit")]
                #[validate(
                        range(min = 1i32, max = 200i32),
              )]
                    #[serde(skip_serializing_if="Option::is_none")]
                    pub limit: Option<i32>,
    }





#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Asset {
    #[serde(rename = "category")]
          #[validate(nested)]
    pub category: models::AssetCategory,

    #[serde(rename = "label")]
    #[validate(
            length(max = 120),
          custom(function = "check_xss_string"),
    )]
    pub label: String,

    #[serde(rename = "notes")]
    #[validate(
            length(max = 4000),
          custom(function = "check_xss_string"),
    )]
    #[serde(skip_serializing_if="Option::is_none")]
    pub notes: Option<String>,

    #[serde(rename = "sensitivity")]
          #[validate(nested)]
    #[serde(skip_serializing_if="Option::is_none")]
    pub sensitivity: Option<models::SensitivityTier>,

    #[serde(rename = "asset_id")]
    pub asset_id: uuid::Uuid,

    #[serde(rename = "created_at")]
    pub created_at: chrono::DateTime::<chrono::Utc>,

}



impl Asset {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(category: models::AssetCategory, label: String, asset_id: uuid::Uuid, created_at: chrono::DateTime::<chrono::Utc>, ) -> Asset {
        Asset {
 category,
 label,
 notes: None,
 sensitivity: None,
 asset_id,
 created_at,
        }
    }
}

/// Converts the Asset value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for Asset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            // Skipping category in query parameter serialization


            Some("label".to_string()),
            Some(self.label.to_string()),


            self.notes.as_ref().map(|notes| {
                [
                    "notes".to_string(),
                    notes.to_string(),
                ].join(",")
            }),

            // Skipping sensitivity in query parameter serialization

            // Skipping asset_id in query parameter serialization

            // Skipping created_at in query parameter serialization

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Asset value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Asset {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub category: Vec<models::AssetCategory>,
            pub label: Vec<String>,
            pub notes: Vec<String>,
            pub sensitivity: Vec<models::SensitivityTier>,
            pub asset_id: Vec<uuid::Uuid>,
            pub created_at: Vec<chrono::DateTime::<chrono::Utc>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing Asset".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "category" => intermediate_rep.category.push(<models::AssetCategory as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "label" => intermediate_rep.label.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "notes" => intermediate_rep.notes.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "sensitivity" => intermediate_rep.sensitivity.push(<models::SensitivityTier as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "asset_id" => intermediate_rep.asset_id.push(<uuid::Uuid as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "created_at" => intermediate_rep.created_at.push(<chrono::DateTime::<chrono::Utc> as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing Asset".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Asset {
            category: intermediate_rep.category.into_iter().next().ok_or_else(|| "category missing in Asset".to_string())?,
            label: intermediate_rep.label.into_iter().next().ok_or_else(|| "label missing in Asset".to_string())?,
            notes: intermediate_rep.notes.into_iter().next(),
            sensitivity: intermediate_rep.sensitivity.into_iter().next(),
            asset_id: intermediate_rep.asset_id.into_iter().next().ok_or_else(|| "asset_id missing in Asset".to_string())?,
            created_at: intermediate_rep.created_at.into_iter().next().ok_or_else(|| "created_at missing in Asset".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Asset> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<Asset>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<Asset>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for Asset - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<Asset> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <Asset as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into Asset - {err}"#))
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
pub enum AssetCategory {
    #[serde(rename = "property")]
    Property,
    #[serde(rename = "bank_account")]
    BankAccount,
    #[serde(rename = "insurance_policy")]
    InsurancePolicy,
    #[serde(rename = "vehicle")]
    Vehicle,
    #[serde(rename = "business_interest")]
    BusinessInterest,
    #[serde(rename = "digital_account")]
    DigitalAccount,
    #[serde(rename = "other")]
    Other,
}

impl validator::Validate for AssetCategory
{
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {
        std::result::Result::Ok(())
    }
}

impl std::fmt::Display for AssetCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            AssetCategory::Property => write!(f, "property"),
            AssetCategory::BankAccount => write!(f, "bank_account"),
            AssetCategory::InsurancePolicy => write!(f, "insurance_policy"),
            AssetCategory::Vehicle => write!(f, "vehicle"),
            AssetCategory::BusinessInterest => write!(f, "business_interest"),
            AssetCategory::DigitalAccount => write!(f, "digital_account"),
            AssetCategory::Other => write!(f, "other"),
        }
    }
}

impl std::str::FromStr for AssetCategory {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "property" => std::result::Result::Ok(AssetCategory::Property),
            "bank_account" => std::result::Result::Ok(AssetCategory::BankAccount),
            "insurance_policy" => std::result::Result::Ok(AssetCategory::InsurancePolicy),
            "vehicle" => std::result::Result::Ok(AssetCategory::Vehicle),
            "business_interest" => std::result::Result::Ok(AssetCategory::BusinessInterest),
            "digital_account" => std::result::Result::Ok(AssetCategory::DigitalAccount),
            "other" => std::result::Result::Ok(AssetCategory::Other),
            _ => std::result::Result::Err(format!(r#"Value not valid: {s}"#)),
        }
    }
}


#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct AssetCreate {
    #[serde(rename = "category")]
          #[validate(nested)]
    pub category: models::AssetCategory,

    #[serde(rename = "label")]
    #[validate(
            length(max = 120),
          custom(function = "check_xss_string"),
    )]
    pub label: String,

    #[serde(rename = "notes")]
    #[validate(
            length(max = 4000),
          custom(function = "check_xss_string"),
    )]
    #[serde(skip_serializing_if="Option::is_none")]
    pub notes: Option<String>,

    #[serde(rename = "sensitivity")]
          #[validate(nested)]
    #[serde(skip_serializing_if="Option::is_none")]
    pub sensitivity: Option<models::SensitivityTier>,

}



impl AssetCreate {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(category: models::AssetCategory, label: String, ) -> AssetCreate {
        AssetCreate {
 category,
 label,
 notes: None,
 sensitivity: None,
        }
    }
}

/// Converts the AssetCreate value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for AssetCreate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            // Skipping category in query parameter serialization


            Some("label".to_string()),
            Some(self.label.to_string()),


            self.notes.as_ref().map(|notes| {
                [
                    "notes".to_string(),
                    notes.to_string(),
                ].join(",")
            }),

            // Skipping sensitivity in query parameter serialization

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a AssetCreate value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for AssetCreate {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub category: Vec<models::AssetCategory>,
            pub label: Vec<String>,
            pub notes: Vec<String>,
            pub sensitivity: Vec<models::SensitivityTier>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing AssetCreate".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "category" => intermediate_rep.category.push(<models::AssetCategory as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "label" => intermediate_rep.label.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "notes" => intermediate_rep.notes.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "sensitivity" => intermediate_rep.sensitivity.push(<models::SensitivityTier as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing AssetCreate".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(AssetCreate {
            category: intermediate_rep.category.into_iter().next().ok_or_else(|| "category missing in AssetCreate".to_string())?,
            label: intermediate_rep.label.into_iter().next().ok_or_else(|| "label missing in AssetCreate".to_string())?,
            notes: intermediate_rep.notes.into_iter().next(),
            sensitivity: intermediate_rep.sensitivity.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<AssetCreate> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<AssetCreate>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<AssetCreate>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for AssetCreate - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<AssetCreate> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <AssetCreate as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into AssetCreate - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}



#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Instruction {
    #[serde(rename = "title")]
    #[validate(
            length(max = 120),
          custom(function = "check_xss_string"),
    )]
    pub title: String,

    #[serde(rename = "body")]
    #[validate(
            length(max = 10000),
          custom(function = "check_xss_string"),
    )]
    pub body: String,

    #[serde(rename = "sensitivity")]
          #[validate(nested)]
    #[serde(skip_serializing_if="Option::is_none")]
    pub sensitivity: Option<models::SensitivityTier>,

    #[serde(rename = "instruction_id")]
    pub instruction_id: uuid::Uuid,

    #[serde(rename = "created_at")]
    pub created_at: chrono::DateTime::<chrono::Utc>,

}



impl Instruction {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(title: String, body: String, instruction_id: uuid::Uuid, created_at: chrono::DateTime::<chrono::Utc>, ) -> Instruction {
        Instruction {
 title,
 body,
 sensitivity: None,
 instruction_id,
 created_at,
        }
    }
}

/// Converts the Instruction value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![

            Some("title".to_string()),
            Some(self.title.to_string()),


            Some("body".to_string()),
            Some(self.body.to_string()),

            // Skipping sensitivity in query parameter serialization

            // Skipping instruction_id in query parameter serialization

            // Skipping created_at in query parameter serialization

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Instruction value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Instruction {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub title: Vec<String>,
            pub body: Vec<String>,
            pub sensitivity: Vec<models::SensitivityTier>,
            pub instruction_id: Vec<uuid::Uuid>,
            pub created_at: Vec<chrono::DateTime::<chrono::Utc>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing Instruction".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "title" => intermediate_rep.title.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "body" => intermediate_rep.body.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "sensitivity" => intermediate_rep.sensitivity.push(<models::SensitivityTier as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "instruction_id" => intermediate_rep.instruction_id.push(<uuid::Uuid as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "created_at" => intermediate_rep.created_at.push(<chrono::DateTime::<chrono::Utc> as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing Instruction".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Instruction {
            title: intermediate_rep.title.into_iter().next().ok_or_else(|| "title missing in Instruction".to_string())?,
            body: intermediate_rep.body.into_iter().next().ok_or_else(|| "body missing in Instruction".to_string())?,
            sensitivity: intermediate_rep.sensitivity.into_iter().next(),
            instruction_id: intermediate_rep.instruction_id.into_iter().next().ok_or_else(|| "instruction_id missing in Instruction".to_string())?,
            created_at: intermediate_rep.created_at.into_iter().next().ok_or_else(|| "created_at missing in Instruction".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Instruction> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<Instruction>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<Instruction>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for Instruction - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<Instruction> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <Instruction as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into Instruction - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}



#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct InstructionCreate {
    #[serde(rename = "title")]
    #[validate(
            length(max = 120),
          custom(function = "check_xss_string"),
    )]
    pub title: String,

    #[serde(rename = "body")]
    #[validate(
            length(max = 10000),
          custom(function = "check_xss_string"),
    )]
    pub body: String,

    #[serde(rename = "sensitivity")]
          #[validate(nested)]
    #[serde(skip_serializing_if="Option::is_none")]
    pub sensitivity: Option<models::SensitivityTier>,

}



impl InstructionCreate {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(title: String, body: String, ) -> InstructionCreate {
        InstructionCreate {
 title,
 body,
 sensitivity: None,
        }
    }
}

/// Converts the InstructionCreate value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for InstructionCreate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![

            Some("title".to_string()),
            Some(self.title.to_string()),


            Some("body".to_string()),
            Some(self.body.to_string()),

            // Skipping sensitivity in query parameter serialization

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a InstructionCreate value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for InstructionCreate {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub title: Vec<String>,
            pub body: Vec<String>,
            pub sensitivity: Vec<models::SensitivityTier>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing InstructionCreate".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "title" => intermediate_rep.title.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "body" => intermediate_rep.body.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "sensitivity" => intermediate_rep.sensitivity.push(<models::SensitivityTier as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing InstructionCreate".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(InstructionCreate {
            title: intermediate_rep.title.into_iter().next().ok_or_else(|| "title missing in InstructionCreate".to_string())?,
            body: intermediate_rep.body.into_iter().next().ok_or_else(|| "body missing in InstructionCreate".to_string())?,
            sensitivity: intermediate_rep.sensitivity.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<InstructionCreate> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<InstructionCreate>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<InstructionCreate>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for InstructionCreate - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<InstructionCreate> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <InstructionCreate as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into InstructionCreate - {err}"#))
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
pub struct Person {
    #[serde(rename = "full_name")]
    #[validate(
            length(max = 120),
          custom(function = "check_xss_string"),
    )]
    pub full_name: String,

    #[serde(rename = "email")]
          #[validate(custom(function = "check_xss_string"))]
    #[serde(skip_serializing_if="Option::is_none")]
    pub email: Option<String>,

    #[serde(rename = "phone_e164")]
    #[validate(
            length(max = 20),
          custom(function = "check_xss_string"),
    )]
    #[serde(skip_serializing_if="Option::is_none")]
    pub phone_e164: Option<String>,

    #[serde(rename = "relationship")]
    #[validate(
            length(max = 80),
          custom(function = "check_xss_string"),
    )]
    #[serde(skip_serializing_if="Option::is_none")]
    pub relationship: Option<String>,

    #[serde(rename = "person_id")]
    pub person_id: uuid::Uuid,

    #[serde(rename = "created_at")]
    pub created_at: chrono::DateTime::<chrono::Utc>,

}



impl Person {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(full_name: String, person_id: uuid::Uuid, created_at: chrono::DateTime::<chrono::Utc>, ) -> Person {
        Person {
 full_name,
 email: None,
 phone_e164: None,
 relationship: None,
 person_id,
 created_at,
        }
    }
}

/// Converts the Person value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for Person {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![

            Some("full_name".to_string()),
            Some(self.full_name.to_string()),


            self.email.as_ref().map(|email| {
                [
                    "email".to_string(),
                    email.to_string(),
                ].join(",")
            }),


            self.phone_e164.as_ref().map(|phone_e164| {
                [
                    "phone_e164".to_string(),
                    phone_e164.to_string(),
                ].join(",")
            }),


            self.relationship.as_ref().map(|relationship| {
                [
                    "relationship".to_string(),
                    relationship.to_string(),
                ].join(",")
            }),

            // Skipping person_id in query parameter serialization

            // Skipping created_at in query parameter serialization

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Person value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Person {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub full_name: Vec<String>,
            pub email: Vec<String>,
            pub phone_e164: Vec<String>,
            pub relationship: Vec<String>,
            pub person_id: Vec<uuid::Uuid>,
            pub created_at: Vec<chrono::DateTime::<chrono::Utc>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing Person".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "full_name" => intermediate_rep.full_name.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "email" => intermediate_rep.email.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "phone_e164" => intermediate_rep.phone_e164.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "relationship" => intermediate_rep.relationship.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "person_id" => intermediate_rep.person_id.push(<uuid::Uuid as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "created_at" => intermediate_rep.created_at.push(<chrono::DateTime::<chrono::Utc> as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing Person".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Person {
            full_name: intermediate_rep.full_name.into_iter().next().ok_or_else(|| "full_name missing in Person".to_string())?,
            email: intermediate_rep.email.into_iter().next(),
            phone_e164: intermediate_rep.phone_e164.into_iter().next(),
            relationship: intermediate_rep.relationship.into_iter().next(),
            person_id: intermediate_rep.person_id.into_iter().next().ok_or_else(|| "person_id missing in Person".to_string())?,
            created_at: intermediate_rep.created_at.into_iter().next().ok_or_else(|| "created_at missing in Person".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Person> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<Person>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<Person>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for Person - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<Person> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <Person as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into Person - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}



#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct PersonCreate {
    #[serde(rename = "full_name")]
    #[validate(
            length(max = 120),
          custom(function = "check_xss_string"),
    )]
    pub full_name: String,

    #[serde(rename = "email")]
          #[validate(custom(function = "check_xss_string"))]
    #[serde(skip_serializing_if="Option::is_none")]
    pub email: Option<String>,

    #[serde(rename = "phone_e164")]
    #[validate(
            length(max = 20),
          custom(function = "check_xss_string"),
    )]
    #[serde(skip_serializing_if="Option::is_none")]
    pub phone_e164: Option<String>,

    #[serde(rename = "relationship")]
    #[validate(
            length(max = 80),
          custom(function = "check_xss_string"),
    )]
    #[serde(skip_serializing_if="Option::is_none")]
    pub relationship: Option<String>,

}



impl PersonCreate {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(full_name: String, ) -> PersonCreate {
        PersonCreate {
 full_name,
 email: None,
 phone_e164: None,
 relationship: None,
        }
    }
}

/// Converts the PersonCreate value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for PersonCreate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![

            Some("full_name".to_string()),
            Some(self.full_name.to_string()),


            self.email.as_ref().map(|email| {
                [
                    "email".to_string(),
                    email.to_string(),
                ].join(",")
            }),


            self.phone_e164.as_ref().map(|phone_e164| {
                [
                    "phone_e164".to_string(),
                    phone_e164.to_string(),
                ].join(",")
            }),


            self.relationship.as_ref().map(|relationship| {
                [
                    "relationship".to_string(),
                    relationship.to_string(),
                ].join(",")
            }),

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a PersonCreate value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for PersonCreate {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub full_name: Vec<String>,
            pub email: Vec<String>,
            pub phone_e164: Vec<String>,
            pub relationship: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing PersonCreate".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "full_name" => intermediate_rep.full_name.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "email" => intermediate_rep.email.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "phone_e164" => intermediate_rep.phone_e164.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "relationship" => intermediate_rep.relationship.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing PersonCreate".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(PersonCreate {
            full_name: intermediate_rep.full_name.into_iter().next().ok_or_else(|| "full_name missing in PersonCreate".to_string())?,
            email: intermediate_rep.email.into_iter().next(),
            phone_e164: intermediate_rep.phone_e164.into_iter().next(),
            relationship: intermediate_rep.relationship.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<PersonCreate> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<PersonCreate>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<PersonCreate>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for PersonCreate - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<PersonCreate> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <PersonCreate as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into PersonCreate - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
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
pub enum Role {
    #[serde(rename = "principal")]
    Principal,
    #[serde(rename = "proxy")]
    Proxy,
    #[serde(rename = "executor_nominee")]
    ExecutorNominee,
    #[serde(rename = "emergency_contact")]
    EmergencyContact,
}

impl validator::Validate for Role
{
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {
        std::result::Result::Ok(())
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Role::Principal => write!(f, "principal"),
            Role::Proxy => write!(f, "proxy"),
            Role::ExecutorNominee => write!(f, "executor_nominee"),
            Role::EmergencyContact => write!(f, "emergency_contact"),
        }
    }
}

impl std::str::FromStr for Role {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "principal" => std::result::Result::Ok(Role::Principal),
            "proxy" => std::result::Result::Ok(Role::Proxy),
            "executor_nominee" => std::result::Result::Ok(Role::ExecutorNominee),
            "emergency_contact" => std::result::Result::Ok(Role::EmergencyContact),
            _ => std::result::Result::Err(format!(r#"Value not valid: {s}"#)),
        }
    }
}


#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct RoleGrant {
    #[serde(rename = "person_id")]
    pub person_id: uuid::Uuid,

    #[serde(rename = "role")]
          #[validate(nested)]
    pub role: models::Role,

    #[serde(rename = "scope")]
          #[validate(nested)]
    pub scope: models::RoleGrantCreateScope,

    #[serde(rename = "grant_id")]
    pub grant_id: uuid::Uuid,

    /// Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "status")]
          #[validate(custom(function = "check_xss_string"))]
    pub status: String,

    #[serde(rename = "created_at")]
    pub created_at: chrono::DateTime::<chrono::Utc>,

}



impl RoleGrant {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(person_id: uuid::Uuid, role: models::Role, scope: models::RoleGrantCreateScope, grant_id: uuid::Uuid, status: String, created_at: chrono::DateTime::<chrono::Utc>, ) -> RoleGrant {
        RoleGrant {
 person_id,
 role,
 scope,
 grant_id,
 status,
 created_at,
        }
    }
}

/// Converts the RoleGrant value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for RoleGrant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            // Skipping person_id in query parameter serialization

            // Skipping role in query parameter serialization

            // Skipping scope in query parameter serialization

            // Skipping grant_id in query parameter serialization


            Some("status".to_string()),
            Some(self.status.to_string()),

            // Skipping created_at in query parameter serialization

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a RoleGrant value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for RoleGrant {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub person_id: Vec<uuid::Uuid>,
            pub role: Vec<models::Role>,
            pub scope: Vec<models::RoleGrantCreateScope>,
            pub grant_id: Vec<uuid::Uuid>,
            pub status: Vec<String>,
            pub created_at: Vec<chrono::DateTime::<chrono::Utc>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing RoleGrant".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "person_id" => intermediate_rep.person_id.push(<uuid::Uuid as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "role" => intermediate_rep.role.push(<models::Role as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "scope" => intermediate_rep.scope.push(<models::RoleGrantCreateScope as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "grant_id" => intermediate_rep.grant_id.push(<uuid::Uuid as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "status" => intermediate_rep.status.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "created_at" => intermediate_rep.created_at.push(<chrono::DateTime::<chrono::Utc> as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing RoleGrant".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(RoleGrant {
            person_id: intermediate_rep.person_id.into_iter().next().ok_or_else(|| "person_id missing in RoleGrant".to_string())?,
            role: intermediate_rep.role.into_iter().next().ok_or_else(|| "role missing in RoleGrant".to_string())?,
            scope: intermediate_rep.scope.into_iter().next().ok_or_else(|| "scope missing in RoleGrant".to_string())?,
            grant_id: intermediate_rep.grant_id.into_iter().next().ok_or_else(|| "grant_id missing in RoleGrant".to_string())?,
            status: intermediate_rep.status.into_iter().next().ok_or_else(|| "status missing in RoleGrant".to_string())?,
            created_at: intermediate_rep.created_at.into_iter().next().ok_or_else(|| "created_at missing in RoleGrant".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<RoleGrant> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<RoleGrant>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<RoleGrant>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for RoleGrant - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<RoleGrant> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <RoleGrant as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into RoleGrant - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}



#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct RoleGrantCreate {
    #[serde(rename = "person_id")]
    pub person_id: uuid::Uuid,

    #[serde(rename = "role")]
          #[validate(nested)]
    pub role: models::Role,

    #[serde(rename = "scope")]
          #[validate(nested)]
    pub scope: models::RoleGrantCreateScope,

}



impl RoleGrantCreate {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(person_id: uuid::Uuid, role: models::Role, scope: models::RoleGrantCreateScope, ) -> RoleGrantCreate {
        RoleGrantCreate {
 person_id,
 role,
 scope,
        }
    }
}

/// Converts the RoleGrantCreate value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for RoleGrantCreate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            // Skipping person_id in query parameter serialization

            // Skipping role in query parameter serialization

            // Skipping scope in query parameter serialization

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a RoleGrantCreate value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for RoleGrantCreate {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub person_id: Vec<uuid::Uuid>,
            pub role: Vec<models::Role>,
            pub scope: Vec<models::RoleGrantCreateScope>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing RoleGrantCreate".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "person_id" => intermediate_rep.person_id.push(<uuid::Uuid as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "role" => intermediate_rep.role.push(<models::Role as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "scope" => intermediate_rep.scope.push(<models::RoleGrantCreateScope as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing RoleGrantCreate".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(RoleGrantCreate {
            person_id: intermediate_rep.person_id.into_iter().next().ok_or_else(|| "person_id missing in RoleGrantCreate".to_string())?,
            role: intermediate_rep.role.into_iter().next().ok_or_else(|| "role missing in RoleGrantCreate".to_string())?,
            scope: intermediate_rep.scope.into_iter().next().ok_or_else(|| "scope missing in RoleGrantCreate".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<RoleGrantCreate> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<RoleGrantCreate>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<RoleGrantCreate>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for RoleGrantCreate - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<RoleGrantCreate> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <RoleGrantCreate as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into RoleGrantCreate - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}



#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct RoleGrantCreateScope {
    /// Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "access_level")]
          #[validate(custom(function = "check_xss_string"))]
    pub access_level: String,

    #[serde(rename = "allowed_tiers")]
    #[validate(
            length(max = 3),
          nested,
    )]
    #[serde(skip_serializing_if="Option::is_none")]
    pub allowed_tiers: Option<Vec<models::SensitivityTier>>,

    #[serde(rename = "expires_at")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub expires_at: Option<chrono::DateTime::<chrono::Utc>>,

}



impl RoleGrantCreateScope {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(access_level: String, ) -> RoleGrantCreateScope {
        RoleGrantCreateScope {
 access_level,
 allowed_tiers: None,
 expires_at: None,
        }
    }
}

/// Converts the RoleGrantCreateScope value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for RoleGrantCreateScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![

            Some("access_level".to_string()),
            Some(self.access_level.to_string()),

            // Skipping allowed_tiers in query parameter serialization

            // Skipping expires_at in query parameter serialization

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a RoleGrantCreateScope value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for RoleGrantCreateScope {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub access_level: Vec<String>,
            pub allowed_tiers: Vec<Vec<models::SensitivityTier>>,
            pub expires_at: Vec<chrono::DateTime::<chrono::Utc>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing RoleGrantCreateScope".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "access_level" => intermediate_rep.access_level.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    "allowed_tiers" => return std::result::Result::Err("Parsing a container in this style is not supported in RoleGrantCreateScope".to_string()),
                    #[allow(clippy::redundant_clone)]
                    "expires_at" => intermediate_rep.expires_at.push(<chrono::DateTime::<chrono::Utc> as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing RoleGrantCreateScope".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(RoleGrantCreateScope {
            access_level: intermediate_rep.access_level.into_iter().next().ok_or_else(|| "access_level missing in RoleGrantCreateScope".to_string())?,
            allowed_tiers: intermediate_rep.allowed_tiers.into_iter().next(),
            expires_at: intermediate_rep.expires_at.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<RoleGrantCreateScope> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<RoleGrantCreateScope>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<RoleGrantCreateScope>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for RoleGrantCreateScope - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<RoleGrantCreateScope> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <RoleGrantCreateScope as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into RoleGrantCreateScope - {err}"#))
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
pub struct V1PeopleGet200Response {
    #[serde(rename = "items")]
    #[validate(
            length(max = 200),
          nested,
    )]
    pub items: Vec<models::Person>,

}



impl V1PeopleGet200Response {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(items: Vec<models::Person>, ) -> V1PeopleGet200Response {
        V1PeopleGet200Response {
 items,
        }
    }
}

/// Converts the V1PeopleGet200Response value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for V1PeopleGet200Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            // Skipping items in query parameter serialization

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a V1PeopleGet200Response value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for V1PeopleGet200Response {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub items: Vec<Vec<models::Person>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing V1PeopleGet200Response".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    "items" => return std::result::Result::Err("Parsing a container in this style is not supported in V1PeopleGet200Response".to_string()),
                    _ => return std::result::Result::Err("Unexpected key while parsing V1PeopleGet200Response".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(V1PeopleGet200Response {
            items: intermediate_rep.items.into_iter().next().ok_or_else(|| "items missing in V1PeopleGet200Response".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<V1PeopleGet200Response> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<V1PeopleGet200Response>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<V1PeopleGet200Response>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for V1PeopleGet200Response - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<V1PeopleGet200Response> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <V1PeopleGet200Response as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into V1PeopleGet200Response - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}



#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct V1PeopleGet400Response {
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



impl V1PeopleGet400Response {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(r_type: String, title: String, status: u16, ) -> V1PeopleGet400Response {
        V1PeopleGet400Response {
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

/// Converts the V1PeopleGet400Response value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for V1PeopleGet400Response {
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

/// Converts Query Parameters representation (style=form, explode=false) to a V1PeopleGet400Response value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for V1PeopleGet400Response {
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
                None => return std::result::Result::Err("Missing value while parsing V1PeopleGet400Response".to_string())
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
                    "errors" => return std::result::Result::Err("Parsing a container in this style is not supported in V1PeopleGet400Response".to_string()),
                    _ => return std::result::Result::Err("Unexpected key while parsing V1PeopleGet400Response".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(V1PeopleGet400Response {
            r_type: intermediate_rep.r_type.into_iter().next().ok_or_else(|| "type missing in V1PeopleGet400Response".to_string())?,
            title: intermediate_rep.title.into_iter().next().ok_or_else(|| "title missing in V1PeopleGet400Response".to_string())?,
            status: intermediate_rep.status.into_iter().next().ok_or_else(|| "status missing in V1PeopleGet400Response".to_string())?,
            detail: intermediate_rep.detail.into_iter().next(),
            instance: intermediate_rep.instance.into_iter().next(),
            request_id: intermediate_rep.request_id.into_iter().next(),
            r_errors: intermediate_rep.r_errors.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<V1PeopleGet400Response> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<V1PeopleGet400Response>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<V1PeopleGet400Response>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for V1PeopleGet400Response - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<V1PeopleGet400Response> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <V1PeopleGet400Response as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into V1PeopleGet400Response - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}


