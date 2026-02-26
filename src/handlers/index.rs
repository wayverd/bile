use std::fs;

use axum::{
    extract::State,
    response::{IntoResponse as _, Response},
};

use crate::{
    BileState,
    config::Config,
    error::{Context as _, Result},
    git::Repository,
    http::response::Html,
    utils::filters,
};

#[derive(askama::Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    config: &'a Config,
    sections: Vec<Section>,
}

struct Section {
    name: Option<String>,
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
            sections: Vec::new(),
        })
        .into_response());
    };

    let mut sections = Vec::new();

    for entry in read {
        let entry = entry.context("failed to open directory entry")?;
        let metadata = entry.metadata().context("failed to get file metadata")?;

        if !metadata.is_dir() {
            continue;
        }

        if entry
            .file_name()
            .to_str()
            .is_some_and(|p| p != "." && p.starts_with('.'))
        {
            continue;
        }

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

        let repo_section = repo.section();
        let section = sections
            .iter_mut()
            .find(|s: &&mut Section| s.name == repo_section);
        match section {
            Some(section) => section.repos.push(repo),
            None => sections.push(Section {
                name: repo_section,
                repos: vec![repo],
            }),
        }
    }

    sections.sort_by(|a, b| a.name.cmp(&b.name));
    for section in &mut sections {
        section.repos.sort_by(|a, b| a.name().cmp(&b.name()));
    }

    Ok(Html(IndexTemplate {
        config: &state.config,
        sections,
    })
    .into_response())
}
