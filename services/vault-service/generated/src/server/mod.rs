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
    A: apis::documents::Documents<E, Claims = C> + apis::ApiAuthBasic<Claims = C> + Send + Sync + 'static,
    E: std::fmt::Debug + Send + Sync + 'static,
    C: Send + Sync + 'static,
{
    // build our application with a route
    Router::new()
        .route("/vault/readyz",
            get(readyz_get::<I, A, E, C>)
        )
        .route("/vault/v1/documents",
            get(v1_documents_get::<I, A, E, C>).post(v1_documents_post::<I, A, E, C>)
        )
        .route("/vault/v1/documents/{document_id}",
            get(v1_documents_document_id_get::<I, A, E, C>)
        )
        .route("/vault/v1/documents/{document_id}/download",
            get(v1_documents_document_id_download_get::<I, A, E, C>)
        )
        .route("/vault/v1/documents/{document_id}/versions",
            post(v1_documents_document_id_versions_post::<I, A, E, C>)
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
/// ReadyzGet - GET /vault/readyz
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
    A: apis::documents::Documents<E, Claims = C>+ apis::ApiAuthBasic<Claims = C> + Send + Sync,
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
                                                apis::documents::ReadyzGetResponse::Status200_ServiceAndCriticalDependenciesAreReadyToServeTraffic
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
                                                apis::documents::ReadyzGetResponse::Status503_ServiceIsReachableButCriticalDependenciesAreNotReady
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
fn v1_documents_document_id_download_get_validation(
  path_params: models::V1DocumentsDocumentIdDownloadGetPathParams,
  query_params: models::V1DocumentsDocumentIdDownloadGetQueryParams,
) -> std::result::Result<(
  models::V1DocumentsDocumentIdDownloadGetPathParams,
  models::V1DocumentsDocumentIdDownloadGetQueryParams,
), ValidationErrors>
{
  path_params.validate()?;
  query_params.validate()?;

Ok((
  path_params,
  query_params,
))
}
/// V1DocumentsDocumentIdDownloadGet - GET /vault/v1/documents/{document_id}/download
#[tracing::instrument(skip_all)]
async fn v1_documents_document_id_download_get<I, A, E, C>(
  method: Method,
  TypedHeader(host): TypedHeader<Host>,
  cookies: CookieJar,
  headers: HeaderMap,
  Path(path_params): Path<models::V1DocumentsDocumentIdDownloadGetPathParams>,
  QueryExtra(query_params): QueryExtra<models::V1DocumentsDocumentIdDownloadGetQueryParams>,
 State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::documents::Documents<E, Claims = C>+ apis::ApiAuthBasic<Claims = C> + Send + Sync,
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
    v1_documents_document_id_download_get_validation(
        path_params,
        query_params,
    )
  ).await.unwrap();

  let Ok((
    path_params,
    query_params,
  )) = validation else {
    return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
  };



let result = api_impl.as_ref().v1_documents_document_id_download_get(
      
      &method,
      &host,
      &cookies,
        &claims,
        &path_params,
        &query_params,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::documents::V1DocumentsDocumentIdDownloadGetResponse::Status200_DocumentBytes
                                                    {
                                                        body,
                                                        x_request_id,
                                                        content_disposition,
                                                        x_document_sha256
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
                                                    if let Some(content_disposition) = content_disposition {
                                                    let content_disposition = match header::IntoHeaderValue(content_disposition).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling content_disposition header - {e}"))).map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR });
                                                        }
                                                    };


                                                    {
                                                      let mut response_headers = response.headers_mut().unwrap();
                                                      response_headers.insert(
                                                          HeaderName::from_static("content-disposition"),
                                                          content_disposition
                                                      );
                                                    }
                                                    }
                                                    if let Some(x_document_sha256) = x_document_sha256 {
                                                    let x_document_sha256 = match header::IntoHeaderValue(x_document_sha256).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling x_document_sha256 header - {e}"))).map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR });
                                                        }
                                                    };


                                                    {
                                                      let mut response_headers = response.headers_mut().unwrap();
                                                      response_headers.insert(
                                                          HeaderName::from_static("x-document-sha256"),
                                                          x_document_sha256
                                                      );
                                                    }
                                                    }
                                                  let mut response = response.status(200);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/octet-stream"));
                                                  }

                                                  let body_content = body.0;
                                                  response.body(Body::from(body_content))
                                                },
                                                apis::documents::V1DocumentsDocumentIdDownloadGetResponse::Status400_ErrorResponse
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
                                                apis::documents::V1DocumentsDocumentIdDownloadGetResponse::Status401_Unauthorized
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
                                                apis::documents::V1DocumentsDocumentIdDownloadGetResponse::Status403_Forbidden
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
                                                apis::documents::V1DocumentsDocumentIdDownloadGetResponse::Status404_NotFound
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
                                                  let mut response = response.status(404);
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
                                                apis::documents::V1DocumentsDocumentIdDownloadGetResponse::Status500_InternalServerError
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
fn v1_documents_document_id_get_validation(
  path_params: models::V1DocumentsDocumentIdGetPathParams,
) -> std::result::Result<(
  models::V1DocumentsDocumentIdGetPathParams,
), ValidationErrors>
{
  path_params.validate()?;

Ok((
  path_params,
))
}
/// V1DocumentsDocumentIdGet - GET /vault/v1/documents/{document_id}
#[tracing::instrument(skip_all)]
async fn v1_documents_document_id_get<I, A, E, C>(
  method: Method,
  TypedHeader(host): TypedHeader<Host>,
  cookies: CookieJar,
  headers: HeaderMap,
  Path(path_params): Path<models::V1DocumentsDocumentIdGetPathParams>,
 State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::documents::Documents<E, Claims = C>+ apis::ApiAuthBasic<Claims = C> + Send + Sync,
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
    v1_documents_document_id_get_validation(
        path_params,
    )
  ).await.unwrap();

  let Ok((
    path_params,
  )) = validation else {
    return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
  };



let result = api_impl.as_ref().v1_documents_document_id_get(
      
      &method,
      &host,
      &cookies,
        &claims,
        &path_params,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::documents::V1DocumentsDocumentIdGetResponse::Status200_OK
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
                                                apis::documents::V1DocumentsDocumentIdGetResponse::Status400_ErrorResponse
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
                                                apis::documents::V1DocumentsDocumentIdGetResponse::Status401_Unauthorized
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
                                                apis::documents::V1DocumentsDocumentIdGetResponse::Status403_Forbidden
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
                                                apis::documents::V1DocumentsDocumentIdGetResponse::Status404_NotFound
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
                                                  let mut response = response.status(404);
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
                                                apis::documents::V1DocumentsDocumentIdGetResponse::Status500_InternalServerError
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
    struct V1DocumentsDocumentIdVersionsPostBodyValidator<'a> {
          #[validate(nested)]
          body: &'a models::DocumentCommit,
    }


#[tracing::instrument(skip_all)]
fn v1_documents_document_id_versions_post_validation(
  path_params: models::V1DocumentsDocumentIdVersionsPostPathParams,
        body: models::DocumentCommit,
) -> std::result::Result<(
  models::V1DocumentsDocumentIdVersionsPostPathParams,
        models::DocumentCommit,
), ValidationErrors>
{
  path_params.validate()?;
              let b = V1DocumentsDocumentIdVersionsPostBodyValidator { body: &body };
              b.validate()?;

Ok((
  path_params,
    body,
))
}
/// V1DocumentsDocumentIdVersionsPost - POST /vault/v1/documents/{document_id}/versions
#[tracing::instrument(skip_all)]
async fn v1_documents_document_id_versions_post<I, A, E, C>(
  method: Method,
  TypedHeader(host): TypedHeader<Host>,
  cookies: CookieJar,
  headers: HeaderMap,
  Path(path_params): Path<models::V1DocumentsDocumentIdVersionsPostPathParams>,
 State(api_impl): State<I>,
          Json(body): Json<models::DocumentCommit>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::documents::Documents<E, Claims = C>+ apis::ApiAuthBasic<Claims = C> + Send + Sync,
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
    v1_documents_document_id_versions_post_validation(
        path_params,
          body,
    )
  ).await.unwrap();

  let Ok((
    path_params,
      body,
  )) = validation else {
    return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
  };



let result = api_impl.as_ref().v1_documents_document_id_versions_post(
      
      &method,
      &host,
      &cookies,
        &claims,
        &path_params,
              &body,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::documents::V1DocumentsDocumentIdVersionsPostResponse::Status201_VersionCommitted
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
                                                apis::documents::V1DocumentsDocumentIdVersionsPostResponse::Status400_ErrorResponse
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
                                                apis::documents::V1DocumentsDocumentIdVersionsPostResponse::Status401_Unauthorized
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
                                                apis::documents::V1DocumentsDocumentIdVersionsPostResponse::Status403_Forbidden
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
                                                apis::documents::V1DocumentsDocumentIdVersionsPostResponse::Status404_NotFound
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
                                                  let mut response = response.status(404);
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
                                                apis::documents::V1DocumentsDocumentIdVersionsPostResponse::Status409_Conflict
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
                                                apis::documents::V1DocumentsDocumentIdVersionsPostResponse::Status422_UnprocessableEntity
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
                                                apis::documents::V1DocumentsDocumentIdVersionsPostResponse::Status500_InternalServerError
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
fn v1_documents_get_validation(
  query_params: models::V1DocumentsGetQueryParams,
) -> std::result::Result<(
  models::V1DocumentsGetQueryParams,
), ValidationErrors>
{
  query_params.validate()?;

Ok((
  query_params,
))
}
/// V1DocumentsGet - GET /vault/v1/documents
#[tracing::instrument(skip_all)]
async fn v1_documents_get<I, A, E, C>(
  method: Method,
  TypedHeader(host): TypedHeader<Host>,
  cookies: CookieJar,
  headers: HeaderMap,
  QueryExtra(query_params): QueryExtra<models::V1DocumentsGetQueryParams>,
 State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::documents::Documents<E, Claims = C>+ apis::ApiAuthBasic<Claims = C> + Send + Sync,
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
    v1_documents_get_validation(
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



let result = api_impl.as_ref().v1_documents_get(
      
      &method,
      &host,
      &cookies,
        &claims,
        &query_params,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::documents::V1DocumentsGetResponse::Status200_OK
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
                                                apis::documents::V1DocumentsGetResponse::Status400_ErrorResponse
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
                                                apis::documents::V1DocumentsGetResponse::Status401_Unauthorized
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
                                                apis::documents::V1DocumentsGetResponse::Status403_Forbidden
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
                                                apis::documents::V1DocumentsGetResponse::Status500_InternalServerError
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
    struct V1DocumentsPostBodyValidator<'a> {
          #[validate(nested)]
          body: &'a models::DocumentInit,
    }


#[tracing::instrument(skip_all)]
fn v1_documents_post_validation(
        body: models::DocumentInit,
) -> std::result::Result<(
        models::DocumentInit,
), ValidationErrors>
{
              let b = V1DocumentsPostBodyValidator { body: &body };
              b.validate()?;

Ok((
    body,
))
}
/// V1DocumentsPost - POST /vault/v1/documents
#[tracing::instrument(skip_all)]
async fn v1_documents_post<I, A, E, C>(
  method: Method,
  TypedHeader(host): TypedHeader<Host>,
  cookies: CookieJar,
  headers: HeaderMap,
 State(api_impl): State<I>,
          Json(body): Json<models::DocumentInit>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::documents::Documents<E, Claims = C>+ apis::ApiAuthBasic<Claims = C> + Send + Sync,
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
    v1_documents_post_validation(
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



let result = api_impl.as_ref().v1_documents_post(
      
      &method,
      &host,
      &cookies,
        &claims,
              &body,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::documents::V1DocumentsPostResponse::Status201_InitOK
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
                                                apis::documents::V1DocumentsPostResponse::Status400_ErrorResponse
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
                                                apis::documents::V1DocumentsPostResponse::Status401_Unauthorized
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
                                                apis::documents::V1DocumentsPostResponse::Status403_Forbidden
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
                                                apis::documents::V1DocumentsPostResponse::Status422_UnprocessableEntity
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
                                                apis::documents::V1DocumentsPostResponse::Status500_InternalServerError
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
