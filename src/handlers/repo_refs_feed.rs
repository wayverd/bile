use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse as _, Response},
};

use crate::{
    BileState,
    error::{Context as _, Result},
    git::{Repository, TagEntry},
    http::{
        extractor::RepoName,
        path::Path,
        response::{ErrorPage, Xml},
    },
    utils::filters,
};

#[derive(askama::Template)]
#[template(path = "refs.xml")]
struct RepoRefFeedTemplate<'a> {
    repo: &'a Repository,
    tags: Vec<TagEntry>,
    base_url: &'a str,
}

#[tracing::instrument(skip_all)]
pub(crate) async fn get(state: State<BileState>, Path(repo_name): Path<RepoName>) -> Response {
    state.spawn(move |state| inner(&state, &repo_name)).await
}

#[tracing::instrument(skip_all)]
fn inner(state: &BileState, repo_name: &RepoName) -> Result<Response> {
    let Some(repo) = Repository::open(&state.config, repo_name).context("opening repository")?
    else {
        return Ok(ErrorPage::from(state)
            .with_status(StatusCode::NOT_FOUND)
            .into_response());
    };

    if repo.is_empty()? {
        // show a server error
        return Ok(ErrorPage::from(state)
            .with_status(StatusCode::SERVICE_UNAVAILABLE)
            .into_response());
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
