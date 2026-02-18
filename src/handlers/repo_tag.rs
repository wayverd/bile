use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use git2::Tag;

use crate::utils::{
    Error, Result,
    error::Context as _,
    extractor::repo_name_checks,
    filters,
    git::Repository,
    response::{Html, Redirect},
    spawn_blocking,
};

#[derive(askama::Template)]
#[template(path = "tag.html")]
struct Template<'a> {
    repo: &'a Repository,
    tag: Tag<'a>,
}

#[tracing::instrument(skip_all)]
pub async fn get(Path((repo_name, tag)): Path<(String, String)>) -> Response {
    spawn_blocking(move || inner(&repo_name, &tag).into_response()).await
}

#[tracing::instrument(skip_all)]
fn inner(repo_name: &str, tag: &str) -> Result {
    repo_name_checks(repo_name)?;

    let Some(repo) = Repository::open(repo_name).context("opening repository")? else {
        return Err(Error::new(StatusCode::NOT_FOUND, "repo does not exist"));
    };

    let Ok(repo_tag) = repo.tag(tag) else {
        return Ok(Redirect::permanent(&format!("/{repo_name}/commit/{tag}"))
            .unwrap_or(Redirect::PERMANENT_ROOT)
            .into_response());
    };

    Ok(Html(Template {
        repo: &repo,
        tag: repo_tag,
    })
    .into_response())
}
