use std::{fmt::{self, Display}, ops::{Deref, DerefMut}};

pub struct WebPath<'a> {
    parts: Vec<&'a str>,
}

impl<'a> From<&'a str> for WebPath<'a> {
    fn from(string: &'a str) -> Self {
        let parts = string
            .split('/')
            .filter(|a| !a.is_empty())
            .collect();
        WebPath { parts }
    }
}

impl<'a> Display for WebPath<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.parts.join("/"))
    }
}

impl<'a> Clone for WebPath<'a> {
    fn clone(&self) -> Self {
        WebPath { parts: self.parts.clone() }
    }
}

impl<'a> Deref for WebPath<'a> {
    type Target = Vec<&'a str>;

    fn deref(&self) -> &Self::Target {
        &self.parts
    }
}

impl<'a> DerefMut for WebPath<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.parts
    }
}
