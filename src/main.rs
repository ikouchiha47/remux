mod renderer;

use eframe::egui;
use renderer::WezTerminalPane;
use std::error::Error as StdError;
use std::sync::{Arc, Mutex};
use termwiz::Error;

use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError>> {
    let pane = Arc::new(Mutex::new(WezTerminalPane::new(80, 24)?));

    // Test: Spawn a background task that appends ANSI text every 2 seconds
    {
        let pane_for_task = Arc::clone(&pane);
        tokio::spawn(async move {
            loop {
                {
                    let mut locked = pane_for_task.lock().unwrap();
                    // Red foreground on black background
                    locked.append_raw("\u{1b}[31;40mHello from Tokio!\u{1b}[0m\n");
                }
                sleep(Duration::from_secs(2)).await;
            }
        });
    }

    // Print the initial contents of the pane (if any)
    println!("{}", pane.lock().unwrap().read());

    // Now launch eframe. We pass our Arc<Mutex<WezTerminalPane>> into the app constructor.
    let native_options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "Remux with async Termwiz",
        native_options,
        Box::new(|cc| {
            let app = RemuxApp::new_with_pane(cc, pane.clone())
                .expect("should have successfully opened a pane");
            Ok(Box::new(app))
        }),
    );

    Ok(())
}

struct RemuxApp {
    pane: Arc<Mutex<WezTerminalPane>>, // Shared terminal pane
    input: String,
}

impl RemuxApp {
    /// A new constructor that re-uses an existing Arc<Mutex<WezTerminalPane>>
    fn new_with_pane(
        _cc: &eframe::CreationContext<'_>,
        pane: Arc<Mutex<WezTerminalPane>>,
    ) -> Result<Self, Error> {
        Ok(Self {
            pane,
            input: String::new(),
        })
    }

    fn simulate_output(&self, raw_output: &str) {
        if let Ok(mut pane) = self.pane.lock() {
            pane.append_raw(raw_output);
        }
    }

    fn process_input(&mut self) {
        self.simulate_output(&self.input);
        self.input.clear();
    }
}

impl eframe::App for RemuxApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top Panel, title bar
        egui::TopBottomPanel::top("Title Bar").show(ctx, |ui| {
            ui.label(egui::RichText::new("Terminal Emulator - [tmux session]").strong());
        });

        // Center Panel, for main display
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                // Lock the pane and read its text
                if let Ok(pane) = self.pane.lock() {
                    let lines = pane.read_colored_lines();

                    for line_segments in lines {
                        ui.horizontal_wrapped(|ui| {
                            for (text_chunk, fg_color) in line_segments {
                                // println!("{:?} {:?}", text_chunk, fg_color);
                                let text = egui::RichText::new(text_chunk).color(fg_color);
                                ui.label(text);
                            }
                        });
                    }
                }
            });
        });

        // Status Bar + Command Input
        egui::TopBottomPanel::bottom("Status Bar").show(ctx, |ui| {
            let rect = ui.available_rect_before_wrap();
            ui.painter()
                .rect_filled(rect, 0.0, egui::Color32::from_rgb(30, 30, 30));

            ui.horizontal(|ui| {
                ui.add_space(rect.width() - 120.0); // push to the right

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

            ui.separator(); // line separating status bar and input

            let input_rect = ui.available_rect_before_wrap();
            let input_height = 20.0;

            // custom background for the input
            ui.painter().rect_filled(
                egui::Rect::from_min_size(
                    input_rect.min,
                    egui::vec2(input_rect.width(), input_height),
                ),
                0.0,
                egui::Color32::from_gray(20),
            );

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

                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        self.process_input();
                    }
                },
            );
        });

        // Request repaint for continuous updates
        ctx.request_repaint();
    }
}
