use std::fmt::Debug;
use std::hash::Hash;
use std::collections::HashMap;
use eframe::egui;

// Internal state tracking
#[derive(Debug, Clone)]
struct TreeNodeState {
    expanded: bool,
}

// TreeNode holds data for each node
pub struct TreeNode<ID: Clone + Eq + Hash + Debug> {
    
    #[allow(dead_code)]
    pub id: ID,
    #[allow(dead_code)]
    pub parent: Option<ID>,
    pub children: Vec<ID>,
    pub label: String,
    // Other fields...
}

pub struct Tree<ID: Clone + Eq + Hash + Debug> {
    nodes: HashMap<ID, TreeNode<ID>>,
    states: HashMap<ID, TreeNodeState>,
    root_nodes: Vec<ID>,
    selected_node: Option<ID>,
}

// Helper struct to store node data needed for display
struct NodeDisplayData<ID: Clone + Eq + Hash + Debug> {
    #[allow(dead_code)]
    id: ID,
    children: Vec<ID>,
    label: String,
}

impl<ID: Clone + Eq + Hash + Debug> Tree<ID> {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            states: HashMap::new(),
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
        };

        // Initialize state
        self.states.insert(id.clone(), TreeNodeState { expanded: false });

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

    pub fn show(&mut self, ui: &mut egui::Ui) -> Option<ID> {
        let mut clicked_node = None;

        // Clone root nodes to avoid borrow issues
        let root_ids = self.root_nodes.clone();

        ui.vertical(|ui| {
            for root_id in root_ids {
                if let Some(clicked) = self.show_node(ui, &root_id, 0) {
                    clicked_node = Some(clicked);
                }
            }
        });

        clicked_node
    }

    fn show_node(&mut self, ui: &mut egui::Ui, id: &ID, depth: usize) -> Option<ID> {
        let mut clicked_node = None;

        // First, collect all the data we need
        let (node_data, state_expanded, is_selected) = match (self.nodes.get(id), self.states.get(id)) {
            (Some(node), Some(state)) => {
                let node_data = NodeDisplayData {
                    id: id.clone(),
                    label: node.label.clone(),
                    children: node.children.clone(),
                };
                let state_expanded = state.expanded;
                let is_selected = self.selected_node.as_ref().map_or(false, |sel_id| sel_id == id);
                (Some(node_data), state_expanded, is_selected)
            },
            _ => (None, false, false),
        };

        if let Some(node_data) = node_data {
            let is_leaf = node_data.children.is_empty();

            // Render the node UI
            ui.horizontal(|ui| {
                ui.add_space((depth * 20) as f32); // Indent

                // Show expand/collapse indicator (if not a leaf)
                if !is_leaf {
                    if ui.button(if state_expanded { "▼" } else { "►" }).clicked() {
                        // Toggle expansion state
                        if let Some(state) = self.states.get_mut(id) {
                            state.expanded = !state.expanded;
                        }
                    }
                } else {
                    // Just add some space for leaf nodes to align them
                    ui.add_space(20.0);
                }

                // Use selectable for node text
                if ui.selectable_label(is_selected, &node_data.label).clicked() {
                    self.selected_node = Some(id.clone());
                    clicked_node = Some(id.clone());
                }
            });

            // If expanded, show children
            if state_expanded {
                // Clone children list to avoid borrow issues
                let children = node_data.children;
                for child_id in children {
                    if let Some(clicked) = self.show_node(ui, &child_id, depth + 1) {
                        // Only set the clicked node if we haven't already set one
                        if clicked_node.is_none() {
                            clicked_node = Some(clicked);
                        }
                    }
                }
            }
        }

        clicked_node
    }
}