use std::{fmt::Write as _, path};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse as _, Response},
};
use syntect::{
    html::{ClassStyle, ClassedHTMLGenerator},
    parsing::SyntaxSet,
    util::LinesWithEndings,
};

use crate::{
    BileState,
    config::Config,
    error::Context as _,
    git::Repository,
    http::{
        extractor::{ObjectName, Ref, RepoName},
        response::{ErrorPage, Html, Redirect, Result},
    },
    utils::{blob_mime, filters},
};

#[derive(askama::Template)]
#[template(path = "tree.html")]
struct RepoTreeTemplate<'a> {
    config: &'a Config,
    repo: &'a Repository,
    tree: git2::Tree<'a>,
    path: &'a path::Path,
    spec: &'a str,
    last_commit: git2::Commit<'a>,
}

#[derive(askama::Template)]
#[template(path = "file.html")]
struct RepoFileTemplate<'a> {
    config: &'a Config,
    repo: &'a Repository,
    path: &'a path::Path,
    file_text: &'a str,
    spec: &'a str,
    last_commit: git2::Commit<'a>,
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
        return Ok(ErrorPage::new(&state.config)
            .with_status(StatusCode::NOT_FOUND)
            .into_response());
    };

    if repo.is_empty()? {
        return Ok(Redirect::permanent(&format!("/{repo_name}"))
            .unwrap_or(Redirect::PERMANENT_ROOT)
            .into_response());
    }

    let spec = repo.ref_or_head_shorthand(r#ref)?;
    let Some((commit, tree)) = repo
        .commit_tree(&spec)
        .context("failed to get commit tree")?
    else {
        return Ok(ErrorPage::new(&state.config)
            .with_status(StatusCode::NOT_FOUND)
            .into_response());
    };

    let (path, tree_obj) = if let Some(path) = object_name {
        let path = path::Path::new(&path.0);

        (path, repo.tree_object(&tree, path)?)
    } else {
        (path::Path::new(""), Some(tree.into_object()))
    };

    let Some(tree_obj) = tree_obj else {
        return Ok(ErrorPage::new(&state.config)
            .with_status(StatusCode::NOT_FOUND)
            .into_response());
    };

    let Some(last_commit) = repo.file_last_commit(&spec, path)? else {
        return Ok(ErrorPage::new(&state.config)
            .with_status(StatusCode::NOT_FOUND)
            .into_response());
    };

    let tree_obj = match tree_obj.into_tree() {
        // this is a subtree
        Ok(sub_tree) => {
            return Ok(Html(RepoTreeTemplate {
                config: &state.config,
                repo: &repo,
                tree: sub_tree,
                path,
                spec: &spec,
                last_commit,
            })
            .into_response());
        }
        // this is not a subtree, so it should be a blob i.e. file
        Err(tree_obj) => tree_obj,
    };

    let Some(blob) = tree_obj.as_blob() else {
        return Ok(ErrorPage::new(&state.config)
            .with_status(StatusCode::NOT_FOUND)
            .into_response());
    };

    let output = render(&state.syntax, repo_name, path, &spec, &commit, blob)?;

    Ok(Html(RepoFileTemplate {
        config: &state.config,
        repo: &repo,
        path,
        file_text: &output,
        spec: &spec,
        last_commit,
    })
    .into_response())
}

// TODO: make sure I am escaping html properly here
// TODO: allow disabling of syntax highlighting
// TODO: -- dont pull in memory, use iterators if possible
fn render(
    syntaxes: &SyntaxSet,
    repo_name: &RepoName,
    path: &path::Path,
    spec: &str,
    commit: &git2::Commit<'_>,
    blob: &git2::Blob<'_>,
) -> crate::error::Result<String> {
    let extension = path
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or_default();

    if blob.is_binary() {
        // this is not a text file, but try to serve the file if the MIME type
        // can give a hint at how
        let mime = blob_mime(blob, extension);

        let output = match mime.type_() {
            mime::TEXT => unreachable!("git detected this file as binary"),
            mime::IMAGE => format!(
                "<img src=\"/{}/tree/{}/raw/{}\" />",
                repo_name,
                spec,
                path.display()
            ),
            tag @ (mime::AUDIO | mime::VIDEO) => format!(
                "<{} src=\"/{}/tree/{}/raw/{}\" controls>Your browser does not have support for playing this {0} file.</{0}>",
                tag,
                repo_name,
                spec,
                path.display()
            ),
            _ => "Cannot display binary file.".to_string(),
        };

        return Ok(output);
    }

    let syntax = syntaxes
        .find_syntax_by_extension(extension)
        .unwrap_or_else(|| syntaxes.find_syntax_plain_text());

    // get file contents from git object
    let file_string = str::from_utf8(blob.content())?;

    // create a highlighter that uses CSS classes so we can use prefers-color-scheme
    let mut highlighter =
        ClassedHTMLGenerator::new_with_class_style(syntax, syntaxes, ClassStyle::Spaced);
    LinesWithEndings::from(file_string).for_each(|line| {
        if let Err(err) = highlighter.parse_html_for_line_which_includes_newline(line) {
            tracing::error!(err=?err, "failed to highlight code");
        }
    });

    // use oid so it is a permalink
    let prefix = format!(
        "/{}/tree/{}/item/{}",
        repo_name,
        commit.id(),
        path.display()
    );

    let mut output = String::from("<pre>\n");
    for (n, line) in highlighter.finalize().lines().enumerate() {
        let _ = writeln!(
            &mut output,
            "<a href='{1}#L{0}' id='L{0}' class='line'>{0}</a>{2}",
            n + 1,
            prefix,
            line,
        );
    }
    output.push_str("</pre>\n");

    Ok(output)
}
