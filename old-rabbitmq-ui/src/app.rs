// rabbitmq_ui/src/app.rs

use eframe::egui::{self, Color32, RichText};
use egui_components::TreeNodeId;
use std::collections::HashMap;
use lapin::types::FieldTable;
use rabbitmq_info::api::RabbitMQApiClient;

use rabbitmq_config::{
    ExchangeInfo, MessageProperties, QueueInfo, RabbitMQClient, RabbitMQConfig, RabbitMQMessage,
};
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;

pub enum UiAction {
    Connect,
    Disconnect,
    Refresh,
    PublishMessage,
    DeclareQueue,
    DeclareExchange,
}

#[derive(Default)]
pub struct TreeState {
    #[allow(dead_code)]
    queue_tree_nodes: HashMap<TreeNodeId, String>,
    #[allow(dead_code)]
    queue_children: HashMap<TreeNodeId, Vec<TreeNodeId>>,
    #[allow(dead_code)]
    selected_queue: Option<TreeNodeId>,
}

pub struct AppState {
    runtime: Arc<Runtime>,
    amqp_client: Option<Arc<Mutex<RabbitMQClient>>>,
    api_client: Option<RabbitMQApiClient>,
    connection_status: bool,
    config: RabbitMQConfig,
    message: RabbitMQMessage,
    status_message: String,
    available_queues: Vec<String>,
    available_exchanges: Vec<String>,
    new_queue: QueueInfo,
    new_exchange: ExchangeInfo,
    show_queue_dialog: bool,
    show_exchange_dialog: bool,
    #[allow(dead_code)]
    tree_state: TreeState,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            runtime: Arc::new(Runtime::new().expect("Failed to create Tokio runtime")),
            amqp_client: None,
            api_client: None,
            connection_status: false,
            config: RabbitMQConfig::default(),
            message: RabbitMQMessage {
                exchange: "".to_string(),
                routing_key: "".to_string(),
                payload: Vec::new(),
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
                arguments: FieldTable::default(),
            },
            new_exchange: ExchangeInfo {
                name: "".to_string(),
                kind: "direct".to_string(),
                durable: true,
                auto_delete: false,
                internal: false,
                arguments: FieldTable::default(),
            },
            show_queue_dialog: false,
            show_exchange_dialog: false,
            tree_state: TreeState::default(),
        }
    }
}

impl AppState {
    pub fn connect_to_rabbitmq(&mut self) {
        let config = self.config.clone();
        let runtime = self.runtime.clone();

        let client_future = async move { RabbitMQClient::new(config.clone()).await };

        match runtime.block_on(client_future) {
            Ok(client) => {
                self.amqp_client = Some(Arc::new(Mutex::new(client)));
                self.api_client = RabbitMQApiClient::new(&self.config).ok();
                self.connection_status = true;
                self.status_message = format!("Connected to RabbitMQ at {}", self.config.host);
                self.refresh_queues_and_exchanges();
            }
            Err(err) => {
                self.status_message = format!("Connection failed: {err:?}");
            }
        }
    }

    pub fn disconnect_from_rabbitmq(&mut self) {
        if let Some(client) = &self.amqp_client {
            let client = client.clone();
            let runtime = self.runtime.clone();
            let close_future = async move { client.lock().await.close().await };
            match runtime.block_on(close_future) {
                Ok(_) => self.status_message = "Disconnected from RabbitMQ".to_string(),
                Err(err) => self.status_message = format!("Error during disconnect: {err:?}"),
            }
        }
        self.amqp_client = None;
        self.api_client = None;
        self.connection_status = false;
        self.available_queues.clear();
        self.available_exchanges.clear();
    }

    pub fn refresh_queues_and_exchanges(&mut self) {
        if let Some(api_client) = &self.api_client {
            let runtime = self.runtime.clone();
            
            let queues_future = api_client.get_queues();
            match runtime.block_on(queues_future) {
                Ok(queues) => {
                    self.available_queues = queues.into_iter()
                        .filter_map(|q| q.get("name").and_then(|n| n.as_str().map(String::from)))
                        .collect();
                }
                Err(err) => self.status_message = format!("Failed to fetch queues: {err:?}"),
            }

            let exchanges_future = api_client.get_exchanges();
            match runtime.block_on(exchanges_future) {
                Ok(exchanges) => {
                    self.available_exchanges = exchanges.into_iter()
                        .filter_map(|e| e.get("name").and_then(|n| n.as_str().map(String::from)))
                        .collect();
                }
                Err(err) => self.status_message = format!("Failed to fetch exchanges: {err:?}"),
            }
        }
    }

    pub fn publish_message(&mut self) {
        if let Some(client) = &self.amqp_client {
            let client = client.clone();
            let runtime = self.runtime.clone();
            let message = self.message.clone();
            let publish_future = async move { client.lock().await.publish_message(&message).await };
            match runtime.block_on(publish_future) {
                Ok(_) => self.status_message = "Message published successfully".to_string(),
                Err(err) => self.status_message = format!("Failed to publish message: {err:?}"),
            }
        } else {
            self.status_message = "Not connected to RabbitMQ".to_string();
        }
    }

    pub fn declare_queue(&mut self) {
        if let Some(client) = &self.amqp_client {
            let client = client.clone();
            let runtime = self.runtime.clone();
            let queue_info = self.new_queue.clone();
            let queue_name = queue_info.name.clone();
            let queue_future = async move { client.lock().await.declare_queue(&queue_info).await };
            match runtime.block_on(queue_future) {
                Ok(_) => {
                    self.status_message = format!("Queue '{queue_name}' created successfully");
                    self.new_queue.name = "".to_string();
                    self.show_queue_dialog = false;
                    self.refresh_queues_and_exchanges();
                }
                Err(err) => self.status_message = format!("Failed to create queue: {err:?}"),
            }
        } else {
            self.status_message = "Not connected to RabbitMQ".to_string();
        }
    }

    pub fn declare_exchange(&mut self) {
        if let Some(client) = &self.amqp_client {
            let client = client.clone();
            let runtime = self.runtime.clone();
            let exchange_info = self.new_exchange.clone();
            let exchange_name = exchange_info.name.clone();
            let exchange_future = async move { client.lock().await.declare_exchange(&exchange_info).await };
            match runtime.block_on(exchange_future) {
                Ok(_) => {
                    self.status_message = format!("Exchange '{exchange_name}' created successfully");
                    self.new_exchange.name = "".to_string();
                    self.show_exchange_dialog = false;
                    self.refresh_queues_and_exchanges();
                }
                Err(err) => self.status_message = format!("Failed to create exchange: {err:?}"),
            }
        } else {
            self.status_message = "Not connected to RabbitMQ".to_string();
        }
    }
}

#[derive(Default)]
pub struct App {
    state: AppState,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let screen_rect = ctx.screen_rect();
        let north_height = 50.0;
        let south_height = 30.0;
        let west_width = 150.0;
        let north_rect = egui::Rect::from_min_size(screen_rect.min, egui::Vec2::new(screen_rect.width(), north_height));
        let south_rect = egui::Rect::from_min_size(egui::Pos2::new(screen_rect.min.x, screen_rect.max.y - south_height), egui::Vec2::new(screen_rect.width(), south_height));
        let remaining_rect = egui::Rect::from_min_max(egui::Pos2::new(screen_rect.min.x, north_rect.max.y), egui::Pos2::new(screen_rect.max.x, south_rect.min.y));
        let west_rect = egui::Rect::from_min_size(remaining_rect.min, egui::Vec2::new(west_width, remaining_rect.height()));
        let center_rect = egui::Rect::from_min_max(egui::Pos2::new(west_rect.max.x, remaining_rect.min.y), remaining_rect.max);
        let mut actions = Vec::new();

        egui::Area::new("north_panel").fixed_pos(north_rect.min).show(ctx, |ui| {
            ui.set_max_size(north_rect.size());
            ui.set_width(north_rect.width());
            ui.set_height(north_rect.height());
            ui.painter().rect_filled(north_rect, 0.0, Color32::from_rgb(173, 216, 230));
            ui.horizontal(|ui| {
                let status_text = if self.state.connection_status {
                    RichText::new("Connected").color(Color32::GREEN)
                } else {
                    RichText::new("Disconnected").color(Color32::RED)
                };
                ui.label(status_text);
                if !self.state.connection_status {
                    if ui.button("Connect").clicked() {
                        actions.push(UiAction::Connect);
                    }
                    ui.label("Host:");
                    ui.text_edit_singleline(&mut self.state.config.host);
                    ui.label("Port:");
                    let mut port_str = self.state.config.amqp_port.to_string();
                    if ui.text_edit_singleline(&mut port_str).changed() {
                        if let Ok(port) = port_str.parse::<u16>() {
                            self.state.config.amqp_port = port;
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

        egui::Area::new("south_panel").fixed_pos(south_rect.min).show(ctx, |ui| {
            ui.set_max_size(south_rect.size());
            ui.set_width(south_rect.width());
            ui.set_height(south_rect.height());
            ui.painter().rect_filled(south_rect, 0.0, Color32::from_rgb(144, 238, 144));
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.label(&self.state.status_message);
            });
        });

        egui::Area::new("west_panel").fixed_pos(west_rect.min).show(ctx, |ui| {
            ui.set_max_size(west_rect.size());
            ui.set_width(west_rect.width());
            ui.set_height(west_rect.height());
            ui.painter().rect_filled(west_rect, 0.0, Color32::from_rgb(255, 218, 185));
            ui.vertical(|ui| {
                ui.heading("Queues");
                ui.separator();
                for queue in &self.state.available_queues {
                    if ui.selectable_label(false, queue).clicked() {
                        self.state.status_message = format!("Selected queue: {queue}");
                    }
                }
                ui.add_space(10.0);
                ui.heading("Exchanges");
                ui.separator();
                for exchange in &self.state.available_exchanges {
                    if ui.selectable_label(false, exchange).clicked() {
                        self.state.status_message = format!("Selected exchange: {exchange}");
                    }
                }
                ui.add_space(10.0);
                if ui.button("Declare Queue").clicked() {
                    self.state.show_queue_dialog = true;
                }
                if ui.button("Declare Exchange").clicked() {
                    self.state.show_exchange_dialog = true;
                }
            });
        });

        egui::Area::new("center_panel").fixed_pos(center_rect.min).show(ctx, |ui| {
            ui.set_max_size(center_rect.size());
            ui.set_width(center_rect.width());
            ui.set_height(center_rect.height());
            ui.painter().rect_filled(center_rect, 0.0, Color32::from_rgb(230, 230, 250));
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
                let mut payload_text = String::from_utf8_lossy(&self.state.message.payload).to_string();
                if ui.text_edit_multiline(&mut payload_text).changed() {
                    self.state.message.payload = payload_text.into_bytes();
                }
                if ui.button("Publish").clicked() {
                    actions.push(UiAction::PublishMessage);
                }
            });
        });

        if self.state.show_queue_dialog {
            egui::Window::new("Declare Queue").collapsible(false).resizable(false).show(ctx, |ui| {
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

        if self.state.show_exchange_dialog {
            egui::Window::new("Declare Exchange").collapsible(false).resizable(false).show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Exchange Name:");
                    ui.text_edit_singleline(&mut self.state.new_exchange.name);
                });
                ui.horizontal(|ui| {
                    ui.label("Type:");
                    egui::ComboBox::from_id_source("exchange_type").selected_text(&self.state.new_exchange.kind).show_ui(ui, |ui| {
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

        for action in actions {
            match action {
                UiAction::Connect => self.state.connect_to_rabbitmq(),
                UiAction::Disconnect => self.state.disconnect_from_rabbitmq(),
                UiAction::Refresh => self.state.refresh_queues_and_exchanges(),
                UiAction::PublishMessage => self.state.publish_message(),
                UiAction.DeclareQueue => self.state.declare_queue(),
                UiAction.DeclareExchange => self.state.declare_exchange(),
            }
        }
    }
}
