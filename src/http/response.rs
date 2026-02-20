use std::sync::Arc;

use axum::{
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};

use crate::{config::Config, http::BileState};

pub(crate) type Result<T = Response, E = crate::error::Error> = std::result::Result<T, E>;

pub(crate) struct Css<T>(pub T);

impl<T: IntoResponse> IntoResponse for Css<T> {
    fn into_response(self) -> Response {
        (
            [
                (
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(mime::TEXT_CSS_UTF_8.as_ref()),
                ),
                (
                    header::CACHE_CONTROL,
                    HeaderValue::from_static("max-age=31536000, immutable"),
                ),
            ],
            self.0,
        )
            .into_response()
    }
}

pub(crate) struct Ico<T>(pub T);

impl<T: IntoResponse> IntoResponse for Ico<T> {
    fn into_response(self) -> Response {
        (
            [
                (
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("image/x-icon"),
                ),
                (
                    header::CACHE_CONTROL,
                    HeaderValue::from_static("max-age=31536000, immutable"),
                ),
            ],
            self.0,
        )
            .into_response()
    }
}

pub(crate) struct Json<T>(pub T);

impl<T: IntoResponse> IntoResponse for Json<T> {
    fn into_response(self) -> Response {
        (
            [
                (
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
                ),
                (
                    header::CACHE_CONTROL,
                    HeaderValue::from_static("max-age=31536000, immutable"),
                ),
            ],
            self.0,
        )
            .into_response()
    }
}

pub(crate) struct Png<T>(pub T);

impl<T: IntoResponse> IntoResponse for Png<T> {
    fn into_response(self) -> Response {
        (
            [
                (
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(mime::IMAGE_PNG.as_ref()),
                ),
                (
                    header::CACHE_CONTROL,
                    HeaderValue::from_static("max-age=31536000, immutable"),
                ),
            ],
            self.0,
        )
            .into_response()
    }
}

pub(crate) struct Text<T>(pub T);

impl<T: IntoResponse> IntoResponse for Text<T> {
    fn into_response(self) -> Response {
        (
            [
                (
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(mime::TEXT_PLAIN_UTF_8.as_ref()),
                ),
                (
                    header::CACHE_CONTROL,
                    HeaderValue::from_static("max-age=300, private"),
                ),
            ],
            self.0,
        )
            .into_response()
    }
}

pub(crate) struct Html<T: askama::Template>(pub T);

impl<T: askama::Template> IntoResponse for Html<T> {
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(rendered) => (
                [
                    (
                        header::CONTENT_TYPE,
                        HeaderValue::from_static(mime::TEXT_HTML_UTF_8.as_ref()),
                    ),
                    (
                        header::CACHE_CONTROL,
                        HeaderValue::from_static("max-age=300, private"),
                    ),
                ],
                rendered,
            )
                .into_response(),
            Err(err) => {
                tracing::error!(err=?err, "failed to render html response");

                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    [(
                        header::CONTENT_TYPE,
                        HeaderValue::from_static(mime::TEXT_HTML_UTF_8.as_ref()),
                    )],
                    "a serious error has occured",
                )
                    .into_response()
            }
        }
    }
}

pub(crate) struct Xml<T: askama::Template>(pub T);

impl<T: askama::Template> IntoResponse for Xml<T> {
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(rendered) => (
                [
                    (
                        header::CONTENT_TYPE,
                        HeaderValue::from_static(mime::TEXT_XML.as_ref()),
                    ),
                    (
                        header::CACHE_CONTROL,
                        HeaderValue::from_static("max-age=300, private"),
                    ),
                ],
                rendered,
            )
                .into_response(),
            Err(err) => {
                tracing::error!(err=?err, "failed to render xml response");

                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    [(
                        header::CONTENT_TYPE,
                        HeaderValue::from_static(mime::TEXT_HTML_UTF_8.as_ref()),
                    )],
                    "a serious error has occured",
                )
                    .into_response()
            }
        }
    }
}

#[must_use = "needs to be returned from a handler or otherwise turned into a Response to be useful"]
#[derive(Debug, Clone)]
pub(crate) struct Redirect {
    status_code: StatusCode,
    location: HeaderValue,
}

impl Redirect {
    pub(crate) const PERMANENT_ROOT: Self = Self {
        status_code: StatusCode::PERMANENT_REDIRECT,
        location: HeaderValue::from_static("/"),
    };
    pub(crate) const TEMPORARY_ROOT: Self = Self {
        status_code: StatusCode::TEMPORARY_REDIRECT,
        location: HeaderValue::from_static("/"),
    };

    #[tracing::instrument(skip_all)]
    pub(crate) fn to(uri: &str) -> Option<Self> {
        Self::with_status_code(StatusCode::SEE_OTHER, uri)
    }

    #[tracing::instrument(skip_all)]
    pub(crate) fn temporary(uri: &str) -> Option<Self> {
        Self::with_status_code(StatusCode::TEMPORARY_REDIRECT, uri)
    }

    #[tracing::instrument(skip_all)]
    pub(crate) fn permanent(uri: &str) -> Option<Self> {
        Self::with_status_code(StatusCode::PERMANENT_REDIRECT, uri)
    }

    #[tracing::instrument(skip_all)]
    fn with_status_code(status_code: StatusCode, uri: &str) -> Option<Self> {
        assert!(
            status_code.is_redirection(),
            "not a redirection status code"
        );

        let location = match HeaderValue::try_from(uri) {
            Ok(location) => location,
            Err(err) => {
                tracing::error!(err=?err, "failed to convert uri to header");

                return None;
            }
        };

        Some(Self {
            status_code,
            location,
        })
    }
}

impl IntoResponse for Redirect {
    fn into_response(self) -> Response {
        (self.status_code, [(header::LOCATION, self.location)]).into_response()
    }
}

#[derive(askama::Template)]
#[template(path = "error.html")]
pub(crate) struct ErrorPage {
    config: Arc<Config>,
    status: StatusCode,
}

impl ErrorPage {
    pub(crate) fn with_status(self, status: StatusCode) -> Self {
        Self {
            config: self.config,
            status,
        }
    }
}

impl From<BileState> for ErrorPage {
    fn from(value: BileState) -> Self {
        Self {
            config: value.config,
            status: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl<'s> From<&'s BileState> for ErrorPage {
    fn from(value: &'s BileState) -> Self {
        Self {
            config: value.config.clone(),
            status: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for ErrorPage {
    fn into_response(self) -> Response {
        (self.status, Html(self)).into_response()
    }
}
