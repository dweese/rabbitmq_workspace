use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use eframe::egui;
use rabbitmq_config::{RabbitMQClient, RabbitMQConfig};
use rabbitmq_info::api::RabbitMQApiClient;

// Enums for communication between UI and async tasks
#[derive(Debug, Clone)]
pub enum UiRequest {
    Connect(RabbitMQConfig),
    Disconnect,
    Refresh,
}

#[derive(Debug)]
pub enum ServerResponse {
    Connected,
    Disconnected,
    Error(String),
    Queues(Vec<String>),
    Exchanges(Vec<String>),
}

// The state of our application
pub struct AppState {
    runtime: Arc<Runtime>,
    request_tx: mpsc::Sender<UiRequest>,
    response_rx: mpsc::Receiver<ServerResponse>,
    
    config: RabbitMQConfig,
    password_input: String,
    connection_status: bool,
    status_message: String,
    available_queues: Vec<String>,
    available_exchanges: Vec<String>,
}

impl AppState {
    pub fn new() -> Self {
        let runtime = Arc::new(Runtime::new().expect("Failed to create Tokio runtime"));
        let (request_tx, mut request_rx) = mpsc::channel(10);
        let (response_tx, response_rx) = mpsc::channel(10);

        // --- The Async Task ---
        runtime.spawn(async move {
            let mut amqp_client: Option<Arc<tokio::sync::Mutex<RabbitMQClient>>> = None;
            let mut api_client: Option<RabbitMQApiClient> = None;

            while let Some(request) = request_rx.recv().await {
                match request {
                    UiRequest::Connect(config) => {
                        let connect_future = RabbitMQClient::new(config.clone());
                        match connect_future.await {
                            Ok(client) => {
                                amqp_client = Some(Arc::new(tokio::sync::Mutex::new(client)));
                                api_client = RabbitMQApiClient::new(&config).ok();
                                response_tx.send(ServerResponse::Connected).await.ok();
                            }
                            Err(e) => {
                                response_tx.send(ServerResponse::Error(format!("Connection failed: {}", e))).await.ok();
                            }
                        }
                    }
                    UiRequest::Disconnect => {
                        if let Some(client) = amqp_client.take() {
                            let _ = client.lock().await.close().await;
                        }
                        api_client = None;
                        response_tx.send(ServerResponse::Disconnected).await.ok();
                    }
                    UiRequest::Refresh => {
                        if let Some(client) = &api_client {
                            match client.get_queues().await {
                                Ok(queues) => {
                                    let queue_names = queues.into_iter()
                                        .filter_map(|q| q.get("name").and_then(|n| n.as_str().map(String::from)))
                                        .collect();
                                    response_tx.send(ServerResponse::Queues(queue_names)).await.ok();
                                }
                                Err(e) => {
                                    response_tx.send(ServerResponse::Error(format!("Failed to get queues: {}", e))).await.ok();
                                }
                            }
                            match client.get_exchanges().await {
                                Ok(exchanges) => {
                                    let exchange_names = exchanges.into_iter()
                                        .filter_map(|e| e.get("name").and_then(|n| n.as_str().map(String::from)))
                                        .collect();
                                    response_tx.send(ServerResponse::Exchanges(exchange_names)).await.ok();
                                }
                                Err(e) => {
                                    response_tx.send(ServerResponse::Error(format!("Failed to get exchanges: {}", e))).await.ok();
                                }
                            }
                        } else {
                            response_tx.send(ServerResponse::Error("Not connected".to_string())).await.ok();
                        }
                    }
                }
            }
        });

        Self {
            runtime,
            request_tx,
            response_rx,
            config: RabbitMQConfig::default(),
            password_input: String::new(),
            connection_status: false,
            status_message: "Welcome to the new RabbitMQ UI".to_string(),
            available_queues: Vec::new(),
            available_exchanges: Vec::new(),
        }
    }
}

pub struct App {
    state: AppState,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: AppState::new(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(response) = self.state.response_rx.try_recv() {
            match response {
                ServerResponse::Connected => {
                    self.state.connection_status = true;
                    self.state.status_message = "Connected!".to_string();
                }
                ServerResponse::Disconnected => {
                    self.state.connection_status = false;
                    self.state.status_message = "Disconnected.".to_string();
                    self.state.available_queues.clear();
                    self.state.available_exchanges.clear();
                }
                ServerResponse::Error(msg) => {
                    self.state.status_message = format!("Error: {}", msg);
                }
                ServerResponse::Queues(queues) => {
                    self.state.available_queues = queues;
                }
                ServerResponse::Exchanges(exchanges) => {
                    self.state.available_exchanges = exchanges;
                }
            }
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if !self.state.connection_status {
                    ui.vertical(|ui| {
                        egui::Grid::new("connection_grid")
                            .num_columns(2)
                            .spacing([10.0, 4.0])
                            .striped(true)
                            .show(ui, |ui| {
                                ui.label("Host:");
                                ui.text_edit_singleline(&mut self.state.config.host);
                                ui.end_row();

                                ui.label("VHost:");
                                ui.text_edit_singleline(&mut self.state.config.vhost);
                                ui.end_row();

                                ui.label("Username:");
                                ui.text_edit_singleline(&mut self.state.config.username);
                                ui.end_row();

                                ui.label("Password:");
                                ui.add(egui::TextEdit::singleline(&mut self.state.password_input).password(true));
                                ui.end_row();
                            });
                        
                        if ui.button("Connect").clicked() {
                            let mut config = self.state.config.clone();
                            config.password = self.state.password_input.clone();
                            self.state.request_tx.blocking_send(UiRequest::Connect(config)).ok();
                        }
                    });
                } else {
                    if ui.button("Disconnect").clicked() {
                        self.state.request_tx.blocking_send(UiRequest::Disconnect).ok();
                    }
                    if ui.button("Refresh").clicked() {
                        self.state.request_tx.blocking_send(UiRequest::Refresh).ok();
                    }
                }
            });
        });

        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            ui.heading("Queues");
            egui::ScrollArea::vertical().id_source("queues_scroll").show(ui, |ui| {
                for queue in &self.state.available_queues {
                    ui.label(queue);
                }
            });
            ui.separator();
            ui.heading("Exchanges");
            egui::ScrollArea::vertical().id_source("exchanges_scroll").show(ui, |ui| {
                for exchange in &self.state.available_exchanges {
                    ui.label(exchange);
                }
            });
        });
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("RabbitMQ UI");
            ui.label(format!("Connection Status: {}", if self.state.connection_status { "Connected" } else { "Disconnected" }));
            ui.separator();
            ui.label(&self.state.status_message);
        });
    }
}
