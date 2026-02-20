use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse as _, Response},
};

use crate::{
    BileState,
    error::Context as _,
    git::Repository,
    http::{
        extractor::{Ref, RepoName},
        path::Path,
        response::{ErrorPage, Result, Xml},
    },
    utils::filters,
};

#[derive(askama::Template)]
#[template(path = "log.xml")]
struct RepoLogFeedTemplate<'a> {
    repo: &'a Repository,
    commits: Vec<git2::Commit<'a>>,
    branch: String,
    base_url: &'a str,
}

#[tracing::instrument(skip_all)]
pub(crate) async fn get_1(state: State<BileState>, Path(repo_name): Path<RepoName>) -> Response {
    state
        .spawn(move |state| inner(&state, &repo_name, None))
        .await
}

#[tracing::instrument(skip_all)]
pub(crate) async fn get_2(
    state: State<BileState>,
    Path((repo_name, r#ref)): Path<(RepoName, Ref)>,
) -> Response {
    state
        .spawn(move |state| inner(&state, &repo_name, Some(&r#ref)))
        .await
}

fn inner(state: &BileState, repo_name: &RepoName, r#ref: Option<&Ref>) -> Result<Response> {
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

    let r = r#ref.map_or("HEAD", |r| r.0.as_str());

    let Some(commits) = repo.commits(r, state.config.log_per_page)? else {
        return Ok(ErrorPage::from(state)
            .with_status(StatusCode::NOT_FOUND)
            .into_response());
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
