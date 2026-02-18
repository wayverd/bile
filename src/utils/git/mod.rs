mod branch;
mod commit;
mod core;
mod tag;
mod tree;

use std::path::Path;

use git2::{Object, Signature};

use crate::utils::error::{Context as _, Result};

pub struct TagEntry {
    pub link: String,
    pub tag: String,
    pub message: String,
    pub signature: Signature<'static>,
}

impl TagEntry {
    #[tracing::instrument(skip_all)]
    fn try_from_commit(name: String, obj: &Object<'_>) -> Result<Self> {
        Ok(Self {
            link: format!("refs/{name}"),
            tag: name,
            message: String::new(),
            signature: obj
                .as_commit()
                .context("git object is not a commit")?
                .committer()
                .to_owned(),
        })
    }

    #[tracing::instrument(skip_all)]
    fn try_from_tag(name: String, obj: &Object<'_>) -> Result<Self> {
        let tag = obj.as_tag().context("git object is not a tag")?;

        Ok(Self {
            link: format!("refs/{name}"),
            tag: name,
            message: tag.message().unwrap_or("").to_string(),
            signature: tag
                .tagger()
                .context("git tag does not have a tagger")
                .or_else(|_| -> Result<Signature<'_>> {
                    let signature = obj
                        .peel_to_commit()
                        .context("failed to peel object to commit")?
                        .committer()
                        .to_owned();

                    Ok(signature)
                })?
                .to_owned(),
        })
    }
}

pub struct Repository {
    inner: git2::Repository,
}

impl Repository {
    #[tracing::instrument(skip_all)]
    pub fn open<P>(path: P) -> Result<Option<Self>>
    where
        P: AsRef<Path>,
    {
        let config = crate::config();

        let path = path.as_ref();

        let path = config.project_root.join(path);

        if !path.exists() {
            return Ok(None);
        }

        let path = path.canonicalize().context("failed to canonicalize path")?;

        if !path.starts_with(&config.project_root) {
            tracing::warn!(
                root=?config.project_root.display(),
                requested=?path.display(),
                "attempted path traversal",
            );

            return Ok(None);
        }

        if path == config.project_root {
            return Ok(None);
        }

        if !path.exists() {
            return Ok(None);
        }

        let inner = git2::Repository::open(path).context("failed to actually open the repo")?;

        if !inner.path().join(&config.export_ok).exists() {
            tracing::warn!("tried to access private repo");

            return Ok(None);
        }

        Ok(Some(Self { inner }))
    }

    #[must_use]
    pub const fn as_inner(&self) -> &git2::Repository {
        &self.inner
    }
}
