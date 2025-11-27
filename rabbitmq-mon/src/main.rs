use chrono::Local;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use rabbitmq_config::{load_config_file, RabbitMQConfig};
use rabbitmq_info::api::RabbitMQApiClient;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use serde_json::Value;
use std::io;
use std::time::{Duration, Instant};

/// App holds the state of the application
struct App {
    client: RabbitMQApiClient,
    queues: Vec<Value>,
    should_quit: bool,
    status: String,
}

impl App {
    fn new(client: RabbitMQApiClient) -> Self {
        Self {
            client,
            queues: Vec::new(),
            should_quit: false,
            status: "Fetching data...".to_string(),
        }
    }

    /// Fetches queue data from the RabbitMQ API and updates the app state.
    async fn on_tick(&mut self) {
        match self.client.get_queues().await {
            Ok(queues) => {
                self.queues = queues;
                self.status = format!("Updated at {}", Local::now().format("%H:%M:%S"));
            }
            Err(e) => {
                self.status = format!("Error fetching data: {}", e);
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // Load non-sensitive info from the config file
    let file_config = load_config_file()?;
    let conn_info = file_config.connection;

    // Prompt for the password
    println!("Connecting as user: '{}'", conn_info.username);
    let password = rpassword::prompt_password("Enter password: ")?;

    // Assemble the final config
    let config = RabbitMQConfig {
        host: conn_info.host,
        port: conn_info.port,
        username: conn_info.username,
        password,
        vhost: conn_info.vhost,
    };

    let client = RabbitMQApiClient::new(&config)?;

    // Check if RabbitMQ is alive before proceeding
    println!("Checking RabbitMQ connection...");
    if let Err(e) = client.is_alive().await {
        eprintln!("Failed to connect to RabbitMQ: {}", e);
        eprintln!("Please ensure RabbitMQ is running and the configuration is correct.");
        return Ok(());
    }
    println!("RabbitMQ is alive. Launching monitor...");
    tokio::time::sleep(Duration::from_secs(1)).await;


    // Create app state
    let mut app = App::new(client);

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run the app
    let res = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

async fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<()> {
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_secs(5); // Refresh every 5 seconds
    
    // Initial data fetch
    app.on_tick().await;

    loop {
        if app.should_quit {
            return Ok(());
        }

        terminal.draw(|f| ui(f, app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    app.should_quit = true;
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.on_tick().await;
            last_tick = Instant::now();
        }
    }
}

fn ui(f: &mut ratatui::Frame<'_>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Footer
        ].as_ref())
        .split(f.size());

    // Header
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "RabbitMQ Monitor",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title("Header"));
    f.render_widget(header, chunks[0]);

    // Main content area - Display Queues
    let queue_lines: Vec<Line> = app.queues.iter().map(|q| {
        let name = q["name"].as_str().unwrap_or("N/A");
        let messages = q["messages"].as_i64().unwrap_or(0);
        let consumers = q["consumers"].as_i64().unwrap_or(0);
        Line::from(format!("Queue: {:<30} | Messages: {:<10} | Consumers: {}", name, messages, consumers))
    }).collect();

    let content = Paragraph::new(queue_lines)
        .block(Block::default().borders(Borders::ALL).title("Queues"));
    f.render_widget(content, chunks[1]);

    // Footer
    let footer_text = format!("Status: {} | Press 'q' to quit", app.status);
    let footer = Paragraph::new(Line::from(footer_text))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[2]);
}
