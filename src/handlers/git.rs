use axum::{
    extract::Path,
    http::{HeaderValue, StatusCode, Uri, header},
    response::{IntoResponse as _, Response},
};

use crate::utils::{
    Error, Result, error::Context as _, extractor::repo_name_checks, git::Repository,
    spawn_blocking,
};

#[tracing::instrument(skip_all)]
pub async fn get_1(uri: Uri, Path(repo_name): Path<String>) -> Response {
    spawn_blocking(move || inner(&uri, &repo_name).into_response()).await
}

#[tracing::instrument(skip_all)]
pub async fn get_2(uri: Uri, Path((repo_name, _)): Path<(String, String)>) -> Response {
    spawn_blocking(move || inner(&uri, &repo_name).into_response()).await
}

fn inner(uri: &Uri, repo_name: &str) -> Result {
    repo_name_checks(repo_name)?;

    let Some(repo) = Repository::open(repo_name).context("opening repository")? else {
        return Err(Error::new(StatusCode::NOT_FOUND, "repo does not exist"));
    };

    let path = uri
        .path()
        .strip_prefix(&format!("/{repo_name}/"))
        .unwrap_or_default();

    let path = repo.path().join(path);

    // cant canonicalize if it doesnt exist
    if !path.exists() {
        return Err(Error::new(
            StatusCode::NOT_FOUND,
            "This page does not exist.",
        ));
    }

    let path = path.canonicalize().context("canonicalize new path")?;

    // that path got us outside of the repository structure somehow
    if !path.starts_with(repo.path()) {
        tracing::warn!("Attempt to acces file outside of repo dir: {:?}", path);
        return Err(Error::new(
            StatusCode::FORBIDDEN,
            "You do not have access to this file.",
        ));
    }

    // Either the requested resource does not exist or it is not
    // a file, i.e. a directory.
    if !path.is_file() {
        return Err(Error::new(
            StatusCode::NOT_FOUND,
            "This page does not exist.",
        ));
    }

    let body = std::fs::read(&path).context("reading file")?;

    Ok((
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::APPLICATION_OCTET_STREAM.as_ref()),
        )],
        body,
    )
        .into_response())
}
