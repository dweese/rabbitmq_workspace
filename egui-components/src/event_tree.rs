
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use eframe::egui;
use crate::tree_node_id::TreeNodeId;

#[derive(Debug, Clone)]
pub enum TreeEvent<ID: Clone + Eq + Hash + Debug> {
    NodeSelected(ID),
    NodeDoubleClicked(ID),
    NodeExpanded(ID),
    NodeCollapsed(ID),
    NodeDeleted(ID),
    NodeRenamed(ID, String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TreeNodeState {
    Expanded,
    Collapsed,
}

// TreeNode holds data for each node
#[derive(Debug, Clone)]
pub struct TreeNode<ID: Clone + Eq + Hash + Debug> {
    pub id: ID,
    pub parent: Option<ID>,
    pub children: Vec<ID>,
    pub label: String,
    pub state: TreeNodeState,
}

pub struct EventTree<ID: Clone + Eq + Hash + Debug = TreeNodeId> {
    pub name: String,
    pub nodes: HashMap<ID, TreeNode<ID>>,
    pub root_nodes: Vec<ID>,
    pub selected_node: Option<ID>,
}

// Helper struct to store node data needed for display
// Removed the unused id field
#[derive(Clone)]
struct NodeDisplayData<ID: Clone + Eq + Hash + Debug> {
    // Removed: id: ID,
    label: String,
    is_expanded: bool,
    is_leaf: bool,
    children: Vec<ID>,
}

impl<ID: Clone + Eq + Hash + Debug> EventTree<ID> {
    pub fn new(name: String) -> Self {
        Self {
            name,
            nodes: HashMap::new(),
            root_nodes: Vec::new(),
            selected_node: None,
        }
    }

    pub fn add_node(&mut self, id: ID, parent: Option<ID>, label: String) -> ID {
        let node = TreeNode {
            id: id.clone(),
            parent: parent.clone(),
            children: Vec::new(),
            label,
            state: TreeNodeState::Collapsed,
        };

        // If this node has a parent, add it to the parent's children
        if let Some(parent_id) = &parent {
            if let Some(parent_node) = self.nodes.get_mut(parent_id) {
                parent_node.children.push(id.clone());
            }
        } else {
            // If no parent, it's a root node
            self.root_nodes.push(id.clone());
        }

        // Add the node to our map
        self.nodes.insert(id.clone(), node);

        id
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Option<TreeEvent<ID>> {
        let mut event = None;

        // Clone the root nodes to avoid borrow issues
        let root_ids = self.root_nodes.clone();

        ui.vertical(|ui| {
            for root_id in root_ids {
                if let Some(new_event) = self.show_node(ui, &root_id, 0) {
                    event = Some(new_event);
                }
            }
        });

        event
    }

    fn show_node(&mut self, ui: &mut egui::Ui, id: &ID, depth: usize) -> Option<TreeEvent<ID>> {
        // First, collect all the data we need
        let node_data = self.nodes.get(id).map(|node| {
            NodeDisplayData {
                // Removed: id: id.clone(),
                label: node.label.clone(),
                is_expanded: node.state == TreeNodeState::Expanded,
                is_leaf: node.children.is_empty(),
                children: node.children.clone(),
            }
        });

        if node_data.is_none() {
            return None;
        }

        let node_data = node_data.unwrap();
        let is_selected = self.selected_node.as_ref().map_or(false, |sel_id| sel_id == id);

        // Handle UI and events
        let mut event = None;

        ui.horizontal(|ui| {
            ui.add_space((depth * 20) as f32); // Indent

            // Show expand/collapse indicator (if not a leaf)
            if !node_data.is_leaf {
                if ui.button(if node_data.is_expanded { "▼" } else { "►" }).clicked() {
                    // Toggle expansion state
                    let new_state = if node_data.is_expanded {
                        TreeNodeState::Collapsed
                    } else {
                        TreeNodeState::Expanded
                    };

                    // Update the node's state outside the closure
                    if let Some(node) = self.nodes.get_mut(id) {
                        node.state = new_state.clone();
                    }

                    // Create appropriate event
                    event = Some(if new_state == TreeNodeState::Expanded {
                        TreeEvent::NodeExpanded(id.clone())
                    } else {
                        TreeEvent::NodeCollapsed(id.clone())
                    });
                }
            } else {
                // Just add some space for leaf nodes to align them
                ui.add_space(20.0);
            }

            // Use selectable for node text
            let resp = ui.selectable_label(is_selected, &node_data.label);

            if resp.clicked() {
                self.selected_node = Some(id.clone());
                event = Some(TreeEvent::NodeSelected(id.clone()));
            }

            if resp.double_clicked() {
                event = Some(TreeEvent::NodeDoubleClicked(id.clone()));
            }
        });

        // If expanded, show children
        if node_data.is_expanded {
            // Clone the children to avoid borrow issues
            let children = node_data.children;
            for child_id in children {
                if let Some(child_event) = self.show_node(ui, &child_id, depth + 1) {
                    // Only set the event if we haven't already set one
                    if event.is_none() {
                        event = Some(child_event);
                    }
                }
            }
        }

        event
    }

    #[allow(dead_code)]
    pub fn get_node(&self, id: &ID) -> Option<&TreeNode<ID>> {
        self.nodes.get(id)
    }

    #[allow(dead_code)]
    pub fn get_selected_node(&self) -> Option<&TreeNode<ID>> {
        self.selected_node.as_ref().and_then(|id| self.nodes.get(id))
    }
}