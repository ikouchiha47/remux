mod drawer;
use eframe::egui;
use nix::pty::{forkpty, ForkptyResult, Winsize};
use nix::unistd::execvp;
use std::ffi::CString;
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::os::fd::AsRawFd;
use std::os::fd::{FromRawFd, OwnedFd};

struct RemuxApp {
    stdin: File,
    stdout: OwnedFd,
    output: String,
    input: String,
}

// man fnctl
// file descriptor flag: changes the flags of the file descriptor iteself
// descriptor status flag: changes the flags of the underlying file`

impl RemuxApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, _width: usize, _height: usize) -> Self {
        // let mut surface = Surface::new(width, height);
        // surface.add_change(Change::ClearScreen(ColorAttribute::Default));
        //
        let (stdin, stdout) = spawn_pty().expect("Failed to start tmux session");

        let flags = nix::fcntl::fcntl(stdout.as_raw_fd(), nix::fcntl::FcntlArg::F_GETFL).unwrap();
        let mut flags = nix::fcntl::OFlag::from_bits(flags).unwrap();

        flags.set(nix::fcntl::OFlag::O_NONBLOCK, true);
        nix::fcntl::fcntl(stdout.as_raw_fd(), nix::fcntl::FcntlArg::F_SETFL(flags)).unwrap();

        Self {
            stdin,
            stdout,
            output: String::new(),
            input: String::new(),
        }
    }

    // fn process_input(&mut self) {
    //     if !self.input.is_empty() {
    //         self.history.push(self.input.clone());
    //         self.surface.add_change(Change::Text(self.input.clone()));
    //         self.input.clear();
    //     }
    // }

    fn process_tmux_output(&mut self) {
        let mut buffer = [0u8; 1024];
        if let Ok(bytes_read) = nix::unistd::read(self.stdout.as_raw_fd(), &mut buffer) {
            let raw_output = &String::from_utf8_lossy(&buffer[..bytes_read]);
            self.output.push_str(raw_output);

            println!("more raw_output {:?}", raw_output);
        }
    }

    fn send_tmux_command(&mut self, command: &str) {
        if let Err(e) = self.stdin.write_all(format!("{}\n", command).as_bytes()) {
            eprintln!("Failed to send command to tmux: {}", e);
        }
    }
}

fn spawn_pty() -> io::Result<(File, OwnedFd)> {
    let winsize = Winsize {
        ws_row: 24,
        ws_col: 80,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };

    let fork_result = unsafe { forkpty(Some(&winsize), None).unwrap() };

    match fork_result {
        ForkptyResult::Parent { master, .. } => {
            let reader = unsafe { std::fs::File::from_raw_fd(master.as_raw_fd()) };
            Ok((reader, master))
        }
        ForkptyResult::Child => {
            let tmux = CString::new("tmux").unwrap();
            let args = vec![CString::new("-C").unwrap()];
            execvp(&tmux, &args).unwrap();
            unreachable!()
        }
    }
}

impl eframe::App for RemuxApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let previous_output_len = self.output.len();

        self.process_tmux_output();

        if self.output.len() > previous_output_len {
            println!(
                "needs repaint {:?} {:?}",
                self.output.len(),
                previous_output_len
            );
            // println!("output {:?}", self.output);
        }

        egui::TopBottomPanel::top("Title Bar").show(ctx, |ui| {
            ui.label(egui::RichText::new("Terminal Emulator - [tmux session]").strong());
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.text_edit_singleline(&mut self.input).lost_focus()
                    && ctx.input(|i| i.key_pressed(egui::Key::Enter))
                {
                    let command = self.input.clone();
                    self.input.clear();
                    self.send_tmux_command(&command);
                }

                if ui.button("New Session").clicked() {
                    self.send_tmux_command("new-session");
                }
            });
            // Display tmux output
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .max_height(ui.available_height())
                .show(ui, |ui| {
                    ui.label(&self.output);
                });

            ui.separator();
        });
    }
}

fn main() {
    let native_options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "Remux with async Termwiz",
        native_options,
        Box::new(|cc| {
            let app = RemuxApp::new(cc, 800, 600);
            Ok(Box::new(app))
        }),
    );
}

// use drawer::draw_surface;
// use termwiz::color::ColorAttribute;
// use termwiz::surface::{Change, Surface};
// use eframe::egui::{Color32, TextEdit, Vec2};

// egui::CentralPanel::default().show(ctx, |ui| {
//     egui::ScrollArea::vertical().show(ui, |ui| {
//         ui.text_edit_multiline(&mut self.output);
//     });
//
//     if ui.text_edit_singleline(&mut self.input).lost_focus()
//         && ctx.input(|i| i.key_pressed(egui::Key::Enter))
//     {
//         let command = self.input.clone();
//         println!("changed {:?}", command);
//         self.input.clear();
//         self.stdin
//             .write_all(format!("{}\n", command).as_bytes())
//             .expect("Failed to send command to tmux");
//     }
// });

// egui::TopBottomPanel::bottom("Input Panel").show(ctx, |ui| {
//     let rect = ui.available_rect_before_wrap();
//     ui.painter()
//         .rect_filled(rect, 0.0, Color32::from_rgb(30, 30, 30));
//
//     ui.set_min_height(40.0);
//     ui.separator();

// let input_rect = ui.available_rect_before_wrap();
// let input_height = 20.0;

// ui.allocate_ui_with_layout(
//     Vec2::new(input_rect.width(), input_height),
//     egui::Layout::top_down(egui::Align::Min),
//     |ui| {
//         let response = ui.add_sized(
//             Vec2::new(input_rect.width(), input_height),
//             TextEdit::singleline(&mut self.input)
//                 .hint_text(egui::RichText::new(": ").monospace())
//                 .text_color(Color32::GREEN)
//                 .frame(true),
//         );
//
//         if response.lost_focus() && ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
//             self.process_input();
//         }
//     },
// );
//
// });

// ctx.request_repaint();
