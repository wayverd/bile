use axum::{
    extract,
    http::{self, StatusCode, request},
    response::{self, IntoResponse as _},
};

use crate::utils::{Error, Result};

fn get_param<'x>(ext: &'x http::Extensions, name: &str) -> Option<&'x str> {
    let raw = ext.get::<extract::RawPathParams>()?;

    for (key, value) in raw {
        if key == name {
            return Some(value);
        }
    }

    None
}

pub struct Commit(pub String);

impl<S> extract::FromRequestParts<S> for Commit
where
    S: Sync,
{
    type Rejection = response::Response;

    async fn from_request_parts(
        parts: &mut request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let Some(raw) = get_param(&parts.extensions, "commit") else {
            return Err(Error::new(StatusCode::NOT_FOUND, "page not found").into_response());
        };

        if raw.is_empty() {
            return Err(
                Error::new(StatusCode::NOT_FOUND, "commit does not exist in repo").into_response(),
            );
        }

        for c in raw.bytes() {
            if !c.is_ascii_hexdigit() {
                return Err(
                    Error::new(StatusCode::NOT_FOUND, "commit does not exist in repo")
                        .into_response(),
                );
            }
        }

        Ok(Self(raw.to_string()))
    }
}

pub struct Obj(pub String);

impl<S> extract::FromRequestParts<S> for Obj
where
    S: Sync,
{
    type Rejection = response::Response;

    async fn from_request_parts(
        parts: &mut request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let Some(raw) = get_param(&parts.extensions, "obj") else {
            return Err(Error::new(StatusCode::NOT_FOUND, "page not found").into_response());
        };

        if raw.is_empty() {
            return Err(
                Error::new(StatusCode::NOT_FOUND, "object does not exist in repo").into_response(),
            );
        }

        Ok(Self(raw.to_string()))
    }
}

pub struct ObjectName(pub String);

impl<S> extract::FromRequestParts<S> for ObjectName
where
    S: Sync,
{
    type Rejection = response::Response;

    async fn from_request_parts(
        parts: &mut request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let Some(raw) = get_param(&parts.extensions, "object_name") else {
            return Err(Error::new(StatusCode::NOT_FOUND, "page not found").into_response());
        };

        if raw.is_empty() {
            return Err(
                Error::new(StatusCode::NOT_FOUND, "object does not exist in repo").into_response(),
            );
        }

        Ok(Self(raw.to_string()))
    }
}

pub struct Ref(pub String);

impl<S> extract::FromRequestParts<S> for Ref
where
    S: Sync,
{
    type Rejection = response::Response;

    async fn from_request_parts(
        parts: &mut request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let Some(raw) = get_param(&parts.extensions, "ref") else {
            return Err(Error::new(StatusCode::NOT_FOUND, "page not found").into_response());
        };

        if raw.is_empty() {
            return Err(
                Error::new(StatusCode::NOT_FOUND, "ref does not exist in repo").into_response(),
            );
        }

        Ok(Self(raw.to_string()))
    }
}

pub struct RepoName(pub String);

impl<S> extract::FromRequestParts<S> for RepoName
where
    S: Sync,
{
    type Rejection = response::Response;

    async fn from_request_parts(
        parts: &mut request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let Some(raw) = get_param(&parts.extensions, "repo_name") else {
            return Err(Error::new(StatusCode::NOT_FOUND, "page not found").into_response());
        };

        if raw.is_empty() {
            return Err(Error::new(StatusCode::NOT_FOUND, "repo does not exist").into_response());
        }

        Ok(Self(raw.to_string()))
    }
}

pub struct Tag(pub String);

impl<S> extract::FromRequestParts<S> for Tag
where
    S: Sync,
{
    type Rejection = response::Response;

    async fn from_request_parts(
        parts: &mut request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let Some(raw) = get_param(&parts.extensions, "tag") else {
            return Err(Error::new(StatusCode::NOT_FOUND, "page not found").into_response());
        };

        if raw.is_empty() {
            return Err(
                Error::new(StatusCode::NOT_FOUND, "tag does not exist in repo").into_response(),
            );
        }

        Ok(Self(raw.to_string()))
    }
}

pub fn repo_name_checks(name: &str) -> Result<()> {
    let name = name.trim();

    if name.is_empty() {
        return Err(Error::new(StatusCode::NOT_FOUND, "repo does not exist"));
    }

    Ok(())
}

pub fn commit_checks(name: &str) -> Result<()> {
    let name = name.trim();

    if name.is_empty() {
        return Err(Error::new(StatusCode::NOT_FOUND, "repo does not exist"));
    }

    for c in name.bytes() {
        if !c.is_ascii_hexdigit() {
            return Err(Error::new(StatusCode::NOT_FOUND, "repo does not exist"));
        }
    }

    Ok(())
}
