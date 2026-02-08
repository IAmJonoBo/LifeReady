use std::collections::HashMap;

use axum::{body::Body, extract::*, response::Response, routing::*};
use axum_extra::{
    extract::{CookieJar, Query as QueryExtra},
    TypedHeader,
};
use bytes::Bytes;
use headers::Host;
use http::{header::CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue, Method, StatusCode};
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
    A: apis::audit::Audit<E, Claims = C> + apis::ApiAuthBasic<Claims = C> + Send + Sync + 'static,
    E: std::fmt::Debug + Send + Sync + 'static,
    C: Send + Sync + 'static,
{
    // build our application with a route
    Router::new()
        .route(
            "/audit/v1/audit/events",
            post(v1_audit_events_post::<I, A, E, C>),
        )
        .route(
            "/audit/v1/audit/export",
            get(v1_audit_export_get::<I, A, E, C>),
        )
        .with_state(api_impl)
}

#[derive(validator::Validate)]
#[allow(dead_code)]
struct V1AuditEventsPostBodyValidator<'a> {
    #[validate(nested)]
    body: &'a models::AuditAppend,
}

#[tracing::instrument(skip_all)]
fn v1_audit_events_post_validation(
    body: models::AuditAppend,
) -> std::result::Result<(models::AuditAppend,), ValidationErrors> {
    let b = V1AuditEventsPostBodyValidator { body: &body };
    b.validate()?;

    Ok((body,))
}
/// V1AuditEventsPost - POST /audit/v1/audit/events
#[tracing::instrument(skip_all)]
async fn v1_audit_events_post<I, A, E, C>(
    method: Method,
    TypedHeader(host): TypedHeader<Host>,
    cookies: CookieJar,
    headers: HeaderMap,
    State(api_impl): State<I>,
    Json(body): Json<models::AuditAppend>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::audit::Audit<E, Claims = C> + apis::ApiAuthBasic<Claims = C> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
{
    // Authentication
    let claims_in_auth_header = api_impl
        .as_ref()
        .extract_claims_from_auth_header(apis::BasicAuthKind::Bearer, &headers, "authorization")
        .await;
    let claims = None.or(claims_in_auth_header);
    let Some(claims) = claims else {
        return response_with_status_code_only(StatusCode::UNAUTHORIZED);
    };

    #[allow(clippy::redundant_closure)]
    let validation = tokio::task::spawn_blocking(move || v1_audit_events_post_validation(body))
        .await
        .unwrap();

    let Ok((
      body,
  )) = validation else {
    return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
  };

    let result = api_impl
        .as_ref()
        .v1_audit_events_post(&method, &host, &cookies, &claims, &body)
        .await;

    let mut response = Response::builder();

    let resp = match result {
        Ok(rsp) => match rsp {
            apis::audit::V1AuditEventsPostResponse::Status201_Appended { body, x_request_id } => {
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
                        response_headers
                            .insert(HeaderName::from_static("x-request-id"), x_request_id);
                    }
                }
                let mut response = response.status(201);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers
                        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
            apis::audit::V1AuditEventsPostResponse::Status400_ErrorResponse {
                body,
                x_request_id,
            } => {
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
                        response_headers
                            .insert(HeaderName::from_static("x-request-id"), x_request_id);
                    }
                }
                let mut response = response.status(400);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_static("application/problem+json"),
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
            apis::audit::V1AuditEventsPostResponse::Status401_Unauthorized {
                body,
                x_request_id,
            } => {
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
                        response_headers
                            .insert(HeaderName::from_static("x-request-id"), x_request_id);
                    }
                }
                let mut response = response.status(401);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_static("application/problem+json"),
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
            apis::audit::V1AuditEventsPostResponse::Status403_Forbidden { body, x_request_id } => {
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
                        response_headers
                            .insert(HeaderName::from_static("x-request-id"), x_request_id);
                    }
                }
                let mut response = response.status(403);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_static("application/problem+json"),
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
            apis::audit::V1AuditEventsPostResponse::Status409_Conflict { body, x_request_id } => {
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
                        response_headers
                            .insert(HeaderName::from_static("x-request-id"), x_request_id);
                    }
                }
                let mut response = response.status(409);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_static("application/problem+json"),
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
            apis::audit::V1AuditEventsPostResponse::Status422_UnprocessableEntity {
                body,
                x_request_id,
            } => {
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
                        response_headers
                            .insert(HeaderName::from_static("x-request-id"), x_request_id);
                    }
                }
                let mut response = response.status(422);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_static("application/problem+json"),
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
            apis::audit::V1AuditEventsPostResponse::Status500_InternalServerError {
                body,
                x_request_id,
            } => {
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
                        response_headers
                            .insert(HeaderName::from_static("x-request-id"), x_request_id);
                    }
                }
                let mut response = response.status(500);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_static("application/problem+json"),
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
        },
        Err(why) => {
            // Application code returned an error. This should not happen, as the implementation should
            // return a valid response.
            return api_impl
                .as_ref()
                .handle_error(&method, &host, &cookies, why)
                .await;
        }
    };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[tracing::instrument(skip_all)]
fn v1_audit_export_get_validation(
    query_params: models::V1AuditExportGetQueryParams,
) -> std::result::Result<(models::V1AuditExportGetQueryParams,), ValidationErrors> {
    query_params.validate()?;

    Ok((query_params,))
}
/// V1AuditExportGet - GET /audit/v1/audit/export
#[tracing::instrument(skip_all)]
async fn v1_audit_export_get<I, A, E, C>(
    method: Method,
    TypedHeader(host): TypedHeader<Host>,
    cookies: CookieJar,
    headers: HeaderMap,
    QueryExtra(query_params): QueryExtra<models::V1AuditExportGetQueryParams>,
    State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::audit::Audit<E, Claims = C> + apis::ApiAuthBasic<Claims = C> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
{
    // Authentication
    let claims_in_auth_header = api_impl
        .as_ref()
        .extract_claims_from_auth_header(apis::BasicAuthKind::Bearer, &headers, "authorization")
        .await;
    let claims = None.or(claims_in_auth_header);
    let Some(claims) = claims else {
        return response_with_status_code_only(StatusCode::UNAUTHORIZED);
    };

    #[allow(clippy::redundant_closure)]
    let validation =
        tokio::task::spawn_blocking(move || v1_audit_export_get_validation(query_params))
            .await
            .unwrap();

    let Ok((
    query_params,
  )) = validation else {
    return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
  };

    let result = api_impl
        .as_ref()
        .v1_audit_export_get(&method, &host, &cookies, &claims, &query_params)
        .await;

    let mut response = Response::builder();

    let resp = match result {
        Ok(rsp) => match rsp {
            apis::audit::V1AuditExportGetResponse::Status200_ExportReady { body, x_request_id } => {
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
                        response_headers
                            .insert(HeaderName::from_static("x-request-id"), x_request_id);
                    }
                }
                let mut response = response.status(200);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers
                        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
            apis::audit::V1AuditExportGetResponse::Status400_ErrorResponse {
                body,
                x_request_id,
            } => {
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
                        response_headers
                            .insert(HeaderName::from_static("x-request-id"), x_request_id);
                    }
                }
                let mut response = response.status(400);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_static("application/problem+json"),
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
            apis::audit::V1AuditExportGetResponse::Status401_Unauthorized {
                body,
                x_request_id,
            } => {
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
                        response_headers
                            .insert(HeaderName::from_static("x-request-id"), x_request_id);
                    }
                }
                let mut response = response.status(401);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_static("application/problem+json"),
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
            apis::audit::V1AuditExportGetResponse::Status403_Forbidden { body, x_request_id } => {
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
                        response_headers
                            .insert(HeaderName::from_static("x-request-id"), x_request_id);
                    }
                }
                let mut response = response.status(403);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_static("application/problem+json"),
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
            apis::audit::V1AuditExportGetResponse::Status404_NotFound { body, x_request_id } => {
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
                        response_headers
                            .insert(HeaderName::from_static("x-request-id"), x_request_id);
                    }
                }
                let mut response = response.status(404);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_static("application/problem+json"),
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
            apis::audit::V1AuditExportGetResponse::Status500_InternalServerError {
                body,
                x_request_id,
            } => {
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
                        response_headers
                            .insert(HeaderName::from_static("x-request-id"), x_request_id);
                    }
                }
                let mut response = response.status(500);
                {
                    let mut response_headers = response.headers_mut().unwrap();
                    response_headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_static("application/problem+json"),
                    );
                }

                let body_content = tokio::task::spawn_blocking(move || {
                    serde_json::to_vec(&body).map_err(|e| {
                        error!(error = ?e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                })
                .await
                .unwrap()?;
                response.body(Body::from(body_content))
            }
        },
        Err(why) => {
            // Application code returned an error. This should not happen, as the implementation should
            // return a valid response.
            return api_impl
                .as_ref()
                .handle_error(&method, &host, &cookies, why)
                .await;
        }
    };

    resp.map_err(|e| {
        error!(error = ?e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[allow(dead_code)]
#[inline]
fn response_with_status_code_only(code: StatusCode) -> Result<Response, StatusCode> {
    Response::builder()
        .status(code)
        .body(Body::empty())
        .map_err(|_| code)
}
