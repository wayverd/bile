use std::path::Path;

use git2::{Reference, Time};

use crate::utils::{
    error::{Context as _, Result},
    git::Repository,
};

impl Repository {
    #[must_use]
    pub fn description(&self) -> String {
        let content = std::fs::read_to_string(self.path().join("description")).unwrap_or_default();

        let first = content.lines().next().unwrap_or_default();

        first.to_string()
    }

    #[tracing::instrument(skip_all)]
    pub fn head(&self) -> Result<Reference<'_>> {
        let head = self.inner.head()?;

        Ok(head)
    }

    #[tracing::instrument(skip_all)]
    pub fn is_empty(&self) -> Result<bool> {
        Ok(self.inner.is_empty()?)
    }

    #[must_use]
    pub fn is_shallow(&self) -> bool {
        self.inner.is_shallow()
    }

    #[tracing::instrument(skip_all)]
    pub fn last_modified(&self) -> Result<Time> {
        let head = self.head()?;
        let commit = head.peel_to_commit()?;
        let time = commit.committer().when();

        Ok(time)
    }

    pub fn name(&self) -> Option<&str> {
        self.inner
            .workdir()
            // use the path for bare repositories
            .unwrap_or_else(|| self.inner.path())
            .file_name()
            .and_then(std::ffi::OsStr::to_str)
    }

    #[must_use]
    pub fn owner(&self) -> String {
        self.inner
            .config()
            .and_then(|config| config.get_string("gitweb.owner"))
            .unwrap_or_default()
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        self.inner.path()
    }

    #[must_use]
    pub fn readme(&self) -> String {
        use askama::filters::Escaper as _;

        enum ReadmeFormat {
            Plaintext,
            Html,
            Markdown,
        }

        let mut format = ReadmeFormat::Plaintext;

        self.inner
            .revparse_single("HEAD:readme")
            .or_else(|_| self.inner.revparse_single("HEAD:README.txt"))
            .or_else(|_| self.inner.revparse_single("HEAD:readme.txt"))
            .or_else(|_| self.inner.revparse_single("HEAD:README.txt"))
            .or_else(|_| {
                format = ReadmeFormat::Markdown;
                self.inner.revparse_single("HEAD:readme.md")
            })
            .or_else(|_| self.inner.revparse_single("HEAD:README.mdown"))
            .or_else(|_| self.inner.revparse_single("HEAD:readme.mdown"))
            .or_else(|_| self.inner.revparse_single("HEAD:README.mdown"))
            .or_else(|_| self.inner.revparse_single("HEAD:readme.markdown"))
            .or_else(|_| self.inner.revparse_single("HEAD:README.markdown"))
            .or_else(|_| {
                format = ReadmeFormat::Html;
                self.inner.revparse_single("HEAD:readme.html")
            })
            .or_else(|_| self.inner.revparse_single("HEAD:README.html"))
            .or_else(|_| self.inner.revparse_single("HEAD:readme.htm"))
            .or_else(|_| self.inner.revparse_single("HEAD:README.htm"))
            .ok()
            .and_then(|readme| readme.into_blob().ok())
            .map(|blob| {
                let text = str::from_utf8(blob.content()).unwrap_or_default();

                // render the file contents to HTML
                match format {
                    // render plaintext as preformatted text
                    ReadmeFormat::Plaintext => {
                        let mut output = "<pre>".to_string();
                        if let Err(err) = askama::filters::Html.write_escaped_str(&mut output, text)
                        {
                            tracing::error!(err=?err, "failed to write escaped plaintext readme");
                        }
                        output.push_str("</pre>");
                        output
                    }
                    // already is HTML
                    ReadmeFormat::Html => text.to_string(),
                    // render Markdown to HTML
                    ReadmeFormat::Markdown => crate::utils::markdown::render(text),
                }
            })
            .unwrap_or_default()
    }

    #[tracing::instrument(skip_all)]
    pub fn ref_or_head_shorthand(&self, r#ref: Option<&str>) -> Result<String> {
        let head = self.head()?;

        let spec = r#ref
            .map(str::to_string)
            .or_else(move || head.shorthand().map(str::to_string))
            .context("failed to get repo ref spec")?;

        Ok(spec)
    }
}
