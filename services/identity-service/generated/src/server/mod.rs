use std::collections::HashMap;

use axum::{body::Body, extract::*, response::Response, routing::*};
use axum_extra::{
    TypedHeader,
    extract::{CookieJar, Query as QueryExtra},
};
use bytes::Bytes;
use headers::Host;
use http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode, header::CONTENT_TYPE};
use tracing::error;
use validator::{Validate, ValidationErrors};

#[allow(unused_imports)]
use crate::{apis, models};
use crate::{header, types::*};
#[allow(unused_imports)]
use crate::{
    models::check_xss_map, models::check_xss_map_nested, models::check_xss_map_string,
    models::check_xss_string, models::check_xss_vec_string,
};


/// Setup API Server.
pub fn new<I, A, E, C>(api_impl: I) -> Router
where
    I: AsRef<A> + Clone + Send + Sync + 'static,
    A: apis::auth::Auth<E, Claims = C> + apis::ApiAuthBasic<Claims = C> + Send + Sync + 'static,
    E: std::fmt::Debug + Send + Sync + 'static,
    C: Send + Sync + 'static,
{
    // build our application with a route
    Router::new()
        .route("/identity/readyz",
            get(readyz_get::<I, A, E, C>)
        )
        .route("/identity/v1/auth/login",
            post(v1_auth_login_post::<I, A, E, C>)
        )
        .route("/identity/v1/auth/mfa/verify",
            post(v1_auth_mfa_verify_post::<I, A, E, C>)
        )
        .route("/identity/v1/me",
            get(v1_me_get::<I, A, E, C>)
        )
        .with_state(api_impl)
}


#[tracing::instrument(skip_all)]
fn readyz_get_validation(
) -> std::result::Result<(
), ValidationErrors>
{

Ok((
))
}
/// ReadyzGet - GET /identity/readyz
#[tracing::instrument(skip_all)]
async fn readyz_get<I, A, E, C>(
  method: Method,
  TypedHeader(host): TypedHeader<Host>,
  cookies: CookieJar,
  headers: HeaderMap,
 State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::auth::Auth<E, Claims = C>+ apis::ApiAuthBasic<Claims = C> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {


    // Authentication
    let claims_in_auth_header = api_impl.as_ref().extract_claims_from_auth_header(apis::BasicAuthKind::Bearer, &headers, "authorization").await;
    let claims = None
             .or(claims_in_auth_header)
          ;
    let Some(claims) = claims else {
        return response_with_status_code_only(StatusCode::UNAUTHORIZED);
    };


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    readyz_get_validation(
    )
  ).await.unwrap();

  let Ok((
  )) = validation else {
    return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
  };



let result = api_impl.as_ref().readyz_get(
      
      &method,
      &host,
      &cookies,
        &claims,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::auth::ReadyzGetResponse::Status200_ServiceAndCriticalDependenciesAreReadyToServeTraffic
                                                    {
                                                        body,
                                                        x_request_id
                                                    }
                                                => {
                                                    if let Some(x_request_id) = x_request_id {
                                                    let x_request_id = match header::IntoHeaderValue(x_request_id).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling x_request_id header - {e}"))).map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR });
                                                        }
                                                    };


                                                    {
                                                      let mut response_headers = response.headers_mut().unwrap();
                                                      response_headers.insert(
                                                          HeaderName::from_static("x-request-id"),
                                                          x_request_id
                                                      );
                                                    }
                                                    }
                                                  let mut response = response.status(200);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/json"));
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::auth::ReadyzGetResponse::Status503_ServiceIsReachableButCriticalDependenciesAreNotReady
                                                    {
                                                        body,
                                                        x_request_id
                                                    }
                                                => {
                                                    if let Some(x_request_id) = x_request_id {
                                                    let x_request_id = match header::IntoHeaderValue(x_request_id).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling x_request_id header - {e}"))).map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR });
                                                        }
                                                    };


                                                    {
                                                      let mut response_headers = response.headers_mut().unwrap();
                                                      response_headers.insert(
                                                          HeaderName::from_static("x-request-id"),
                                                          x_request_id
                                                      );
                                                    }
                                                    }
                                                  let mut response = response.status(503);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/json"));
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(why) => {
                                                    // Application code returned an error. This should not happen, as the implementation should
                                                    // return a valid response.
                                                    return api_impl.as_ref().handle_error(&method, &host, &cookies, why).await;
                                            },
                                        };


                                        resp.map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })
}

    #[derive(validator::Validate)]
    #[allow(dead_code)]
    struct V1AuthLoginPostBodyValidator<'a> {
          #[validate(nested)]
          body: &'a models::LoginRequest,
    }


#[tracing::instrument(skip_all)]
fn v1_auth_login_post_validation(
        body: models::LoginRequest,
) -> std::result::Result<(
        models::LoginRequest,
), ValidationErrors>
{
              let b = V1AuthLoginPostBodyValidator { body: &body };
              b.validate()?;

Ok((
    body,
))
}
/// V1AuthLoginPost - POST /identity/v1/auth/login
#[tracing::instrument(skip_all)]
async fn v1_auth_login_post<I, A, E, C>(
  method: Method,
  TypedHeader(host): TypedHeader<Host>,
  cookies: CookieJar,
  headers: HeaderMap,
 State(api_impl): State<I>,
          Json(body): Json<models::LoginRequest>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::auth::Auth<E, Claims = C>+ apis::ApiAuthBasic<Claims = C> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {


    // Authentication
    let claims_in_auth_header = api_impl.as_ref().extract_claims_from_auth_header(apis::BasicAuthKind::Bearer, &headers, "authorization").await;
    let claims = None
             .or(claims_in_auth_header)
          ;
    let Some(claims) = claims else {
        return response_with_status_code_only(StatusCode::UNAUTHORIZED);
    };


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    v1_auth_login_post_validation(
          body,
    )
  ).await.unwrap();

  let Ok((
      body,
  )) = validation else {
    return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
  };



let result = api_impl.as_ref().v1_auth_login_post(
      
      &method,
      &host,
      &cookies,
        &claims,
              &body,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::auth::V1AuthLoginPostResponse::Status200_ChallengeCreated
                                                    {
                                                        body,
                                                        x_request_id
                                                    }
                                                => {
                                                    if let Some(x_request_id) = x_request_id {
                                                    let x_request_id = match header::IntoHeaderValue(x_request_id).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling x_request_id header - {e}"))).map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR });
                                                        }
                                                    };


                                                    {
                                                      let mut response_headers = response.headers_mut().unwrap();
                                                      response_headers.insert(
                                                          HeaderName::from_static("x-request-id"),
                                                          x_request_id
                                                      );
                                                    }
                                                    }
                                                  let mut response = response.status(200);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/json"));
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::auth::V1AuthLoginPostResponse::Status400_ErrorResponse
                                                    {
                                                        body,
                                                        x_request_id
                                                    }
                                                => {
                                                    if let Some(x_request_id) = x_request_id {
                                                    let x_request_id = match header::IntoHeaderValue(x_request_id).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling x_request_id header - {e}"))).map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR });
                                                        }
                                                    };


                                                    {
                                                      let mut response_headers = response.headers_mut().unwrap();
                                                      response_headers.insert(
                                                          HeaderName::from_static("x-request-id"),
                                                          x_request_id
                                                      );
                                                    }
                                                    }
                                                  let mut response = response.status(400);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/problem+json"));
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::auth::V1AuthLoginPostResponse::Status401_Unauthorized
                                                    {
                                                        body,
                                                        x_request_id
                                                    }
                                                => {
                                                    if let Some(x_request_id) = x_request_id {
                                                    let x_request_id = match header::IntoHeaderValue(x_request_id).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling x_request_id header - {e}"))).map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR });
                                                        }
                                                    };


                                                    {
                                                      let mut response_headers = response.headers_mut().unwrap();
                                                      response_headers.insert(
                                                          HeaderName::from_static("x-request-id"),
                                                          x_request_id
                                                      );
                                                    }
                                                    }
                                                  let mut response = response.status(401);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/problem+json"));
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::auth::V1AuthLoginPostResponse::Status403_Forbidden
                                                    {
                                                        body,
                                                        x_request_id
                                                    }
                                                => {
                                                    if let Some(x_request_id) = x_request_id {
                                                    let x_request_id = match header::IntoHeaderValue(x_request_id).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling x_request_id header - {e}"))).map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR });
                                                        }
                                                    };


                                                    {
                                                      let mut response_headers = response.headers_mut().unwrap();
                                                      response_headers.insert(
                                                          HeaderName::from_static("x-request-id"),
                                                          x_request_id
                                                      );
                                                    }
                                                    }
                                                  let mut response = response.status(403);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/problem+json"));
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::auth::V1AuthLoginPostResponse::Status422_UnprocessableEntity
                                                    {
                                                        body,
                                                        x_request_id
                                                    }
                                                => {
                                                    if let Some(x_request_id) = x_request_id {
                                                    let x_request_id = match header::IntoHeaderValue(x_request_id).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling x_request_id header - {e}"))).map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR });
                                                        }
                                                    };


                                                    {
                                                      let mut response_headers = response.headers_mut().unwrap();
                                                      response_headers.insert(
                                                          HeaderName::from_static("x-request-id"),
                                                          x_request_id
                                                      );
                                                    }
                                                    }
                                                  let mut response = response.status(422);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/problem+json"));
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::auth::V1AuthLoginPostResponse::Status500_InternalServerError
                                                    {
                                                        body,
                                                        x_request_id
                                                    }
                                                => {
                                                    if let Some(x_request_id) = x_request_id {
                                                    let x_request_id = match header::IntoHeaderValue(x_request_id).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling x_request_id header - {e}"))).map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR });
                                                        }
                                                    };


                                                    {
                                                      let mut response_headers = response.headers_mut().unwrap();
                                                      response_headers.insert(
                                                          HeaderName::from_static("x-request-id"),
                                                          x_request_id
                                                      );
                                                    }
                                                    }
                                                  let mut response = response.status(500);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/problem+json"));
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(why) => {
                                                    // Application code returned an error. This should not happen, as the implementation should
                                                    // return a valid response.
                                                    return api_impl.as_ref().handle_error(&method, &host, &cookies, why).await;
                                            },
                                        };


                                        resp.map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })
}

    #[derive(validator::Validate)]
    #[allow(dead_code)]
    struct V1AuthMfaVerifyPostBodyValidator<'a> {
          #[validate(nested)]
          body: &'a models::MfaVerifyRequest,
    }


#[tracing::instrument(skip_all)]
fn v1_auth_mfa_verify_post_validation(
        body: models::MfaVerifyRequest,
) -> std::result::Result<(
        models::MfaVerifyRequest,
), ValidationErrors>
{
              let b = V1AuthMfaVerifyPostBodyValidator { body: &body };
              b.validate()?;

Ok((
    body,
))
}
/// V1AuthMfaVerifyPost - POST /identity/v1/auth/mfa/verify
#[tracing::instrument(skip_all)]
async fn v1_auth_mfa_verify_post<I, A, E, C>(
  method: Method,
  TypedHeader(host): TypedHeader<Host>,
  cookies: CookieJar,
  headers: HeaderMap,
 State(api_impl): State<I>,
          Json(body): Json<models::MfaVerifyRequest>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::auth::Auth<E, Claims = C>+ apis::ApiAuthBasic<Claims = C> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {


    // Authentication
    let claims_in_auth_header = api_impl.as_ref().extract_claims_from_auth_header(apis::BasicAuthKind::Bearer, &headers, "authorization").await;
    let claims = None
             .or(claims_in_auth_header)
          ;
    let Some(claims) = claims else {
        return response_with_status_code_only(StatusCode::UNAUTHORIZED);
    };


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    v1_auth_mfa_verify_post_validation(
          body,
    )
  ).await.unwrap();

  let Ok((
      body,
  )) = validation else {
    return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
  };



let result = api_impl.as_ref().v1_auth_mfa_verify_post(
      
      &method,
      &host,
      &cookies,
        &claims,
              &body,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::auth::V1AuthMfaVerifyPostResponse::Status200_SessionIssued
                                                    {
                                                        body,
                                                        x_request_id
                                                    }
                                                => {
                                                    if let Some(x_request_id) = x_request_id {
                                                    let x_request_id = match header::IntoHeaderValue(x_request_id).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling x_request_id header - {e}"))).map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR });
                                                        }
                                                    };


                                                    {
                                                      let mut response_headers = response.headers_mut().unwrap();
                                                      response_headers.insert(
                                                          HeaderName::from_static("x-request-id"),
                                                          x_request_id
                                                      );
                                                    }
                                                    }
                                                  let mut response = response.status(200);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/json"));
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::auth::V1AuthMfaVerifyPostResponse::Status400_ErrorResponse
                                                    {
                                                        body,
                                                        x_request_id
                                                    }
                                                => {
                                                    if let Some(x_request_id) = x_request_id {
                                                    let x_request_id = match header::IntoHeaderValue(x_request_id).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling x_request_id header - {e}"))).map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR });
                                                        }
                                                    };


                                                    {
                                                      let mut response_headers = response.headers_mut().unwrap();
                                                      response_headers.insert(
                                                          HeaderName::from_static("x-request-id"),
                                                          x_request_id
                                                      );
                                                    }
                                                    }
                                                  let mut response = response.status(400);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/problem+json"));
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::auth::V1AuthMfaVerifyPostResponse::Status401_Unauthorized
                                                    {
                                                        body,
                                                        x_request_id
                                                    }
                                                => {
                                                    if let Some(x_request_id) = x_request_id {
                                                    let x_request_id = match header::IntoHeaderValue(x_request_id).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling x_request_id header - {e}"))).map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR });
                                                        }
                                                    };


                                                    {
                                                      let mut response_headers = response.headers_mut().unwrap();
                                                      response_headers.insert(
                                                          HeaderName::from_static("x-request-id"),
                                                          x_request_id
                                                      );
                                                    }
                                                    }
                                                  let mut response = response.status(401);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/problem+json"));
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::auth::V1AuthMfaVerifyPostResponse::Status403_Forbidden
                                                    {
                                                        body,
                                                        x_request_id
                                                    }
                                                => {
                                                    if let Some(x_request_id) = x_request_id {
                                                    let x_request_id = match header::IntoHeaderValue(x_request_id).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling x_request_id header - {e}"))).map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR });
                                                        }
                                                    };


                                                    {
                                                      let mut response_headers = response.headers_mut().unwrap();
                                                      response_headers.insert(
                                                          HeaderName::from_static("x-request-id"),
                                                          x_request_id
                                                      );
                                                    }
                                                    }
                                                  let mut response = response.status(403);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/problem+json"));
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::auth::V1AuthMfaVerifyPostResponse::Status422_UnprocessableEntity
                                                    {
                                                        body,
                                                        x_request_id
                                                    }
                                                => {
                                                    if let Some(x_request_id) = x_request_id {
                                                    let x_request_id = match header::IntoHeaderValue(x_request_id).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling x_request_id header - {e}"))).map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR });
                                                        }
                                                    };


                                                    {
                                                      let mut response_headers = response.headers_mut().unwrap();
                                                      response_headers.insert(
                                                          HeaderName::from_static("x-request-id"),
                                                          x_request_id
                                                      );
                                                    }
                                                    }
                                                  let mut response = response.status(422);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/problem+json"));
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::auth::V1AuthMfaVerifyPostResponse::Status500_InternalServerError
                                                    {
                                                        body,
                                                        x_request_id
                                                    }
                                                => {
                                                    if let Some(x_request_id) = x_request_id {
                                                    let x_request_id = match header::IntoHeaderValue(x_request_id).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling x_request_id header - {e}"))).map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR });
                                                        }
                                                    };


                                                    {
                                                      let mut response_headers = response.headers_mut().unwrap();
                                                      response_headers.insert(
                                                          HeaderName::from_static("x-request-id"),
                                                          x_request_id
                                                      );
                                                    }
                                                    }
                                                  let mut response = response.status(500);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/problem+json"));
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(why) => {
                                                    // Application code returned an error. This should not happen, as the implementation should
                                                    // return a valid response.
                                                    return api_impl.as_ref().handle_error(&method, &host, &cookies, why).await;
                                            },
                                        };


                                        resp.map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })
}


#[tracing::instrument(skip_all)]
fn v1_me_get_validation(
) -> std::result::Result<(
), ValidationErrors>
{

Ok((
))
}
/// V1MeGet - GET /identity/v1/me
#[tracing::instrument(skip_all)]
async fn v1_me_get<I, A, E, C>(
  method: Method,
  TypedHeader(host): TypedHeader<Host>,
  cookies: CookieJar,
  headers: HeaderMap,
 State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::auth::Auth<E, Claims = C>+ apis::ApiAuthBasic<Claims = C> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {


    // Authentication
    let claims_in_auth_header = api_impl.as_ref().extract_claims_from_auth_header(apis::BasicAuthKind::Bearer, &headers, "authorization").await;
    let claims = None
             .or(claims_in_auth_header)
          ;
    let Some(claims) = claims else {
        return response_with_status_code_only(StatusCode::UNAUTHORIZED);
    };


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    v1_me_get_validation(
    )
  ).await.unwrap();

  let Ok((
  )) = validation else {
    return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
  };



let result = api_impl.as_ref().v1_me_get(
      
      &method,
      &host,
      &cookies,
        &claims,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::auth::V1MeGetResponse::Status200_CurrentPrincipalProfile
                                                    {
                                                        body,
                                                        x_request_id
                                                    }
                                                => {
                                                    if let Some(x_request_id) = x_request_id {
                                                    let x_request_id = match header::IntoHeaderValue(x_request_id).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling x_request_id header - {e}"))).map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR });
                                                        }
                                                    };


                                                    {
                                                      let mut response_headers = response.headers_mut().unwrap();
                                                      response_headers.insert(
                                                          HeaderName::from_static("x-request-id"),
                                                          x_request_id
                                                      );
                                                    }
                                                    }
                                                  let mut response = response.status(200);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/json"));
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::auth::V1MeGetResponse::Status401_Unauthorized
                                                    {
                                                        body,
                                                        x_request_id
                                                    }
                                                => {
                                                    if let Some(x_request_id) = x_request_id {
                                                    let x_request_id = match header::IntoHeaderValue(x_request_id).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling x_request_id header - {e}"))).map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR });
                                                        }
                                                    };


                                                    {
                                                      let mut response_headers = response.headers_mut().unwrap();
                                                      response_headers.insert(
                                                          HeaderName::from_static("x-request-id"),
                                                          x_request_id
                                                      );
                                                    }
                                                    }
                                                  let mut response = response.status(401);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/problem+json"));
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::auth::V1MeGetResponse::Status403_Forbidden
                                                    {
                                                        body,
                                                        x_request_id
                                                    }
                                                => {
                                                    if let Some(x_request_id) = x_request_id {
                                                    let x_request_id = match header::IntoHeaderValue(x_request_id).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling x_request_id header - {e}"))).map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR });
                                                        }
                                                    };


                                                    {
                                                      let mut response_headers = response.headers_mut().unwrap();
                                                      response_headers.insert(
                                                          HeaderName::from_static("x-request-id"),
                                                          x_request_id
                                                      );
                                                    }
                                                    }
                                                  let mut response = response.status(403);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/problem+json"));
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::auth::V1MeGetResponse::Status500_InternalServerError
                                                    {
                                                        body,
                                                        x_request_id
                                                    }
                                                => {
                                                    if let Some(x_request_id) = x_request_id {
                                                    let x_request_id = match header::IntoHeaderValue(x_request_id).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling x_request_id header - {e}"))).map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR });
                                                        }
                                                    };


                                                    {
                                                      let mut response_headers = response.headers_mut().unwrap();
                                                      response_headers.insert(
                                                          HeaderName::from_static("x-request-id"),
                                                          x_request_id
                                                      );
                                                    }
                                                    }
                                                  let mut response = response.status(500);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/problem+json"));
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(why) => {
                                                    // Application code returned an error. This should not happen, as the implementation should
                                                    // return a valid response.
                                                    return api_impl.as_ref().handle_error(&method, &host, &cookies, why).await;
                                            },
                                        };


                                        resp.map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })
}


#[allow(dead_code)]
#[inline]
fn response_with_status_code_only(code: StatusCode) -> Result<Response, StatusCode> {
   Response::builder()
          .status(code)
          .body(Body::empty())
          .map_err(|_| code)
}
