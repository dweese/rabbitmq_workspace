// egui-components/src/tree.rs

use eframe::egui::{self, Color32, Stroke, Ui};
use std::collections::HashMap;

// Make sure all these types are public
pub type ChildrenFn = Box<dyn Fn(TreeNodeId) -> Vec<TreeNodeId>>;
pub type RenderFn = Box<dyn Fn(&mut Ui, &TreeNode, bool, bool) -> bool>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TreeNodeId(pub u64);

impl TreeNodeId {
    pub fn root() -> Self {
        TreeNodeId(0)
    }
}

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub id: TreeNodeId,
    pub depth: u32,
}

#[derive(Debug, Clone)]
struct TreeNodeState {
    expanded: bool,
}

/// A tree component for displaying hierarchical data
pub struct Tree {
    id: String,
    root_id: TreeNodeId,
    children_fn: Option<ChildrenFn>,
    render_fn: Option<RenderFn>,
    state: HashMap<TreeNodeId, TreeNodeState>,
    indent_size: f32,
    node_spacing: f32,
    icon_size: f32,
    background_color: Option<Color32>,
    selection_color: Color32,
    hover_color: Color32,
    line_stroke: Stroke,
    selected: Option<TreeNodeId>,
}

impl Tree {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            root_id: TreeNodeId::root(),
            children_fn: None,
            render_fn: None,
            state: HashMap::new(),
            indent_size: 20.0,
            node_spacing: 4.0,
            icon_size: 16.0,
            background_color: None,
            selection_color: Color32::from_rgb(100, 150, 250),
            hover_color: Color32::from_rgb(180, 200, 250),
            line_stroke: Stroke::new(1.0, Color32::from_gray(180)),
            selected: None,
        }
    }

    pub fn root_id(mut self, root_id: TreeNodeId) -> Self {
        self.root_id = root_id;
        self
    }

    pub fn children_fn(mut self, children_fn: impl Fn(TreeNodeId) -> Vec<TreeNodeId> + 'static) -> Self {
        self.children_fn = Some(Box::new(children_fn));
        self
    }

    pub fn render_fn(mut self, render_fn: impl Fn(&mut Ui, &TreeNode, bool, bool) -> bool + 'static) -> Self {
        self.render_fn = Some(Box::new(render_fn));
        self
    }

    pub fn with_expanded(mut self, id: TreeNodeId, expanded: bool) -> Self {
        self.state.entry(id).or_insert(TreeNodeState { expanded }).expanded = expanded;
        self
    }

    pub fn selected(mut self, selected: Option<TreeNodeId>) -> Self {
        self.selected = selected;
        self
    }

    pub fn ui(mut self, ui: &mut Ui) -> Option<TreeNodeId> {
        let mut selected = self.selected;

        if let (Some(children_fn), Some(render_fn)) = (self.children_fn.as_ref(), self.render_fn.as_ref()) {
            let mut nodes = Vec::new();

            // Start with the root
            self.collect_visible_nodes(&mut nodes, self.root_id, 0, children_fn);

            // Render each visible node
            egui::Frame::none()
                .fill(self.background_color.unwrap_or_default())
                .show(ui, |ui| {
                    for node in nodes {
                        let is_selected = selected == Some(node.id);
                        let indent = node.depth as f32 * self.indent_size;

                        ui.horizontal(|ui| {
                            ui.add_space(indent);

                            // Check if this node has children
                            let children = children_fn(node.id);
                            let has_children = !children.is_empty();

                            // Get expansion state
                            let node_state = self.state.entry(node.id).or_insert(TreeNodeState { expanded: false });

                            if has_children {
                                // Toggle button for expansion
                                let toggle_text = if node_state.expanded { "▼" } else { "►" };
                                if ui.button(toggle_text).clicked() {
                                    node_state.expanded = !node_state.expanded;
                                }
                            } else {
                                // Spacer for leaf nodes
                                ui.add_space(self.icon_size);
                            }

                            // Draw the actual item with selection highlighting
                            let response = ui.scope(|ui| {
                                let is_hovered = ui.rect_contains_pointer(ui.max_rect());

                                // Background for selection/hover
                                if is_selected || is_hovered {
                                    let color = if is_selected { self.selection_color } else { self.hover_color };
                                    ui.painter().rect_filled(ui.max_rect(), 4.0, color);
                                }

                                // Render the node using the provided function
                                render_fn(ui, &node, is_selected, is_hovered)
                            });

                            // Handle selection on click
                            if response.inner {
                                selected = Some(node.id);
                            }
                        });

                        ui.add_space(self.node_spacing);
                    }
                });
        }

        selected
    }

    // Helper to build the list of visible nodes based on expansion state
    fn collect_visible_nodes(&self, nodes: &mut Vec<TreeNode>, id: TreeNodeId, depth: u32, children_fn: &ChildrenFn) {
        if depth > 0 {  // Skip the root node from display
            nodes.push(TreeNode { id, depth: depth - 1 });
        }

        // Check if this node is expanded
        let is_expanded = self.state.get(&id).map_or(false, |state| state.expanded);

        if is_expanded || depth == 0 {  // Always process children of root
            for child_id in children_fn(id) {
                self.collect_visible_nodes(nodes, child_id, depth + 1, children_fn);
            }
        }
    }
}