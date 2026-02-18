use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse as _, Response},
};
use git2::Commit;

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
#[template(path = "log.html")]
struct RepoLogTemplate<'a> {
    repo: &'a Repository,
    commits: Vec<Commit<'a>>,
    branch: String,
    // the spec the user should be linked to to see the next page of commits
    next_page: Option<String>,
}

#[tracing::instrument(skip_all)]
pub async fn get_1(Path(repo_name): Path<String>) -> Response {
    spawn_blocking(move || inner(&repo_name, None, None).into_response()).await
}

#[tracing::instrument(skip_all)]
pub async fn get_2(Path((repo_name, r#ref)): Path<(String, String)>) -> Response {
    spawn_blocking(move || inner(&repo_name, Some(&r#ref), None).into_response()).await
}

#[tracing::instrument(skip_all)]
pub async fn get_3(
    Path((repo_name, r#ref, object_name)): Path<(String, String, String)>,
) -> Response {
    spawn_blocking(move || inner(&repo_name, Some(&r#ref), Some(&object_name)).into_response())
        .await
}

fn inner(repo_name: &str, r#ref: Option<&str>, object_name: Option<&str>) -> Result {
    repo_name_checks(repo_name)?;

    let config = crate::config();

    let Some(repo) = Repository::open(repo_name).context("opening repository")? else {
        return Err(Error::new(StatusCode::NOT_FOUND, "repo does not exist"));
    };

    if repo.is_empty()? {
        return Ok(Redirect::permanent(&format!("/{repo_name}"))
            .unwrap_or(Redirect::PERMANENT_ROOT)
            .into_response());
    }

    let r = r#ref.unwrap_or("HEAD");

    let next_page_spec = if repo.is_shallow() {
        String::new()
    } else if let Some(i) = r.rfind('~') {
        // there is a tilde, try to find a number too
        let n = r[i + 1..].parse::<usize>().ok().unwrap_or(1);
        format!("{}~{}", &r[..i], n + config.log_per_page)
    } else {
        // there was no tilde
        format!("{}~{}", r, config.log_per_page)
    };

    let Some(mut commits) = repo
        .commits_for_obj(r, config.log_per_page + 1, object_name)
        .context("failed to get commits for object")?
    else {
        return Err(Error::new(StatusCode::NOT_FOUND, "entry does not exist"));
    };

    // check if there even is a next page
    let next_page = if commits.len() < config.log_per_page + 1 {
        None
    } else {
        // remove additional commit from next page check
        commits.pop();
        Some(next_page_spec)
    };

    let branch = repo.ref_or_head_shorthand(r#ref)?;

    Ok(Html(RepoLogTemplate {
        repo: &repo,
        commits,
        branch,
        next_page,
    })
    .into_response())
}
