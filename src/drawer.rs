use eframe::egui;

use eframe::egui::{Color32, FontId, TextEdit, Vec2};
use termwiz::color::ColorAttribute;

use termwiz::surface::{Change, Surface};

pub fn draw_surface(surface: &Surface, ui: &mut egui::Ui) {
    println!("drawing surface");

    let (_width, _height) = surface.dimensions();
    let cell_size = Vec2::new(10.0, 18.0);

    ui.vertical(|ui| {
        for (_y, line) in surface.screen_lines().iter().enumerate() {
            ui.horizontal(|ui| {
                for (_x, cell) in line.visible_cells().enumerate() {
                    let text = cell.str();
                    if text != " " {
                        println!("printing {:?}", text);
                    }
                    let color = match cell.attrs().foreground() {
                        ColorAttribute::TrueColorWithPaletteFallback(rgb, _) => Color32::from_rgb(
                            (rgb.0 * 255.0) as u8,
                            (rgb.1 * 255.0) as u8,
                            (rgb.2 * 255.0) as u8,
                        ),
                        ColorAttribute::TrueColorWithDefaultFallback(rgb) => Color32::from_rgb(
                            (rgb.0 * 255.0) as u8,
                            (rgb.1 * 255.0) as u8,
                            (rgb.2 * 255.0) as u8,
                        ),
                        ColorAttribute::PaletteIndex(_) => Color32::GRAY,
                        ColorAttribute::Default => Color32::WHITE,
                    };

                    ui.add_sized(
                        cell_size,
                        egui::Label::new(
                            egui::RichText::new(text)
                                .color(color)
                                .font(FontId::monospace(12.0)),
                        ),
                    );
                }
            });
        }
    });
}
