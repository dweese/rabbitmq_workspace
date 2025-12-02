use anyhow::Result;
use rabbitmq_config::{get_password, load_config_file, RabbitMQClient, RabbitMQConfig, RabbitMQMessage, MessageProperties};
use serde::Deserialize;
use serde_json::json;
use std::fs;
use std::io;
use std::path::PathBuf;
use tui_textarea::TextArea;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
    Terminal,
};

// --- Serde Structs for message_types.json ---

#[derive(Debug, Deserialize)]
struct MessageTypes {
    message_types: Vec<Category>,
}

#[derive(Debug, Deserialize, Clone)]
struct Category {
    category: String,
    types: Vec<MessageType>,
}

#[derive(Debug, Deserialize, Clone)]
struct MessageType {
    name: String,
    #[serde(default)]
    priority: u8,
    durable: bool,
}

// --- Application State ---

struct App<'a> {
    should_quit: bool,
    editor: TextArea<'a>,
    categories: Vec<Category>,
    template_list_state: ListState,
    client: RabbitMQClient,
    logs: Vec<String>,
}

impl<'a> App<'a> {
    async fn new(categories: Vec<Category>) -> Result<Self> {
        let mut editor = TextArea::default();
        editor.set_block(Block::default().borders(Borders::ALL).title("Message Editor"));

        let mut template_list_state = ListState::default();
        if !categories.is_empty() {
            template_list_state.select(Some(0));
        }

        // --- Connect to RabbitMQ ---
        let file_config = load_config_file()?;
        let conn_info = file_config.connection;
        let password = get_password()?;
        let config = RabbitMQConfig {
            host: conn_info.host,
            amqp_port: conn_info.amqp_port,
            management_port: conn_info.management_port,
            username: conn_info.username,
            password,
            vhost: conn_info.vhost,
        };
        let client = RabbitMQClient::new(config).await?;

        let mut app = App {
            should_quit: false,
            editor,
            categories,
            template_list_state,
            client,
            logs: Vec::new(),
        };

        app.update_editor_payload();
        Ok(app)
    }

    fn get_flat_templates(&self) -> Vec<(MessageType, Category)> {
        self.categories
            .iter()
            .flat_map(|c| c.types.iter().map(move |t| (t.clone(), c.clone())))
            .collect()
    }

    fn next_template(&mut self) {
        let flat_templates = self.get_flat_templates();
        let i = match self.template_list_state.selected() {
            Some(i) => {
                if i >= flat_templates.len() - 1 { 0 } else { i + 1 }
            }
            None => 0,
        };
        self.template_list_state.select(Some(i));
        self.update_editor_payload();
    }

    fn previous_template(&mut self) {
        let flat_templates = self.get_flat_templates();
        let i = match self.template_list_state.selected() {
            Some(i) => {
                if i == 0 { flat_templates.len() - 1 } else { i - 1 }
            }
            None => 0,
        };
        self.template_list_state.select(Some(i));
        self.update_editor_payload();
    }

    fn update_editor_payload(&mut self) {
        if let Some(selected_index) = self.template_list_state.selected() {
            let flat_templates = self.get_flat_templates();
            if let Some((template, _)) = flat_templates.get(selected_index) {
                let payload = generate_sample_payload(&template.name);
                self.editor.select_all();
                self.editor.delete_char();
                self.editor.insert_str(&payload);
            }
        }
    }

    async fn publish_current_message(&mut self) {
        if let Some(selected_index) = self.template_list_state.selected() {
            let flat_templates = self.get_flat_templates();
            if let Some((template, category)) = flat_templates.get(selected_index) {
                let payload = self.editor.lines().join("\n");

                let message = RabbitMQMessage {
                    exchange: category.category.clone(),
                    routing_key: template.name.clone(),
                    payload: payload.into_bytes(),
                    properties: Some(MessageProperties {
                        priority: if template.priority > 0 { Some(template.priority) } else { None },
                        ..Default::default()
                    }),
                };

                match self.client.publish_message(&message).await {
                    Ok(_) => self.logs.push(format!("Published message to '{}'", template.name)),
                    Err(e) => self.logs.push(format!("Error publishing: {}", e)),
                }
            }
        }
    }
}

// --- Main Application Logic ---

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let categories = load_message_categories()?;
    let mut app = App::new(categories).await?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    run_app(&mut terminal, &mut app).await?;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn load_message_categories() -> Result<Vec<Category>> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../artifacts/message_types.json");
    let topology_str = fs::read_to_string(path)?;
    let topology: MessageTypes = serde_json::from_str(&topology_str)?;
    Ok(topology.message_types)
}

/// Generates a sample JSON payload based on the message name.
fn generate_sample_payload(message_name: &str) -> String {
    let payload = match message_name {
        "user.create" => json!({ "email": "user@example.com", "name": "Test User" }),
        "user.update" => json!({ "id": "user-123", "name": "Updated Name" }),
        "user.delete" => json!({ "id": "user-123" }),
        "order.placed" => json!({ "order_id": "ord-456", "amount": 99.99, "items": [ "item-1", "item-2" ] }),
        "payment.processed" => json!({ "order_id": "ord-456", "status": "success", "transaction_id": "txn-789" }),
        "inventory.updated" => json!({ "item_id": "item-1", "quantity": 50 }),
        "user.lookup" => json!({ "email": "user@example.com" }),
        "order.status" => json!({ "order_id": "ord-456" }),
        _ => json!({ "message": "No sample payload defined." }),
    };
    serde_json::to_string_pretty(&payload).unwrap_or_default()
}

async fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App<'_>) -> io::Result<()> {
    loop {
        if app.should_quit {
            return Ok(());
        }

        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => app.should_quit = true,
                KeyCode::Char('p') => app.publish_current_message().await,
                KeyCode::Up => app.previous_template(),
                KeyCode::Down => app.next_template(),
                _ => {
                    app.editor.input(key);
                }
            }
        }
    }
}

fn ui(f: &mut ratatui::Frame<'_>, app: &mut App) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
        ])
        .split(f.area());

    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(70),
        ])
        .split(main_chunks[0]);

    // --- Producers Pane ---
    let templates = app.get_flat_templates();
    let items: Vec<ListItem> = templates
        .iter()
        .map(|(t, _)| ListItem::new(t.name.as_str()))
        .collect();

    let producers_list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Producers (Templates)"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan))
        .highlight_symbol(">> ");

    f.render_stateful_widget(producers_list, top_chunks[0], &mut app.template_list_state);

    // --- Editor Pane ---
    f.render_widget(app.editor.widget(), top_chunks[1]);

    // --- Consumers Pane ---
    let consumers_pane = Block::default().title("Consumers (Queues)").borders(Borders::ALL);
    f.render_widget(consumers_pane, main_chunks[1]);

    // --- Logs Pane ---
    let log_items: Vec<ListItem> = app.logs.iter().rev().map(|l| ListItem::new(l.as_str())).collect();
    let logs_list = List::new(log_items)
        .block(Block::default().borders(Borders::ALL).title("Logs"));
    f.render_widget(logs_list, main_chunks[2]);
}
