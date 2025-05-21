// rabbitmq_ui/src/main.rs
// rabbitmq_ui/src/main.rs
mod app;

use eframe::NativeOptions;
use eframe::egui::{self, Color32, RichText, ScrollArea, Stroke};
use std::collections::HashMap;
use egui_components::{BorderLayout, Tree, TreeNodeId};

use rabbitmq_config::{
    RabbitMQClient, RabbitMQConfig, RabbitMQMessage,
    MessageProperties, QueueInfo, ExchangeInfo,
};
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;

// Create a simple enum for UI actions
pub enum UiAction {
    Connect,
    Disconnect,
    Refresh,
    PublishMessage,
    DeclareQueue,
    DeclareExchange,
}

// Application state that contains all data needed by the UI
pub struct TreeState {
    queue_tree_nodes: HashMap<TreeNodeId, String>,
    queue_children: HashMap<TreeNodeId, Vec<TreeNodeId>>,
    selected_queue: Option<TreeNodeId>,
}

impl Default for TreeState {
    fn default() -> Self {
        Self {
            queue_tree_nodes: HashMap::new(),
            queue_children: HashMap::new(),
            selected_queue: None,
        }
    }
}

pub struct AppState {
    runtime: Arc<Runtime>,
    client: Option<Arc<Mutex<RabbitMQClient>>>,
    connection_status: bool,
    config: RabbitMQConfig,
    message: RabbitMQMessage,
    status_message: String,
    available_queues: Vec<String>,
    available_exchanges: Vec<String>,

    // Queue and exchange creation form data
    new_queue: QueueInfo,
    new_exchange: ExchangeInfo,

    // UI state flags
    show_queue_dialog: bool,
    show_exchange_dialog: bool,

    // Tree state in a separate struct
    tree_state: TreeState,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            runtime: Arc::new(Runtime::new().expect("Failed to create Tokio runtime")),
            client: None,
            connection_status: false,
            config: RabbitMQConfig::default(),
            message: RabbitMQMessage {
                exchange: "".to_string(),
                routing_key: "".to_string(),
                payload: "".to_string(),
                properties: Some(MessageProperties::default()),
            },
            status_message: "Welcome to RabbitMQ UI".to_string(),
            available_queues: Vec::new(),
            available_exchanges: Vec::new(),
            new_queue: QueueInfo {
                name: "".to_string(),
                durable: true,
                auto_delete: false,
                exclusive: false,
            },
            new_exchange: ExchangeInfo {
                name: "".to_string(),
                kind: "direct".to_string(),
                durable: true,
                auto_delete: false,
            },
            show_queue_dialog: false,
            show_exchange_dialog: false,
            tree_state: TreeState::default(),
        }
    }
}

impl AppState {
    // Your existing methods remain unchanged
    pub fn connect_to_rabbitmq(&mut self) {
        let config = self.config.clone();
        let runtime = self.runtime.clone();

        let client_future = async move {
            RabbitMQClient::new(config).await
        };

        match runtime.block_on(client_future) {
            Ok(client) => {
                self.client = Some(Arc::new(Mutex::new(client)));
                self.connection_status = true;
                self.status_message = format!("Connected to RabbitMQ at {}", self.config.host);
                self.refresh_queues_and_exchanges();
            },
            Err(err) => {
                self.status_message = format!("Connection failed: {:?}", err);
            }
        }
    }

    pub fn disconnect_from_rabbitmq(&mut self) {
        if let Some(client) = &self.client {
            let client = client.clone();
            let runtime = self.runtime.clone();

            let close_future = async move {
                let client = client.lock().await;
                client.close().await
            };

            match runtime.block_on(close_future) {
                Ok(_) => {
                    self.status_message = "Disconnected from RabbitMQ".to_string();
                },
                Err(err) => {
                    self.status_message = format!("Error during disconnect: {:?}", err);
                }
            }
        }

        self.client = None;
        self.connection_status = false;
        self.available_queues.clear();
        self.available_exchanges.clear();
    }

    pub fn refresh_queues_and_exchanges(&mut self) {
        // Existing refresh code...
        if let Some(client) = &self.client {
            let client = client.clone();
            let runtime = self.runtime.clone();

            // Fetch queues
            let queues_future = async move {
                let client = client.lock().await;
                client.list_queues().await
            };

            match runtime.block_on(queues_future) {
                Ok(queues) => {
                    self.available_queues = queues;
                },
                Err(err) => {
                    self.status_message = format!("Failed to fetch queues: {:?}", err);
                }
            }

            // Fetch exchanges
            if let Some(client) = &self.client {
                let client = client.clone();
                let runtime = self.runtime.clone();

                let exchanges_future = async move {
                    let client = client.lock().await;
                    client.list_exchanges().await
                };

                match runtime.block_on(exchanges_future) {
                    Ok(exchanges) => {
                        self.available_exchanges = exchanges;
                    },
                    Err(err) => {
                        self.status_message = format!("Failed to fetch exchanges: {:?}", err);
                    }
                }
            }
        }
    }

    pub fn publish_message(&mut self) {
        // Existing publish code...
        if let Some(client) = &self.client {
            let client = client.clone();
            let runtime = self.runtime.clone();
            let message = self.message.clone();

            let publish_future = async move {
                let client = client.lock().await;
                client.publish_message(&message).await
            };

            match runtime.block_on(publish_future) {
                Ok(_) => {
                    self.status_message = "Message published successfully".to_string();
                },
                Err(err) => {
                    self.status_message = format!("Failed to publish message: {:?}", err);
                }
            }
        } else {
            self.status_message = "Not connected to RabbitMQ".to_string();
        }
    }

    pub fn declare_queue(&mut self) {
        // Existing queue creation code...
        if let Some(client) = &self.client {
            let client = client.clone();
            let runtime = self.runtime.clone();
            let queue_info = self.new_queue.clone();
            let queue_name = queue_info.name.clone();

            let queue_future = async move {
                let client = client.lock().await;
                client.declare_queue(&queue_info).await
            };

            match runtime.block_on(queue_future) {
                Ok(_) => {
                    self.status_message = format!("Queue '{}' created successfully", queue_name);
                    self.new_queue.name = "".to_string();
                    self.show_queue_dialog = false;
                    self.refresh_queues_and_exchanges();
                },
                Err(err) => {
                    self.status_message = format!("Failed to create queue: {:?}", err);
                }
            }
        } else {
            self.status_message = "Not connected to RabbitMQ".to_string();
        }
    }

    pub fn declare_exchange(&mut self) {
        // Existing exchange creation code...
        if let Some(client) = &self.client {
            let client = client.clone();
            let runtime = self.runtime.clone();
            let exchange_info = self.new_exchange.clone();
            let exchange_name = exchange_info.name.clone();

            let exchange_future = async move {
                let client = client.lock().await;
                client.declare_exchange(&exchange_info).await
            };

            match runtime.block_on(exchange_future) {
                Ok(_) => {
                    self.status_message = format!("Exchange '{}' created successfully", exchange_name);
                    self.new_exchange.name = "".to_string();
                    self.show_exchange_dialog = false;
                    self.refresh_queues_and_exchanges();
                },
                Err(err) => {
                    self.status_message = format!("Failed to create exchange: {:?}", err);
                }
            }
        } else {
            self.status_message = "Not connected to RabbitMQ".to_string();
        }
    }
}

// The main App struct
pub struct App {
    state: AppState,
}

impl Default for App {
    fn default() -> Self {
        Self {
            state: AppState::default(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Track UI actions with a Vec
        let mut actions = Vec::new();

        // ======= TOP PANEL (Connection controls) =======
        egui::TopBottomPanel::top("north_panel")
            .frame(egui::Frame::none().fill(Color32::from_rgb(173, 216, 230))) // Light blue
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Connection status indicator
                    let status_text = if self.state.connection_status {
                        RichText::new("Connected").color(Color32::GREEN)
                    } else {
                        RichText::new("Disconnected").color(Color32::RED)
                    };
                    ui.label(status_text);

                    // Connection controls
                    if !self.state.connection_status {
                        if ui.button("Connect").clicked() {
                            actions.push(UiAction::Connect);
                        }

                        // Basic config fields for quick editing
                        ui.label("Host:");
                        ui.text_edit_singleline(&mut self.state.config.host);

                        ui.label("Port:");
                        let mut port_str = self.state.config.port.to_string();
                        if ui.text_edit_singleline(&mut port_str).changed() {
                            if let Ok(port) = port_str.parse::<u16>() {
                                self.state.config.port = port;
                            }
                        }
                    } else {
                        if ui.button("Disconnect").clicked() {
                            actions.push(UiAction::Disconnect);
                        }

                        if ui.button("Refresh Lists").clicked() {
                            actions.push(UiAction::Refresh);
                        }
                    }
                });
            });

        // ======= LEFT PANEL (Resources tree) =======
        egui::SidePanel::left("west_panel")
            .default_width(200.0)
            .frame(egui::Frame::none().fill(Color32::from_rgb(255, 218, 185))) // Peach
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    // Try using a String instead of &str to ensure ownership:
                    let tree = Tree::new(String::from("resource_tree"))
                        .root_id(TreeNodeId::root())
                        .children_fn(|node_id| {
                            if node_id == TreeNodeId::root() {
                                // Root level has two nodes: Queues and Exchanges
                                vec![TreeNodeId(1), TreeNodeId(2)]
                            } else if node_id.0 == 1 {
                                // Children of "Queues"
                                self.state.available_queues.iter().enumerate()
                                    .map(|(i, _)| TreeNodeId(100 + i as u64))
                                    .collect()
                            } else if node_id.0 == 2 {
                                // Children of "Exchanges"
                                self.state.available_exchanges.iter().enumerate()
                                    .map(|(i, _)| TreeNodeId(200 + i as u64))
                                    .collect()
                            } else {
                                vec![]
                            }
                        })
                        .render_fn(|ui, node, is_selected, is_hovered| {
                            let text = match node.id.0 {
                                0 => "Resources".to_string(),
                                1 => "Queues".to_string(),
                                2 => "Exchanges".to_string(),
                                id if id >= 100 && id < 200 => {
                                    let idx = (id - 100) as usize;
                                    if idx < self.state.available_queues.len() {
                                        self.state.available_queues[idx].clone()
                                    } else {
                                        "Unknown Queue".to_string()
                                    }
                                },
                                id if id >= 200 && id < 300 => {
                                    let idx = (id - 200) as usize;
                                    if idx < self.state.available_exchanges.len() {
                                        self.state.available_exchanges[idx].clone()
                                    } else {
                                        "Unknown Exchange".to_string()
                                    }
                                },
                                _ => "Unknown Node".to_string(),
                            };

                            let response = ui.selectable_label(is_selected, text);
                            response.clicked()
                        })
                        .with_expanded(TreeNodeId(1), true)
                        .with_expanded(TreeNodeId(2), true)
                        .selected(self.state.tree_state.selected_queue);

                    self.state.tree_state.selected_queue = tree.ui(ui);

                    // Don't need to track the selected node in a separate struct field since we have it in tree_state
                    if let Some(selected) = self.state.tree_state.selected_queue {
                        match selected.0 {
                            id if id >= 100 && id < 200 => {
                                let idx = (id - 100) as usize;
                                if idx < self.state.available_queues.len() {
                                    let queue = &self.state.available_queues[idx];
                                    self.state.message.routing_key = queue.clone();
                                    self.state.status_message = format!("Selected queue: {}", queue);
                                }
                            },
                            id if id >= 200 && id < 300 => {
                                let idx = (id - 200) as usize;
                                if idx < self.state.available_exchanges.len() {
                                    let exchange = &self.state.available_exchanges[idx];
                                    self.state.message.exchange = exchange.clone();
                                    self.state.status_message = format!("Selected exchange: {}", exchange);
                                }
                            },
                            _ => {}
                        }
                    }

                    ui.separator();

                    if ui.button("Declare Queue").clicked() {
                        self.state.show_queue_dialog = true;
                    }

                    if ui.button("Declare Exchange").clicked() {
                        self.state.show_exchange_dialog = true;
                    }
                });
            });

        // ======= BOTTOM PANEL (Status bar) =======
        egui::TopBottomPanel::bottom("south_panel")
            .frame(egui::Frame::none().fill(Color32::from_rgb(144, 238, 144))) // Light green
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.label(&self.state.status_message);
                });
            });

        // ======= CENTRAL PANEL (Message publishing) =======
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(Color32::from_rgb(230, 230, 250))) // Lavender
            .show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.heading("Publish Message");
                        ui.separator();

                        ui.horizontal(|ui| {
                            ui.label("Exchange:");
                            ui.text_edit_singleline(&mut self.state.message.exchange);
                        });

                        ui.horizontal(|ui| {
                            ui.label("Routing Key:");
                            ui.text_edit_singleline(&mut self.state.message.routing_key);
                        });

                        ui.label("Payload:");
                        ui.text_edit_multiline(&mut self.state.message.payload)
                            .on_hover_text("Enter message content here");

                        // Message properties collapsing section
                        ui.collapsing("Message Properties", |ui| {
                            if let Some(props) = &mut self.state.message.properties {
                                // Content Type
                                {
                                    let mut content_type = props.content_type.clone().unwrap_or_default();
                                    ui.horizontal(|ui| {
                                        ui.label("Content Type:");
                                        if ui.text_edit_singleline(&mut content_type).changed() {
                                            props.content_type = if content_type.is_empty() { None } else { Some(content_type) };
                                        }
                                    });
                                }

                                // Content Encoding
                                {
                                    let mut content_encoding = props.content_encoding.clone().unwrap_or_default();
                                    ui.horizontal(|ui| {
                                        ui.label("Content Encoding:");
                                        if ui.text_edit_singleline(&mut content_encoding).changed() {
                                            props.content_encoding = if content_encoding.is_empty() { None } else { Some(content_encoding) };
                                        }
                                    });
                                }

                                // Delivery Mode
                                {
                                    let mut persistent = props.delivery_mode.unwrap_or(1) == 2;
                                    if ui.checkbox(&mut persistent, "Persistent").changed() {
                                        props.delivery_mode = Some(if persistent { 2 } else { 1 });
                                    }
                                }

                                // Add more properties as needed
                            }

                        });

                        if ui.add_enabled(self.state.connection_status, egui::Button::new("Publish")).clicked() {
                            actions.push(UiAction::PublishMessage);
                        }

                        ui.separator();
                        ui.heading("Message Browser");
                        ui.label("Select a queue to browse messages");
                        // Message browsing implementation would go here
                    });
                });
            });

        // Queue declaration dialog
        if self.state.show_queue_dialog {
            egui::Window::new("Declare Queue")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Queue Name:");
                        ui.text_edit_singleline(&mut self.state.new_queue.name);
                    });

                    ui.checkbox(&mut self.state.new_queue.durable, "Durable");
                    ui.checkbox(&mut self.state.new_queue.auto_delete, "Auto Delete");
                    ui.checkbox(&mut self.state.new_queue.exclusive, "Exclusive");

                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            self.state.show_queue_dialog = false;
                        }

                        if ui.button("Create Queue").clicked() {
                            if self.state.new_queue.name.is_empty() {
                                self.state.status_message = "Queue name cannot be empty".to_string();
                            } else {
                                actions.push(UiAction::DeclareQueue);
                            }
                        }
                    });
                });
        }

        // Exchange declaration dialog
        if self.state.show_exchange_dialog {
            egui::Window::new("Declare Exchange")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Exchange Name:");
                        ui.text_edit_singleline(&mut self.state.new_exchange.name);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Type:");
                        egui::ComboBox::from_id_source("exchange_type")
                            .selected_text(&self.state.new_exchange.kind)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.state.new_exchange.kind, "direct".to_string(), "Direct");
                                ui.selectable_value(&mut self.state.new_exchange.kind, "fanout".to_string(), "Fanout");
                                ui.selectable_value(&mut self.state.new_exchange.kind, "topic".to_string(), "Topic");
                                ui.selectable_value(&mut self.state.new_exchange.kind, "headers".to_string(), "Headers");
                            });
                    });

                    ui.checkbox(&mut self.state.new_exchange.durable, "Durable");
                    ui.checkbox(&mut self.state.new_exchange.auto_delete, "Auto Delete");

                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            self.state.show_exchange_dialog = false;
                        }

                        if ui.button("Create Exchange").clicked() {
                            if self.state.new_exchange.name.is_empty() {
                                self.state.status_message = "Exchange name cannot be empty".to_string();
                            } else {
                                actions.push(UiAction::DeclareExchange);
                            }
                        }
                    });
                });
        }

        // Process any actions that were triggered
        for action in actions {
            match action {
                UiAction::Connect => self.state.connect_to_rabbitmq(),
                UiAction::Disconnect => self.state.disconnect_from_rabbitmq(),
                UiAction::Refresh => self.state.refresh_queues_and_exchanges(),
                UiAction::PublishMessage => self.state.publish_message(),
                UiAction::DeclareQueue => self.state.declare_queue(),
                UiAction::DeclareExchange => self.state.declare_exchange(),
            }
        }
    }
}

fn main() {
    env_logger::init();

    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0]),
        ..Default::default()
    };

    eframe::run_native(
        "RabbitMQ UI",
        native_options,
        Box::new(|_cc| Box::new(app::App::default())),
    ).unwrap();
}

