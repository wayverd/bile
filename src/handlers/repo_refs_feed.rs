use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse as _, Response},
};

use crate::utils::{
    Error, Result,
    error::Context as _,
    extractor::repo_name_checks,
    filters,
    git::{Repository, TagEntry},
    response::Xml,
    spawn_blocking,
};

#[derive(askama::Template)]
#[template(path = "refs.xml")]
struct RepoRefFeedTemplate<'a> {
    repo: &'a Repository,
    tags: Vec<TagEntry>,
    base_url: &'a str,
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
        // show a server error
        return Err(Error::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "Cannot show feed because there is nothing here.",
        ));
    }

    let mut tags = repo.tag_entries()?;

    // sort so that newest tags are at the top
    tags.sort_unstable_by(|a, b| a.signature.when().cmp(&b.signature.when()).reverse());

    Ok(Xml(RepoRefFeedTemplate {
        repo: &repo,
        tags,
        base_url: &format!("/{repo_name}"),
    })
    .into_response())
}
