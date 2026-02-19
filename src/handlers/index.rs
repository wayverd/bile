use std::fs;

use axum::{
    extract::State,
    response::{IntoResponse as _, Response},
};

use crate::{
    BileState,
    config::Config,
    error::Context as _,
    git::Repository,
    http::response::{Html, Result},
    utils::filters,
};

#[derive(askama::Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    config: &'a Config,
    repos: Vec<Repository>,
}

#[tracing::instrument(skip_all)]
pub(crate) async fn get(state: State<BileState>) -> Response {
    state.spawn(move |state| inner(&state)).await
}

fn inner(state: &BileState) -> Result<Response> {
    let Ok(read) = fs::read_dir(&state.config.project_root) else {
        return Ok(Html(IndexTemplate {
            config: &state.config,
            repos: Vec::new(),
        })
        .into_response());
    };

    let mut repos = Vec::new();

    for entry in read {
        let entry = entry.context("failed to open directory entry")?;

        let Some(repo) = Repository::open_path(&state.config, &entry.path())
            .context("failed to open repository")?
        else {
            continue;
        };

        // check for the export file in the git directory
        // (the .git subfolder for non-bare repos)
        if !repo.path().join(&state.config.export_ok).exists() {
            continue;
        }

        repos.push(repo);
    }

    Ok(Html(IndexTemplate {
        config: &state.config,
        repos,
    })
    .into_response())
}
