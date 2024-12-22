mod veeteee;

use eframe::egui::{self, TextBuffer};
use std::sync::{Arc, Mutex};
use veeteee::TerminalPane;

fn main() {
    let mut pane = TerminalPane::new();
    pane.append_raw("\033[32;44mColored Text\033[0m\n");
    pane.append_raw("\033[10;20HMove Cursor\n");
    pane.append_raw("\033[2JClear Screen\n");

    println!("{}", pane.read());

    let native_options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "Remux",
        native_options,
        Box::new(|cc| Ok(Box::new(RemuxApp::new(cc)))),
    );
}

#[derive(Default)]
struct RemuxApp {
    pane: Arc<Mutex<TerminalPane>>, // Shared terminal pane
    input: String,
}

impl RemuxApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self::default();
        Self {
            pane: Arc::new(Mutex::new(TerminalPane::new())),
            input: String::new(),
        }
    }

    fn simulate_output(&self, raw_output: &str) {
        if let Ok(mut pane) = self.pane.lock() {
            pane.append_raw(raw_output);
        }
    }

    fn process_input(&mut self) {
        self.simulate_output(&format!("Command : {}\n", self.input));
        self.input.clear();
    }
}

impl eframe::App for RemuxApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Top Panel, title bar
        egui::TopBottomPanel::top("Title Bar").show(ctx, |ui| {
            ui.label(egui::RichText::new("Terminal Emulator - [tmux session]").strong());
        });

        // Center Panel, for main display
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                if let Ok(pane) = self.pane.lock() {
                    ui.label(pane.read());
                }
            });
        });

        // Status Bar + Command Input
        egui::TopBottomPanel::bottom("Status Bar").show(ctx, |ui| {
            // Thin bar styling for the Vim-like status line
            let rect = ui.available_rect_before_wrap();
            ui.painter().rect_filled(
                rect,
                0.0,                                 // No corner rounding
                egui::Color32::from_rgb(30, 30, 30), // Dark background
            );

            ui.horizontal(|ui| {
                ui.add_space(rect.width() - 120.0); // Push the line/column to the right

                if let Ok(pane) = self.pane.lock() {
                    ui.label(
                        egui::RichText::new(format!(
                            "Line: {}, Col: {}",
                            pane.cursor.0, pane.cursor.1
                        ))
                        .monospace()
                        .color(egui::Color32::LIGHT_GREEN),
                    );
                }
            });

            ui.separator(); // Line separating status bar and input

            // Input section
            let input_rect = ui.available_rect_before_wrap(); // Get the available space
            let input_height = 20.0;

            // Draw the custom background for the input box
            ui.painter().rect_filled(
                egui::Rect::from_min_size(
                    input_rect.min,
                    egui::vec2(input_rect.width(), input_height),
                ),
                0.0,
                egui::Color32::from_gray(20),
            );

            // Render the single-line input
            ui.allocate_ui_with_layout(
                egui::vec2(input_rect.width(), input_height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    let response = ui.add_sized(
                        egui::vec2(input_rect.width(), input_height),
                        egui::TextEdit::singleline(&mut self.input)
                            .hint_text(egui::RichText::new(": ").monospace())
                            .frame(false),
                    );

                    // Handle input processing when Enter is pressed
                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        self.process_input();
                    }
                },
            );

            ui.add_space(2.0);
            // ui.horizontal(|ui| {
            //     ui.label(egui::RichText::new(": ").monospace());
            //     let response = ui.text_edit_singleline(&mut self.input);
            //     if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            //         self.simulate_output(&format!("Command : {}\n", self.input));
            //         self.input.clear();
            //     }
            // });
        });

        // Request repaint (ensures smooth updates)
        ctx.request_repaint();
    }
}
