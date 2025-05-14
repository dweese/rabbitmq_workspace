// egui-components/src/border_layout.rs
use eframe::egui::{self, Ui, Widget, Response, Color32};

type WidgetFn = Box<dyn FnOnce(&mut Ui)>;

pub struct BorderLayout {
    #[allow(dead_code)]
    id: String,
    north: Option<WidgetFn>,
    south: Option<WidgetFn>,
    east: Option<WidgetFn>,
    west: Option<WidgetFn>,
    center: Option<WidgetFn>,
    north_height: f32,
    south_height: f32,
    east_width: f32,
    west_width: f32,
    north_color: Option<Color32>,
    south_color: Option<Color32>,
    east_color: Option<Color32>,
    west_color: Option<Color32>,
    center_color: Option<Color32>,
}

impl BorderLayout {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            north: None,
            south: None,
            east: None,
            west: None,
            center: None,
            north_height: 40.0,  // Default heights/widths
            south_height: 40.0,
            east_width: 100.0,
            west_width: 100.0,
            north_color: None,
            south_color: None,
            east_color: None,
            west_color: None,
            center_color: None,
        }
    }

    pub fn north(mut self, add_contents: impl FnOnce(&mut Ui) + 'static) -> Self {
        self.north = Some(Box::new(add_contents));
        self
    }

    pub fn south(mut self, add_contents: impl FnOnce(&mut Ui) + 'static) -> Self {
        self.south = Some(Box::new(add_contents));
        self
    }

    pub fn east(mut self, add_contents: impl FnOnce(&mut Ui) + 'static) -> Self {
        self.east = Some(Box::new(add_contents));
        self
    }

    pub fn west(mut self, add_contents: impl FnOnce(&mut Ui) + 'static) -> Self {
        self.west = Some(Box::new(add_contents));
        self
    }

    pub fn center(mut self, add_contents: impl FnOnce(&mut Ui) + 'static) -> Self {
        self.center = Some(Box::new(add_contents));
        self
    }

    // Optional methods to configure sizes
    pub fn north_height(mut self, height: f32) -> Self {
        self.north_height = height;
        self
    }

    pub fn south_height(mut self, height: f32) -> Self {
        self.south_height = height;
        self
    }

    pub fn east_width(mut self, width: f32) -> Self {
        self.east_width = width;
        self
    }

    pub fn west_width(mut self, width: f32) -> Self {
        self.west_width = width;
        self
    }

    // Methods to set colors for each region
    pub fn north_color(mut self, color: Color32) -> Self {
        self.north_color = Some(color);
        self
    }

    pub fn south_color(mut self, color: Color32) -> Self {
        self.south_color = Some(color);
        self
    }

    pub fn east_color(mut self, color: Color32) -> Self {
        self.east_color = Some(color);
        self
    }

    pub fn west_color(mut self, color: Color32) -> Self {
        self.west_color = Some(color);
        self
    }

    pub fn center_color(mut self, color: Color32) -> Self {
        self.center_color = Some(color);
        self
    }
}

impl Widget for BorderLayout {
    fn ui(self, ui: &mut Ui) -> Response {
        let available_rect = ui.available_rect_before_wrap();

        // Calculate regions
        let mut center_rect = available_rect;

        // North region
        let north_rect = if self.north.is_some() {
            let height = self.north_height;
            let rect = egui::Rect::from_min_size(
                available_rect.min,
                egui::Vec2::new(available_rect.width(), height),
            );
            center_rect.min.y += height;
            Some(rect)
        } else {
            None
        };

        // South region
        let south_rect = if self.south.is_some() {
            let height = self.south_height;
            let rect = egui::Rect::from_min_size(
                egui::Pos2::new(available_rect.min.x, available_rect.max.y - height),
                egui::Vec2::new(available_rect.width(), height),
            );
            center_rect.max.y -= height;
            Some(rect)
        } else {
            None
        };

        // West region
        let west_rect = if self.west.is_some() {
            let width = self.west_width;
            let rect = egui::Rect::from_min_size(
                egui::Pos2::new(center_rect.min.x, center_rect.min.y),
                egui::Vec2::new(width, center_rect.height()),
            );
            center_rect.min.x += width;
            Some(rect)
        } else {
            None
        };

        // East region
        let east_rect = if self.east.is_some() {
            let width = self.east_width;
            let rect = egui::Rect::from_min_size(
                egui::Pos2::new(center_rect.max.x - width, center_rect.min.y),
                egui::Vec2::new(width, center_rect.height()),
            );
            center_rect.max.x -= width;
            Some(rect)
        } else {
            None
        };

        // Draw each region

        // Using temp variables to handle Option consumption
        let north = self.north;
        let south = self.south;
        let west = self.west;
        let east = self.east;
        let center = self.center;
        let north_color = self.north_color;
        let south_color = self.south_color;
        let west_color = self.west_color;
        let east_color = self.east_color;
        let center_color = self.center_color;

        // North region
        if let Some(rect) = north_rect {
            if let Some(function) = north {
                let _ = ui.allocate_ui_at_rect(rect, |ui| {
                    if let Some(color) = north_color {
                        let painter = ui.painter();
                        painter.rect_filled(rect, 0.0, color);
                    }
                    function(ui);
                });
            }
        }

        // South region
        if let Some(rect) = south_rect {
            if let Some(function) = south {
                let _ = ui.allocate_ui_at_rect(rect, |ui| {
                    if let Some(color) = south_color {
                        let painter = ui.painter();
                        painter.rect_filled(rect, 0.0, color);
                    }
                    function(ui);
                });
            }
        }

        // West region
        if let Some(rect) = west_rect {
            if let Some(function) = west {
                let _ = ui.allocate_ui_at_rect(rect, |ui| {
                    if let Some(color) = west_color {
                        let painter = ui.painter();
                        painter.rect_filled(rect, 0.0, color);
                    }
                    function(ui);
                });
            }
        }

        // East region
        if let Some(rect) = east_rect {
            if let Some(function) = east {
                let _ = ui.allocate_ui_at_rect(rect, |ui| {
                    if let Some(color) = east_color {
                        let painter = ui.painter();
                        painter.rect_filled(rect, 0.0, color);
                    }
                    function(ui);
                });
            }
        }

        // Center region (always available)
        let center_response = if let Some(function) = center {
            ui.allocate_ui_at_rect(center_rect, |ui| {
                if let Some(color) = center_color {
                    let painter = ui.painter();
                    painter.rect_filled(center_rect, 0.0, color);
                }
                function(ui);
            }).response
        } else {
            ui.allocate_rect(center_rect, egui::Sense::hover())
        };

        // Consume available space and return
        ui.allocate_rect(available_rect, egui::Sense::hover());
        center_response
    }
}