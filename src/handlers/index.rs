use std::fs;

use axum::response::{IntoResponse as _, Response};

use crate::utils::{
    Result, error::Context as _, filters, git::Repository, response::Html, spawn_blocking,
};

#[derive(askama::Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    repos: Vec<Repository>,
}

#[tracing::instrument(skip_all)]
pub async fn get() -> Response {
    spawn_blocking(move || inner().into_response()).await
}

fn inner() -> Result {
    let Ok(read) = fs::read_dir(&crate::config().project_root) else {
        return Ok(Html(IndexTemplate { repos: Vec::new() }).into_response());
    };

    let config = crate::config();

    let mut repos = Vec::new();

    for entry in read {
        let entry = entry.context("failed to open directory entry")?;

        let Some(repo) = Repository::open(entry.path()).context("failed to open repository")?
        else {
            continue;
        };

        // check for the export file in the git directory
        // (the .git subfolder for non-bare repos)
        if !repo.path().join(&config.export_ok).exists() {
            continue;
        }

        repos.push(repo);
    }

    Ok(Html(IndexTemplate { repos }).into_response())
}
