use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

pub struct TemuxClient {
    pub session_name: String,
    pub output: Arc<Mutex<String>>,
    command_sender: mpsc::Sender<String>,
}

impl Clone for TemuxClient {
    fn clone(&self) -> Self {
        Self {
            session_name: self.session_name.clone(),
            output: Arc::clone(&self.output),
            command_sender: self.command_sender.clone(),
        }
    }
}

impl TemuxClient {
    pub fn new(session_name: &str) -> Self {
        let output = Arc::new(Mutex::new(String::new()));
        let (command_sender, command_receiver) = mpsc::channel(32);

        // let command_receiver = Arc::new(Mutex::new(command_receiver));

        let client = Self {
            session_name: session_name.to_string(),
            output: Arc::clone(&output),
            command_sender,
        };

        let output_clone = Arc::clone(&output);
        let session_name = session_name.to_string();

        thread::spawn(move || {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("Failed to create tokio runtime")
                .block_on(async {
                    run_tmux_task(output_clone, command_receiver, session_name).await;
                });
        });

        client
    }

    pub async fn create_session(&self, session_name: &str) -> Result<(), String> {
        self.send_command(&format!("new-session -d -s {}", session_name))
            .await
    }

    pub async fn create_window(&self, window_name: &str) -> Result<(), String> {
        self.send_command(&format!("new-window -n {}", window_name))
            .await
    }

    pub async fn create_split(&self, split_type: SplitType) -> Result<(), String> {
        let split_cmd = match split_type {
            SplitType::Horizontal => "split-window -h",
            SplitType::Vertical => "split-window -v",
        };
        self.send_command(split_cmd).await
    }

    pub async fn kill_session(&self, session_name: &str) -> Result<(), String> {
        self.send_command(&format!("kill-session -t {}", session_name))
            .await
    }

    pub async fn detach_client(&self) -> Result<(), String> {
        self.send_command("detach-client").await
    }

    pub async fn save_session(&self, path: &str) -> Result<(), String> {
        // First capture the session info
        let capture_cmd = format!(
            "list-windows -F \"#{{window_index}} #{{window_name}} #{{window_layout}}\" -t {}",
            self.session_name
        );
        self.send_command(&capture_cmd).await?;

        // Wait a moment for the output to be captured
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Get the captured output
        let output = self.output.lock().map_err(|e| e.to_string())?;

        // Save to file
        tokio::fs::write(path, output.as_bytes())
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub async fn load_session(&self, path: &str) -> Result<(), String> {
        let config = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| e.to_string())?;

        // Parse and recreate the session
        for line in config.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let _window_index = parts[0];
                let window_name = parts[1];
                let window_layout = parts[2];

                // Recreate window
                self.create_window(window_name).await?;
                // Set layout
                self.send_command(&format!("select-layout {}", window_layout))
                    .await?;
            }
        }

        Ok(())
    }

    pub async fn select_window(&self, window_index: usize) -> Result<(), String> {
        self.send_command(&format!("select-window -t {}", window_index))
            .await
    }

    pub async fn rename_window(&self, window_index: usize, new_name: &str) -> Result<(), String> {
        self.send_command(&format!("rename-window -t {} {}", window_index, new_name))
            .await
    }

    pub async fn list_windows(&self) -> Result<(), String> {
        self.send_command("list-windows").await
    }

    pub async fn list_sessions(&self) -> Result<(), String> {
        self.send_command("list-sessions").await
    }

    pub async fn send_keys(&self, target: &str, keys: &str) -> Result<(), String> {
        self.send_command(&format!("send-keys -t {} {}", target, keys))
            .await
    }

    pub async fn set_option(&self, option: &str, value: &str) -> Result<(), String> {
        self.send_command(&format!("set-option {} {}", option, value))
            .await
    }

    pub async fn send_command(&self, command: &str) -> Result<(), String> {
        self.command_sender
            .send(command.to_string())
            .await
            .map_err(|e| format!("Failed to send command: {}", e))
    }
}

pub enum SplitType {
    Horizontal,
    Vertical,
}

/// Runs the tmux client task in a new thread
pub async fn run_tmux_task(
    output: Arc<Mutex<String>>,
    mut command_receiver: mpsc::Receiver<String>,
    session_name: String,
) {
    // Start tmux in control mode with a unique session
    let mut child = Command::new("tmux")
        .arg("-C") // Control mode
        .arg("new-session")
        .arg("-A") // Attach to existing session if it exists
        .arg("-s")
        .arg(&session_name)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start tmux");

    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    let stdout = child.stdout.take().expect("Failed to open stdout");

    let mut reader = BufReader::new(stdout);
    let mut buffer = String::new();

    tokio::spawn(async move {
        while let Some(command) = command_receiver.recv().await {
            if let Err(e) = stdin.write_all(format!("{}\n", command).as_bytes()).await {
                eprintln!("Failed to send command to tmux: {}", e);
            }
        }
    });

    // Main loop to read tmux output
    loop {
        match reader.read_line(&mut buffer).await {
            Ok(0) => break, // EOF reached
            Ok(_) => {
                let mut output_lock = output.lock().unwrap();
                output_lock.push_str(&buffer);
                buffer.clear();
            }
            Err(e) => eprintln!("Error reading tmux output: {}", e),
        }
    }
}
