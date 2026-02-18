use std::fmt::Write as _;

use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse as _, Response},
};
use git2::{
    BranchType, Commit, DescribeFormatOptions, DescribeOptions, Diff, DiffFindOptions, DiffFormat,
};
use syntect::{
    html::{ClassStyle, ClassedHTMLGenerator},
    util::LinesWithEndings,
};

use crate::{
    SYNTAXES,
    utils::{
        Error, Result,
        error::Context as _,
        extractor::{commit_checks, repo_name_checks},
        filters,
        git::Repository,
        response::Html,
        spawn_blocking,
    },
};

#[derive(askama::Template)]
#[template(path = "commit.html")]
struct RepoCommitTemplate<'a> {
    repo: &'a Repository,
    commit: Commit<'a>,
    diff: &'a Diff<'a>,
}

impl RepoCommitTemplate<'_> {
    fn parent_ids(&self) -> Vec<git2::Oid> {
        self.commit.parent_ids().collect()
    }

    fn diff(&self) -> String {
        let mut buf = String::new();

        let _ = self.diff.print(DiffFormat::Patch, |_, _, line| {
            let Ok(content) = str::from_utf8(line.content()) else {
                buf.push_str("Cannot display diff for binary file.");

                return false;
            };

            match line.origin() {
                'F' | 'H' => {}
                c if matches!(c, ' ' | '+' | '-' | '=' | '<' | '>') => buf.push(c),
                _ => unreachable!(),
            }

            buf.push_str(content);

            true
        });

        // highlight the diff
        let syntax = SYNTAXES
            .find_syntax_by_name("Diff")
            .expect("diff syntax missing");

        let mut highlighter =
            ClassedHTMLGenerator::new_with_class_style(syntax, &SYNTAXES, ClassStyle::Spaced);

        LinesWithEndings::from(&buf).for_each(|line| {
            if let Err(err) = highlighter.parse_html_for_line_which_includes_newline(line) {
                tracing::error!(err=?err, "failed to highlight code");
            }
        });

        highlighter.finalize()
    }

    fn refs(&self) -> String {
        let mut html = String::new();

        // add badge if this commit is a tag
        let descr = self.commit.as_object().describe(
            DescribeOptions::new()
                .describe_tags()
                .max_candidates_tags(0),
        );
        if let Ok(descr) = descr {
            // this can be a tag or lightweight tag, the refs path will redirect
            let _ = write!(
                &mut html,
                r#"<a href="/{0}/refs/{1}" class="badge tag">{1}</a>"#,
                self.repo.name().unwrap_or("<unknown>"),
                descr
                    .format(Some(DescribeFormatOptions::new().abbreviated_size(0)))
                    .unwrap_or_else(|_| "<unknown>".to_string()),
            );
        }

        // also add badge if this is the tip of a branch
        let branches = match self.repo.branches_of_type(BranchType::Local) {
            Ok(branches) => branches,
            Err(err) => {
                tracing::error!(err=?err, "failed to create branches iterator");
                return html;
            }
        };
        let branches = branches.into_iter().filter(|branch| {
            branch.get().peel_to_commit().map(|commit| commit.id()) == Ok(self.commit.id())
        });
        for branch in branches {
            // branch is not a reference, just a fancy name for a commit
            let _ = write!(
                &mut html,
                r#" <a href="/{0}/log/{1}" class="badge branch">{1}</a>"#,
                self.repo.name().unwrap_or("<unknown>"),
                branch
                    .name()
                    .unwrap_or(Some("<unknown>"))
                    .unwrap_or("<unknown>"),
            );
        }

        html
    }
}

#[tracing::instrument(skip_all)]
pub async fn get(Path((repo_name, commit)): Path<(String, String)>) -> Response {
    spawn_blocking(move || inner(&repo_name, &commit).into_response()).await
}

#[tracing::instrument(skip_all)]
fn inner(repo_name: &str, commit: &str) -> Result {
    repo_name_checks(repo_name)?;
    commit_checks(commit)?;

    let Some(repo) = Repository::open(repo_name).context("opening repository")? else {
        return Err(Error::new(StatusCode::NOT_FOUND, "repo does not exist"));
    };

    let Some(commit) = repo.commit(commit).context("failed to get commit")? else {
        return Err(Error::new(
            StatusCode::NOT_FOUND,
            "commit does not exist in repo",
        ));
    };

    let mut diff = repo
        .commit_diff(&commit)
        .context("failed to get commits diff")?;

    let mut find_options = DiffFindOptions::new();
    // try to find moved/renamed files
    find_options.all(true);
    if let Err(err) = diff.find_similar(Some(&mut find_options)) {
        tracing::error!(err=?err, "failed to mark similar files in diff");
    }

    Ok(Html(RepoCommitTemplate {
        repo: &repo,
        commit,
        diff: &diff,
    })
    .into_response())
}
