use axum::extract::{FromRequestParts, path::ErrorKind, rejection::PathRejection};
use http::{StatusCode, request::Parts};
use serde::de::DeserializeOwned;

use crate::http::{BileState, response::ErrorPage};

pub(crate) struct Path<T>(pub T);

impl<T> FromRequestParts<BileState> for Path<T>
where
    T: DeserializeOwned + Send,
{
    type Rejection = ErrorPage;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &BileState,
    ) -> Result<Self, Self::Rejection> {
        match axum::extract::Path::<T>::from_request_parts(parts, state).await {
            Ok(value) => Ok(Self(value.0)),
            Err(rejection) => match rejection {
                PathRejection::FailedToDeserializePathParams(inner) => {
                    let status = match inner.kind() {
                        ErrorKind::UnsupportedType { .. } => StatusCode::INTERNAL_SERVER_ERROR,
                        _ => StatusCode::BAD_REQUEST,
                    };

                    Err(ErrorPage::from(state).with_status(status))
                }
                _ => Err(ErrorPage::from(state).with_status(StatusCode::INTERNAL_SERVER_ERROR)),
            },
        }
    }
}
