use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse as _, Response},
};

use crate::{
    BileState,
    config::Config,
    error::Context as _,
    git::{Repository, TagEntry},
    http::{
        extractor::RepoName,
        response::{ErrorPage, Html, Redirect, Result},
    },
    utils::filters,
};

#[derive(askama::Template)]
#[template(path = "refs.html")]
struct RepoRefTemplate<'a> {
    config: &'a Config,
    repo: &'a Repository,
    branches: Vec<git2::Reference<'a>>,
    tags: Vec<TagEntry>,
}

#[tracing::instrument(skip_all)]
pub(crate) async fn get(state: State<BileState>, Path(repo_name): Path<RepoName>) -> Response {
    state.spawn(move |state| inner(&state, &repo_name)).await
}

#[tracing::instrument(skip_all)]
fn inner(state: &BileState, repo_name: &RepoName) -> Result<Response> {
    let Some(repo) = Repository::open(&state.config, repo_name).context("opening repository")?
    else {
        return Ok(ErrorPage::new(&state.config)
            .with_status(StatusCode::NOT_FOUND)
            .into_response());
    };

    if repo.is_empty()? {
        return Ok(Redirect::permanent(&format!("/{repo_name}"))
            .unwrap_or(Redirect::PERMANENT_ROOT)
            .into_response());
    }

    let branches = repo.branches()?;

    let mut tags = repo.tag_entries()?;

    // sort so that newest tags are at the top
    tags.sort_unstable_by(|a, b| a.signature.when().cmp(&b.signature.when()).reverse());

    Ok(Html(RepoRefTemplate {
        config: &state.config,
        repo: &repo,
        branches,
        tags,
    })
    .into_response())
}
