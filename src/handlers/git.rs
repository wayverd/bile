use axum::{
    extract::State,
    http::{HeaderValue, StatusCode, Uri, header},
    response::{IntoResponse as _, Response},
};

use crate::{
    BileState,
    error::Context as _,
    git::Repository,
    http::{
        extractor::RepoName,
        path::Path,
        response::{ErrorPage, Result},
    },
};

#[tracing::instrument(skip_all)]
pub(crate) async fn get_1(
    state: State<BileState>,
    uri: Uri,
    Path(repo_name): Path<RepoName>,
) -> Response {
    state
        .spawn(move |state| inner(&state, &uri, &repo_name))
        .await
}

#[tracing::instrument(skip_all)]
pub(crate) async fn get_2(
    state: State<BileState>,
    uri: Uri,
    Path((repo_name, _)): Path<(RepoName, String)>,
) -> Response {
    state
        .spawn(move |state| inner(&state, &uri, &repo_name))
        .await
}

fn inner(state: &BileState, uri: &Uri, repo_name: &RepoName) -> Result<Response> {
    let Some(repo) = Repository::open(&state.config, repo_name).context("opening repository")?
    else {
        return Ok(ErrorPage::from(state)
            .with_status(StatusCode::NOT_FOUND)
            .into_response());
    };

    let path = uri
        .path()
        .strip_prefix(&format!("/{repo_name}/"))
        .unwrap_or_default();

    let path = repo.path().join(path);

    // cant canonicalize if it doesnt exist
    if !path.exists() {
        return Ok(ErrorPage::from(state)
            .with_status(StatusCode::NOT_FOUND)
            .into_response());
    }

    let path = path.canonicalize().context("canonicalize new path")?;

    // that path got us outside of the repository structure somehow
    if !path.starts_with(repo.path()) {
        tracing::warn!("Attempt to acces file outside of repo dir: {:?}", path);
        return Ok(ErrorPage::from(state)
            .with_status(StatusCode::FORBIDDEN)
            .into_response());
    }

    // Either the requested resource does not exist or it is not
    // a file, i.e. a directory.
    if !path.is_file() {
        return Ok(ErrorPage::from(state)
            .with_status(StatusCode::NOT_FOUND)
            .into_response());
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
