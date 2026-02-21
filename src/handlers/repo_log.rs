use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse as _, Response},
};

use crate::{
    BileState,
    config::Config,
    error::{Context as _, Result},
    git::Repository,
    http::{
        extractor::{ObjectName, Ref, RepoName},
        path::Path,
        response::{ErrorPage, Html, Redirect},
    },
    utils::filters,
};

#[derive(askama::Template)]
#[template(path = "log.html")]
struct RepoLogTemplate<'a> {
    config: &'a Config,
    repo: &'a Repository,
    commits: Vec<git2::Commit<'a>>,
    branch: String,
    // the spec the user should be linked to to see the next page of commits
    next_page: Option<String>,
}

#[tracing::instrument(skip_all)]
pub(crate) async fn get_1(state: State<BileState>, Path(repo_name): Path<RepoName>) -> Response {
    state
        .spawn(move |state| inner(&state, &repo_name, None, None))
        .await
}

#[tracing::instrument(skip_all)]
pub(crate) async fn get_2(
    state: State<BileState>,
    Path((repo_name, r#ref)): Path<(RepoName, Ref)>,
) -> Response {
    state
        .spawn(move |state| inner(&state, &repo_name, Some(&r#ref), None))
        .await
}

#[tracing::instrument(skip_all)]
pub(crate) async fn get_3(
    state: State<BileState>,
    Path((repo_name, r#ref, object_name)): Path<(RepoName, Ref, ObjectName)>,
) -> Response {
    state
        .spawn(move |state| inner(&state, &repo_name, Some(&r#ref), Some(&object_name)))
        .await
}

fn inner(
    state: &BileState,
    repo_name: &RepoName,
    r#ref: Option<&Ref>,
    object_name: Option<&ObjectName>,
) -> Result<Response> {
    let Some(repo) = Repository::open(&state.config, repo_name).context("opening repository")?
    else {
        return Ok(ErrorPage::from(state)
            .with_status(StatusCode::NOT_FOUND)
            .into_response());
    };

    if repo.is_empty()? {
        return Ok(Redirect::permanent(&format!("/{repo_name}"))
            .unwrap_or(Redirect::PERMANENT_ROOT)
            .into_response());
    }

    let r = r#ref.map_or("HEAD", |r| r.0.as_str());

    let next_page_spec = if repo.is_shallow() {
        String::new()
    } else if let Some(i) = r.rfind('~') {
        // there is a tilde, try to find a number too
        let n = r[i + 1..].parse::<usize>().ok().unwrap_or(1);
        format!("{}~{}", &r[..i], n + state.config.log_per_page)
    } else {
        // there was no tilde
        format!("{}~{}", r, state.config.log_per_page)
    };

    let Some(mut commits) = repo
        .commits_for_obj(r, state.config.log_per_page + 1, object_name)
        .context("failed to get commits for object")?
    else {
        return Ok(ErrorPage::from(state)
            .with_status(StatusCode::NOT_FOUND)
            .into_response());
    };

    // check if there even is a next page
    let next_page = if commits.len() < state.config.log_per_page + 1 {
        None
    } else {
        // remove additional commit from next page check
        commits.pop();
        Some(next_page_spec)
    };

    let branch = repo.ref_or_head_shorthand(r#ref)?;

    Ok(Html(RepoLogTemplate {
        config: &state.config,
        repo: &repo,
        commits,
        branch,
        next_page,
    })
    .into_response())
}
