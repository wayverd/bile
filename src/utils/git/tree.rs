use std::path::Path;

use git2::{Blob, Object, Tree};

use crate::utils::{
    error::{Context as _, Result},
    git::Repository,
};

impl Repository {
    #[tracing::instrument(skip_all)]
    pub fn tree_blob(&self, tree: &Tree<'_>, path: &Path) -> Result<Option<Blob<'_>>> {
        let Some(obj) = self.tree_object(tree, path)? else {
            return Ok(None);
        };

        let blob = obj.peel_to_blob()?;

        Ok(Some(blob))
    }

    #[tracing::instrument(skip_all)]
    pub fn tree_object(&self, tree: &Tree<'_>, path: &Path) -> Result<Option<Object<'_>>> {
        let entry = match tree
            .get_path(path)
            .context("failed to get object from tree")
        {
            Ok(entry) => entry,
            Err(err) => {
                tracing::error!(err=?err, "failed to get object from tree");
                return Ok(None);
            }
        };

        let obj = entry.to_object(&self.inner)?;

        Ok(Some(obj))
    }
}
