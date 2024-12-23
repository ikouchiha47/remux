use std::error::Error;
use std::usize;

use termwiz::cell::{Cell, CellAttributes, Line};
use termwiz::color::{ColorAttribute, SrgbaTuple};
use termwiz::escape::parser::Parser;
use termwiz::escape::{Action, ControlCode, CSI};
use termwiz::surface::{Change, Position, Surface};

use eframe::egui::Color32;

/// A terminal pane that processes ANSI sequences

pub struct WezTerminalPane {
    surface: Surface,
    attrs: CellAttributes,
    pub cursor: (usize, usize), // Cursor position (row, col)
    width: usize,
    height: usize,
}

impl WezTerminalPane {
    pub fn new(width: usize, height: usize) -> Result<Self, Box<dyn Error>> {
        let surface = Surface::new(width, height);
        let attrs = CellAttributes::default();

        // let caps = Capabilities::new_from_env()?;
        // let terminal = SystemTerminal::new(caps)?;
        // let term = BufferedTerminal::new(terminal)?;

        Ok(Self {
            surface,
            attrs,
            cursor: (0, 0),
            width,
            height,
        })
    }

    /// Append raw ANSI sequences to the terminal and update the surface
    pub fn append_raw(&mut self, raw_input: &str) -> Result<(), Box<dyn Error>> {
        let mut parser = Parser::new();
        let mut ignoring = false;
        let mut buf = String::new(); // accumulate the strings/chars until Ctrl Sequence is
                                     // encountered

        parser.parse(raw_input.as_bytes(), |action| {
            if ignoring {
                return;
            }
            match action {
                Action::Print(c) => {
                    buf.push(c);
                }
                Action::PrintString(s) => buf.push_str(&s),
                Action::CSI(csi) => {
                    // Handle CSI sequences (e.g., color, cursor movement)
                    self.handle_csi(csi);
                }
                Action::Control(c) => match c {
                    ControlCode::CarriageReturn | ControlCode::LineFeed => {
                        ignoring = true;
                    }
                    _ => {}
                },
                _ => {
                    // Handle other terminal actions as needed
                }
            }
        });

        Ok(())
    }

    fn flush_buf(buf: &mut String, cells: &mut Vec<Cell>, attrs: &CellAttributes) {}

    fn handle_csi(&mut self, csi: CSI) {}

    /// Read the terminal buffer content as a string
    pub fn read(&self) -> String {
        self.surface.screen_chars_to_string()
    }

    /// Move the cursor to a specific position
    pub fn move_cursor(&mut self, row: usize, col: usize) {
        let clamped_row = row.min(self.height.saturating_sub(1));
        let clamped_col = col.min(self.width.saturating_sub(1));
        self.cursor = (clamped_row, clamped_col);

        // now apply the change
        self.surface.add_change(Change::CursorPosition {
            x: Position::Absolute(self.cursor.1 as usize),
            y: Position::Absolute(self.cursor.0 as usize),
        });
    }

    /// Clear the terminal screen
    pub fn clear(&mut self) {
        self.surface
            .add_change(Change::ClearScreen(ColorAttribute::Default));
    }

    pub fn read_colored_lines(&self) -> Vec<Vec<(String, Color32)>> {
        let mut out_lines = Vec::new();

        let screen_lines = self.surface.screen_lines(); // returns Vec<Cow<'_, Line>>
        for line_cow in screen_lines {
            let mut line = line_cow.into_owned(); // convert Cow<Line> → &Line
            let mut segments = Vec::new();

            // We'll accumulate text for cells with the same color
            let mut current_color = Color32::WHITE;
            let mut current_text = String::new();

            for cell in line.cells_mut() {
                let fg = color_to_egui(cell.attrs().foreground());
                let ch = cell.str(); // the character(s) in this cell

                if ch != " " {
                    print!("{:?} {:?}", ch, cell);
                }

                if fg == current_color {
                    current_text.push_str(ch);
                } else {
                    if !current_text.is_empty() {
                        segments.push((current_text, current_color));
                    }
                    current_color = fg;
                    current_text = ch.to_string();
                }
            }

            // End of line => push leftover text if any
            if !current_text.is_empty() {
                segments.push((current_text, current_color));
            }

            out_lines.push(segments);
            println!("");
        }

        out_lines
    }
}

/// Convert a Termwiz foreground color to an egui::Color32
fn color_to_egui(attr: ColorAttribute) -> Color32 {
    match attr {
        // The user didn't specify a color => fallback to white
        ColorAttribute::Default => Color32::WHITE,

        // The user specified an index in the color palette (e.g. the standard 256 or extended).
        ColorAttribute::PaletteIndex(idx) => {
            // For simplicity, let's do a handful of indices:
            match idx {
                0 => Color32::BLACK,
                1 => Color32::from_rgb(128, 0, 0),   // maroon
                2 => Color32::from_rgb(0, 128, 0),   // green
                3 => Color32::from_rgb(128, 128, 0), // olive
                4 => Color32::from_rgb(0, 0, 128),   // navy
                5 => Color32::from_rgb(128, 0, 128), // purple
                6 => Color32::from_rgb(0, 128, 128), // teal
                7 => Color32::LIGHT_GRAY,
                8 => Color32::GRAY,
                9 => Color32::RED,
                10 => Color32::from_rgb(0, 255, 0), // lime
                11 => Color32::YELLOW,
                12 => Color32::BLUE,
                13 => Color32::from_rgb(255, 0, 255), // fuchsia
                14 => Color32::from_rgb(0, 255, 255), // aqua
                15 => Color32::WHITE,
                _ => {
                    // If you want to handle indexes beyond 15, either build a big table
                    // or just fallback to white for now:
                    Color32::WHITE
                }
            }
        }

        ColorAttribute::TrueColorWithDefaultFallback(srgba) => srgba_to_egui(srgba),

        ColorAttribute::TrueColorWithPaletteFallback(srgba, _fallback_idx) => {
            // For a GUI context (like Egui), we *do* support sRGB,
            // so let's just use the srgba. If you prefer to actually
            // respect the fallback, you'd detect if "true color is not available"
            // and then call the palette logic above for fallback_idx.
            srgba_to_egui(srgba)
        }
    }
}

fn srgba_to_egui(srgba: SrgbaTuple) -> Color32 {
    // SrgbaTuple is typically (red, green, blue, alpha) in floating-point 0..1 range.
    // Egui uses 0..255 u8 range, so convert:
    Color32::from_rgba_unmultiplied(
        (srgba.0 * 255.0) as u8, // red
        (srgba.1 * 255.0) as u8, // blue
        (srgba.2 * 255.0) as u8, // green
        (srgba.3 * 255.0) as u8, // alpha
    )
}
