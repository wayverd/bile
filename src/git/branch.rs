use git2::{Branch, BranchType, Reference};

use crate::{error::Result, git::Repository};

impl Repository {
    #[tracing::instrument(skip_all)]
    pub(crate) fn branches(&self) -> Result<Vec<Reference<'_>>> {
        let references = self.inner.references()?;

        let branches = references
            .filter_map(Result::ok)
            .filter(Reference::is_branch)
            .collect();

        Ok(branches)
    }

    #[tracing::instrument(skip_all)]
    pub(crate) fn branches_of_type(&self, typ: BranchType) -> Result<Vec<Branch<'_>>> {
        let branches = self
            .inner
            .branches(Some(typ))?
            .filter_map(|x| if let Ok(x) = x { Some(x.0) } else { None })
            .collect();

        Ok(branches)
    }
}
