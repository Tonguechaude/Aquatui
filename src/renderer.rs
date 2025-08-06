use crate::entities::{Bubble, Fish, Seaweed};
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

pub const KAME_HOUSE: &str = r#"
  ___________
 /   KAME    \
/    HOUSE    \
|  []  __  []  |
|_____|__|_____|
"#;

pub fn draw_seaweed(lines: &mut Vec<Line>, cols: u16, rows: u16, frame_count: usize) {
    let seaweed_height = 4;
    let sand_chars = [".", "·", "°", ".", "·", "°", "˙", ".", "·"];
    let wave_offset = (frame_count / 3) % sand_chars.len();

    for i in 0..seaweed_height {
        let y = rows - 1 - i;
        if y as usize >= lines.len() {
            continue;
        }

        let seaweed_line: String = if i == 0 {
            // Ligne de sable animée
            (0..cols as usize)
                .map(|x| {
                    let char_idx = (x + wave_offset) % sand_chars.len();
                    sand_chars[char_idx]
                })
                .collect()
        } else {
            let seaweed_chars = ["~", "≈", "∼", "≋", "~", "≈"];
            seaweed_chars
                .iter()
                .map(|&s| s)
                .cycle()
                .take(cols as usize)
                .collect()
        };

        for (dx, ch) in seaweed_line.chars().enumerate() {
            if dx >= cols as usize || dx >= lines[y as usize].spans.len() {
                break;
            }
            let color = if i == 0 {
                Color::Yellow // Sable
            } else {
                Color::Green // Algues
            };
            lines[y as usize].spans[dx] = Span::styled(ch.to_string(), Style::default().fg(color));
        }
    }
}

pub fn render_fish(lines: &mut Vec<Line>, fish: &Fish, inner_width: u16) {
    for (dy, line) in fish.body.iter().enumerate() {
        let y = fish.y as usize + dy;
        if y >= lines.len() {
            continue;
        }

        let chars: Vec<_> = line.chars().collect();
        let start_x = fish.x as i16;
        for (i, ch) in chars.into_iter().enumerate() {
            let screen_x = start_x + i as i16;
            if screen_x >= 0 && (screen_x as u16) < inner_width {
                let x_idx = screen_x as usize;
                if x_idx < lines[y].spans.len() {
                    lines[y].spans[x_idx] =
                        Span::styled(ch.to_string(), Style::default().fg(fish.color));
                }
            }
        }
    }
}

pub fn render_bubble(lines: &mut Vec<Line>, bubble: &Bubble, inner_width: u16) {
    let y_idx = bubble.y as usize;
    if y_idx < lines.len() {
        let x_idx = (bubble.x.min(inner_width - 1)) as usize;
        if x_idx < lines[y_idx].spans.len() {
            let symbol = bubble.current_symbol();
            lines[y_idx].spans[x_idx] = Span::styled(symbol, Style::default().fg(bubble.color));
        }
    }
}

pub fn render_seaweed(
    lines: &mut Vec<Line>,
    seaweed: &Seaweed,
    inner_width: u16,
    inner_height: u16,
) {
    let x = seaweed.x.min(inner_width - 1) as usize;
    let y = seaweed.y.min(inner_height - 1) as usize;
    let symbol = seaweed.current_symbol();
    let symbol_line = symbol.iter().rev().take(seaweed.height).collect::<Vec<_>>();
    for (dy, line) in symbol_line.iter().enumerate() {
        let draw_y = y.saturating_sub(dy);
        if draw_y >= lines.len() {
            continue;
        }
        for (dx, char) in line.chars().enumerate() {
            let draw_x = x + dx;
            if draw_x >= lines[draw_y].spans.len() || draw_y >= lines.len() {
                continue;
            }
            lines[draw_y].spans[draw_x] =
                Span::styled(char.to_string(), Style::default().fg(Color::Green));
        }
    }
}

pub fn render_kame_house(lines: &mut Vec<Line>, cols: u16, rows: u16) {
    let house_lines: Vec<&str> = KAME_HOUSE.lines().collect();
    let house_width = house_lines.iter().map(|line| line.len()).max().unwrap_or(0);
    let house_height = house_lines.len();
    let start_x = cols.saturating_sub(house_width as u16 + 2) as usize;
    let seaweed_height = 4;
    let start_y = rows.saturating_sub(house_height as u16 + seaweed_height) as usize;

    for (dy, line) in house_lines.iter().enumerate() {
        let y = start_y + dy;
        if y >= lines.len() {
            continue;
        }

        for (dx, ch) in line.chars().enumerate() {
            let x = start_x + dx;
            if x >= lines[y].spans.len() || y >= lines.len() {
                break;
            }

            let color = match ch {
                '[' | ']' => Color::Blue,                // Fenêtres
                '_' | '|' | '/' | '\\' => Color::Yellow, // Structure
                _ => Color::Magenta,                     // Texte
            };
            lines[y].spans[x] = Span::styled(ch.to_string(), Style::default().fg(color));
        }
    }
}
