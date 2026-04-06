use crate::diagnostic::SourceLocation;
use std::fmt;

/// A wrapper type that attaches source location information to any AST or IR node.
/// This allows error reporting to point back to the original source location
/// even after multiple transformation passes.
#[derive(Debug, Clone)]
pub struct Spanned<T> {
    pub node: T,
    pub location: SourceLocation,
}

impl<T> Spanned<T> {
    /// Create a new spanned node with the given location
    pub fn new(node: T, location: SourceLocation) -> Self {
        Self { node, location }
    }

    /// Transform the inner node while preserving the location
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Spanned<U> {
        Spanned {
            node: f(self.node),
            location: self.location,
        }
    }

    /// Get a reference to the inner node
    pub fn inner(&self) -> &T {
        &self.node
    }

    /// Get a mutable reference to the inner node
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.node
    }

    /// Unwrap to get the inner node, discarding the location
    pub fn into_inner(self) -> T {
        self.node
    }
}

impl<T: fmt::Display> fmt::Display for Spanned<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.node.fmt(f)
    }
}

impl<T: PartialEq> PartialEq for Spanned<T> {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node && self.location == other.location
    }
}

impl<T: Eq> Eq for Spanned<T> {}
