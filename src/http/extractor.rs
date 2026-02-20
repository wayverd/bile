use std::fmt;

use serde::de::Error as _;
use trim_in_place::TrimInPlace as _;

pub(crate) struct Commit(pub String);

impl fmt::Display for Commit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<'de> serde::Deserialize<'de> for Commit {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut value = String::deserialize(deserializer)?;

        value.trim_in_place();

        if value.is_empty() || value.starts_with('.') {
            return Err(D::Error::custom("invalid commit ref"));
        }

        for c in value.bytes() {
            if !c.is_ascii_hexdigit() {
                return Err(D::Error::custom("invalid commit ref"));
            }
        }

        Ok(Self(value))
    }
}

pub(crate) struct Obj(pub String);

impl fmt::Display for Obj {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<'de> serde::Deserialize<'de> for Obj {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut value = String::deserialize(deserializer)?;

        value.trim_in_place();

        if value.is_empty() {
            return Err(D::Error::custom("invalid object ref"));
        }

        Ok(Self(value))
    }
}

pub(crate) struct ObjectName(pub String);

impl fmt::Display for ObjectName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<'de> serde::Deserialize<'de> for ObjectName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut value = String::deserialize(deserializer)?;

        value.trim_in_place();

        if value.is_empty() {
            return Err(D::Error::custom("invalid object name"));
        }

        Ok(Self(value))
    }
}

pub(crate) struct Ref(pub String);

impl fmt::Display for Ref {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<'de> serde::Deserialize<'de> for Ref {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut value = String::deserialize(deserializer)?;

        value.trim_in_place();

        if value.is_empty() || value.starts_with('.') {
            return Err(D::Error::custom("invalid ref"));
        }

        Ok(Self(value))
    }
}

pub(crate) struct RepoName(pub String);

impl fmt::Display for RepoName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<'de> serde::Deserialize<'de> for RepoName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut value = String::deserialize(deserializer)?;

        value.trim_in_place();

        if value.is_empty() || value.starts_with('.') {
            return Err(D::Error::custom("invalid repo name"));
        }

        Ok(Self(value))
    }
}

pub(crate) struct Tag(pub String);

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<'de> serde::Deserialize<'de> for Tag {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut value = String::deserialize(deserializer)?;

        value.trim_in_place();

        if value.is_empty() {
            return Err(D::Error::custom("invalid tag"));
        }

        Ok(Self(value))
    }
}
