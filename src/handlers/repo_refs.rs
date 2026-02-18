use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse as _, Response},
};
use git2::Reference;

use crate::utils::{
    Error, Result,
    error::Context as _,
    extractor::repo_name_checks,
    filters,
    git::{Repository, TagEntry},
    response::{Html, Redirect},
    spawn_blocking,
};

#[derive(askama::Template)]
#[template(path = "refs.html")]
struct RepoRefTemplate<'a> {
    repo: &'a Repository,
    branches: Vec<Reference<'a>>,
    tags: Vec<TagEntry>,
}

#[tracing::instrument(skip_all)]
pub async fn get(Path(repo_name): Path<String>) -> Response {
    spawn_blocking(move || inner(&repo_name).into_response()).await
}

#[tracing::instrument(skip_all)]
fn inner(repo_name: &str) -> Result {
    repo_name_checks(repo_name)?;

    let Some(repo) = Repository::open(repo_name).context("opening repository")? else {
        return Err(Error::new(StatusCode::NOT_FOUND, "repo does not exist"));
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
        repo: &repo,
        branches,
        tags,
    })
    .into_response())
}
