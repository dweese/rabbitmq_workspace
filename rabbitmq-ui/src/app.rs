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
                        response_tx.send(ServerResponse::Error("Refresh not implemented".to_string())).await.ok();
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
                }
                ServerResponse::Error(msg) => {
                    self.state.status_message = format!("Error: {}", msg);
                }
            }
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if !self.state.connection_status {
                    if ui.button("Connect").clicked() {
                        let mut config = self.state.config.clone();
                        config.password = self.state.password_input.clone();
                        self.state.request_tx.blocking_send(UiRequest::Connect(config)).ok();
                    }
                    ui.label("Host:");
                    ui.text_edit_singleline(&mut self.state.config.host);

                    ui.label("VHost:");
                    ui.text_edit_singleline(&mut self.state.config.vhost);
                    
                    ui.label("Username:");
                    ui.text_edit_singleline(&mut self.state.config.username);

                    ui.label("Password:");
                    ui.add(egui::TextEdit::singleline(&mut self.state.password_input).password(true));
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
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("RabbitMQ UI");
            ui.label(format!("Connection Status: {}", if self.state.connection_status { "Connected" } else { "Disconnected" }));
            ui.separator();
            ui.label(&self.state.status_message);
        });
    }
}
