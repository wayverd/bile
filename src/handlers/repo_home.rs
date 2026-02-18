use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse as _, Response},
};
use git2::Commit;

use crate::utils::{
    Error, Result, error::Context as _, extractor::repo_name_checks, filters, git::Repository,
    response::Html, spawn_blocking,
};

#[derive(askama::Template)]
#[template(path = "repo.html")]
struct RepoHomeTemplate<'a> {
    repo: &'a Repository,
    commits: Vec<Commit<'a>>,
    readme_text: String,
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

    let readme_text = repo.readme();

    // TODO: let r = req.param("ref").unwrap_or("HEAD");
    let r = "HEAD";
    let Some(commits) = repo.commits(r, 3)? else {
        return Err(Error::new(StatusCode::NOT_FOUND, "crepo does not exist"));
    };

    Ok(Html(RepoHomeTemplate {
        repo: &repo,
        commits,
        readme_text,
    })
    .into_response())
}
