// egui-components/src/tree_node_id.rs

use std::fmt::{self, Debug, Display};
use std::hash::{Hash, Hasher};

/// Unique identifier for tree nodes
#[derive(Clone)]
pub struct TreeNodeId {
    /// Unique ID string for the node
    pub id: String,
}

impl TreeNodeId {
    /// Create a new TreeNodeId with the given string ID
    pub fn new<S: Into<String>>(id: S) -> Self {
        Self { id: id.into() }
    }

    /// Create a root node ID
    pub fn root() -> Self {
        Self { id: "root".to_string() }
    }

    /// Get the string representation of this ID
    pub fn as_string(&self) -> &str {
        &self.id
    }

    /// Check if this is the root node
    pub fn is_root(&self) -> bool {
        self.id == "root"
    }

    /// Create a child ID from this node with the given suffix
    pub fn child<S: Display>(&self, suffix: S) -> Self {
        Self { id: format!("{}_{}", self.id, suffix) }
    }
}

// Implement required traits
impl PartialEq for TreeNodeId {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for TreeNodeId {}

impl Hash for TreeNodeId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Debug for TreeNodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TreeNodeId({})", self.id)
    }
}

impl Display for TreeNodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

// Implement Default trait so we can use it with generics that require Default
impl Default for TreeNodeId {
    fn default() -> Self {
        Self::root()
    }
}