use rand::{Rng, rngs::ThreadRng};

use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};

pub struct Seaweed {
    pub x: u16,
    pub y: u16,
    pub frame: usize,
    pub z_index: u8,
    pub height: usize,
}

impl Seaweed {
    const FRAMES: [&'static str; 2] = [
        "
        (
        )
        (
        )
        (",
        "
        )
        (
        )
        (
        )",
    ];

    pub fn new(x: u16, y: u16, z_index: u8, rng: &mut ThreadRng) -> Self {
        Self {
            x,
            y,
            frame: 0,
            z_index,
            height: rng.random_range(2..=6),
        }
    }

    pub fn update(&mut self) {
        self.frame = (self.frame + 1) % Self::FRAMES.len();
    }

    pub fn current_symbol(&self) -> Vec<&str> {
        Self::FRAMES[self.frame].lines().map(str::trim).collect()
    }
}

pub fn draw_seaweed(lines: &mut Vec<Line>, cols: u16, rows: u16) {
    let seaweed_height = 4;
    let seaweed_chars = ["~", "≡", "≡", "~", "≡"];
    for i in 0..seaweed_height {
        let y = rows - 1 - i;
        if y as usize >= lines.len() {
            continue;
        }

        let seaweed_line = seaweed_chars
            .iter()
            .map(|&s| s) // Dé-référencement ici
            .cycle()
            .take(cols as usize)
            .collect::<String>();

        let mut content = lines[y as usize].clone();
        for (dx, ch) in seaweed_line.chars().enumerate() {
            if dx >= cols as usize {
                break;
            }
            if dx < content.spans.len() {
                content.spans[dx] = Span::styled(ch.to_string(), Style::default().fg(Color::Green));
            }
        }

        lines[y as usize] = content;
    }
}
