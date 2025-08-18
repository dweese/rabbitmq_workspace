// egui-components/src/tree_vis/topology.rs
use crate::event_tree::{EventTree, TreeEvent};
use crate::tree_node_id::TreeNodeId;

use std::collections::HashMap;
use std::fmt::Debug;
// Removed unused import: std::hash::Hash
use eframe::egui;

// Define a trait for topology data sources
pub trait TopologyDataSource {
    fn get_exchanges(&self) -> Vec<String>;
    fn get_queues(&self) -> Vec<String>;
    // Add other methods as needed
}

#[derive(Debug, Clone, PartialEq)]
pub enum TopologyNodeType {
    VHost,
    Exchange(String), // exchange type
    Queue,
    Binding(String), // routing key
}

#[derive(Debug, Clone)]
pub enum TopologyAction {
    ShowExchangeDetails(TreeNodeId),
    ShowQueueDetails(TreeNodeId),
    EditExchange(TreeNodeId),
    EditQueue(TreeNodeId),
}

// Adding allow(dead_code) to suppress warnings about unused fields
#[allow(dead_code)]
pub struct TopologyNode {
    id: TreeNodeId,
    name: String,
    node_type: TopologyNodeType,
    metadata: HashMap<String, String>, // Store additional properties
}

pub struct TopologyVisualizer {
    tree: EventTree<TreeNodeId>,
    nodes: HashMap<TreeNodeId, TopologyNode>,
    selected_node: Option<TreeNodeId>,
}

impl Default for TopologyVisualizer {
    fn default() -> Self {
        Self::new()
    }
}

impl TopologyVisualizer {
    pub fn new() -> Self {
        Self {
            tree: EventTree::new("topology_tree".to_string()),
            nodes: HashMap::new(),
            selected_node: None,
        }
    }

    // Change the parameter type to use our trait instead of concrete RabbitMQClient
    pub fn update_from_client<T: TopologyDataSource>(&mut self, client: &T) {
        // Example usage of the trait methods:
        // Prefix unused variables with underscore
        let _exchanges = client.get_exchanges();
        let _queues = client.get_queues();

        // Add implementation here using these lists
        // To be implemented in the future
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) -> Option<TreeEvent<TreeNodeId>> {
        self.tree.show(ui)
    }

    // Add allow(dead_code) to methods that are defined but not yet used
    #[allow(dead_code)]
    fn add_exchange(&mut self, vhost_id: TreeNodeId, name: &str, ex_type: &str) -> TreeNodeId {
        let exchange_id = TreeNodeId {
            id: format!("exchange:{name}"),
        };

        // Create a topology node
        let node = TopologyNode {
            id: exchange_id.clone(),
            name: name.to_string(),
            node_type: TopologyNodeType::Exchange(ex_type.to_string()),
            metadata: HashMap::new(),
        };

        // Add to the topology nodes
        self.nodes.insert(exchange_id.clone(), node);

        // Add to the tree
        self.tree
            .add_node(exchange_id.clone(), Some(vhost_id), name.to_string());

        exchange_id
    }

    #[allow(dead_code)]
    fn add_queue(&mut self, vhost_id: TreeNodeId, name: &str) -> TreeNodeId {
        let queue_id = TreeNodeId {
            id: format!("queue:{name}"),
        };

        // Create a topology node
        let node = TopologyNode {
            id: queue_id.clone(),
            name: name.to_string(),
            node_type: TopologyNodeType::Queue,
            metadata: HashMap::new(),
        };

        // Add to the topology nodes
        self.nodes.insert(queue_id.clone(), node);

        // Add to the tree
        self.tree
            .add_node(queue_id.clone(), Some(vhost_id), name.to_string());

        queue_id
    }

    #[allow(dead_code)]
    fn add_binding(&mut self, exchange_id: TreeNodeId, queue_id: TreeNodeId, routing_key: &str) {
        // Option 1: Add a binding node
        let binding_id = TreeNodeId {
            id: format!(
                "binding:{}->{}:{}",
                exchange_id.id, queue_id.id, routing_key
            ),
        };

        let node = TopologyNode {
            id: binding_id.clone(),
            name: routing_key.to_string(),
            node_type: TopologyNodeType::Binding(routing_key.to_string()),
            metadata: HashMap::new(),
        };

        self.nodes.insert(binding_id.clone(), node);

        // Add to the tree as a child of the exchange
        self.tree.add_node(
            binding_id,
            Some(exchange_id),
            format!(
                "→ {} ({})",
                self.nodes.get(&queue_id).map_or("Unknown", |n| &n.name),
                routing_key
            ),
        );
    }

    pub fn handle_event(&mut self, event: TreeEvent<TreeNodeId>) -> Option<TopologyAction> {
        match event {
            TreeEvent::NodeSelected(id) => {
                self.selected_node = Some(id.clone());

                // Return an action based on node type
                if let Some(node) = self.nodes.get(&id) {
                    match node.node_type {
                        TopologyNodeType::Exchange(_) => {
                            Some(TopologyAction::ShowExchangeDetails(id))
                        }
                        TopologyNodeType::Queue => Some(TopologyAction::ShowQueueDetails(id)),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            TreeEvent::NodeDoubleClicked(id) => {
                // Handle double-click - maybe edit the item
                if let Some(node) = self.nodes.get(&id) {
                    match node.node_type {
                        TopologyNodeType::Exchange(_) => Some(TopologyAction::EditExchange(id)),
                        TopologyNodeType::Queue => Some(TopologyAction::EditQueue(id)),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            // Handle other events
            _ => None,
        }
    }
}
