use core::f32;
use eframe::glow::Context;
use nix::pty::{forkpty, ForkptyResult, Winsize};
use nix::unistd::execvp;
use nix::NixPath;
use std::ffi::CString;
use std::fs::File;
use std::io::{self, Write};
use std::os::fd::AsRawFd;
use std::os::fd::{FromRawFd, OwnedFd};

// mod sessionmanager;
mod temux;
use eframe::{egui, epaint};

use std::sync::{Arc, Mutex};
use temux::TemuxClient;

struct RemuxApp {
    input: Arc<Mutex<Vec<u8>>>,
    input_offset: usize,
    termbuff: Arc<Mutex<Vec<u8>>>,
    client: TemuxClient,
    stdin: Option<File>,
    stdout: Option<OwnedFd>,
    history: Vec<String>,
    history_index: usize,
}

impl RemuxApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, _width: usize, _height: usize) -> Self {
        let client = TemuxClient::new("session0");
        let (stdin, stdout) = spawn_pty("bash").expect("should not have failed to open");

        let flags = nix::fcntl::fcntl(stdout.as_raw_fd(), nix::fcntl::FcntlArg::F_GETFL).unwrap();
        let mut flags =
            nix::fcntl::OFlag::from_bits(flags & nix::fcntl::OFlag::O_ACCMODE.bits()).unwrap();

        flags.set(nix::fcntl::OFlag::O_NONBLOCK, true);
        nix::fcntl::fcntl(stdout.as_raw_fd(), nix::fcntl::FcntlArg::F_SETFL(flags)).unwrap();

        Self {
            input: Arc::new(Mutex::new(Vec::new())),
            input_offset: 0,
            termbuff: Arc::new(Mutex::new(Vec::new())),
            client,
            stdin: Some(stdin),
            stdout: Some(stdout),
            history: Vec::new(),
            history_index: 0,
        }
    }
}

impl Drop for RemuxApp {
    fn drop(&mut self) {
        if let Some(stdin) = self.stdin.take() {
            drop(stdin);
        }

        if let Some(stdout) = self.stdout.take() {
            drop(stdout);
        }

        // Other Clean up
    }
}

impl eframe::App for RemuxApp {
    fn on_exit(&mut self, _gl: Option<&Context>) {
        println!("App is exiting...");
        if let Some(mut stdin) = self.stdin.take() {
            stdin.flush().expect("to have flushed data");
            let _ = nix::unistd::close(stdin.as_raw_fd());
        }
        if let Some(stdout) = self.stdout.take() {
            let _ = nix::unistd::close(stdout.as_raw_fd());
        }
    }
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut buffer = vec![0; 1024];

        // Read from stdout
        if let Ok(n) = nix::unistd::read(self.stdout.as_mut().unwrap().as_raw_fd(), &mut buffer) {
            if n > 0 {
                {
                    let mut tb = self.termbuff.lock().unwrap();

                    tb.extend_from_slice(&buffer[..n]);
                    self.input_offset = tb.len();
                };
            }
        }

        egui::TopBottomPanel::top("Title Bar").show(ctx, |ui| {
            ui.label(egui::RichText::new("Terminal Emulator - [tmux session]").strong());
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .max_height(ui.available_height())
                .show(ui, |ui| {
                    let mut input_buffer = self.termbuff.lock().unwrap();
                    let input_display = String::from_utf8_lossy(&input_buffer);

                    ui.label(egui::RichText::new(input_display).monospace());

                    for event in ctx.input(|i| i.raw.events.clone()) {
                        if let egui::Event::Key { key, .. } = event {
                            if key == egui::Key::Enter {
                                let user_input = &input_buffer[self.input_offset..];

                                println!("{:?}", String::from_utf8_lossy(user_input));

                                if let Some(stdin) = &mut self.stdin {
                                    stdin.write_all(user_input).unwrap();
                                    stdin.write_all(b"\n").unwrap();
                                }

                                self.input_offset = input_buffer.len();
                            }
                        };

                        let egui::Event::Text(text) = event else {
                            continue;
                        };

                        input_buffer.extend(text.into_bytes());
                    }
                });

            ui.separator();

            // Separate panel for tmux commands
            egui::CollapsingHeader::new("Temux Commands").show(ui, |_ui| {
                let client = self.client.clone();

                tokio::spawn(async move {
                    if let Err(err) = client.create_session("new-session").await {
                        eprintln!("Failed to create session: {:?}", err);
                    }
                });
            });
        });

        ctx.request_repaint();
    }
}

fn spawn_pty(shell: &str) -> io::Result<(File, OwnedFd)> {
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
            let shell = CString::new(shell).unwrap();
            let args = vec![CString::new("--noprofile").unwrap()];

            execvp(&shell, &args).unwrap();
            unreachable!()
        }
    }
}

#[tokio::main]
async fn main() {
    let viewport = egui::ViewportBuilder {
        inner_size: Some(epaint::Vec2 { x: 800.0, y: 600.0 }),
        ..Default::default()
    };
    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    let _ = eframe::run_native(
        "Remux with async Termwiz",
        native_options,
        Box::new(|cc| {
            let app = RemuxApp::new(cc, 800, 600);
            Ok(Box::new(app))
        }),
    );
}
