use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::{
    BileState,
    config::Config,
    error::Context as _,
    git::Repository,
    http::{
        extractor::{RepoName, Tag},
        path::Path,
        response::{ErrorPage, Html, Redirect, Result},
    },
    utils::filters,
};

#[derive(askama::Template)]
#[template(path = "tag.html")]
struct Template<'a> {
    config: &'a Config,
    repo: &'a Repository,
    tag: git2::Tag<'a>,
}

#[tracing::instrument(skip_all)]
pub(crate) async fn get(
    state: State<BileState>,
    Path((repo_name, tag)): Path<(RepoName, Tag)>,
) -> Response {
    state
        .spawn(move |state| inner(&state, &repo_name, &tag))
        .await
}

#[tracing::instrument(skip_all)]
fn inner(state: &BileState, repo_name: &RepoName, tag: &Tag) -> Result<Response> {
    let Some(repo) = Repository::open(&state.config, repo_name).context("opening repository")?
    else {
        return Ok(ErrorPage::from(state)
            .with_status(StatusCode::NOT_FOUND)
            .into_response());
    };

    let Ok(repo_tag) = repo.tag(tag) else {
        return Ok(Redirect::permanent(&format!("/{repo_name}/commit/{tag}"))
            .unwrap_or(Redirect::PERMANENT_ROOT)
            .into_response());
    };

    Ok(Html(Template {
        config: &state.config,
        repo: &repo,
        tag: repo_tag,
    })
    .into_response())
}
