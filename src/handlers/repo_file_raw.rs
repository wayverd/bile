use std::path;

use axum::{
    extract::Path,
    http::{StatusCode, header},
    response::{IntoResponse as _, Response},
};

use crate::utils::{
    Error, Result, blob_mime, error::Context as _, extractor::repo_name_checks, git::Repository,
    spawn_blocking,
};

#[tracing::instrument(skip_all)]
pub async fn get(
    Path((repo_name, r#ref, object_name)): Path<(String, String, String)>,
) -> Response {
    spawn_blocking(move || inner(&repo_name, &r#ref, &object_name).into_response()).await
}

#[tracing::instrument(skip_all)]
fn inner(repo_name: &str, r#ref: &str, object_name: &str) -> Result {
    repo_name_checks(repo_name)?;

    let Some(repo) = Repository::open(repo_name).context("opening repository")? else {
        return Err(Error::new(StatusCode::NOT_FOUND, "repo does not exist"));
    };

    let path = path::Path::new(&object_name);

    let Some((_, tree)) = repo
        .commit_tree(r#ref)
        .context("failed to get commit tree")?
    else {
        return Err(Error::new(
            StatusCode::NOT_FOUND,
            "commit does not exist in repo",
        ));
    };

    let Some(blob) = repo.tree_blob(&tree, path)? else {
        return Err(Error::new(
            StatusCode::NOT_FOUND,
            "file does not exist into repo",
        ));
    };

    let extension = path
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or_default();

    let mime = blob_mime(&blob, extension);

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, mime.as_ref())],
        blob.content().to_vec(),
    )
        .into_response())
}
