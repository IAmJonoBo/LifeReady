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







#[derive(Debug, Clone, PartialEq, PartialOrd,  serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Email(pub String);

impl validator::Validate for Email {
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {

        std::result::Result::Ok(())
    }
}

impl std::convert::From<String> for Email {
    fn from(x: String) -> Self {
        Email(x)
    }
}

impl std::fmt::Display for Email {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
       write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for Email {
    type Err = std::string::ParseError;
    fn from_str(x: &str) -> std::result::Result<Self, Self::Err> {
        std::result::Result::Ok(Email(x.to_string()))
    }
}

impl std::convert::From<Email> for String {
    fn from(x: Email) -> Self {
        x.0
    }
}

impl std::ops::Deref for Email {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}

impl std::ops::DerefMut for Email {
    fn deref_mut(&mut self) -> &mut String {
        &mut self.0
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
pub struct LoginChallenge {
    #[serde(rename = "challenge_id")]
    pub challenge_id: uuid::Uuid,

    #[serde(rename = "mfa_required")]
    pub mfa_required: bool,

    /// Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "mfa_methods")]
    #[validate(
            length(max = 5),
          custom(function = "check_xss_vec_string"),
    )]
    #[serde(skip_serializing_if="Option::is_none")]
    pub mfa_methods: Option<Vec<String>>,

}



impl LoginChallenge {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(challenge_id: uuid::Uuid, mfa_required: bool, ) -> LoginChallenge {
        LoginChallenge {
 challenge_id,
 mfa_required,
 mfa_methods: None,
        }
    }
}

/// Converts the LoginChallenge value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for LoginChallenge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            // Skipping challenge_id in query parameter serialization


            Some("mfa_required".to_string()),
            Some(self.mfa_required.to_string()),


            self.mfa_methods.as_ref().map(|mfa_methods| {
                [
                    "mfa_methods".to_string(),
                    mfa_methods.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","),
                ].join(",")
            }),

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a LoginChallenge value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for LoginChallenge {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub challenge_id: Vec<uuid::Uuid>,
            pub mfa_required: Vec<bool>,
            pub mfa_methods: Vec<Vec<String>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing LoginChallenge".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "challenge_id" => intermediate_rep.challenge_id.push(<uuid::Uuid as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "mfa_required" => intermediate_rep.mfa_required.push(<bool as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    "mfa_methods" => return std::result::Result::Err("Parsing a container in this style is not supported in LoginChallenge".to_string()),
                    _ => return std::result::Result::Err("Unexpected key while parsing LoginChallenge".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(LoginChallenge {
            challenge_id: intermediate_rep.challenge_id.into_iter().next().ok_or_else(|| "challenge_id missing in LoginChallenge".to_string())?,
            mfa_required: intermediate_rep.mfa_required.into_iter().next().ok_or_else(|| "mfa_required missing in LoginChallenge".to_string())?,
            mfa_methods: intermediate_rep.mfa_methods.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<LoginChallenge> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<LoginChallenge>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<LoginChallenge>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for LoginChallenge - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<LoginChallenge> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <LoginChallenge as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into LoginChallenge - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}



#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct LoginRequest {
    #[serde(rename = "email")]
    #[validate(
            length(max = 320),
          custom(function = "check_xss_string"),
    )]
    pub email: String,

    #[serde(rename = "password")]
    #[validate(
            length(min = 8, max = 256),
          custom(function = "check_xss_string"),
    )]
    #[serde(skip_serializing_if="Option::is_none")]
    pub password: Option<String>,

    #[serde(rename = "client")]
          #[validate(nested)]
    #[serde(skip_serializing_if="Option::is_none")]
    pub client: Option<models::LoginRequestClient>,

}



impl LoginRequest {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(email: String, ) -> LoginRequest {
        LoginRequest {
 email,
 password: None,
 client: None,
        }
    }
}

/// Converts the LoginRequest value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for LoginRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![

            Some("email".to_string()),
            Some(self.email.to_string()),


            self.password.as_ref().map(|password| {
                [
                    "password".to_string(),
                    password.to_string(),
                ].join(",")
            }),

            // Skipping client in query parameter serialization

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a LoginRequest value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for LoginRequest {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub email: Vec<String>,
            pub password: Vec<String>,
            pub client: Vec<models::LoginRequestClient>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing LoginRequest".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "email" => intermediate_rep.email.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "password" => intermediate_rep.password.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "client" => intermediate_rep.client.push(<models::LoginRequestClient as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing LoginRequest".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(LoginRequest {
            email: intermediate_rep.email.into_iter().next().ok_or_else(|| "email missing in LoginRequest".to_string())?,
            password: intermediate_rep.password.into_iter().next(),
            client: intermediate_rep.client.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<LoginRequest> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<LoginRequest>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<LoginRequest>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for LoginRequest - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<LoginRequest> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <LoginRequest as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into LoginRequest - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}



#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct LoginRequestClient {
    /// Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "platform")]
          #[validate(custom(function = "check_xss_string"))]
    pub platform: String,

    #[serde(rename = "device_name")]
    #[validate(
            length(max = 64),
          custom(function = "check_xss_string"),
    )]
    #[serde(skip_serializing_if="Option::is_none")]
    pub device_name: Option<String>,

}



impl LoginRequestClient {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(platform: String, ) -> LoginRequestClient {
        LoginRequestClient {
 platform,
 device_name: None,
        }
    }
}

/// Converts the LoginRequestClient value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for LoginRequestClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![

            Some("platform".to_string()),
            Some(self.platform.to_string()),


            self.device_name.as_ref().map(|device_name| {
                [
                    "device_name".to_string(),
                    device_name.to_string(),
                ].join(",")
            }),

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a LoginRequestClient value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for LoginRequestClient {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub platform: Vec<String>,
            pub device_name: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing LoginRequestClient".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "platform" => intermediate_rep.platform.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "device_name" => intermediate_rep.device_name.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing LoginRequestClient".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(LoginRequestClient {
            platform: intermediate_rep.platform.into_iter().next().ok_or_else(|| "platform missing in LoginRequestClient".to_string())?,
            device_name: intermediate_rep.device_name.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<LoginRequestClient> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<LoginRequestClient>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<LoginRequestClient>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for LoginRequestClient - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<LoginRequestClient> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <LoginRequestClient as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into LoginRequestClient - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}



#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Me {
    #[serde(rename = "principal_id")]
    pub principal_id: uuid::Uuid,

    #[serde(rename = "email")]
    #[validate(
            length(max = 320),
          custom(function = "check_xss_string"),
    )]
    pub email: String,

    #[serde(rename = "display_name")]
    #[validate(
            length(max = 80),
          custom(function = "check_xss_string"),
    )]
    #[serde(skip_serializing_if="Option::is_none")]
    pub display_name: Option<String>,

}



impl Me {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(principal_id: uuid::Uuid, email: String, ) -> Me {
        Me {
 principal_id,
 email,
 display_name: None,
        }
    }
}

/// Converts the Me value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for Me {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            // Skipping principal_id in query parameter serialization


            Some("email".to_string()),
            Some(self.email.to_string()),


            self.display_name.as_ref().map(|display_name| {
                [
                    "display_name".to_string(),
                    display_name.to_string(),
                ].join(",")
            }),

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Me value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Me {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub principal_id: Vec<uuid::Uuid>,
            pub email: Vec<String>,
            pub display_name: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing Me".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "principal_id" => intermediate_rep.principal_id.push(<uuid::Uuid as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "email" => intermediate_rep.email.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "display_name" => intermediate_rep.display_name.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing Me".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Me {
            principal_id: intermediate_rep.principal_id.into_iter().next().ok_or_else(|| "principal_id missing in Me".to_string())?,
            email: intermediate_rep.email.into_iter().next().ok_or_else(|| "email missing in Me".to_string())?,
            display_name: intermediate_rep.display_name.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Me> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<Me>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<Me>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for Me - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<Me> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <Me as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into Me - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}



#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct MfaVerifyRequest {
    #[serde(rename = "challenge_id")]
    pub challenge_id: uuid::Uuid,

    /// Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "method")]
          #[validate(custom(function = "check_xss_string"))]
    pub method: String,

    #[serde(rename = "code")]
    #[validate(
            length(min = 4, max = 64),
          custom(function = "check_xss_string"),
    )]
    pub code: String,

}



impl MfaVerifyRequest {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(challenge_id: uuid::Uuid, method: String, code: String, ) -> MfaVerifyRequest {
        MfaVerifyRequest {
 challenge_id,
 method,
 code,
        }
    }
}

/// Converts the MfaVerifyRequest value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for MfaVerifyRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            // Skipping challenge_id in query parameter serialization


            Some("method".to_string()),
            Some(self.method.to_string()),


            Some("code".to_string()),
            Some(self.code.to_string()),

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a MfaVerifyRequest value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for MfaVerifyRequest {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub challenge_id: Vec<uuid::Uuid>,
            pub method: Vec<String>,
            pub code: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing MfaVerifyRequest".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "challenge_id" => intermediate_rep.challenge_id.push(<uuid::Uuid as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "method" => intermediate_rep.method.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "code" => intermediate_rep.code.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing MfaVerifyRequest".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(MfaVerifyRequest {
            challenge_id: intermediate_rep.challenge_id.into_iter().next().ok_or_else(|| "challenge_id missing in MfaVerifyRequest".to_string())?,
            method: intermediate_rep.method.into_iter().next().ok_or_else(|| "method missing in MfaVerifyRequest".to_string())?,
            code: intermediate_rep.code.into_iter().next().ok_or_else(|| "code missing in MfaVerifyRequest".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<MfaVerifyRequest> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<MfaVerifyRequest>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<MfaVerifyRequest>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for MfaVerifyRequest - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<MfaVerifyRequest> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <MfaVerifyRequest as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into MfaVerifyRequest - {err}"#))
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



#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Session {
    #[serde(rename = "access_token")]
    #[validate(
            length(min = 16),
          custom(function = "check_xss_string"),
    )]
    pub access_token: String,

    #[serde(rename = "expires_at")]
    pub expires_at: chrono::DateTime::<chrono::Utc>,

}



impl Session {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(access_token: String, expires_at: chrono::DateTime::<chrono::Utc>, ) -> Session {
        Session {
 access_token,
 expires_at,
        }
    }
}

/// Converts the Session value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![

            Some("access_token".to_string()),
            Some(self.access_token.to_string()),

            // Skipping expires_at in query parameter serialization

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Session value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Session {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub access_token: Vec<String>,
            pub expires_at: Vec<chrono::DateTime::<chrono::Utc>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing Session".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "access_token" => intermediate_rep.access_token.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "expires_at" => intermediate_rep.expires_at.push(<chrono::DateTime::<chrono::Utc> as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing Session".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Session {
            access_token: intermediate_rep.access_token.into_iter().next().ok_or_else(|| "access_token missing in Session".to_string())?,
            expires_at: intermediate_rep.expires_at.into_iter().next().ok_or_else(|| "expires_at missing in Session".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Session> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<Session>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<Session>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for Session - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<Session> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <Session as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into Session - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
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
pub struct V1AuthLoginPost400Response {
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



impl V1AuthLoginPost400Response {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(r_type: String, title: String, status: u16, ) -> V1AuthLoginPost400Response {
        V1AuthLoginPost400Response {
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

/// Converts the V1AuthLoginPost400Response value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for V1AuthLoginPost400Response {
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

/// Converts Query Parameters representation (style=form, explode=false) to a V1AuthLoginPost400Response value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for V1AuthLoginPost400Response {
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
                None => return std::result::Result::Err("Missing value while parsing V1AuthLoginPost400Response".to_string())
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
                    "errors" => return std::result::Result::Err("Parsing a container in this style is not supported in V1AuthLoginPost400Response".to_string()),
                    _ => return std::result::Result::Err("Unexpected key while parsing V1AuthLoginPost400Response".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(V1AuthLoginPost400Response {
            r_type: intermediate_rep.r_type.into_iter().next().ok_or_else(|| "type missing in V1AuthLoginPost400Response".to_string())?,
            title: intermediate_rep.title.into_iter().next().ok_or_else(|| "title missing in V1AuthLoginPost400Response".to_string())?,
            status: intermediate_rep.status.into_iter().next().ok_or_else(|| "status missing in V1AuthLoginPost400Response".to_string())?,
            detail: intermediate_rep.detail.into_iter().next(),
            instance: intermediate_rep.instance.into_iter().next(),
            request_id: intermediate_rep.request_id.into_iter().next(),
            r_errors: intermediate_rep.r_errors.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<V1AuthLoginPost400Response> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<V1AuthLoginPost400Response>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<V1AuthLoginPost400Response>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for V1AuthLoginPost400Response - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<V1AuthLoginPost400Response> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <V1AuthLoginPost400Response as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into V1AuthLoginPost400Response - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}


