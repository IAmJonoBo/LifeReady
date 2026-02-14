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
    A: apis::assets::Assets<E, Claims = C> + apis::instructions::Instructions<E, Claims = C> + apis::people::People<E, Claims = C> + apis::roles::Roles<E, Claims = C> + apis::ApiAuthBasic<Claims = C> + Send + Sync + 'static,
    E: std::fmt::Debug + Send + Sync + 'static,
    C: Send + Sync + 'static,
{
    // build our application with a route
    Router::new()
        .route("/estate/readyz",
            get(readyz_get::<I, A, E, C>)
        )
        .route("/estate/v1/assets",
            post(v1_assets_post::<I, A, E, C>)
        )
        .route("/estate/v1/instructions",
            post(v1_instructions_post::<I, A, E, C>)
        )
        .route("/estate/v1/people",
            get(v1_people_get::<I, A, E, C>).post(v1_people_post::<I, A, E, C>)
        )
        .route("/estate/v1/roles/grants",
            post(v1_roles_grants_post::<I, A, E, C>)
        )
        .with_state(api_impl)
}

    #[derive(validator::Validate)]
    #[allow(dead_code)]
    struct V1AssetsPostBodyValidator<'a> {
          #[validate(nested)]
          body: &'a models::AssetCreate,
    }


#[tracing::instrument(skip_all)]
fn v1_assets_post_validation(
        body: models::AssetCreate,
) -> std::result::Result<(
        models::AssetCreate,
), ValidationErrors>
{
              let b = V1AssetsPostBodyValidator { body: &body };
              b.validate()?;

Ok((
    body,
))
}
/// V1AssetsPost - POST /estate/v1/assets
#[tracing::instrument(skip_all)]
async fn v1_assets_post<I, A, E, C>(
  method: Method,
  TypedHeader(host): TypedHeader<Host>,
  cookies: CookieJar,
  headers: HeaderMap,
 State(api_impl): State<I>,
          Json(body): Json<models::AssetCreate>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::assets::Assets<E, Claims = C>+ apis::ApiAuthBasic<Claims = C> + Send + Sync,
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
    v1_assets_post_validation(
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



let result = api_impl.as_ref().v1_assets_post(
      
      &method,
      &host,
      &cookies,
        &claims,
              &body,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::assets::V1AssetsPostResponse::Status201_Created
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
                                                  let mut response = response.status(201);
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
                                                apis::assets::V1AssetsPostResponse::Status400_ErrorResponse
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
                                                apis::assets::V1AssetsPostResponse::Status401_Unauthorized
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
                                                apis::assets::V1AssetsPostResponse::Status403_Forbidden
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
                                                apis::assets::V1AssetsPostResponse::Status409_Conflict
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
                                                  let mut response = response.status(409);
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
                                                apis::assets::V1AssetsPostResponse::Status422_UnprocessableEntity
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
                                                apis::assets::V1AssetsPostResponse::Status500_InternalServerError
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
    struct V1InstructionsPostBodyValidator<'a> {
          #[validate(nested)]
          body: &'a models::InstructionCreate,
    }


#[tracing::instrument(skip_all)]
fn v1_instructions_post_validation(
        body: models::InstructionCreate,
) -> std::result::Result<(
        models::InstructionCreate,
), ValidationErrors>
{
              let b = V1InstructionsPostBodyValidator { body: &body };
              b.validate()?;

Ok((
    body,
))
}
/// V1InstructionsPost - POST /estate/v1/instructions
#[tracing::instrument(skip_all)]
async fn v1_instructions_post<I, A, E, C>(
  method: Method,
  TypedHeader(host): TypedHeader<Host>,
  cookies: CookieJar,
  headers: HeaderMap,
 State(api_impl): State<I>,
          Json(body): Json<models::InstructionCreate>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::instructions::Instructions<E, Claims = C>+ apis::ApiAuthBasic<Claims = C> + Send + Sync,
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
    v1_instructions_post_validation(
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



let result = api_impl.as_ref().v1_instructions_post(
      
      &method,
      &host,
      &cookies,
        &claims,
              &body,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::instructions::V1InstructionsPostResponse::Status201_Created
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
                                                  let mut response = response.status(201);
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
                                                apis::instructions::V1InstructionsPostResponse::Status400_ErrorResponse
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
                                                apis::instructions::V1InstructionsPostResponse::Status401_Unauthorized
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
                                                apis::instructions::V1InstructionsPostResponse::Status403_Forbidden
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
                                                apis::instructions::V1InstructionsPostResponse::Status409_Conflict
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
                                                  let mut response = response.status(409);
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
                                                apis::instructions::V1InstructionsPostResponse::Status422_UnprocessableEntity
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
                                                apis::instructions::V1InstructionsPostResponse::Status500_InternalServerError
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
fn readyz_get_validation(
) -> std::result::Result<(
), ValidationErrors>
{

Ok((
))
}
/// ReadyzGet - GET /estate/readyz
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
    A: apis::people::People<E, Claims = C>+ apis::ApiAuthBasic<Claims = C> + Send + Sync,
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
                                                apis::people::ReadyzGetResponse::Status200_ServiceAndCriticalDependenciesAreReadyToServeTraffic
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
                                                apis::people::ReadyzGetResponse::Status503_ServiceIsReachableButCriticalDependenciesAreNotReady
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


#[tracing::instrument(skip_all)]
fn v1_people_get_validation(
  query_params: models::V1PeopleGetQueryParams,
) -> std::result::Result<(
  models::V1PeopleGetQueryParams,
), ValidationErrors>
{
  query_params.validate()?;

Ok((
  query_params,
))
}
/// V1PeopleGet - GET /estate/v1/people
#[tracing::instrument(skip_all)]
async fn v1_people_get<I, A, E, C>(
  method: Method,
  TypedHeader(host): TypedHeader<Host>,
  cookies: CookieJar,
  headers: HeaderMap,
  QueryExtra(query_params): QueryExtra<models::V1PeopleGetQueryParams>,
 State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::people::People<E, Claims = C>+ apis::ApiAuthBasic<Claims = C> + Send + Sync,
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
    v1_people_get_validation(
        query_params,
    )
  ).await.unwrap();

  let Ok((
    query_params,
  )) = validation else {
    return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
  };



let result = api_impl.as_ref().v1_people_get(
      
      &method,
      &host,
      &cookies,
        &claims,
        &query_params,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::people::V1PeopleGetResponse::Status200_OK
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
                                                apis::people::V1PeopleGetResponse::Status400_ErrorResponse
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
                                                apis::people::V1PeopleGetResponse::Status401_Unauthorized
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
                                                apis::people::V1PeopleGetResponse::Status403_Forbidden
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
                                                apis::people::V1PeopleGetResponse::Status500_InternalServerError
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
    struct V1PeoplePostBodyValidator<'a> {
          #[validate(nested)]
          body: &'a models::PersonCreate,
    }


#[tracing::instrument(skip_all)]
fn v1_people_post_validation(
        body: models::PersonCreate,
) -> std::result::Result<(
        models::PersonCreate,
), ValidationErrors>
{
              let b = V1PeoplePostBodyValidator { body: &body };
              b.validate()?;

Ok((
    body,
))
}
/// V1PeoplePost - POST /estate/v1/people
#[tracing::instrument(skip_all)]
async fn v1_people_post<I, A, E, C>(
  method: Method,
  TypedHeader(host): TypedHeader<Host>,
  cookies: CookieJar,
  headers: HeaderMap,
 State(api_impl): State<I>,
          Json(body): Json<models::PersonCreate>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::people::People<E, Claims = C>+ apis::ApiAuthBasic<Claims = C> + Send + Sync,
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
    v1_people_post_validation(
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



let result = api_impl.as_ref().v1_people_post(
      
      &method,
      &host,
      &cookies,
        &claims,
              &body,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::people::V1PeoplePostResponse::Status201_Created
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
                                                  let mut response = response.status(201);
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
                                                apis::people::V1PeoplePostResponse::Status400_ErrorResponse
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
                                                apis::people::V1PeoplePostResponse::Status401_Unauthorized
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
                                                apis::people::V1PeoplePostResponse::Status403_Forbidden
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
                                                apis::people::V1PeoplePostResponse::Status409_Conflict
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
                                                  let mut response = response.status(409);
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
                                                apis::people::V1PeoplePostResponse::Status422_UnprocessableEntity
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
                                                apis::people::V1PeoplePostResponse::Status500_InternalServerError
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
    struct V1RolesGrantsPostBodyValidator<'a> {
          #[validate(nested)]
          body: &'a models::RoleGrantCreate,
    }


#[tracing::instrument(skip_all)]
fn v1_roles_grants_post_validation(
        body: models::RoleGrantCreate,
) -> std::result::Result<(
        models::RoleGrantCreate,
), ValidationErrors>
{
              let b = V1RolesGrantsPostBodyValidator { body: &body };
              b.validate()?;

Ok((
    body,
))
}
/// V1RolesGrantsPost - POST /estate/v1/roles/grants
#[tracing::instrument(skip_all)]
async fn v1_roles_grants_post<I, A, E, C>(
  method: Method,
  TypedHeader(host): TypedHeader<Host>,
  cookies: CookieJar,
  headers: HeaderMap,
 State(api_impl): State<I>,
          Json(body): Json<models::RoleGrantCreate>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::roles::Roles<E, Claims = C>+ apis::ApiAuthBasic<Claims = C> + Send + Sync,
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
    v1_roles_grants_post_validation(
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



let result = api_impl.as_ref().v1_roles_grants_post(
      
      &method,
      &host,
      &cookies,
        &claims,
              &body,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::roles::V1RolesGrantsPostResponse::Status201_Created
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
                                                  let mut response = response.status(201);
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
                                                apis::roles::V1RolesGrantsPostResponse::Status400_ErrorResponse
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
                                                apis::roles::V1RolesGrantsPostResponse::Status401_Unauthorized
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
                                                apis::roles::V1RolesGrantsPostResponse::Status403_Forbidden
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
                                                apis::roles::V1RolesGrantsPostResponse::Status409_Conflict
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
                                                  let mut response = response.status(409);
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
                                                apis::roles::V1RolesGrantsPostResponse::Status422_UnprocessableEntity
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
                                                apis::roles::V1RolesGrantsPostResponse::Status500_InternalServerError
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
