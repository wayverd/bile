use std::{fmt::Write as _, path};

use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse as _, Response},
};
use git2::{Blob, Commit, Tree};
use syntect::{
    html::{ClassStyle, ClassedHTMLGenerator},
    util::LinesWithEndings,
};

use crate::{
    SYNTAXES,
    utils::{
        Error, Result, blob_mime,
        error::Context as _,
        extractor::repo_name_checks,
        filters,
        git::Repository,
        response::{Html, Redirect},
        spawn_blocking,
    },
};

#[derive(askama::Template)]
#[template(path = "tree.html")]
struct RepoTreeTemplate<'a> {
    repo: &'a Repository,
    tree: Tree<'a>,
    path: &'a path::Path,
    spec: &'a str,
    last_commit: Commit<'a>,
}

#[derive(askama::Template)]
#[template(path = "file.html")]
struct RepoFileTemplate<'a> {
    repo: &'a Repository,
    path: &'a path::Path,
    file_text: &'a str,
    spec: &'a str,
    last_commit: Commit<'a>,
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

    let Some(repo) = Repository::open(repo_name).context("opening repository")? else {
        return Err(Error::new(StatusCode::NOT_FOUND, "repo does not exist"));
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
        return Err(Error::new(
            StatusCode::NOT_FOUND,
            "commit does not exist in repo",
        ));
    };

    let (path, tree_obj) = if let Some(path) = object_name {
        let path = path::Path::new(path);

        (path, repo.tree_object(&tree, path)?)
    } else {
        (path::Path::new(""), Some(tree.into_object()))
    };

    let Some(tree_obj) = tree_obj else {
        return Err(Error::new(
            StatusCode::NOT_FOUND,
            "file does not exist into repo",
        ));
    };

    let Some(last_commit) = repo.file_last_commit(&spec, path)? else {
        return Err(Error::new(
            StatusCode::NOT_FOUND,
            "commit does not exist in repo",
        ));
    };

    let tree_obj = match tree_obj.into_tree() {
        // this is a subtree
        Ok(sub_tree) => {
            return Ok(Html(RepoTreeTemplate {
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
        return Err(Error::new(StatusCode::NOT_FOUND, "File not found"));
    };

    let output = render(repo_name, path, &spec, &commit, blob)?;

    Ok(Html(RepoFileTemplate {
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
    repo_name: &str,
    path: &path::Path,
    spec: &str,
    commit: &Commit<'_>,
    blob: &Blob<'_>,
) -> crate::utils::error::Result<String> {
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

    let syntax = SYNTAXES
        .find_syntax_by_extension(extension)
        .unwrap_or_else(|| SYNTAXES.find_syntax_plain_text());

    // get file contents from git object
    let file_string = str::from_utf8(blob.content())?;

    // create a highlighter that uses CSS classes so we can use prefers-color-scheme
    let mut highlighter =
        ClassedHTMLGenerator::new_with_class_style(syntax, &SYNTAXES, ClassStyle::Spaced);
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
