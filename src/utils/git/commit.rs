use std::ffi::CString;

use git2::{Commit, Diff, DiffOptions, DiffStats, Sort, Tree};

use crate::utils::{
    error::{Context as _, Result},
    git::Repository,
};

impl Repository {
    #[tracing::instrument(skip_all)]
    pub fn commit(&self, spec: &str) -> Result<Option<Commit<'_>>> {
        let obj = match self.inner.revparse_single(spec) {
            Ok(obj) => obj,
            Err(err) => {
                tracing::warn!(err=?err, "failed to revparse commit");
                return Ok(None);
            }
        };

        let commit = match obj.peel_to_commit() {
            Ok(commit) => commit,
            Err(err) => {
                tracing::warn!(err=?err, "failed to peel object to commit");
                return Ok(None);
            }
        };

        Ok(Some(commit))
    }

    #[tracing::instrument(skip_all)]
    pub fn commit_diff(&self, commit: &Commit<'_>) -> Result<Diff<'_>> {
        let mut options = DiffOptions::new();

        // This is identical to getting "commit^" and on merges this will be the
        // merged into branch before the merge.
        let parent = commit.parents().next();

        let diff = self.inner.diff_tree_to_tree(
            parent.and_then(|parent| parent.tree().ok()).as_ref(),
            commit.tree().ok().as_ref(),
            Some(&mut options),
        )?;

        Ok(diff)
    }

    #[tracing::instrument(skip_all)]
    pub fn commit_stats(&self, commit: &Commit<'_>) -> Result<DiffStats> {
        let diff = self.commit_diff(commit)?;

        let stats = diff.stats()?;

        Ok(stats)
    }

    #[tracing::instrument(skip_all)]
    pub fn commit_tree(&self, spec: &str) -> Result<Option<(Commit<'_>, Tree<'_>)>> {
        let Some(commit) = self.commit(spec)? else {
            return Ok(None);
        };

        let tree = commit.tree()?;

        Ok(Some((commit, tree)))
    }

    #[tracing::instrument(skip_all)]
    pub fn commits(&self, spec: &str, amount: usize) -> Result<Option<Vec<Commit<'_>>>> {
        if self.is_shallow() {
            return self.commits_shallow();
        }

        self.commits_full(spec, amount)
    }

    #[tracing::instrument(skip_all)]
    pub fn commits_full(&self, spec: &str, amount: usize) -> Result<Option<Vec<Commit<'_>>>> {
        let mut revwalk = self.inner.revwalk()?;

        let Some(commit) = self.commit(spec)? else {
            return Ok(None);
        };

        revwalk.push(commit.id())?;

        revwalk.set_sorting(Sort::TIME)?;

        let commits = revwalk
            .filter_map(|oid| oid.ok().and_then(|oid| self.inner.find_commit(oid).ok()))
            .take(amount)
            .collect();

        Ok(Some(commits))
    }

    #[tracing::instrument(skip_all)]
    pub fn commits_for_obj(
        &self,
        spec: &str,
        amount: usize,
        obj: Option<&str>,
    ) -> Result<Option<Vec<Commit<'_>>>> {
        tracing::info!("is_shallow");
        if self.is_shallow() {
            return self
                .commits_shallow()
                .context("failed to get commits on shallow repo");
        }

        let mut revwalk = self.inner.revwalk().context("failed to create revwalk")?;

        let Some(commit) = self.commit(spec).context("failed to get commit")? else {
            tracing::info!("commit None");
            return Ok(None);
        };

        revwalk
            .push(commit.id())
            .context("failed to set root commit for revwalk")?;

        revwalk
            .set_sorting(Sort::TIME)
            .context("failed to set revwalk sorting mode")?;

        let commits =
            revwalk.filter_map(|oid| oid.ok().and_then(|oid| self.inner.find_commit(oid).ok()));

        let Some(Ok(path)) = obj.map(CString::new) else {
            return Ok(Some(commits.take(amount).collect()));
        };

        // filter for specific file if necessary
        let mut options = DiffOptions::new();
        options.pathspec(path);

        let commits = commits
            .into_iter()
            .filter(|walked_commit| {
                let old_tree = match walked_commit.tree() {
                    Ok(tree) => tree,
                    Err(err) => {
                        tracing::error!(err=?err, "failed to get commit tree");
                        return false;
                    }
                };

                // check that the given file was affected from any of the parents
                walked_commit.parents().any(|parent| {
                    let new_tree = match parent.tree() {
                        Ok(tree) => tree,
                        Err(err) => {
                            tracing::error!(err=?err, "failed to get parent commit tree");
                            return false;
                        }
                    };

                    let diff = match self.inner.diff_tree_to_tree(
                        Some(&old_tree),
                        Some(&new_tree),
                        Some(&mut options),
                    ) {
                        Ok(diff) => diff,
                        Err(err) => {
                            tracing::error!(err=?err, "failed to diff trees");
                            return false;
                        }
                    };

                    let stats = match diff.stats() {
                        Ok(stats) => stats,
                        Err(err) => {
                            tracing::error!(err=?err, "failed to get diff stats");
                            return false;
                        }
                    };

                    stats.files_changed() > 0
                })
            })
            .take(amount)
            .collect();

        Ok(Some(commits))
    }

    #[tracing::instrument(skip_all)]
    pub fn commits_shallow(&self) -> Result<Option<Vec<Commit<'_>>>> {
        tracing::warn!("repository {:?} is only a shallow clone", self.inner.path());
        let commits = self
            .inner
            .head()?
            .peel_to_commit()
            .map(|commit| vec![commit])
            .unwrap_or_default();

        Ok(Some(commits))
    }

    #[tracing::instrument(skip_all)]
    pub fn file_last_commit<P: git2::IntoCString>(
        &self,
        spec: &str,
        path: P,
    ) -> Result<Option<Commit<'_>>> {
        let mut revwalk = self.inner.revwalk()?;

        let Some(commit) = self.commit(spec)? else {
            return Ok(None);
        };

        revwalk.push(commit.id())?;

        revwalk.set_sorting(Sort::TIME)?;

        let mut options = DiffOptions::new();
        options.pathspec(path.into_c_string()?);

        let commit = revwalk
            .filter_map(|oid| oid.ok().and_then(|oid| self.inner.find_commit(oid).ok()))
            .find(|walked_commit| {
                let commit_tree = match walked_commit.tree() {
                    Ok(tree) => tree,
                    Err(err) => {
                        tracing::error!(err=?err, "failed to get commit tree");
                        return false;
                    }
                };

                let mut files_changed = |child: &Tree<'_>, parent: Option<&Tree<'_>>| {
                    let diff = match self.inner.diff_tree_to_tree(
                        parent,
                        Some(child),
                        Some(&mut options),
                    ) {
                        Ok(diff) => diff,
                        Err(err) => {
                            tracing::error!(err=?err, "failed to diff trees");
                            return false;
                        }
                    };

                    let stats = match diff.stats() {
                        Ok(stats) => stats,
                        Err(err) => {
                            tracing::error!(err=?err, "failed to get diff stats");
                            return false;
                        }
                    };

                    stats.files_changed() > 0
                };

                if walked_commit.parent_count() == 0 {
                    files_changed(&commit_tree, None)
                } else {
                    // check that the given file was affected from any of the parents
                    walked_commit.parents().any(|parent| {
                        let parent_tree = match parent.tree() {
                            Ok(tree) => tree,
                            Err(err) => {
                                tracing::error!(err=?err, "failed to get parent commit tree, this should not happen");
                                return false;
                            }
                        };

                        files_changed(&commit_tree, Some(&parent_tree))
                    })
                }
            });

        Ok(Some(commit.context("file was not part of any commit")?))
    }
}
