pub(crate) mod extractor;
pub(crate) mod path;
pub(crate) mod response;

use std::sync::Arc;

use axum::response::{IntoResponse as _, Response};
use http::StatusCode;
use syntect::parsing::SyntaxSet;

use crate::{config::Config, error::Result, http::response::ErrorPage};

#[derive(Clone)]
pub(crate) struct BileState {
    pub(crate) config: Arc<Config>,
    pub(crate) syntax: Arc<SyntaxSet>,
}

impl BileState {
    pub(crate) fn new(config: Config, syntax: SyntaxSet) -> Self {
        Self {
            config: Arc::new(config),
            syntax: Arc::new(syntax),
        }
    }

    pub(crate) async fn spawn<F>(&self, f: F) -> Response
    where
        F: FnOnce(Self) -> Result<Response> + Send + 'static,
    {
        let span = tracing::Span::current();

        let this = self.clone();

        spawn_blocking(move || span.in_scope(|| wrap_err(this.clone(), f(this)))).await
    }
}

// TODO: https://github.com/rust-lang/rust/issues/110011
// #[track_caller]
async fn spawn_blocking<F, R>(f: F) -> R
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .expect("failed to join spawn_blocking call, this should only happen due to a panic")
}

fn wrap_err(state: BileState, res: Result<Response>) -> Response {
    match res {
        Ok(res) => res,
        Err(err) => {
            tracing::error!(err=?err, "failed to handle response");

            ErrorPage::from(state)
                .with_status(StatusCode::INTERNAL_SERVER_ERROR)
                .into_response()
        }
    }
}

pub(crate) struct PermissionsPolicy;

impl tower_helmet::IntoHeader for PermissionsPolicy {
    fn header_name(&self) -> http::HeaderName {
        http::HeaderName::from_static("permissions-policy")
    }

    fn header_value(
        &self,
    ) -> std::result::Result<http::HeaderValue, http::header::InvalidHeaderValue> {
        http::HeaderValue::from_str(
            "accelerometer=(),ambient-light-sensor=(),attribution-reporting=(),autoplay=(),bluetooth=(),browsing-topics=(),camera=(),captured-surface-control=(),clipboard-read=(),clipboard-write=(),compute-pressure=(),display-capture=(),document-domain=(),encrypted-media=(),fullscreen=(),gamepad=(),geolocation=(),gyroscope=(),hid=(),identity-credentials-get=(),idle-detection=(),interest-cohort=(),local-fonts=(),magnetometer=(),microphone=(),midi=(),on-device-speech-recognition=(),otp-credentials=(),payment=(),picture-in-picture=(),private-state-token-issuance=(),private-state-token-redemption=(),publickey-credentials-create=(),publickey-credentials-get=(),screen-wake-lock=(),serial=(),speaker-selection=(),storage-access=(),sync-xhr=(),usb=(),web-share=(),window-management=(),xr-spatial-tracking=()",
        )
    }
}
