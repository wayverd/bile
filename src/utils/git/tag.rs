use git2::{ObjectType, Tag};

use crate::utils::{
    error::Result,
    git::{Repository, TagEntry},
};

impl Repository {
    #[tracing::instrument(skip_all)]
    pub fn tag_entries(&self) -> Result<Vec<TagEntry>> {
        let mut tags = Vec::new();

        self.inner.tag_foreach(|oid, name_bytes| {
            // remove prefix "ref/tags/"
            let Ok(name) = String::from_utf8(name_bytes[10..].to_vec()) else {
                return true;
            };

            let Ok(obj) = self.inner.find_object(oid, None) else {
                return true;
            };

            let tag = match obj.kind() {
                Some(ObjectType::Tag) => TagEntry::try_from_tag(name, &obj),
                // lightweight tag
                Some(ObjectType::Commit) => TagEntry::try_from_commit(name, &obj),
                _ => unreachable!("a tag was not a tag or lightweight tag"),
            };

            let Ok(tag) = tag else {
                return true;
            };

            tags.push(tag);

            true
        })?;

        Ok(tags)
    }

    #[tracing::instrument(skip_all)]
    pub fn tag(&self, spec: &str) -> Result<Tag<'_>> {
        let tag = self.inner.revparse_single(spec)?.peel_to_tag()?;

        Ok(tag)
    }
}
