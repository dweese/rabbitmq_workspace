use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use eframe::egui;
use egui_components::TreeNodeId;  // Remove unused imports
use serde_json::Value;

pub enum JsonAction {
    LoadFile,
    Refresh,
    ExpandAll,
    CollapseAll,
    SaveExpandedState,
}

#[derive(Default)]
pub struct JsonState {
    tree_nodes: HashMap<TreeNodeId, JsonNodeData>,
    tree_children: HashMap<TreeNodeId, Vec<TreeNodeId>>,
    #[allow(dead_code)]  // Will be used later for tree interaction
    selected_node: Option<TreeNodeId>,
    #[allow(dead_code)]  // Will be used later for tree expansion state
    expanded_nodes: std::collections::HashSet<TreeNodeId>,
}


#[derive(Debug, Clone)]
pub struct JsonNodeData {
    pub key: String,
    pub value_type: String,
    pub preview: String,
    pub full_value: String,
    pub path: String,
}


pub struct AppState {
    json_content: Option<Value>,
    file_path: Option<PathBuf>,
    status_message: String,
    json_state: JsonState,
    show_file_dialog: bool,
    #[allow(dead_code)]  // Will be used later for search functionality
    search_query: String,
    show_raw_json: bool,
}


impl Default for AppState {
    fn default() -> Self {
        Self {
            json_content: None,
            file_path: None,
            status_message: "Ready to load JSON file".to_string(),
            json_state: JsonState::default(),
            show_file_dialog: false,
            search_query: String::new(),
            show_raw_json: false,
        }
    }
}

impl AppState {
    pub fn load_json_file(&mut self, path: PathBuf) {
        match fs::read_to_string(&path) {
            Ok(content) => {
                match serde_json::from_str::<Value>(&content) {
                    Ok(json) => {
                        self.json_content = Some(json.clone());
                        self.file_path = Some(path.clone());
                        self.status_message = format!("Loaded: {:?}", path.file_name().unwrap_or_default());
                        self.build_tree_from_json(&json);
                        println!("Successfully loaded JSON file: {path:?}");  // Changed from log
                    }
                    Err(e) => {
                        self.status_message = format!("JSON parse error: {e}");
                        eprintln!("JSON parse error: {e}");  // Changed from log
                    }
                }
            }
            Err(e) => {
                self.status_message = format!("File read error: {e}");
                eprintln!("File read error: {e}");  // Changed from log
            }
        }
    }

    pub fn load_artifacts_file(&mut self, filename: &str) {
        let path = PathBuf::from("artifacts").join(filename);
        self.load_json_file(path);
    }

    fn build_tree_from_json(&mut self, json: &Value) {
        self.json_state = JsonState::default();
        let root_id = TreeNodeId::new("root");  // Fixed: provide ID argument
        
        let root_data = JsonNodeData {
            key: "root".to_string(),
            value_type: self.get_value_type(json),
            preview: self.get_value_preview(json),
            full_value: json.to_string(),
            path: "".to_string(),
        };
        
        self.json_state.tree_nodes.insert(root_id.clone(), root_data);  // Clone here
        self.build_tree_recursive(json, root_id, "".to_string());  // Use original here
    }

    fn build_tree_recursive(&mut self, value: &Value, parent_id: TreeNodeId, path: String) {
        let children = match value {
            Value::Object(map) => {
                map.iter().map(|(key, val)| {
                    let child_id = parent_id.child(key);  // Use the child() method
                    let child_path = if path.is_empty() { 
                        key.clone() 
                    } else { 
                        format!("{path}.{key}") 
                    };
                    
                    let child_data = JsonNodeData {
                        key: key.clone(),
                        value_type: self.get_value_type(val),
                        preview: self.get_value_preview(val),
                        full_value: val.to_string(),
                        path: child_path.clone(),
                    };
                    
                    self.json_state.tree_nodes.insert(child_id.clone(), child_data);  // Clone here
                    self.build_tree_recursive(val, child_id.clone(), child_path);  // Clone here
                    child_id  // Return original here
                }).collect()
            }
            Value::Array(arr) => {
                arr.iter().enumerate().map(|(idx, val)| {
                    let child_id = parent_id.child(format!("array_{idx}"));  // Use the child() method
                    let child_path = if path.is_empty() { 
                        format!("[{idx}]") 
                    } else { 
                        format!("{path}[{idx}]") 
                    };
                    
                    let child_data = JsonNodeData {
                        key: format!("[{idx}]"),
                        value_type: self.get_value_type(val),
                        preview: self.get_value_preview(val),
                        full_value: val.to_string(),
                        path: child_path.clone(),
                    };
                    
                    self.json_state.tree_nodes.insert(child_id.clone(), child_data);  // Clone here
                    self.build_tree_recursive(val, child_id.clone(), child_path);  // Clone here
                    child_id  // Return original here
                }).collect()
            }
            _ => Vec::new(),
        };
        
        if !children.is_empty() {
            self.json_state.tree_children.insert(parent_id, children);
        }
    }

    fn get_value_type(&self, value: &Value) -> String {
        match value {
            Value::Null => "null".to_string(),
            Value::Bool(_) => "boolean".to_string(),
            Value::Number(_) => "number".to_string(),
            Value::String(_) => "string".to_string(),
            Value::Array(arr) => format!("array[{}]", arr.len()),
            Value::Object(obj) => format!("object{{{}}}", obj.len()),
        }
    }

    fn get_value_preview(&self, value: &Value) -> String {
        match value {
            Value::Null => "null".to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Number(n) => n.to_string(),
            Value::String(s) => {
                if s.len() > 50 {
                    format!("\"{}...\"", &s[..47])
                } else {
                    format!("\"{s}\"")
                }
            }
            Value::Array(arr) => format!("[{} items]", arr.len()),
            Value::Object(obj) => format!("{{{} keys}}", obj.len()),
        }
    }
}

pub struct App {
    state: AppState,
}

impl Default for App {
    fn default() -> Self {
        let mut app = Self {
            state: AppState::default(),
        };
        
        // Try to load a sample file from artifacts
        app.state.load_artifacts_file("rabbitmq_config.json");
        
        app
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Load JSON File").clicked() {
                        self.state.show_file_dialog = true;
                    }
                    ui.separator();
                    if ui.button("Load RabbitMQ Config").clicked() {
                        self.state.load_artifacts_file("rabbitmq_config.json");
                    }
                    if ui.button("Load RabbitMQ Export").clicked() {
                        self.state.load_artifacts_file("rabbit_fedora_2025-5-23.json");
                    }
                });
                
                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.state.show_raw_json, "Show Raw JSON");
                    if ui.button("Expand All").clicked() {
                        // TODO: Implement expand all
                    }
                    if ui.button("Collapse All").clicked() {
                        // TODO: Implement collapse all
                    }
                });
            });
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Status:");
                ui.label(&self.state.status_message);
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(path) = &self.state.file_path {
                        ui.label(format!("File: {:?}", path.file_name().unwrap_or_default()));
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(_json) = &self.state.json_content {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.vertical(|ui| {
                        // Simple tree display for now
                        if let Some(root_data) = self.state.json_state.tree_nodes.iter().next() {
                            ui.label(format!("JSON Structure: {}", root_data.1.value_type));
                            
                            // Show some basic info
                            ui.separator();
                            
                            for data in self.state.json_state.tree_nodes.values() {  // Fixed: prefix with underscore
                                if !data.path.is_empty() {
                                    ui.horizontal(|ui| {
                                        ui.label(&data.path);
                                        ui.label(":");
                                        ui.label(&data.preview);
                                        ui.label(format!("({})", data.value_type));
                                    });
                                }
                            }
                        }
                    });
                });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("No JSON file loaded. Use File menu to load a JSON file.");
                });
            }
        });

        
        
        // Native file dialog
        if self.state.show_file_dialog {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("JSON", &["json"])
                .pick_file()
            {
                self.state.load_json_file(path);
            }
            self.state.show_file_dialog = false; // Reset the flag
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };

    eframe::run_native(
        "YAK JSON Viewer",
        native_options,
        Box::new(|_cc| Box::new(App::default())),
    )
}