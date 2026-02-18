use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse as _, Response},
};
use git2::Commit;

use crate::utils::{
    Error, Result, error::Context as _, extractor::repo_name_checks, filters, git::Repository,
    response::Xml, spawn_blocking,
};

#[derive(askama::Template)]
#[template(path = "log.xml")]
struct RepoLogFeedTemplate<'a> {
    repo: &'a Repository,
    commits: Vec<Commit<'a>>,
    branch: String,
    base_url: &'a str,
}

#[tracing::instrument(skip_all)]
pub async fn get_1(Path(repo_name): Path<String>) -> Response {
    spawn_blocking(move || inner(&repo_name, None).into_response()).await
}

#[tracing::instrument(skip_all)]
pub async fn get_2(Path((repo_name, r#ref)): Path<(String, String)>) -> Response {
    spawn_blocking(move || inner(&repo_name, Some(&r#ref)).into_response()).await
}

fn inner(repo_name: &str, r#ref: Option<&str>) -> Result {
    repo_name_checks(repo_name)?;

    let Some(repo) = Repository::open(repo_name).context("opening repository")? else {
        return Err(Error::new(StatusCode::NOT_FOUND, "repo does not exist"));
    };

    if repo.is_empty()? {
        // show a server error
        return Err(Error::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "Cannot show feed because there are no commits.",
        ));
    }

    let r = r#ref.unwrap_or("HEAD");

    let Some(commits) = repo.commits(r, crate::config().log_per_page)? else {
        return Err(Error::new(StatusCode::NOT_FOUND, "crepo does not exist"));
    };

    let branch = repo.ref_or_head_shorthand(r#ref)?;

    Ok(Xml(RepoLogFeedTemplate {
        repo: &repo,
        commits,
        branch,
        base_url: &format!("/{repo_name}"),
    })
    .into_response())
}
