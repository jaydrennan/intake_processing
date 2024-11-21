use eframe::egui;

pub struct ImageViewer {
    pub zoom: f32,
    pub offset: egui::Vec2,
    pub dragging: bool,
    pub last_cursor_pos: Option<egui::Pos2>,
}

impl Default for ImageViewer {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            offset: egui::Vec2::ZERO,
            dragging: false,
            last_cursor_pos: None,
        }
    }
}

impl ImageViewer {
    pub fn ui(&mut self, ui: &mut egui::Ui, texture: &egui::TextureHandle, original_size: [f32; 2]) {
        // Zoom controls
        ui.horizontal(|ui| {
            if ui.button("🔍 Zoom In").clicked() {
                self.zoom *= 1.2;
            }
            if ui.button("🔍 Zoom Out").clicked() {
                self.zoom /= 1.2;
            }
            if ui.button("↺ Reset").clicked() {
                *self = Default::default();
            }
            ui.label(format!("Zoom: {:.1}x", self.zoom));
        });

        let available_size = ui.available_size();
        let (response, painter) = ui.allocate_painter(
            available_size,
            egui::Sense::drag(),
        );

        // Handle dragging
        if response.dragged() {
            let delta = response.drag_delta();
            self.offset += delta;
            self.dragging = true;
        } else {
            self.dragging = false;
        }
        self.last_cursor_pos = response.hover_pos();

        // Handle scroll for zoom
        if response.hovered() {
            let scroll_delta = ui.input(|i| i.scroll_delta.y);
            if scroll_delta != 0.0 {
                self.zoom *= if scroll_delta > 0.0 { 1.1 } else { 0.9 };
                self.zoom = self.zoom.clamp(0.1, 5.0);
            }
        }

        // Draw image
        let size = [
            original_size[0] * self.zoom,
            original_size[1] * self.zoom,
        ];
        
        let rect = egui::Rect::from_min_size(
            (response.rect.min.to_vec2() + self.offset).to_pos2(),
            egui::vec2(size[0], size[1]),
        );
        
        let uv_rect = egui::Rect::from_min_max(
            egui::pos2(0.0, 0.0),
            egui::pos2(1.0, 1.0),
        );
        
        painter.image(
            texture.id(),
            rect,
            uv_rect,
            egui::Color32::WHITE,
        );
    }
} 