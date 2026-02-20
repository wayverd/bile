use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse as _, Response},
};

use crate::{
    BileState,
    config::Config,
    error::Context as _,
    git::Repository,
    http::{
        extractor::RepoName,
        path::Path,
        response::{ErrorPage, Html, Result},
    },
    utils::filters,
};

#[derive(askama::Template)]
#[template(path = "repo.html")]
struct RepoHomeTemplate<'a> {
    config: &'a Config,
    repo: &'a Repository,
    commits: Vec<git2::Commit<'a>>,
    readme_text: String,
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

    let readme_text = repo.readme(&state.syntax);

    // TODO: let r = req.param("ref").unwrap_or("HEAD");
    let r = "HEAD";
    let Some(commits) = repo.commits(r, 3)? else {
        return Ok(ErrorPage::from(state)
            .with_status(StatusCode::NOT_FOUND)
            .into_response());
    };

    Ok(Html(RepoHomeTemplate {
        config: &state.config,
        repo: &repo,
        commits,
        readme_text,
    })
    .into_response())
}
