//! User defined name attached to an entity for easy searching.
use specs::prelude::*;
use std::{fmt, string::ToString};

#[derive(Component, Debug, Clone)]
pub struct Tag(String);

impl Tag {
    pub fn new<S>(name: S) -> Self
    where
        S: ToString,
    {
        Tag(name.to_string())
    }
}

impl AsRef<str> for Tag {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}
