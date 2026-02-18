pub mod git;

pub mod error;
pub mod extractor;
pub mod filters;
pub mod markdown;
pub mod response;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::utils::response::Html;

pub type Result<T = Response, E = Error> = std::result::Result<T, E>;

// TODO: https://github.com/rust-lang/rust/issues/110011
// #[track_caller]
pub async fn spawn_blocking<F, R>(f: F) -> R
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .expect("failed to join spawn_blocking call, this should only happen due to a panic")
}

#[derive(askama::Template)]
#[template(path = "error.html")]
pub enum Error {
    Failure {
        status: StatusCode,
        err: error::Error,
    },
    Custom {
        status: StatusCode,
        message: String,
    },
}

impl Error {
    pub fn new<M: ToString + ?Sized>(status: StatusCode, message: &M) -> Self {
        Self::Custom {
            status,
            message: message.to_string(),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::Failure { status, err } => {
                tracing::error!(err=?err, "failed to respond to request");

                status
            }
            Self::Custom { status, .. } => status,
        };

        (*status, Html(self)).into_response()
    }
}

impl<E> From<E> for Error
where
    E: Into<error::Error>,
{
    fn from(err: E) -> Self {
        Self::Failure {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            err: err.into(),
        }
    }
}

#[must_use]
pub fn blob_mime(blob: &git2::Blob<'_>, extension: &str) -> mime::Mime {
    extension.parse().unwrap_or_else(|_| {
        if blob.is_binary() {
            mime::APPLICATION_OCTET_STREAM
        } else {
            mime::TEXT_PLAIN_UTF_8
        }
    })
}
