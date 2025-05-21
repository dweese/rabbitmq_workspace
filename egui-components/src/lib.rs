// egui-components/src/lib.rs

// First declare all the modules
mod tree_node_id;
mod tree;
mod border_layout;
mod event_tree;

// Then export the types, but only export TreeNodeId from tree_node_id.rs
pub use tree_node_id::TreeNodeId;
pub use tree::{Tree, TreeNode}; // Remove TreeNodeId from here
pub use border_layout::BorderLayout;
pub use event_tree::Tree as EventTree; // Use an alias to avoid name conflicts