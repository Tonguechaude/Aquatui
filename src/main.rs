use crossterm::{
    event::{Event, KeyCode, poll, read},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use rand::{
    Rng,
    distr::{Distribution, weighted::WeightedIndex},
    prelude::IndexedRandom,
    rngs::ThreadRng,
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use std::{io, thread, time::Duration, time::Instant};

const NUM_FISH: usize = 15;
const NUM_BUBBLES: usize = 40;
const NUM_SEAWEEDS: usize = 30;
const FRAME_DURATION_MS: u64 = 100;
const SLEEP_DURATION_MS: u64 = 5;

#[derive(Clone)]
enum Direction {
    Left,
    Right,
}

#[derive(Clone)]
struct Fish {
    x: u16,
    y: u16,
    body: Vec<String>,
    color: Color,
    speed: u16,
    direction: Direction,
}

impl PartialEq for Fish {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

struct Bubble {
    x: u16,
    y: u16,
    frame: usize, // pour alterner entre ".", "o", "0"
    z_index: u8,
}

struct Seaweed {
    x: u16,
    y: u16,
    frame: usize,
    z_index: u8,
    height: usize,
}

const KAME_HOUSE: &str = r#"
  ____KAME____
 /   HOUSE    \
/______________\
| []   __   [] |
|____|_____|___|
"#;

impl Fish {
    fn new(max_x: u16, max_y: u16, rng: &mut ThreadRng) -> Self {
        let direction = if rng.random_bool(0.5) {
            Direction::Right
        } else {
            Direction::Left
        };
        let x = match direction {
            Direction::Right => 0,
            Direction::Left => max_x,
        };
        let raw_bodies = match direction {
            Direction::Right => vec![
                ("><((°>".to_string(), 3),
                ("><>".to_string(), 3),
                (">º)))>".to_string(), 3),
                (
                    r#"
  \\
 / \\
>=_('>
 \\_/
   /"#
                    .to_string(),
                    1,
                ),
                (
                    r#"
        ,--,_
__    _\\.---'-.
\\ '.-"     // o\\
/_.'-.-_   \\\\  /
        `"--(/"#
                        .to_string(),
                    1,
                ),
            ],
            Direction::Left => vec![
                ("<°))><".to_string(), 3),
                ("<><".to_string(), 3),
                ("<(((º<".to_string(), 3),
                (
                    r#"
  /
 / \\
<')_=<
 \\_/
  \\"#
                    .to_string(),
                    2,
                ),
                (
                    r#"
    _,--,
 .-'---./___    __
/o \\\\     "-.' /
\\  //    _.-'._\\
  `"\\)--"#
                        .to_string(),
                    1,
                ),
            ],
        };

        let speed = rng.random_range(1..3);
        let weights: Vec<u32> = raw_bodies.iter().map(|(_, weight)| *weight).collect();
        let dist = WeightedIndex::new(&weights).unwrap();
        let choice = dist.sample(rng);
        let (raw_body, _) = &raw_bodies[choice];
        let body = raw_body.lines().map(|l| l.trim_end().to_string()).collect();
        let color = *[
            Color::Cyan,
            Color::LightMagenta,
            Color::Yellow,
            Color::LightBlue,
        ]
        .choose(rng)
        .unwrap();

        Self {
            x,
            y: rng.random_range(2..max_y.saturating_sub(4)),
            body,
            color,
            speed,
            direction,
        }
    }

    fn update(&mut self, max_x: u16) -> bool {
        match self.direction {
            Direction::Right => {
                self.x += self.speed;
                self.x < max_x + 10
            }
            Direction::Left => {
                if self.x <= self.speed {
                    false
                } else {
                    self.x -= self.speed;
                    true
                }
            }
        }
    }
}

impl Bubble {
    const FRAMES: [&'static str; 3] = [".", "o", "0"];

    fn new(x: u16, y: u16, z_index: u8) -> Self {
        Self {
            x,
            y,
            frame: 0,
            z_index,
        }
    }

    fn update(&mut self, max_y: u16, max_x: u16, rng: &mut ThreadRng) {
        if self.y > 0 {
            self.y -= 1;
        } else {
            self.y = max_y;
            self.x = rng.random_range(1..max_x);
        }

        self.frame = (self.frame + 1) % Self::FRAMES.len();
    }

    fn current_symbol(&self) -> &'static str {
        Self::FRAMES[self.frame]
    }
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

    fn new(x: u16, y: u16, z_index: u8, rng: &mut ThreadRng) -> Self {
        Self {
            x,
            y,
            frame: 0,
            z_index,
            height: rng.random_range(2..=6),
        }
    }

    fn update(&mut self) {
        self.frame = (self.frame + 1) % Self::FRAMES.len();
    }

    fn current_symbol(&self) -> Vec<&str> {
        Self::FRAMES[self.frame].lines().map(str::trim).collect()
    }
}

fn draw_seaweed(lines: &mut Vec<Line>, cols: u16, rows: u16) {
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

        for (dx, ch) in seaweed_line.chars().enumerate() {
            if dx >= cols as usize || dx >= lines[y as usize].spans.len() {
                break;
            }
            lines[y as usize].spans[dx] = Span::styled(ch.to_string(), Style::default().fg(Color::Green));
        }
    }
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    let size = terminal.size()?;
    let (cols, rows) = (size.width, size.height);
    let mut rng = rand::rng();
    let mut last_frame = Instant::now();

    let mut fishes: Vec<Fish> = (0..NUM_FISH)
        .map(|_| Fish::new(cols - 2, rows - 2, &mut rng))
        .collect();
    let mut bubbles: Vec<Bubble> = (0..NUM_BUBBLES)
        .map(|_| {
            Bubble::new(
                rng.random_range(1..cols - 1),
                rng.random_range(1..rows - 1),
                rng.random_range(0..3),
            )
        })
        .collect();
    let mut seaweeds: Vec<Seaweed> = (0..NUM_SEAWEEDS)
        .map(|_| {
            Seaweed::new(
                rng.random_range(1..cols - 1),
                rows - 5,
                rng.random_range(0..3),
                &mut rng,
            )
        })
        .collect();
    
    // Trier une seule fois au début
    bubbles.sort_by_key(|b| b.z_index);
    seaweeds.sort_by_key(|s| s.z_index);
    
    let mut paused = false;
    let mut speed_multiplier = 1.0f64;
    
    loop {
        terminal.draw(|f| {
            let area = f.area();
            let status = if paused { " [PAUSE]" } else { "" };
            let title = format!("Aquatui - Speed: {:.1}x{} - [q]uit [SPACE]pause [+/-]speed", speed_multiplier, status);
            let block = Block::default()
                .title(title)
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White));
            f.render_widget(&block, area);

            let inner = block.inner(area);
            let mut lines =
                vec![Line::from(vec![Span::raw(" "); inner.width as usize]); inner.height as usize];

            // Affichage des poissons (multi-lignes)
            for fish in &fishes {
                for (dy, line) in fish.body.iter().enumerate() {
                    let y = fish.y as usize + dy;
                    if y >= lines.len() {
                        continue;
                    }

                    let chars: Vec<_> = line.chars().collect();
                    let start_x = fish.x as i16;
                    for (i, ch) in chars.into_iter().enumerate() {
                        let screen_x = start_x + i as i16;
                        if screen_x >= 0 && (screen_x as u16) < inner.width {
                            let x_idx = screen_x as usize;
                            if x_idx < lines[y].spans.len() {
                                lines[y].spans[x_idx] =
                                    Span::styled(ch.to_string(), Style::default().fg(fish.color));
                            }
                        }
                    }
                }
            }

            // Bulles
            for bubble in &bubbles {
                let y_idx = bubble.y as usize;
                if y_idx < lines.len() {
                    let x_idx = (bubble.x.min(inner.width - 1)) as usize;
                    if x_idx < lines[y_idx].spans.len() {
                        let symbol = bubble.current_symbol();
                        lines[y_idx].spans[x_idx] =
                            Span::styled(symbol, Style::default().fg(Color::Cyan));
                    }
                }
            }

            // Algues
            draw_seaweed(&mut lines, cols, rows);
            for seaweed in &seaweeds {
                let x = seaweed.x.min(inner.width - 1) as usize;
                let y = seaweed.y.min(inner.height - 1) as usize;
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

            // Kame House
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

                    lines[y].spans[x] =
                        Span::styled(ch.to_string(), Style::default().fg(Color::Magenta));
                }
            }

            let paragraph = Paragraph::new(lines);
            f.render_widget(paragraph, inner);
        })?;

        let adjusted_duration = Duration::from_millis((FRAME_DURATION_MS as f64 / speed_multiplier) as u64);
        if last_frame.elapsed() >= adjusted_duration && !paused {
            fishes = fishes
                .into_iter()
                .filter_map(|mut fish| {
                    if fish.update(cols - 2) {
                        Some(fish)
                    } else {
                        None
                    }
                })
                .collect();

            while fishes.len() < NUM_FISH {
                fishes.push(Fish::new(cols - 2, rows - 2, &mut rng));
            }
            // Mise à jour des bulles
            for bubble in &mut bubbles {
                bubble.update(rows - 2, cols - 1, &mut rng);
            }
            // Mise à jours des algues
            for seaweed in &mut seaweeds {
                seaweed.update();
            }
            last_frame = Instant::now();
        }

        // Gestion des touches
        if poll(Duration::from_millis(1))? {
            if let Event::Key(key) = read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char(' ') => paused = !paused,
                    KeyCode::Char('+') | KeyCode::Char('=') => {
                        speed_multiplier = (speed_multiplier + 0.1).min(5.0);
                    }
                    KeyCode::Char('-') => {
                        speed_multiplier = (speed_multiplier - 0.1).max(0.1);
                    }
                    KeyCode::Char('r') => {
                        // Reset speed
                        speed_multiplier = 1.0;
                        paused = false;
                    }
                    _ => {}
                }
            }
        }

        // Pause très courte pour éviter 100% CPU
        thread::sleep(Duration::from_millis(SLEEP_DURATION_MS));
    }

    Ok(())
}

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}
