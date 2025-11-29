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
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Terminal,
};
use serde_json::Value;
use std::io;
use std::time::{Duration, Instant};

// Represents the different views or states of the application
#[derive(Clone, Debug)]
enum AppView {
    QueueList,
    QueueDetail { queue_name: String },
}

/// App holds the state of the application
pub struct App {
    client: RabbitMQApiClient,
    queues: Vec<Value>,
    should_quit: bool,
    status: String,
    queue_list_state: TableState,
    view_stack: Vec<AppView>,
}

impl App {
    fn new(client: RabbitMQApiClient) -> Self {
        let mut queue_list_state = TableState::default();
        queue_list_state.select(Some(0));

        Self {
            client,
            queues: Vec::new(),
            should_quit: false,
            status: "Fetching data...".to_string(),
            queue_list_state,
            view_stack: vec![AppView::QueueList], // Start with the queue list view
        }
    }

    fn current_view(&self) -> &AppView {
        self.view_stack.last().unwrap() // The current view is the last one on the stack
    }

    fn push_view(&mut self, view: AppView) {
        self.view_stack.push(view);
    }

    fn pop_view(&mut self) {
        // Don't pop the last view
        if self.view_stack.len() > 1 {
            self.view_stack.pop();
        }
    }

    pub fn next_queue(&mut self) {
        if self.queues.is_empty() {
            return;
        }
        let i = match self.queue_list_state.selected() {
            Some(i) => {
                if i >= self.queues.len() - 1 { 0 } else { i + 1 }
            }
            None => 0,
        };
        self.queue_list_state.select(Some(i));
    }

    pub fn previous_queue(&mut self) {
        if self.queues.is_empty() {
            return;
        }
        let i = match self.queue_list_state.selected() {
            Some(i) => {
                if i == 0 { self.queues.len() - 1 } else { i - 1 }
            }
            None => 0,
        };
        self.queue_list_state.select(Some(i));
    }

    /// Fetches queue data from the RabbitMQ API and updates the app state.
    async fn on_tick(&mut self) {
        match self.client.get_queues().await {
            Ok(queues) => {
                if queues.is_empty() {
                    self.queue_list_state.select(None);
                } else if self.queue_list_state.selected().is_none() {
                    self.queue_list_state.select(Some(0));
                }
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
    let file_config = load_config_file()?;
    let conn_info = file_config.connection;
    println!("Connecting as user: '{}'", conn_info.username);
    let password = rpassword::prompt_password("Enter password: ")?;
    let config = RabbitMQConfig {
        host: conn_info.host,
        amqp_port: conn_info.amqp_port,
        management_port: conn_info.management_port,
        username: conn_info.username,
        password,
        vhost: conn_info.vhost,
    };
    let client = RabbitMQApiClient::new(&config)?;
    println!("Checking RabbitMQ connection...");
    if let Err(e) = client.is_alive().await {
        eprintln!("Failed to connect to RabbitMQ: {}", e);
        return Ok(());
    }
    println!("RabbitMQ is alive. Launching monitor...");
    tokio::time::sleep(Duration::from_secs(1)).await;

    let mut app = App::new(client);
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal, &mut app).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
    }
    Ok(())
}

async fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<()> {
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_secs(5);
    app.on_tick().await;

    loop {
        if app.should_quit {
            return Ok(());
        }
        terminal.draw(|f| ui(f, app))?;

        let timeout = tick_rate.checked_sub(last_tick.elapsed()).unwrap_or_else(|| Duration::from_secs(0));
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                let current_view = app.current_view().clone();
                match current_view {
                    AppView::QueueList => match key.code {
                        KeyCode::Char('q') => app.should_quit = true,
                        KeyCode::Down => app.next_queue(),
                        KeyCode::Up => app.previous_queue(),
                        KeyCode::Enter => {
                            if let Some(selected) = app.queue_list_state.selected() {
                                if let Some(queue) = app.queues.get(selected) {
                                    let queue_name = queue["name"].as_str().unwrap_or("").to_string();
                                    app.push_view(AppView::QueueDetail { queue_name });
                                }
                            }
                        }
                        _ => {}
                    },
                    AppView::QueueDetail { .. } => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc | KeyCode::Backspace => app.pop_view(),
                        _ => {}
                    },
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.on_tick().await;
            last_tick = Instant::now();
        }
    }
}

fn ui(f: &mut ratatui::Frame<'_>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)].as_ref())
        .split(f.area());

    let breadcrumbs = app.view_stack.iter().map(|view| match view {
        AppView::QueueList => "Queues",
        AppView::QueueDetail { queue_name } => queue_name.as_str(),
    }).collect::<Vec<&str>>().join(" > ");
    
    let header_paragraph = Paragraph::new(Line::from(vec![
        Span::styled("RabbitMQ Monitor", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" | "),
        Span::styled(breadcrumbs, Style::default().fg(Color::Yellow)),
    ]))
    .block(Block::default().borders(Borders::ALL).title("Header"));
    f.render_widget(header_paragraph, chunks[0]);

    let current_view = app.current_view().clone();
    match current_view {
        AppView::QueueList => draw_queue_list(f, &app.queues, &mut app.queue_list_state, chunks[1]),
        AppView::QueueDetail { queue_name } => draw_queue_details(f, &app.queues, chunks[1], &queue_name),
    }

    let footer_paragraph = Paragraph::new(Line::from(format!("Status: {} | Press 'q' to quit", app.status)))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer_paragraph, chunks[2]);
}

fn draw_queue_list(f: &mut ratatui::Frame<'_>, queues: &[Value], state: &mut TableState, area: Rect) {
    let header_cells = ["Queue", "Messages", "Consumers"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells).style(Style::default().bg(Color::DarkGray)).height(1);

    let rows = queues.iter().map(|q| {
        let name = q["name"].as_str().unwrap_or("N/A");
        let messages = q["messages"].as_i64().unwrap_or(0).to_string();
        let consumers = q["consumers"].as_i64().unwrap_or(0).to_string();
        Row::new(vec![Cell::from(name), Cell::from(messages), Cell::from(consumers)])
    });

    let table = Table::new(rows, &[Constraint::Percentage(60), Constraint::Percentage(20), Constraint::Percentage(20)])
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Queues"))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");
    
    f.render_stateful_widget(table, area, state);
}

fn draw_queue_details(f: &mut ratatui::Frame<'_>, queues: &[Value], area: Rect, queue_name: &str) {
    let details_text = if let Some(queue) = queues.iter().find(|q| q["name"].as_str() == Some(queue_name)) {
        format!("{:#}", queue) // Pretty-print the JSON for the selected queue
    } else {
        format!("Details for queue '{}' not found.", queue_name)
    };

    let paragraph = Paragraph::new(details_text)
        .block(Block::default().borders(Borders::ALL).title(format!("Details for {}", queue_name)))
        .wrap(ratatui::widgets::Wrap { trim: true });

    f.render_widget(paragraph, area);
}
