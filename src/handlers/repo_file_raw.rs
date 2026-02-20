use std::path;

use axum::{
    extract::State,
    http::{StatusCode, header},
    response::{IntoResponse as _, Response},
};

use crate::{
    BileState,
    error::Context as _,
    git::Repository,
    http::{
        extractor::{ObjectName, Ref, RepoName},
        path::Path,
        response::{ErrorPage, Result},
    },
    utils::blob_mime,
};

#[tracing::instrument(skip_all)]
pub(crate) async fn get(
    state: State<BileState>,
    Path((repo_name, r#ref, object_name)): Path<(RepoName, Ref, ObjectName)>,
) -> Response {
    state
        .spawn(move |state| inner(&state, &repo_name, &r#ref, &object_name))
        .await
}

#[tracing::instrument(skip_all)]
fn inner(
    state: &BileState,
    repo_name: &RepoName,
    r#ref: &Ref,
    object_name: &ObjectName,
) -> Result<Response> {
    let repo = match Repository::open(&state.config, repo_name).context("opening repository") {
        Ok(Some(repo)) => repo,
        Ok(None) => {
            return Ok(ErrorPage::from(state)
                .with_status(StatusCode::NOT_FOUND)
                .into_response());
        }
        Err(err) => {
            tracing::error!(err=?err, "failed to open repository");

            return Ok(ErrorPage::from(state)
                .with_status(StatusCode::NOT_FOUND)
                .into_response());
        }
    };

    let path = path::Path::new(&object_name.0);

    let Some((_, tree)) = repo
        .commit_tree(&r#ref.0)
        .context("failed to get commit tree")?
    else {
        return Ok(ErrorPage::from(state)
            .with_status(StatusCode::NOT_FOUND)
            .into_response());
    };

    let Some(blob) = repo.tree_blob(&tree, path)? else {
        return Ok(ErrorPage::from(state)
            .with_status(StatusCode::NOT_FOUND)
            .into_response());
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
