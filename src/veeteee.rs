use vte::{Params, Parser, Perform};

/// A terminal pane that processes ANSI sequences
#[derive(Default)]
pub struct TerminalPane {
    buffer: String,             // Holds the displayed content
    pub cursor: (usize, usize), // Cursor position (row, col)
}

impl TerminalPane {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            cursor: (0, 0),
        }
    }

    /// Appends raw input (from `%output`) to the pane
    pub fn append_raw(&mut self, raw_input: &str) {
        let mut parser = Parser::new();
        for byte in raw_input.as_bytes() {
            parser.advance(self, *byte);
        }
    }

    pub fn read(&self) -> &String {
        return &self.buffer;
    }
}

/// Implement the `Perform` trait to handle ANSI escape sequences
/// https://docs.rs/vte/latest/vte/trait.Perform.html
impl Perform for TerminalPane {
    fn print(&mut self, c: char) {
        self.buffer.push(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => self.buffer.push('\n'),
            b'\r' => self.buffer.push('\r'),
            _ => {}
        }
    }

    fn csi_dispatch(
        &mut self,
        params: &Params,
        _intermediates: &[u8],
        _ignore: bool,
        action: char,
    ) {
        let mut param_iter = params.iter().flat_map(|param| param);

        match action {
            'm' => {
                // CSI SGR (Set Graphics Rendition), e.g., \033[32m
                while let Some(param) = param_iter.next() {
                    match *param {
                        0 => self.buffer.push_str("[reset]"),
                        30..=37 => self.buffer.push_str(&format!("[fg_color:{}]", param - 30)),
                        40..=47 => self.buffer.push_str(&format!("[bg_color:{}]", param - 40)),
                        _ => self.buffer.push_str("[unknown SGR param]"),
                    }
                }
            }
            'H' => {
                // CSI CUP (Cursor Position), e.g., \033[<row>;<col>H
                let row = param_iter.next().copied().unwrap_or(1); // Default to 1
                let col = param_iter.next().copied().unwrap_or(1); // Default to 1
                self.cursor = (row as usize, col as usize);
                self.buffer
                    .push_str(&format!("[cursor moved to: ({}, {})]", row, col));
            }
            'J' => {
                // CSI ED (Erase Display), e.g., \033[2J
                match param_iter.next().copied().unwrap_or(0) {
                    0 => self.buffer.push_str("[clear to end of screen]"),
                    1 => self.buffer.push_str("[clear to beginning of screen]"),
                    2 => {
                        self.buffer.clear();
                        self.buffer.push_str("[entire screen cleared]");
                    }
                    _ => self.buffer.push_str("[unknown ED param]"),
                }
            }
            _ => {
                // Log unhandled actions for debugging
                self.buffer
                    .push_str(&format!("[unhandled action: {}]", action));
            }
        }
    }
}
