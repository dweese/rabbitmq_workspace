// In egui-components/src/tree.rs or similar

// egui-components/src/event_tree.rs
use crate::tree_node_id::TreeNodeId;

use eframe::egui::{self, Color32, Stroke, Ui};
use std::collections::HashMap;

use std::fmt::Debug;
use std::hash::Hash;

// Define tree-related events
#[derive(Debug, Clone)]
pub enum TreeEvent<ID: Clone + Eq + Hash + Debug> {
    NodeSelected(ID),
    NodeExpanded(ID),
    NodeCollapsed(ID),
    NodeDoubleClicked(ID),
    // Add other events as needed
}

// Create a trait for tree event handlers
pub trait TreeEventHandler<ID: Clone + Eq + Hash + Debug> {
    fn handle_event(&mut self, event: TreeEvent<ID>);
}

// Update the Tree struct to store static data rather than closures
pub struct Tree<ID: Clone + Eq + Hash + Debug = TreeNodeId> {
    id: String,
    root_id: ID,
    nodes: HashMap<ID, TreeNode<ID>>,
    state: HashMap<ID, TreeNodeState>,
    indent_size: f32,
    node_spacing: f32,
    icon_size: f32,
    background_color: Option<Color32>,
    selection_color: Color32,
    hover_color: Color32,
    line_stroke: Stroke,
    selected: Option<ID>,
}

// TreeNode holds data for each node
pub struct TreeNode<ID: Clone + Eq + Hash + Debug> {
    id: ID,
    label: String,
    children: Vec<ID>,
    // Other node-specific properties
}

impl<ID: Clone + Eq + Hash + Debug + Default> Tree<ID> {
    pub fn new(id: String) -> Self {
        Self {
            id,
            root_id: ID::default(),
            nodes: HashMap::new(),
            state: HashMap::new(),
            indent_size: 20.0,
            node_spacing: 4.0,
            icon_size: 16.0,
            background_color: None,
            selection_color: Color32::from_rgb(100, 150, 200),
            hover_color: Color32::from_rgb(150, 180, 200),
            line_stroke: Stroke::new(1.0, Color32::GRAY),
            selected: None,
        }
    }

    // Add a node to the tree
    pub fn add_node(&mut self, id: ID, parent: Option<ID>, label: String) -> &mut Self {
        let node = TreeNode {
            id: id.clone(),
            label,
            children: Vec::new(),
        };

        self.nodes.insert(id.clone(), node);

        // Add to parent's children if parent is specified
        if let Some(parent_id) = parent {
            if let Some(parent_node) = self.nodes.get_mut(&parent_id) {
                parent_node.children.push(id);
            }
        }

        self
    }

    // Set the root ID
    pub fn root_id(&mut self, id: ID) -> &mut Self {
        self.root_id = id;
        self
    }

    // Show the tree in the UI and return any events that occurred
    pub fn show(&mut self, ui: &mut egui::Ui) -> Option<TreeEvent<ID>> {
        let mut event = None;

        let root_id =self.root_id.clone();
                
        // Show tree nodes recursively starting from root
        self.show_node(ui, &root_id, 0, &mut event);

        event
    }

    // Show a single node and its children
    // Refactored show_node method to avoid borrow checker issues

    // Refactored show_node method to avoid borrow checker issues
    // Refactored show_node method to avoid borrow checker issues
    fn show_node(&mut self, ui: &mut egui::Ui, node_id: &ID, level: usize, event: &mut Option<TreeEvent<ID>>) {
        // First, let's clone the necessary data to avoid borrowing issues
        let node = if let Some(node) = self.nodes.get(node_id) {
            node.clone() // Clone the node to avoid immutable borrow
        } else {
            return; // Early return if node doesn't exist
        };

        let indent = level as f32 * self.indent_size;
        let has_children = !node.children.is_empty();

        // Get the current expansion and selection state before UI interaction
        let (is_expanded, is_selected) = if let Some(state) = self.state.get(node_id) {
            (state.expanded, state.selected)
        } else {
            (false, false)
        };

        // Clone the children IDs to avoid borrow during recursion
        let children = if has_children && is_expanded {
            node.children.clone()
        } else {
            vec![]
        };

        ui.horizontal(|ui| {
            ui.add_space(indent);

            // Toggle button for expanding/collapsing
            if has_children {
                if ui.button(if is_expanded { "▼" } else { "▶" }).clicked() {
                    // Update expansion state
                    let new_expanded = !is_expanded;
                    if let Some(state) = self.state.get_mut(node_id) {
                        state.expanded = new_expanded;
                    } else {
                        // Insert new state if it doesn't exist
                        self.state.insert(node_id.clone(), TreeNodeState {
                            expanded: new_expanded,
                            selected: false,
                        });
                    }

                    // Set the event
                    if new_expanded {
                        *event = Some(TreeEvent::NodeExpanded(node_id.clone()));
                    } else {
                        *event = Some(TreeEvent::NodeCollapsed(node_id.clone()));
                    }
                }
            } else {
                ui.add_space(20.0); // Space for alignment where toggle would be
            }

            // The node label with selection highlighting
            let label_response = ui.selectable_label(is_selected, &node.label);

            if label_response.clicked() {
                // Deselect previous selection
                if let Some(selected_id) = &self.selected {
                    if let Some(selected_state) = self.state.get_mut(selected_id) {
                        selected_state.selected = false;
                    }
                }

                // Update selection
                if let Some(state) = self.state.get_mut(node_id) {
                    state.selected = true;
                } else {
                    self.state.insert(node_id.clone(), TreeNodeState {
                        expanded: false,
                        selected: true,
                    });
                }

                self.selected = Some(node_id.clone());
                *event = Some(TreeEvent::NodeSelected(node_id.clone()));
            }

            if label_response.double_clicked() {
                *event = Some(TreeEvent::NodeDoubleClicked(node_id.clone()));
            }
        });

        // Show children if expanded (using the cloned children list)
        if is_expanded {
            for child_id in children {
                self.show_node(ui, &child_id, level + 1, event);
            }
        }
    }
}

// State for each tree node
pub struct TreeNodeState {
    expanded: bool,
    selected: bool,
}

// The Default implementation for TreeNodeId is already defined in tree_node_id.rs